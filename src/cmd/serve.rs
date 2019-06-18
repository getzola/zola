// Contains an embedded version of livereload-js
//
// Copyright (c) 2010-2012 Andrey Tarantsov
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use std::env;
use std::fs::{read_dir, remove_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

use actix_files as fs;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{dev, http, web, App, HttpResponse, HttpServer};
use chrono::prelude::*;
use ctrlc;
use notify::{watcher, RecursiveMode, Watcher};
use ws::{Message, Sender, WebSocket};

use errors::{Error as ZolaError, Result};
use site::Site;
use utils::fs::copy_file;

use console;
use rebuild;

#[derive(Debug, PartialEq)]
enum ChangeKind {
    Content,
    Templates,
    StaticFiles,
    Sass,
    Config,
}

// Uglified using uglifyjs
// Also, commenting out the lines 330-340 (containing `e instanceof ProtocolError`) was needed
// as it seems their build didn't work well and didn't include ProtocolError so it would error on
// errors
const LIVE_RELOAD: &str = include_str!("livereload.js");

struct ErrorFilePaths {
    not_found: PathBuf,
}

fn not_found<B>(
    res: dev::ServiceResponse<B>
) -> std::result::Result<ErrorHandlerResponse<B>, actix_web::Error> {
    let buf: Vec<u8> = {
        let error_files: &ErrorFilePaths = res.request().app_data().unwrap();

        let mut fh = File::open(&error_files.not_found)?;
        let mut buf: Vec<u8> = vec![];
        let _ = fh.read_to_end(&mut buf)?;
        buf
    };

    let new_resp = HttpResponse::build(http::StatusCode::NOT_FOUND)
        .header(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_static("text/html"),
        )
        .body(buf);

    Ok(ErrorHandlerResponse::Response(
        res.into_response(new_resp.into_body()),
    ))
}

fn livereload_handler() -> HttpResponse {
    HttpResponse::Ok().content_type("text/javascript").body(LIVE_RELOAD)
}

fn rebuild_done_handling(broadcaster: &Option<Sender>, res: Result<()>, reload_path: &str) {
    match res {
        Ok(_) => {
            if let Some(broadcaster) = broadcaster.as_ref() {
                broadcaster
                    .send(format!(
                        r#"
                    {{
                        "command": "reload",
                        "path": "{}",
                        "originalPath": "",
                        "liveCSS": true,
                        "liveImg": true,
                        "protocol": ["http://livereload.com/protocols/official-7"]
                    }}"#,
                        reload_path
                    ))
                    .unwrap();
            }
        }
        Err(e) => console::unravel_errors("Failed to build the site", &e),
    }
}

fn create_new_site(
    interface: &str,
    port: u16,
    output_dir: &str,
    base_url: &str,
    config_file: &str,
) -> Result<(Site, String)> {
    let mut site = Site::new(env::current_dir().unwrap(), config_file)?;

    let base_address = format!("{}:{}", base_url, port);
    let address = format!("{}:{}", interface, port);
    let base_url = if site.config.base_url.ends_with('/') {
        format!("http://{}/", base_address)
    } else {
        format!("http://{}", base_address)
    };

    site.set_base_url(base_url);
    site.set_output_path(output_dir);
    site.load()?;
    site.enable_live_reload(port);
    console::notify_site_size(&site);
    console::warn_about_ignored_pages(&site);
    site.build()?;
    Ok((site, address))
}

pub fn serve(
    interface: &str,
    port: u16,
    output_dir: &str,
    base_url: &str,
    config_file: &str,
    watch_only: bool,
) -> Result<()> {
    let start = Instant::now();
    let (mut site, address) = create_new_site(interface, port, output_dir, base_url, config_file)?;
    console::report_elapsed_time(start);

    // Setup watchers
    let mut watching_static = false;
    let mut watching_templates = false;
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher
        .watch("content/", RecursiveMode::Recursive)
        .map_err(|e| ZolaError::chain("Can't watch the `content` folder. Does it exist?", e))?;
    watcher
        .watch(config_file, RecursiveMode::Recursive)
        .map_err(|e| ZolaError::chain("Can't watch the `config` file. Does it exist?", e))?;

    if Path::new("static").exists() {
        watching_static = true;
        watcher
            .watch("static/", RecursiveMode::Recursive)
            .map_err(|e| ZolaError::chain("Can't watch the `static` folder.", e))?;
    }

    if Path::new("templates").exists() {
        watching_templates = true;
        watcher
            .watch("templates/", RecursiveMode::Recursive)
            .map_err(|e| ZolaError::chain("Can't watch the `templates` folder.", e))?;
    }

    // Sass support is optional so don't make it an error to no have a sass folder
    let _ = watcher.watch("sass/", RecursiveMode::Recursive);

    let ws_address = format!("{}:{}", interface, site.live_reload.unwrap());
    let output_path = Path::new(output_dir).to_path_buf();

    // output path is going to need to be moved later on, so clone it for the
    // http closure to avoid contention.
    let static_root = output_path.clone();
    let broadcaster = if !watch_only {
        thread::spawn(move || {
            let s = HttpServer::new(move || {
                let error_handlers = ErrorHandlers::new()
                    .handler(http::StatusCode::NOT_FOUND, not_found);

                App::new()
                    .data(ErrorFilePaths {
                        not_found: static_root.join("404.html"),
                    })
                    .wrap(error_handlers)
                    .route(
                        "/livereload.js",
                        web::get().to(livereload_handler)
                    )
                    // Start a webserver that serves the `output_dir` directory
                    .service(
                        fs::Files::new("/", &static_root)
                            .index_file("index.html"),
                    )
            })
            .bind(&address)
            .expect("Can't start the webserver")
            .shutdown_timeout(20);
            println!("Web server is available at http://{}\n", &address);
            s.run()
        });
        // The websocket for livereload
        let ws_server = WebSocket::new(|output: Sender| {
            move |msg: Message| {
                if msg.into_text().unwrap().contains("\"hello\"") {
                    return output.send(Message::text(
                        r#"
                        {
                            "command": "hello",
                            "protocols": [ "http://livereload.com/protocols/official-7" ],
                            "serverName": "Zola"
                        }
                    "#,
                    ));
                }
                Ok(())
            }
        })
        .unwrap();
        let broadcaster = ws_server.broadcaster();
        thread::spawn(move || {
            ws_server.listen(&*ws_address).unwrap();
        });
        Some(broadcaster)
    } else {
        println!("Watching in watch only mode, no web server will be started");
        None
    };

    let pwd = env::current_dir().unwrap();

    let mut watchers = vec!["content", "config.toml"];
    if watching_static {
        watchers.push("static");
    }
    if watching_templates {
        watchers.push("templates");
    }
    if site.config.compile_sass {
        watchers.push("sass");
    }

    println!(
        "Listening for changes in {}{}{{{}}}",
        pwd.display(),
        MAIN_SEPARATOR,
        watchers.join(", ")
    );

    println!("Press Ctrl+C to stop\n");
    // Delete the output folder on ctrl+C
    ctrlc::set_handler(move || {
        remove_dir_all(&output_path).expect("Failed to delete output directory");
        ::std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    use notify::DebouncedEvent::*;

    let reload_templates = |site: &mut Site, path: &Path| {
        let msg = if path.is_dir() {
            format!("-> Directory in `templates` folder changed {}", path.display())
        } else {
            format!("-> Template changed {}", path.display())
        };
        console::info(&msg);
        // Force refresh
        rebuild_done_handling(&broadcaster, rebuild::after_template_change(site, &path), "/x.js");
    };

    let reload_sass = |site: &Site, path: &Path, partial_path: &Path| {
        let msg = if path.is_dir() {
            format!("-> Directory in `sass` folder changed {}", path.display())
        } else {
            format!("-> Sass file changed {}", path.display())
        };
        console::info(&msg);
        rebuild_done_handling(
            &broadcaster,
            site.compile_sass(&site.base_path),
            &partial_path.to_string_lossy(),
        );
    };

    let copy_static = |site: &Site, path: &Path, partial_path: &Path| {
        // Do nothing if the file/dir was deleted
        if !path.exists() {
            return;
        }

        let msg = if path.is_dir() {
            format!("-> Directory in `static` folder changed {}", path.display())
        } else {
            format!("-> Static file changed {}", path.display())
        };

        console::info(&msg);
        if path.is_dir() {
            rebuild_done_handling(
                &broadcaster,
                site.copy_static_directories(),
                &path.to_string_lossy(),
            );
        } else {
            rebuild_done_handling(
                &broadcaster,
                copy_file(&path, &site.output_path, &site.static_path),
                &partial_path.to_string_lossy(),
            );
        }
    };

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    Rename(old_path, path) => {
                        if path.is_file() && is_temp_file(&path) {
                            continue;
                        }
                        let (change_kind, partial_path) = detect_change_kind(&pwd, &path);

                        // We only care about changes in non-empty folders
                        if path.is_dir() && is_folder_empty(&path) {
                            continue;
                        }

                        println!(
                            "Change detected @ {}",
                            Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
                        );

                        let start = Instant::now();
                        match change_kind {
                            ChangeKind::Content => {
                                console::info(&format!("-> Content renamed {}", path.display()));
                                // Force refresh
                                rebuild_done_handling(
                                    &broadcaster,
                                    rebuild::after_content_rename(&mut site, &old_path, &path),
                                    "/x.js",
                                );
                            }
                            ChangeKind::Templates => reload_templates(&mut site, &path),
                            ChangeKind::StaticFiles => copy_static(&site, &path, &partial_path),
                            ChangeKind::Sass => reload_sass(&site, &path, &partial_path),
                            ChangeKind::Config => {
                                console::info("-> Config changed. The whole site will be reloaded. The browser needs to be refreshed to make the changes visible.");
                                site = create_new_site(
                                    interface,
                                    port,
                                    output_dir,
                                    base_url,
                                    config_file,
                                )
                                .unwrap()
                                .0;
                            }
                        }
                        console::report_elapsed_time(start);
                    }
                    // Intellij does weird things on edit, chmod is there to count those changes
                    // https://github.com/passcod/notify/issues/150#issuecomment-494912080
                    Create(path) | Write(path) | Remove(path) | Chmod(path) => {
                        if is_temp_file(&path) || path.is_dir() {
                            continue;
                        }

                        println!(
                            "Change detected @ {}",
                            Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
                        );

                        let start = Instant::now();
                        match detect_change_kind(&pwd, &path) {
                            (ChangeKind::Content, _) => {
                                console::info(&format!("-> Content changed {}", path.display()));
                                // Force refresh
                                rebuild_done_handling(
                                    &broadcaster,
                                    rebuild::after_content_change(&mut site, &path),
                                    "/x.js",
                                );
                            }
                            (ChangeKind::Templates, _) => reload_templates(&mut site, &path),
                            (ChangeKind::StaticFiles, p) => copy_static(&site, &path, &p),
                            (ChangeKind::Sass, p) => reload_sass(&site, &path, &p),
                            (ChangeKind::Config, _) => {
                                console::info("-> Config changed. The whole site will be reloaded. The browser needs to be refreshed to make the changes visible.");
                                site = create_new_site(
                                    interface,
                                    port,
                                    output_dir,
                                    base_url,
                                    config_file,
                                )
                                .unwrap()
                                .0;
                            }
                        };
                        console::report_elapsed_time(start);
                    }
                    _ => {}
                }
            }
            Err(e) => console::error(&format!("Watch error: {:?}", e)),
        };
    }
}

