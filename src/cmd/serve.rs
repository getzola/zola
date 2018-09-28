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
use std::fs::{remove_dir_all, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::{Instant, Duration};
use std::thread;

use chrono::prelude::*;
use actix_web::{self, fs, http, server, App, HttpRequest, HttpResponse, Responder};
use actix_web::middleware::{Middleware, Started, Response};
use notify::{Watcher, RecursiveMode, watcher};
use ws::{WebSocket, Sender, Message};
use ctrlc;

use site::Site;
use errors::{Result, ResultExt};
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
const LIVE_RELOAD: &'static str = include_str!("livereload.js");

struct NotFoundHandler {
    rendered_template: PathBuf,
}

impl<S> Middleware<S> for NotFoundHandler {
    fn start(&self, _req: &HttpRequest<S>) -> actix_web::Result<Started> {
        Ok(Started::Done)
    }

    fn response(
        &self,
        _req: &HttpRequest<S>,
        mut resp: HttpResponse,
    ) -> actix_web::Result<Response> {
        if http::StatusCode::NOT_FOUND == resp.status() {
            let mut fh = File::open(&self.rendered_template)?;
            let mut buf: Vec<u8> = vec![];
            let _ = fh.read_to_end(&mut buf)?;
            resp.replace_body(buf);
            resp.headers_mut().insert(
                http::header::CONTENT_TYPE,
                http::header::HeaderValue::from_static("text/html"),
            );
        }
        Ok(Response::Done(resp))
    }
}

fn livereload_handler(_: &HttpRequest) -> &'static str {
    LIVE_RELOAD
}

