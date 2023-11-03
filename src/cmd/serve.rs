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

use std::cell::Cell;
use std::fs::read_dir;
use std::future::IntoFuture;
use std::net::{SocketAddrV4, TcpListener};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use hyper::http::HeaderValue;
use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body, header};
use hyper::{Body, Method, Request, Response, StatusCode};
use mime_guess::from_path as mimetype_from_path;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};

use libs::globset::GlobSet;
use libs::percent_encoding;
use libs::relative_path::{RelativePath, RelativePathBuf};
use libs::serde_json;
use notify::{watcher, RecursiveMode, Watcher};
use ws::{Message, Sender, WebSocket};

use errors::{anyhow, Context, Error, Result};
use pathdiff::diff_paths;
use site::sass::compile_sass;
use site::{Site, SITE_CONTENT};
use utils::fs::{clean_site_output_folder, copy_file, is_temp_file};

use crate::messages;
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

static SERVE_ERROR: Mutex<Cell<Option<(&'static str, errors::Error)>>> =
    Mutex::new(Cell::new(None));

fn clear_serve_error() {
    let _ = SERVE_ERROR.lock().map(|error| error.swap(&Cell::new(None)));
}

fn set_serve_error(msg: &'static str, e: errors::Error) {
    if let Ok(serve_error) = SERVE_ERROR.lock() {
        serve_error.swap(&Cell::new(Some((msg, e))));
    }
}

async fn handle_request(req: Request<Body>, mut root: PathBuf) -> Result<Response<Body>> {
    let original_root = root.clone();
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
        return Ok(in_memory_content(&path, content));
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

    // Remove the first slash from the request path
    // otherwise `PathBuf` will interpret it as an absolute path
    root.push(&decoded[1..]);

    // Resolve the root + user supplied path into the absolute path
    // this should hopefully remove any path traversals
    // if we fail to resolve path, we should return 404
    root = match tokio::fs::canonicalize(&root).await {
        Ok(d) => d,
        Err(_) => return Ok(not_found()),
    };

    // Ensure we are only looking for things in our public folder
    if !root.starts_with(original_root) {
        return Ok(not_found());
    }

    let metadata = match tokio::fs::metadata(root.as_path()).await {
        Err(err) => return Ok(io_error(err)),
        Ok(metadata) => metadata,
    };
    if metadata.is_dir() {
        // if root is a directory, append index.html to try to read that instead
        root.push("index.html");
    };

    let result = tokio::fs::read(&root).await;

    let contents = match result {
        Err(err) => return Ok(io_error(err)),
        Ok(contents) => contents,
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            mimetype_from_path(&root).first_or_octet_stream().essence_str(),
        )
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(Body::from(contents))
        .unwrap())
}

/// Inserts build error message boxes into HTML responses when needed.
async fn response_error_injector(
    req: impl IntoFuture<Output = Result<Response<Body>>>,
) -> Result<Response<Body>> {
    let req = req.await;

    // return req as-is if the request is Err(), not HTML, or if there are no error messages.
    if req
        .as_ref()
        .map(|req| {
            req.headers()
                .get(header::CONTENT_TYPE)
                .map(|val| val != &HeaderValue::from_static("text/html"))
                .unwrap_or(true)
        })
        .unwrap_or(true)
        || SERVE_ERROR.lock().unwrap().get_mut().is_none()
    {
        return req;
    }

    let mut req = req.unwrap();
    let mut bytes = body::to_bytes(req.body_mut()).await.unwrap().to_vec();

    if let Some((msg, error)) = SERVE_ERROR.lock().unwrap().get_mut() {
        // Generate an error message similar to the CLI version in messages::unravel_errors.
        let mut error_str = String::new();

        if !msg.is_empty() {
            error_str.push_str(&format!("Error: {msg}\n"));
        }

        error_str.push_str(&format!("Error: {error}\n"));

        let mut cause = error.source();
        while let Some(e) = cause {
            error_str.push_str(&format!("Reason: {}\n", e));
            cause = e.source();
        }

        // Push the error message (wrapped in an HTML dialog box) to the end of the HTML body.
        //
        // The message will be outside of <html> and <body> but web browsers are flexible enough
        // that they will move it to the end of <body> at page load.
        let html_error = format!(
            r#"<div style="all:revert;position:fixed;display:flex;align-items:center;justify-content:center;background-color:rgb(0,0,0,0.5);top:0;right:0;bottom:0;left:0;"><div style="background-color:white;padding:0.5rem;border-radius:0.375rem;filter:drop-shadow(0,25px,25px,rgb(0,0,0/0.15));overflow-x:auto;"><p style="font-weight:700;color:black;font-size:1.25rem;margin:0;margin-bottom:0.5rem;">Zola Build Error:</p><pre style="padding:0.5rem;margin:0;border-radius:0.375rem;background-color:#363636;color:#CE4A2F;font-weight:700;">{error_str}</pre></div></div>"#
        );
        bytes.extend(html_error.as_bytes());

        *req.body_mut() = Body::from(bytes);
    }

    Ok(req)
}

