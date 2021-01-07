// Contains an embedded version of livereload-js 3.2.4
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

use std::fs::{read_dir, remove_dir_all};
use std::net::{SocketAddrV4, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

use hyper::header;
use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, StatusCode};

use chrono::prelude::*;
use notify::{watcher, RecursiveMode, Watcher};
use ws::{Message, Sender, WebSocket};

use errors::{Error as ZolaError, Result};
use globset::GlobSet;
use relative_path::{RelativePath, RelativePathBuf};
use site::sass::compile_sass;
use site::{Site, SITE_CONTENT};
use utils::fs::copy_file;

use crate::console;
use std::ffi::OsStr;

#[derive(Debug, PartialEq)]
enum ChangeKind {
    Content,
    Templates,
    Themes,
    StaticFiles,
    Sass,
    Config,
}

#[derive(Debug, PartialEq)]
enum WatchMode {
    Required,
    Optional,
    Condition(bool),
}

static METHOD_NOT_ALLOWED_TEXT: &[u8] = b"Method Not Allowed";
static NOT_FOUND_TEXT: &[u8] = b"Not Found";

// This is dist/livereload.min.js from the LiveReload.js v3.2.4 release
const LIVE_RELOAD: &str = include_str!("livereload.js");

async fn handle_request(req: Request<Body>, mut root: PathBuf) -> Result<Response<Body>> {
    let mut path = RelativePathBuf::new();
    // https://zola.discourse.group/t/percent-encoding-for-slugs/736
    let decoded = match percent_encoding::percent_decode_str(req.uri().path()).decode_utf8() {
        Ok(d) => d,
        Err(_) => return Ok(not_found()),
    };

    for c in decoded.split('/') {
        path.push(c);
    }

    // livereload.js is served using the LIVE_RELOAD str, not a file
    if path == "livereload.js" {
        if req.method() == Method::GET {
            return Ok(livereload_js());
        } else {
            return Ok(method_not_allowed());
        }
    }

    if let Some(content) = SITE_CONTENT.read().unwrap().get(&path) {
        return Ok(in_memory_html(content));
    }

    // Handle only `GET`/`HEAD` requests
    match *req.method() {
        Method::HEAD | Method::GET => {}
        _ => return Ok(method_not_allowed()),
    }

    // Handle only simple path requests
    if req.uri().scheme_str().is_some() || req.uri().host().is_some() {
        return Ok(not_found());
    }

    // Remove the trailing slash from the request path
    // otherwise `PathBuf` will interpret it as an absolute path
    root.push(&req.uri().path()[1..]);
    let result = tokio::fs::read(root).await;

    let contents = match result {
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => return Ok(not_found()),
            std::io::ErrorKind::PermissionDenied => {
                return Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(Body::empty())
                    .unwrap())
            }
            _ => panic!("{}", err),
        },
        Ok(contents) => contents,
    };

    Ok(Response::builder().status(StatusCode::OK).body(Body::from(contents)).unwrap())
}

fn livereload_js() -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/javascript")
        .status(StatusCode::OK)
        .body(LIVE_RELOAD.into())
        .expect("Could not build livereload.js response")
}

fn in_memory_html(content: &str) -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/html")
        .status(StatusCode::OK)
        .body(content.to_owned().into())
        .expect("Could not build HTML response")
}

fn method_not_allowed() -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/plain")
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(METHOD_NOT_ALLOWED_TEXT.into())
        .expect("Could not build Method Not Allowed response")
}

fn not_found() -> Response<Body> {
    let not_found_path = RelativePath::new("404.html");
    let content = SITE_CONTENT.read().unwrap().get(not_found_path).cloned();

    if let Some(body) = content {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            .status(StatusCode::NOT_FOUND)
            .body(body.into())
            .expect("Could not build Not Found response");
    }

    // Use a plain text response when we can't find the body of the 404
    Response::builder()
        .header(header::CONTENT_TYPE, "text/plain")
        .status(StatusCode::NOT_FOUND)
        .body(NOT_FOUND_TEXT.into())
        .expect("Could not build Not Found response")
}

fn rebuild_done_handling(broadcaster: &Sender, res: Result<()>, reload_path: &str) {
    match res {
        Ok(_) => {
            broadcaster
                .send(format!(
                    r#"
                {{
                    "command": "reload",
                    "path": {},
                    "originalPath": "",
                    "liveCSS": true,
                    "liveImg": true,
                    "protocol": ["http://livereload.com/protocols/official-7"]
                }}"#,
                    serde_json::to_string(&reload_path).unwrap()
                ))
                .unwrap();
        }
        Err(e) => console::unravel_errors("Failed to build the site", &e),
    }
}