fn rebuild_done_handling(broadcaster: &Sender, res: Result<()>, reload_path: &str) {
    match res {
        Ok(_) => {
            broadcaster.send(format!(r#"
                {{
                    "command": "reload",
                    "path": "{}",
                    "originalPath": "",
                    "liveCSS": true,
                    "liveImg": true,
                    "protocol": ["http://livereload.com/protocols/official-7"]
                }}"#, reload_path)
            ).unwrap();
        },
        Err(e) => console::unravel_errors("Failed to build the site", &e)
    }
}

fn create_new_site(interface: &str, port: &str, output_dir: &str, base_url: &str, config_file: &str) -> Result<(Site, String)> {
    let current_dir = env::current_dir().unwrap();
    let mut site = Site::new(&current_dir, config_file)?;

    // If the current site is also contains a theme.toml we need to merge it
    let theme_file = current_dir.join("theme.toml");
    if site.config.theme == None && theme_file.is_file() {
        site.config.merge_with_theme(&theme_file)?;
    }

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
    site.enable_live_reload();
    console::notify_site_size(&site);
    console::warn_about_ignored_pages(&site);
    site.build()?;
    Ok((site, address))
}

/// Attempt to render `index.html` when a directory is requested.
///
/// The default "batteries included" mechanisms for actix to handle directory
/// listings rely on redirection which behaves oddly (the location headers
/// seem to use relative paths for some reason).
/// They also mean that the address in the browser will include the
/// `index.html` on a successful redirect (rare), which is unsightly.
///
/// Rather than deal with all of that, we can hijack a hook for presenting a
/// custom directory listing response and serve it up using their
/// `NamedFile` responder.
fn handle_directory<'a, 'b>(dir: &'a fs::Directory, req: &'b HttpRequest) -> io::Result<HttpResponse> {
    let mut path = PathBuf::from(&dir.base);
    path.push(&dir.path);
    path.push("index.html");
    fs::NamedFile::open(path)?.respond_to(req)
}

pub fn serve(interface: &str, port: &str, output_dir: &str, base_url: &str, config_file: &str) -> Result<()> {
    let start = Instant::now();
    let (mut site, address) = create_new_site(interface, port, output_dir, base_url, config_file)?;
    console::report_elapsed_time(start);

    // Setup watchers
    let mut watching_static = false;
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    watcher.watch("content/", RecursiveMode::Recursive)
        .chain_err(|| "Can't watch the `content` folder. Does it exist?")?;
    watcher.watch("templates/", RecursiveMode::Recursive)
        .chain_err(|| "Can't watch the `templates` folder. Does it exist?")?;
    watcher.watch(config_file, RecursiveMode::Recursive)
        .chain_err(|| "Can't watch the `config` file. Does it exist?")?;

    if Path::new("static").exists() {
        watching_static = true;
        watcher.watch("static/", RecursiveMode::Recursive)
            .chain_err(|| "Can't watch the `static` folder. Does it exist?")?;
    }

    // Sass support is optional so don't make it an error to no have a sass folder
    let _ = watcher.watch("sass/", RecursiveMode::Recursive);

    let ws_address = format!("{}:{}", interface, site.live_reload.unwrap());
    let output_path = Path::new(output_dir).to_path_buf();

    // output path is going to need to be moved later on, so clone it for the
    // http closure to avoid contention.
    let static_root = output_path.clone();
    thread::spawn(move || {
        let s = server::new(move || {
            App::new()
            .middleware(NotFoundHandler { rendered_template: static_root.join("404.html") })
            .resource(r"/livereload.js", |r| r.f(livereload_handler))
            // Start a webserver that serves the `output_dir` directory
            .handler(
                r"/",
                fs::StaticFiles::new(&static_root)
                    .unwrap()
                    .show_files_listing()
                    .files_listing_renderer(handle_directory)
            )
        })
        .bind(&address)
        .expect("Can't start the webserver")
        .shutdown_timeout(20);
        println!("Web server is available at http://{}", &address);
        s.run();
    });

    // The websocket for livereload
    let ws_server = WebSocket::new(|output: Sender| {
        move |msg: Message| {
            if msg.into_text().unwrap().contains("\"hello\"") {
                return output.send(Message::text(r#"
                    {
                        "command": "hello",
                        "protocols": [ "http://livereload.com/protocols/official-7" ],
                        "serverName": "Gutenberg"
                    }
                "#));
            }
            Ok(())
        }
    }).unwrap();
    let broadcaster = ws_server.broadcaster();
    thread::spawn(move || {
        ws_server.listen(&*ws_address).unwrap();
    });

    let pwd = env::current_dir().unwrap();

    let mut watchers = vec!["content", "templates", "config.toml"];
    if watching_static {
        watchers.push("static");
    }
    if site.config.compile_sass {
        watchers.push("sass");
    }

    println!("Listening for changes in {}/{{{}}}", pwd.display(), watchers.join(", "));

    println!("Press Ctrl+C to stop\n");
    // Delete the output folder on ctrl+C
    ctrlc::set_handler(move || {
        remove_dir_all(&output_path).expect("Failed to delete output directory");
        ::std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    use notify::DebouncedEvent::*;

    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    Create(path) |
                    Write(path) |
                    Remove(path) |
                    Rename(_, path) => {
                        if is_temp_file(&path) || path.is_dir() {
                            continue;
                        }

                        println!("Change detected @ {}", Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                        let start = Instant::now();
                        match detect_change_kind(&pwd, &path) {
                            (ChangeKind::Content, _) => {
                                console::info(&format!("-> Content changed {}", path.display()));
                                // Force refresh
                                rebuild_done_handling(&broadcaster, rebuild::after_content_change(&mut site, &path), "/x.js");
                            },
                            (ChangeKind::Templates, _) => {
                                console::info(&format!("-> Template changed {}", path.display()));
                                // Force refresh
                                rebuild_done_handling(&broadcaster, rebuild::after_template_change(&mut site, &path), "/x.js");
                            },
                            (ChangeKind::StaticFiles, p) => {
                                if path.is_file() {
                                    console::info(&format!("-> Static file changes detected {}", path.display()));
                                    rebuild_done_handling(&broadcaster, copy_file(&path, &site.output_path, &site.static_path), &p.to_string_lossy());
                                }
                            },
                            (ChangeKind::Sass, p) => {
                                console::info(&format!("-> Sass file changed {}", path.display()));
                                rebuild_done_handling(&broadcaster, site.compile_sass(&site.base_path), &p.to_string_lossy());
                            },
                            (ChangeKind::Config, _) => {
                                console::info(&format!("-> Config changed. The whole site will be reloaded. The browser needs to be refreshed to make the changes visible."));
                                site = create_new_site(interface, port, output_dir, base_url, config_file).unwrap().0;
                            }
                        };
                        console::report_elapsed_time(start);
                    }
                    _ => {}
                }
            },
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
        None => {
            true
        },
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{is_temp_file, detect_change_kind, ChangeKind};

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
                Path::new("/home/vincent/site"), Path::new("/home/vincent/site/templates/hello.html")
            ),
            (
                (ChangeKind::StaticFiles, PathBuf::from("/static/site.css")),
                Path::new("/home/vincent/site"), Path::new("/home/vincent/site/static/site.css")
            ),
            (
                (ChangeKind::Content, PathBuf::from("/content/posts/hello.md")),
                Path::new("/home/vincent/site"), Path::new("/home/vincent/site/content/posts/hello.md")
            ),
            (
                (ChangeKind::Sass, PathBuf::from("/sass/print.scss")),
                Path::new("/home/vincent/site"), Path::new("/home/vincent/site/sass/print.scss")
            ),
            (
                (ChangeKind::Config, PathBuf::from("/config.toml")),
                Path::new("/home/vincent/site"), Path::new("/home/vincent/site/config.toml")
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