fn livereload_js() -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/javascript")
        .status(StatusCode::OK)
        .body(LIVE_RELOAD.into())
        .expect("Could not build livereload.js response")
}

fn in_memory_content(path: &RelativePathBuf, content: &str) -> Response<Body> {
    let content_type = match path.extension() {
        Some(ext) => match ext {
            "xml" => "text/xml",
            "json" => "application/json",
            _ => "text/html",
        },
        None => "text/html",
    };
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
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

fn io_error(err: std::io::Error) -> Response<Body> {
    match err.kind() {
        std::io::ErrorKind::NotFound => not_found(),
        std::io::ErrorKind::PermissionDenied => {
            Response::builder().status(StatusCode::FORBIDDEN).body(Body::empty()).unwrap()
        }
        _ => panic!("{}", err),
    }
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
            clear_serve_error();
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
        Err(e) => {
            let msg = "Failed to build the site";

            messages::unravel_errors(msg, &e);
            set_serve_error(msg, e);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn create_new_site(
    root_dir: &Path,
    interface: &str,
    interface_port: u16,
    output_dir: Option<&Path>,
    force: bool,
    base_url: &str,
    config_file: &Path,
    include_drafts: bool,
    no_port_append: bool,
    ws_port: Option<u16>,
) -> Result<(Site, String)> {
    SITE_CONTENT.write().unwrap().clear();

    let mut site = Site::new(root_dir, config_file)?;
    let address = format!("{}:{}", interface, interface_port);

    let base_url = if base_url == "/" {
        String::from("/")
    } else {
        let base_address = if no_port_append {
            base_url.to_string()
        } else {
            format!("{}:{}", base_url, interface_port)
        };

        if site.config.base_url.ends_with('/') {
            format!("http://{}/", base_address)
        } else {
            format!("http://{}", base_address)
        }
    };

    site.enable_serve_mode();
    site.set_base_url(base_url);
    if let Some(output_dir) = output_dir {
        if !force && output_dir.exists() {
            return Err(Error::msg(format!(
                "Directory '{}' already exists. Use --force to overwrite.",
                output_dir.display(),
            )));
        }
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
    messages::notify_site_size(&site);
    messages::warn_about_ignored_pages(&site);
    site.build()?;
    Ok((site, address))
}

#[allow(clippy::too_many_arguments)]
pub fn serve(
    root_dir: &Path,
    interface: &str,
    interface_port: u16,
    output_dir: Option<&Path>,
    force: bool,
    base_url: &str,
    config_file: &Path,
    open: bool,
    include_drafts: bool,
    fast_rebuild: bool,
    no_port_append: bool,
    utc_offset: UtcOffset,
) -> Result<()> {
    let start = Instant::now();
    let (mut site, address) = create_new_site(
        root_dir,
        interface,
        interface_port,
        output_dir,
        force,
        base_url,
        config_file,
        include_drafts,
        no_port_append,
        None,
    )?;
    messages::report_elapsed_time(start);

    // Stop right there if we can't bind to the address
    let bind_address: SocketAddrV4 = match address.parse() {
        Ok(a) => a,
        Err(_) => return Err(anyhow!("Invalid address: {}.", address)),
    };
    if (TcpListener::bind(bind_address)).is_err() {
        return Err(anyhow!("Cannot start server on address {}.", address));
    }

    let config_path = PathBuf::from(config_file);
    let config_path_rel = diff_paths(&config_path, root_dir).unwrap_or_else(|| config_path.clone());

    // An array of (path, WatchMode) where the path should be watched for changes,
    // and the WatchMode value indicates whether this file/folder must exist for
    // zola serve to operate
    let watch_this = vec![
        (config_path_rel.to_str().unwrap_or("config.toml"), WatchMode::Required),
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
                .with_context(|| format!("Can't watch `{}` for changes in folder `{}`. Does it exist, and do you have correct permissions?", entry, root_dir.display()))?;
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
                            response_error_injector(handle_request(req, static_root.clone()))
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
            .map_err(|_| anyhow!("Cannot bind to address {} for the websocket server. Maybe the port is already in use?", &ws_address))?;

        thread::spawn(move || {
            ws_server.run().unwrap();
        });

        broadcaster
    };

    println!(
        "Listening for changes in {}{}{{{}}}",
        root_dir.display(),
        MAIN_SEPARATOR,
        watchers.join(",")
    );

    let preserve_dotfiles_in_output = site.config.preserve_dotfiles_in_output;

    println!("Press Ctrl+C to stop\n");
    // Clean the output folder on ctrl+C
    ctrlc::set_handler(move || {
        match clean_site_output_folder(&output_path, preserve_dotfiles_in_output) {
            Ok(()) => (),
            Err(e) => println!("Errored while cleaning output folder: {}", e),
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
        // Do nothing if the file/dir is on the ignore list
        if let Some(gs) = &site.config.ignored_static_globset {
            if gs.is_match(partial_path) {
                return;
            }
        }
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
                copy_file(path, &site.output_path, &site.static_path, site.config.hard_link_static),
                &partial_path.to_string_lossy(),
            );
        }
    };

    let recreate_site = || match create_new_site(
        root_dir,
        interface,
        interface_port,
        output_dir,
        force,
        base_url,
        config_file,
        include_drafts,
        no_port_append,
        ws_port,
    ) {
        Ok((s, _)) => {
            clear_serve_error();
            rebuild_done_handling(&broadcaster, Ok(()), "/x.js");

            Some(s)
        }
        Err(e) => {
            let msg = "Failed to build the site";

            messages::unravel_errors(msg, &e);
            set_serve_error(msg, e);

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

                        if is_temp_file(&path) {
                            continue;
                        }

                        // We only care about changes in non-empty folders
                        if path.is_dir() && is_folder_empty(&path) {
                            continue;
                        }

                        let format =
                            format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
                        let current_time =
                            OffsetDateTime::now_utc().to_offset(utc_offset).format(&format);
                        if let Ok(time_str) = current_time {
                            println!("Change detected @ {}", time_str);
                        } else {
                            // if formatting fails for some reason
                            println!("Change detected");
                        };

                        let start = Instant::now();
                        match detect_change_kind(root_dir, &path, &config_path) {
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
                                            Err(anyhow!("dummy"))
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
                        messages::report_elapsed_time(start);
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

/// Detect what changed from the given path so we have an idea what needs
/// to be reloaded
fn detect_change_kind(pwd: &Path, path: &Path, config_path: &Path) -> (ChangeKind, PathBuf) {
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
    } else if path == config_path {
        ChangeKind::Config
    } else {
        unreachable!("Got a change in an unexpected path: {}", partial_path.display());
    };

    (change_kind, partial_path)
}

/// Check if the directory at path contains any file
fn is_folder_empty(dir: &Path) -> bool {
    // Can panic if we don't have the rights I guess?

    read_dir(dir).expect("Failed to read a directory to see if it was empty").next().is_none()
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
            Path::new(".index.md.kate-swp"),
        ];

        for t in test_cases {
            assert!(is_temp_file(t));
        }
    }

    #[test]
    fn can_detect_kind_of_changes() {
        let test_cases = vec![
            (
                (ChangeKind::Templates, PathBuf::from("/templates/hello.html")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/templates/hello.html"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::Themes, PathBuf::from("/themes/hello.html")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/themes/hello.html"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::StaticFiles, PathBuf::from("/static/site.css")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/static/site.css"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::Content, PathBuf::from("/content/posts/hello.md")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/content/posts/hello.md"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::Sass, PathBuf::from("/sass/print.scss")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/sass/print.scss"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::Config, PathBuf::from("/config.toml")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/config.toml"),
                Path::new("/home/vincent/site/config.toml"),
            ),
            (
                (ChangeKind::Config, PathBuf::from("/config.staging.toml")),
                Path::new("/home/vincent/site"),
                Path::new("/home/vincent/site/config.staging.toml"),
                Path::new("/home/vincent/site/config.staging.toml"),
            ),
        ];

        for (expected, pwd, path, config_filename) in test_cases {
            assert_eq!(expected, detect_change_kind(pwd, path, config_filename));
        }
    }

    #[test]
    #[cfg(windows)]
    fn windows_path_handling() {
        let expected = (ChangeKind::Templates, PathBuf::from("/templates/hello.html"));
        let pwd = Path::new(r#"C:\Users\johan\site"#);
        let path = Path::new(r#"C:\Users\johan\site\templates\hello.html"#);
        let config_filename = Path::new(r#"C:\Users\johan\site\config.toml"#);
        assert_eq!(expected, detect_change_kind(pwd, path, config_filename));
    }

    #[test]
    fn relative_path() {
        let expected = (ChangeKind::Templates, PathBuf::from("/templates/hello.html"));
        let pwd = Path::new("/home/johan/site");
        let path = Path::new("templates/hello.html");
        let config_filename = Path::new("config.toml");
        assert_eq!(expected, detect_change_kind(pwd, path, config_filename));
    }
}