fn create_new_site(
    root_dir: &Path,
    interface: &str,
    interface_port: u16,
    output_dir: Option<&Path>,
    base_url: &str,
    config_file: &Path,
    include_drafts: bool,
    ws_port: Option<u16>,
) -> Result<(Site, String)> {
    let mut site = Site::new(root_dir, config_file)?;

    let base_address = format!("{}:{}", base_url, interface_port);
    let address = format!("{}:{}", interface, interface_port);

    let base_url = if site.config.base_url.ends_with('/') {
        format!("http://{}/", base_address)
    } else {
        format!("http://{}", base_address)
    };

    site.enable_serve_mode();
    site.set_base_url(base_url);
    if let Some(output_dir) = output_dir {
        site.set_output_path(output_dir);
    }
    if include_drafts {
        site.include_drafts();
    }
    site.load()?;
    if let Some(p) = ws_port {
        site.enable_live_reload_with_port(p);
    } else {
        site.enable_live_reload(interface_port);
    }
    console::notify_site_size(&site);
    console::warn_about_ignored_pages(&site);
    site.build()?;
    Ok((site, address))
}

pub fn serve(
    root_dir: &Path,
    interface: &str,
    interface_port: u16,
    output_dir: Option<&Path>,
    base_url: &str,
    config_file: &Path,
    open: bool,
    include_drafts: bool,
    fast_rebuild: bool,
) -> Result<()> {
    let start = Instant::now();
    let (mut site, address) = create_new_site(
        root_dir,
        interface,
        interface_port,
        output_dir,
        base_url,
        config_file,
        include_drafts,
        None,
    )?;
    console::report_elapsed_time(start);

    // Stop right there if we can't bind to the address
    let bind_address: SocketAddrV4 = address.parse().unwrap();
    if (TcpListener::bind(&bind_address)).is_err() {
        return Err(format!("Cannot start server on address {}.", address).into());
    }

    // An array of (path, bool, bool) where the path should be watched for changes, and the boolean value
    // indicates whether this file/folder must exist for zola serve to operate
    let watch_this = vec![
        ("config.toml", WatchMode::Required),
        ("content", WatchMode::Required),
        ("sass", WatchMode::Condition(site.config.compile_sass)),
        ("static", WatchMode::Optional),
        ("templates", WatchMode::Optional),
        ("themes", WatchMode::Condition(site.config.theme.is_some())),
    ];

    // Setup watchers
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();

    // We watch for changes on the filesystem for every entry in watch_this
    // Will fail if either:
    //   - the path is mandatory but does not exist (eg. config.toml)
    //   - the path exists but has incorrect permissions
    // watchers will contain the paths we're actually watching
    let mut watchers = Vec::new();
    for (entry, mode) in watch_this {
        let watch_path = root_dir.join(entry);
        let should_watch = match mode {
            WatchMode::Required => true,
            WatchMode::Optional => watch_path.exists(),
            WatchMode::Condition(b) => b && watch_path.exists(),
        };
        if should_watch {
            watcher
                .watch(root_dir.join(entry), RecursiveMode::Recursive)
                .map_err(|e| ZolaError::chain(format!("Can't watch `{}` for changes in folder `{}`. Does it exist, and do you have correct permissions?", entry, root_dir.display()), e))?;
            watchers.push(entry.to_string());
        }
    }

    let ws_port = site.live_reload;
    let ws_address = format!("{}:{}", interface, ws_port.unwrap());
    let output_path = site.output_path.clone();

    // output path is going to need to be moved later on, so clone it for the
    // http closure to avoid contention.
    let static_root = output_path.clone();
    let broadcaster = {
        thread::spawn(move || {
            let addr = address.parse().unwrap();

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Could not build tokio runtime");

            rt.block_on(async {
                let make_service = make_service_fn(move |_| {
                    let static_root = static_root.clone();

                    async {
                        Ok::<_, hyper::Error>(service_fn(move |req| {
                            handle_request(req, static_root.clone())
                        }))
                    }
                });

                let server = Server::bind(&addr).serve(make_service);

                println!("Web server is available at http://{}\n", &address);
                if open {
                    if let Err(err) = open::that(format!("http://{}", &address)) {
                        eprintln!("Failed to open URL in your browser: {}", err);
                    }
                }

                server.await.expect("Could not start web server");
            });
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

        let ws_server = ws_server
            .bind(&*ws_address)
            .map_err(|_| format!("Cannot bind to address {} for the websocket server. Maybe the port is already in use?", &ws_address))?;

        thread::spawn(move || {
            ws_server.run().unwrap();
        });

        broadcaster
    };

    println!("Listening for changes in {}{{{}}}", root_dir.display(), watchers.join(", "));

    println!("Press Ctrl+C to stop\n");
    // Delete the output folder on ctrl+C
    ctrlc::set_handler(move || {
        match remove_dir_all(&output_path) {
            Ok(()) => (),
            Err(e) => println!("Errored while deleting output folder: {}", e),
        }
        ::std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    use notify::DebouncedEvent::*;

    let reload_sass = |site: &Site, path: &Path, partial_path: &Path| {
        let msg = if path.is_dir() {
            format!("-> Directory in `sass` folder changed {}", path.display())
        } else {
            format!("-> Sass file changed {}", path.display())
        };
        console::info(&msg);
        rebuild_done_handling(
            &broadcaster,
            compile_sass(&site.base_path, &site.output_path),
            &partial_path.to_string_lossy(),
        );
    };

    let reload_templates = |site: &mut Site, path: &Path| {
        rebuild_done_handling(&broadcaster, site.reload_templates(), &path.to_string_lossy());
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
                copy_file(
                    &path,
                    &site.output_path,
                    &site.static_path,
                    site.config.hard_link_static,
                ),
                &partial_path.to_string_lossy(),
            );
        }
    };

    let recreate_site = || match create_new_site(
        root_dir,
        interface,
        interface_port,
        output_dir,
        base_url,
        config_file,
        include_drafts,
        ws_port,
    ) {
        Ok((s, _)) => {
            rebuild_done_handling(&broadcaster, Ok(()), "/x.js");
            Some(s)
        }
        Err(e) => {
            console::error(&format!("{}", e));
            None
        }
    };

    loop {
        match rx.recv() {
            Ok(event) => {
                let can_do_fast_reload = !matches!(event, Remove(_));

                match event {
                    // Intellij does weird things on edit, chmod is there to count those changes
                    // https://github.com/passcod/notify/issues/150#issuecomment-494912080
                    Rename(_, path) | Create(path) | Write(path) | Remove(path) | Chmod(path) => {
                        if is_ignored_file(&site.config.ignored_content_globset, &path) {
                            continue;
                        }
                        if is_temp_file(&path) || path.is_dir() {
                            continue;
                        }
                        // We only care about changes in non-empty folders
                        if path.is_dir() && is_folder_empty(&path) {
                            continue;
                        }

                        println!(
                            "Change detected @ {}",
                            Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
                        );

                        let start = Instant::now();
                        match detect_change_kind(&root_dir, &path) {
                            (ChangeKind::Content, _) => {
                                console::info(&format!("-> Content changed {}", path.display()));

                                if fast_rebuild {
                                    if can_do_fast_reload {
                                        let filename = path
                                            .file_name()
                                            .unwrap_or_else(|| OsStr::new(""))
                                            .to_string_lossy();
                                        let res = if filename == "_index.md" {
                                            site.add_and_render_section(&path)
                                        } else if filename.ends_with(".md") {
                                            site.add_and_render_page(&path)
                                        } else {
                                            // an asset changed? a folder renamed?
                                            // should we make it smarter so it doesn't reload the whole site?
                                            Err("dummy".into())
                                        };

                                        if res.is_err() {
                                            if let Some(s) = recreate_site() {
                                                site = s;
                                            }
                                        } else {
                                            rebuild_done_handling(
                                                &broadcaster,
                                                res,
                                                &path.to_string_lossy(),
                                            );
                                        }
                                    } else {
                                        // Should we be smarter than that? Is it worth it?
                                        if let Some(s) = recreate_site() {
                                            site = s;
                                        }
                                    }
                                } else if let Some(s) = recreate_site() {
                                    site = s;
                                }
                            }
                            (ChangeKind::Templates, partial_path) => {
                                let msg = if path.is_dir() {
                                    format!(
                                        "-> Directory in `templates` folder changed {}",
                                        path.display()
                                    )
                                } else {
                                    format!("-> Template changed {}", path.display())
                                };
                                console::info(&msg);

                                // A shortcode changed, we need to rebuild everything
                                if partial_path.starts_with("/templates/shortcodes") {
                                    if let Some(s) = recreate_site() {
                                        site = s;
                                    }
                                } else {
                                    println!("Reloading only template");
                                    // A normal template changed, no need to re-render Markdown.
                                    reload_templates(&mut site, &path)
                                }
                            }
                            (ChangeKind::StaticFiles, p) => copy_static(&site, &path, &p),
                            (ChangeKind::Sass, p) => reload_sass(&site, &path, &p),
                            (ChangeKind::Themes, _) => {
                                console::info("-> Themes changed.");

                                if let Some(s) = recreate_site() {
                                    site = s;
                                }
                            }
                            (ChangeKind::Config, _) => {
                                console::info("-> Config changed. The browser needs to be refreshed to make the changes visible.");

                                if let Some(s) = recreate_site() {
                                    site = s;
                                }
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

fn is_ignored_file(ignored_content_globset: &Option<GlobSet>, path: &Path) -> bool {
    match ignored_content_globset {
        Some(gs) => gs.is_match(path),
        None => false,
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
    } else if partial_path.starts_with("/themes") {
        ChangeKind::Themes
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
                (ChangeKind::Themes, PathBuf::from("/themes/hello.html")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/themes/hello.html"),
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