/// Returns whether the path we received corresponds to a temp file created
/// by an editor or the OS
fn is_temp_file(path: &Path) -> bool {
    let ext = path.extension();
    match ext {
        Some(ex) => match ex.to_str().unwrap() {
            "swp" | "swx" | "tmp" | ".DS_STORE" => true,
            // jetbrains IDE
            x if x.ends_with("jb_old___") => true,
            x if x.ends_with("jb_tmp___") => true,
            x if x.ends_with("jb_bak___") => true,
            // vim
            x if x.ends_with('~') => true,
            _ => {
                if let Some(filename) = path.file_stem() {
                    // emacs
                    let name = filename.to_str().unwrap();
                    name.starts_with('#') || name.starts_with(".#")
                } else {
                    false
                }
            }
        },
        None => true,
    }
}

/// Detect what changed from the given path so we have an idea what needs
/// to be reloaded
fn detect_change_kind(pwd: &Path, path: &Path) -> (ChangeKind, PathBuf) {
    let mut partial_path = PathBuf::from("/");
    partial_path.push(path.strip_prefix(pwd).unwrap_or(path));

    let change_kind = if partial_path.starts_with("/templates") {
        ChangeKind::Templates
    } else if partial_path.starts_with("/content") {
        ChangeKind::Content
    } else if partial_path.starts_with("/static") {
        ChangeKind::StaticFiles
    } else if partial_path.starts_with("/sass") {
        ChangeKind::Sass
    } else if partial_path == Path::new("/config.toml") {
        ChangeKind::Config
    } else {
        unreachable!("Got a change in an unexpected path: {}", partial_path.display());
    };

    (change_kind, partial_path)
}

/// Check if the directory at path contains any file
fn is_folder_empty(dir: &Path) -> bool {
    // Can panic if we don't have the rights I guess?
    let files: Vec<_> =
        read_dir(dir).expect("Failed to read a directory to see if it was empty").collect();
    files.is_empty()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{detect_change_kind, is_temp_file, ChangeKind};

    #[test]
    fn can_recognize_temp_files() {
        let test_cases = vec![
            Path::new("hello.swp"),
            Path::new("hello.swx"),
            Path::new(".DS_STORE"),
            Path::new("hello.tmp"),
            Path::new("hello.html.__jb_old___"),
            Path::new("hello.html.__jb_tmp___"),
            Path::new("hello.html.__jb_bak___"),
            Path::new("hello.html~"),
            Path::new("#hello.html"),
        ];

        for t in test_cases {
            assert!(is_temp_file(&t));
        }
    }

    #[test]
    fn can_detect_kind_of_changes() {
        let test_cases = vec![
            (
                (ChangeKind::Templates, PathBuf::from("/templates/hello.html")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/templates/hello.html"),
            ),
            (
                (ChangeKind::StaticFiles, PathBuf::from("/static/site.css")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/static/site.css"),
            ),
            (
                (ChangeKind::Content, PathBuf::from("/content/posts/hello.md")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/content/posts/hello.md"),
            ),
            (
                (ChangeKind::Sass, PathBuf::from("/sass/print.scss")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/sass/print.scss"),
            ),
            (
                (ChangeKind::Config, PathBuf::from("/config.toml")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/config.toml"),
            ),
        ];

        for (expected, pwd, path) in test_cases {
            assert_eq!(expected, detect_change_kind(&pwd, &path));
        }
    }

    #[test]
    #[cfg(windows)]
    fn windows_path_handling() {
        let expected = (ChangeKind::Templates, PathBuf::from("/templates/hello.html"));
        let pwd = Path::new(r#"C:\\Users\johan\site"#);
        let path = Path::new(r#"C:\\Users\johan\site\templates\hello.html"#);
        assert_eq!(expected, detect_change_kind(pwd, path));
    }

    #[test]
    fn relative_path() {
        let expected = (ChangeKind::Templates, PathBuf::from("/templates/hello.html"));
        let pwd = Path::new("/home/johan/site");
        let path = Path::new("templates/hello.html");
        assert_eq!(expected, detect_change_kind(pwd, path));
    }
}
