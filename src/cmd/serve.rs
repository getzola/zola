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
use std::ffi::OsStr;
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use axum::{
    Router,
    body::Body,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderValue, Method, StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::get,
};
use mime_guess::from_path as mimetype_from_path;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};
use tokio::sync::broadcast;

use log;
use notify_debouncer_full::{new_debouncer, notify::RecursiveMode};
use relative_path::{RelativePath, RelativePathBuf};

use errors::{Context, Error, Result, anyhow};
use serde_json::json;
use site::sass::compile_sass;
use site::{BuildMode, SITE_CONTENT, Site};
use utils::fs::{clean_site_output_folder, copy_file, create_directory};

use crate::fs_utils::{ChangeKind, SimpleFileSystemEventKind, filter_events};
use crate::messages;

#[derive(Debug, PartialEq)]
enum WatchMode {
    Required,
    Optional,
    Condition(bool),
}

const METHOD_NOT_ALLOWED_TEXT: &[u8] = b"Method Not Allowed";
const NOT_FOUND_TEXT: &[u8] = b"Not Found";

// This is dist/livereload.min.js from the LiveReload.js v3.2.4 release
const LIVE_RELOAD: &str = include_str!("livereload.js");

static SERVE_ERROR: Mutex<Cell<Option<(&'static str, Error)>>> = Mutex::new(Cell::new(None));

struct AppState {
    static_root: PathBuf,
    base_path: String,
    reload_tx: broadcast::Sender<String>,
}

fn clear_serve_error() {
    let _ = SERVE_ERROR.lock().map(|error| error.swap(&Cell::new(None)));
}

fn set_serve_error(msg: &'static str, e: Error) {
    if let Ok(serve_error) = SERVE_ERROR.lock() {
        serve_error.swap(&Cell::new(Some((msg, e))));
    }
}

/// Creates a LiveReload protocol reload message for the given path.
fn make_reload_message(path: &str) -> String {
    json!({
        "command": "reload",
        "path": path,
        "originalPath": "",
        "liveCSS": true,
        "liveImg": true,
        "protocol": ["http://livereload.com/protocols/official-7"]
    })
    .to_string()
}

async fn handle_request(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> Response {
    let path_str = req.uri().path();
    let base_path = &state.base_path;
    let mut root = state.static_root.clone();
    let original_root = root.clone();

    if !path_str.starts_with(base_path) {
        return not_found();
    }

    let trimmed_path = &path_str[base_path.len() - 1..];

    let mut path = RelativePathBuf::new();
    // https://zola.discourse.group/t/percent-encoding-for-slugs/736
    let decoded = match percent_encoding::percent_decode_str(trimmed_path).decode_utf8() {
        Ok(d) => d,
        Err(_) => return not_found(),
    };

    let decoded_path = if *base_path != "/" && decoded.starts_with(base_path) {
        // Remove the base_path from the request path before processing
        decoded[base_path.len()..].to_string()
    } else {
        decoded.to_string()
    };

    for c in decoded_path.split('/') {
        path.push(c);
    }

    if let Some(content) = SITE_CONTENT.read().unwrap().get(&path) {
        return in_memory_content(&path, content);
    }

    // Handle only `GET`/`HEAD` requests
    match *req.method() {
        Method::HEAD | Method::GET => {}
        _ => return method_not_allowed(),
    }

    // Handle only simple path requests
    if req.uri().scheme_str().is_some() || req.uri().host().is_some() {
        return not_found();
    }

    // Remove the first slash from the request path
    // otherwise `PathBuf` will interpret it as an absolute path
    root.push(&decoded[1..]);

    // Resolve the root + user supplied path into the absolute path
    // this should hopefully remove any path traversals
    // if we fail to resolve path, we should return 404
    root = match tokio::fs::canonicalize(&root).await {
        Ok(d) => d,
        Err(_) => return not_found(),
    };

    // Ensure we are only looking for things in our public folder
    if !root.starts_with(original_root) {
        return not_found();
    }

    let metadata = match tokio::fs::metadata(root.as_path()).await {
        Err(err) => return io_error(err),
        Ok(metadata) => metadata,
    };
    if metadata.is_dir() {
        // if root is a directory, append index.html to try to read that instead
        root.push("index.html");
    };

    let result = tokio::fs::read(&root).await;

    let contents = match result {
        Err(err) => return io_error(err),
        Ok(contents) => contents,
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            mimetype_from_path(&root).first_or_octet_stream().essence_str(),
        )
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(Body::from(contents))
        .unwrap()
}

/// WebSocket handler for live reload
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let reload_tx = state.reload_tx.clone();
    ws.on_upgrade(move |socket| handle_websocket(socket, reload_tx))
}

/// Handle WebSocket connection for live reload
async fn handle_websocket(mut socket: WebSocket, reload_tx: broadcast::Sender<String>) {
    let mut rx = reload_tx.subscribe();
    let mut ping_interval = tokio::time::interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            // Periodic ping to keep connection alive
            _ = ping_interval.tick() => {
                if socket.send(Message::Ping(vec![].into())).await.is_err() {
                    break;
                }
            }
            // Send reload messages to client
            Ok(msg) = rx.recv() => {
                if socket.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            // Handle incoming messages (livereload protocol)
            msg_result = socket.recv() => {
                match msg_result {
                    Some(Ok(Message::Text(text))) => {
                        // Handle "hello" message from client
                        if text.contains("\"hello\"") {
                            let hello_response = json!({
                                "command": "hello",
                                "protocols": ["http://livereload.com/protocols/official-7"],
                                "serverName": "Zola"
                            })
                            .to_string();

                            if socket.send(Message::Text(hello_response.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Pong(_))) => continue,
                    Some(Ok(_)) => {} // Ignore other message types
                    Some(Err(e)) => {
                        log::error!("WebSocket error: {e}");
                        break;
                    }
                    None => break,
                }
            }
        }
    }
}

/// Serve livereload.js
async fn serve_livereload_js() -> impl IntoResponse {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/javascript")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .status(StatusCode::OK)
        .body(Body::from(LIVE_RELOAD))
        .expect("Could not build livereload.js response")
}

/// Inserts build error message boxes into HTML responses when needed.
/// Used as axum middleware via `map_response`.
async fn error_injection_middleware(response: Response) -> Response {
    use axum::body::to_bytes;

    // Return response as-is if there are no error messages.
    let has_error = SERVE_ERROR.lock().unwrap().get_mut().is_some();
    if !has_error {
        return response;
    }

    // Only inject errors into HTML responses or 404 responses.
    // Don't interfere with WebSocket upgrades (101) or other special responses.
    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|val| val == HeaderValue::from_static("text/html"))
        .unwrap_or(false);
    let is_not_found = response.status() == StatusCode::NOT_FOUND;

    // Pass through non-HTML, non-404 responses unchanged (e.g., WebSocket upgrades)
    if !is_html && !is_not_found {
        return response;
    }

    let (parts, body) = response.into_parts();
    let bytes = match to_bytes(body, usize::MAX).await {
        Ok(b) => b.to_vec(),
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };

    if let Some((msg, error)) = SERVE_ERROR.lock().unwrap().get_mut() {
        // Generate an error message similar to the CLI version in messages::unravel_errors.
        let mut error_str = String::new();

        if !msg.is_empty() {
            error_str.push_str(&format!("Error: {msg}\n"));
        }

        error_str.push_str(&format!("Error: {error}\n"));

        let mut cause = error.source();
        while let Some(e) = cause {
            error_str.push_str(&format!("Reason: {e}\n"));
            cause = e.source();
        }

        let html_error = format!(
            r#"<div style="all:revert;position:fixed;display:flex;align-items:center;justify-content:center;background-color:rgb(0,0,0,0.5);top:0;right:0;bottom:0;left:0;"><div style="background-color:white;padding:0.5rem;border-radius:0.375rem;filter:drop-shadow(0,25px,25px,rgb(0,0,0/0.15));overflow-x:auto;"><p style="font-weight:700;color:black;font-size:1.25rem;margin:0;margin-bottom:0.5rem;">Zola Build Error:</p><pre style="padding:0.5rem;margin:0;border-radius:0.375rem;background-color:#363636;color:#CE4A2F;font-weight:700;">{error_str}</pre></div></div>"#
        );

        if is_html {
            // Inject error dialog into existing HTML response
            let mut new_bytes = bytes;
            new_bytes.extend(html_error.as_bytes());
            return Response::from_parts(parts, Body::from(new_bytes));
        } else if is_not_found {
            // Return a full HTML page with the error dialog for 404s
            // Include livereload.js so the page can receive reload messages when the error is fixed
            let html_page = format!(
                r#"<!DOCTYPE html><html><head><title>Zola Build Error</title><script src="/livereload.js"></script></head><body>{html_error}</body></html>"#
            );
            return Response::builder()
                .header(header::CONTENT_TYPE, "text/html")
                .status(StatusCode::OK)
                .body(Body::from(html_page))
                .expect("Could not build error response");
        }
    }

    Response::from_parts(parts, Body::from(bytes))
}

fn in_memory_content(path: &RelativePathBuf, content: &str) -> Response {
    let content_type = match path.extension() {
        Some(ext) => match ext {
            "xml" => "text/xml",
            "json" => "application/json",
            "txt" => "text/plain",
            _ => "text/html",
        },
        None => "text/html",
    };
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .status(StatusCode::OK)
        .body(Body::from(content.to_owned()))
        .expect("Could not build HTML response")
}

fn method_not_allowed() -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/plain")
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::from(METHOD_NOT_ALLOWED_TEXT))
        .expect("Could not build Method Not Allowed response")
}

fn io_error(err: std::io::Error) -> Response {
    match err.kind() {
        std::io::ErrorKind::NotFound => not_found(),
        std::io::ErrorKind::PermissionDenied => {
            Response::builder().status(StatusCode::FORBIDDEN).body(Body::empty()).unwrap()
        }
        _ => panic!("{}", err),
    }
}

fn not_found() -> Response {
    let not_found_path = RelativePath::new("404.html");
    let content = SITE_CONTENT.read().unwrap().get(not_found_path).cloned();

    if let Some(body) = content {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(body))
            .expect("Could not build Not Found response");
    }

    // Use a plain text response when we can't find the body of the 404
    Response::builder()
        .header(header::CONTENT_TYPE, "text/plain")
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(NOT_FOUND_TEXT))
        .expect("Could not build Not Found response")
}

fn rebuild_done_handling(
    broadcaster: &broadcast::Sender<String>,
    res: Result<()>,
    reload_path: &str,
) {
    match res {
        Ok(_) => {
            clear_serve_error();
        }
        Err(e) => {
            let msg = "Failed to build the site";
            messages::unravel_errors(msg, &e);
            set_serve_error(msg, e);
        }
    }

    // Always send reload so the client fetches the page (with error dialog if needed)
    let _ = broadcaster.send(make_reload_message(reload_path));
}

fn construct_url(base_url: &str, no_port_append: bool, interface_port: u16) -> String {
    if base_url == "/" {
        return String::from("/");
    }

    let (protocol, stripped_url) = match base_url {
        url if url.starts_with("http://") => ("http://", &url[7..]),
        url if url.starts_with("https://") => ("https://", &url[8..]),
        url => ("http://", url),
    };

    let (domain, path) = {
        let parts: Vec<&str> = stripped_url.splitn(2, '/').collect();
        if parts.len() > 1 {
            (parts[0], format!("/{}", parts[1]))
        } else {
            (parts[0], String::new())
        }
    };

    let full_address = if no_port_append {
        format!("{protocol}{domain}{path}")
    } else {
        format!("{protocol}{domain}:{interface_port}{path}")
    };

    if full_address.ends_with('/') { full_address } else { format!("{full_address}/") }
}

#[allow(clippy::too_many_arguments)]
fn create_new_site(
    root_dir: &Path,
    interface: IpAddr,
    interface_port: u16,
    output_dir: Option<&Path>,
    force: bool,
    base_url: Option<&str>,
    config_file: &Path,
    include_drafts: bool,
    store_html: bool,
    mut no_port_append: bool,
) -> Result<(Site, SocketAddr, String)> {
    SITE_CONTENT.write().unwrap().clear();

    let mut site = Site::new(root_dir, config_file)?;
    let address = SocketAddr::new(interface, interface_port);

    // if no base URL provided, use socket address
    let base_url = base_url.map_or_else(
        || {
            no_port_append = true;
            address.to_string()
        },
        |u| u.to_string(),
    );

    let mut constructed_base_url = construct_url(&base_url, no_port_append, interface_port);

    if !site.config.base_url.ends_with('/') && constructed_base_url != "/" {
        constructed_base_url.truncate(constructed_base_url.len() - 1);
    }

    site.enable_serve_mode(if store_html { BuildMode::Both } else { BuildMode::Memory });
    site.set_base_url(constructed_base_url.clone());
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
    // With Axum, WebSocket runs on the same server as HTTP
    site.enable_live_reload_with_port(interface_port);
    messages::notify_site_size(&site);
    messages::warn_about_ignored_pages(&site);
    site.build()?;
    Ok((site, address, constructed_base_url))
}

#[allow(clippy::too_many_arguments)]
pub fn serve(
    root_dir: &Path,
    interface: IpAddr,
    interface_port: u16,
    output_dir: Option<&Path>,
    force: bool,
    base_url: Option<&str>,
    config_file: &Path,
    open: bool,
    include_drafts: bool,
    store_html: bool,
    fast_rebuild: bool,
    no_port_append: bool,
    utc_offset: UtcOffset,
    extra_watch_paths: Vec<String>,
    debounce: u64,
) -> Result<()> {
    let start = Instant::now();
    let (mut site, bind_address, constructed_base_url) = create_new_site(
        root_dir,
        interface,
        interface_port,
        output_dir,
        force,
        base_url,
        config_file,
        include_drafts,
        store_html,
        no_port_append,
    )?;
    let base_path = match constructed_base_url.splitn(4, '/').nth(3) {
        Some(path) => format!("/{path}"),
        None => "/".to_string(),
    };

    messages::report_elapsed_time(start);

    // Stop right there if we can't bind to the address
    if (TcpListener::bind(bind_address)).is_err() {
        return Err(anyhow!("Cannot start server on address {}.", bind_address));
    }

    let config_path = PathBuf::from(config_file);
    let root_dir_str = root_dir.to_str().expect("Project root dir is not valid UTF-8.");

    // An array of (path, WatchMode, RecursiveMode) where the path is watched for changes,
    // the WatchMode value indicates whether this path must exist for zola serve to operate,
    // and the RecursiveMode value indicates whether to watch nested directories.
    let mut watch_this = vec![
        // The first entry is ultimately to watch config.toml in a more robust manner on Linux when
        // the file changes by way of a caching strategy used by editors such as vim.
        // https://github.com/getzola/zola/issues/2266
        (root_dir_str, WatchMode::Required, RecursiveMode::NonRecursive),
        ("content", WatchMode::Required, RecursiveMode::Recursive),
        ("sass", WatchMode::Condition(site.config.compile_sass), RecursiveMode::Recursive),
        ("static", WatchMode::Optional, RecursiveMode::Recursive),
        ("templates", WatchMode::Optional, RecursiveMode::Recursive),
        ("themes", WatchMode::Condition(site.config.theme.is_some()), RecursiveMode::Recursive),
    ];
    watch_this.extend(
        extra_watch_paths
            .iter()
            .map(|path| (path.as_str(), WatchMode::Required, RecursiveMode::Recursive)),
    );

    // Setup watchers
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_millis(debounce), None, tx).unwrap();

    // We watch for changes on the filesystem for every entry in watch_this
    // Will fail if either:
    //   - the path is mandatory but does not exist (eg. config.toml)
    //   - the path exists but has incorrect permissions
    // watchers will contain the paths we're actually watching
    let mut watchers = Vec::new();
    for (entry, watch_mode, recursive_mode) in watch_this {
        let watch_path = root_dir.join(entry);
        let should_watch = match watch_mode {
            WatchMode::Required => true,
            WatchMode::Optional => watch_path.exists(),
            WatchMode::Condition(b) => b && watch_path.exists(),
        };
        if should_watch {
            debouncer
                .watch(root_dir.join(entry), recursive_mode)
                .with_context(|| format!("Can't watch `{}` for changes in folder `{}`. Does it exist, and do you have correct permissions?", entry, root_dir.display()))?;
            watchers.push(entry.to_string());
        }
    }

    let output_path = site.output_path.clone();
    create_directory(&output_path)?;

    // static_root needs to be canonicalized because we do the same for the http server.
    let static_root = std::fs::canonicalize(&output_path).unwrap();

    // Create broadcast channel for WebSocket live reload
    let (reload_tx, _) = broadcast::channel::<String>(100);
    let broadcaster = reload_tx.clone();

    // Start Axum server in a separate thread
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Could not build tokio runtime");

        rt.block_on(async {
            let state = Arc::new(AppState { static_root, base_path, reload_tx });

            let app = Router::new()
                .route("/livereload.js", get(serve_livereload_js))
                .route("/livereload", get(ws_handler))
                .fallback(handle_request)
                .layer(middleware::map_response(error_injection_middleware))
                .with_state(state);

            let listener = tokio::net::TcpListener::bind(&bind_address)
                .await
                .expect("Could not bind to address");

            let local_addr = listener.local_addr().unwrap();

            log::info!(
                "Web server is available at {} (bound to {})\n",
                &constructed_base_url.replace(&bind_address.to_string(), &local_addr.to_string()),
                &local_addr
            );
            if open && let Err(err) = open::that(&constructed_base_url) {
                log::error!("Failed to open URL in your browser: {err}");
            }

            axum::serve(listener, app).await.expect("Could not start web server");
        });
    });

    // We watch for changes in the config by monitoring its parent directory, but we ignore all
    // ordinary peer files. Map the parent directory back to the config file name to not confuse
    // the end user.
    let config_name =
        config_path.file_name().unwrap().to_str().expect("Config name is not valid UTF-8.");
    let watch_list = watchers
        .iter()
        .map(|w| if w == root_dir_str { config_name } else { w })
        .collect::<Vec<&str>>()
        .join(",");
    log::info!(
        "Listening for changes in {}{}{{{}}}",
        root_dir.display(),
        MAIN_SEPARATOR,
        watch_list
    );

    let preserve_dotfiles_in_output = site.config.preserve_dotfiles_in_output;

    log::info!("Press Ctrl+C to stop\n");
    // Clean the output folder on ctrl+C
    ctrlc::set_handler(move || {
        match clean_site_output_folder(&output_path, preserve_dotfiles_in_output) {
            Ok(()) => (),
            Err(e) => log::error!("Errored while cleaning output folder: {e}"),
        }
        ::std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let reload_sass = |site: &Site, paths: &Vec<&PathBuf>| {
        let combined_paths =
            paths.iter().map(|p| p.display().to_string()).collect::<Vec<String>>().join(", ");
        log::info!("Sass file(s) changed {combined_paths}");
        rebuild_done_handling(
            &broadcaster,
            compile_sass(&site.base_path, &site.output_path),
            &site.sass_path.to_string_lossy(),
        );
    };

    let reload_templates = |site: &mut Site| {
        rebuild_done_handling(
            &broadcaster,
            site.reload_templates(),
            &site.templates_path.to_string_lossy(),
        );
    };

    let copy_static = |site: &Site, path: &Path, partial_path: &Path| {
        // Do nothing if the file/dir is on the ignore list
        if let Some(gs) = &site.config.ignored_static_globset
            && gs.is_match(partial_path)
        {
            return;
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

        log::info!("{msg}");
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
        store_html,
        no_port_append,
    ) {
        Ok((s, _, _)) => {
            clear_serve_error();
            rebuild_done_handling(&broadcaster, Ok(()), "/x.js");

            Some(s)
        }
        Err(e) => {
            let msg = "Failed to build the site";

            messages::unravel_errors(msg, &e);
            set_serve_error(msg, e);

            // Send reload so the client fetches the page with the error dialog
            let _ = broadcaster.send(make_reload_message("/x.js"));

            None
        }
    };

    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                let changes = filter_events(
                    events,
                    root_dir,
                    &config_path,
                    &site.config.ignored_content_globset,
                );
                if changes.is_empty() {
                    continue;
                }
                let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");

                for (change_kind, change_group) in changes.iter() {
                    let current_time =
                        OffsetDateTime::now_utc().to_offset(utc_offset).format(&format);
                    if let Ok(time_str) = current_time {
                        log::info!("Change detected @ {time_str}");
                    } else {
                        // if formatting fails for some reason
                        log::info!("Change detected");
                    };

                    let start = Instant::now();
                    match change_kind {
                        ChangeKind::Content => {
                            for (_, full_path, event_kind) in change_group.iter() {
                                log::info!("-> Content changed {}", full_path.display());

                                let can_do_fast_reload =
                                    *event_kind != SimpleFileSystemEventKind::Remove;

                                if fast_rebuild {
                                    if can_do_fast_reload {
                                        let filename = full_path
                                            .file_name()
                                            .unwrap_or_else(|| OsStr::new(""))
                                            .to_string_lossy();
                                        let res = if filename == "_index.md" {
                                            site.add_and_render_section(full_path)
                                        } else if filename.ends_with(".md") {
                                            site.add_and_render_page(full_path)
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
                                                &full_path.to_string_lossy(),
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
                        }
                        ChangeKind::Templates => {
                            let partial_paths: Vec<&PathBuf> =
                                change_group.iter().map(|(p, _, _)| p).collect();
                            let full_paths: Vec<&PathBuf> =
                                change_group.iter().map(|(_, p, _)| p).collect();
                            let combined_paths = full_paths
                                .iter()
                                .map(|p| p.display().to_string())
                                .collect::<Vec<String>>()
                                .join(", ");
                            log::info!("-> Template file(s) changed {combined_paths}");

                            let shortcodes_updated = partial_paths
                                .iter()
                                .any(|p| p.starts_with("/templates/shortcodes"));
                            // Rebuild site if shortcodes change; otherwise, just update template.
                            if shortcodes_updated {
                                if let Some(s) = recreate_site() {
                                    site = s;
                                }
                            } else {
                                log::info!("Reloading only template");
                                reload_templates(&mut site)
                            }
                        }
                        ChangeKind::StaticFiles => {
                            for (partial_path, full_path, _) in change_group.iter() {
                                copy_static(&site, full_path, partial_path);
                            }
                        }
                        ChangeKind::Sass => {
                            let full_paths = change_group.iter().map(|(_, p, _)| p).collect();
                            reload_sass(&site, &full_paths);
                        }
                        ChangeKind::Themes => {
                            // No need to iterate over change group since we're rebuilding the site.
                            log::info!("-> Themes changed.");

                            if let Some(s) = recreate_site() {
                                site = s;
                            }
                        }
                        ChangeKind::Config => {
                            // No need to iterate over change group since we're rebuilding the site.
                            log::info!(
                                "-> Config changed. The browser needs to be refreshed to make the changes visible.",
                            );

                            if let Some(s) = recreate_site() {
                                site = s;
                            }
                        }
                        ChangeKind::ExtraPath => {
                            let full_paths: Vec<&PathBuf> =
                                change_group.iter().map(|(_, p, _)| p).collect();
                            let combined_paths = full_paths
                                .iter()
                                .map(|p| p.display().to_string())
                                .collect::<Vec<String>>()
                                .join(", ");
                            log::info!("-> {combined_paths} changed. Recreating whole site.");

                            // We can't know exactly what to update when a user provides the path.
                            if let Some(s) = recreate_site() {
                                site = s;
                            }
                        }
                    };
                    messages::report_elapsed_time(start);
                }
            }
            Ok(Err(e)) => log::error!("File system event errors: {e:?}"),
            Err(e) => log::error!("File system event receiver errors: {e:?}"),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{construct_url, create_new_site};
    use crate::get_config_file_path;
    use std::net::{IpAddr, SocketAddr};
    use std::path::Path;
    use std::str::FromStr;
    use url::Url;

    #[test]
    fn test_construct_url_base_url_is_slash() {
        let result = construct_url("/", false, 8080);
        assert_eq!(result, "/");
    }

    #[test]
    fn test_construct_url_http_protocol() {
        let result = construct_url("http://example.com", false, 8080);
        assert_eq!(result, "http://example.com:8080/");
    }

    #[test]
    fn test_construct_url_https_protocol() {
        let result = construct_url("https://example.com", false, 8080);
        assert_eq!(result, "https://example.com:8080/");
    }

    #[test]
    fn test_construct_url_no_protocol() {
        let result = construct_url("example.com", false, 8080);
        assert_eq!(result, "http://example.com:8080/");
    }

    #[test]
    fn test_construct_url_no_port_append() {
        let result = construct_url("https://example.com", true, 8080);
        assert_eq!(result, "https://example.com/");
    }

    #[test]
    fn test_construct_url_trailing_slash() {
        let result = construct_url("http://example.com/", false, 8080);
        assert_eq!(result, "http://example.com:8080/");
    }

    fn create_and_verify_new_site(
        interface: IpAddr,
        interface_port: u16,
        output_dir: Option<&Path>,
        base_url: Option<&str>,
        no_port_append: bool,
        expected_base_url: String,
    ) {
        let cli_dir = Path::new("./test_site").canonicalize().unwrap();
        let cli_config = Path::new("./test_site/config.toml").canonicalize().unwrap();

        let (root_dir, config_file) = get_config_file_path(&cli_dir, &cli_config);
        assert_eq!(cli_dir, root_dir);
        assert_eq!(cli_config, root_dir.join("config.toml"));

        let force = false;
        let include_drafts = false;

        let (site, bind_address, constructed_base_url) = create_new_site(
            &root_dir,
            interface,
            interface_port,
            output_dir,
            force,
            base_url,
            &config_file,
            include_drafts,
            false,
            no_port_append,
        )
        .unwrap();

        assert_eq!(bind_address, SocketAddr::new(interface, interface_port));
        assert_eq!(constructed_base_url, expected_base_url);
        assert!(site.base_path.exists());
        assert_eq!(site.base_path, root_dir);
        assert_eq!(site.config.base_url, constructed_base_url);
        // With Axum, WebSocket runs on the same port as HTTP
        assert_eq!(site.live_reload, Some(interface_port));
        assert_eq!(site.output_path, root_dir.join(&site.config.output_dir));
        assert_eq!(site.static_path, root_dir.join("static"));

        let base_url = Url::parse(&expected_base_url).unwrap();
        for (_, permalink) in site.permalinks {
            let permalink_url = Url::parse(&permalink).unwrap();
            assert_eq!(base_url.scheme(), permalink_url.scheme());
            assert_eq!(base_url.host(), permalink_url.host());
            assert_eq!(base_url.port(), permalink_url.port());
            assert!(!permalink_url.path().starts_with("//"));
            assert!(!permalink_url.path().ends_with("//"));
            assert!(permalink_url.path().starts_with("/"));
            assert!(permalink_url.path().starts_with(base_url.path()));
        }
    }

    #[test]
    #[cfg(not(windows))]
    fn test_create_new_site() {
        let interface = IpAddr::from_str("127.0.0.1").unwrap();
        let interface_port = 1111;

        // without_protocol_with_port_without_mounted_path
        create_and_verify_new_site(
            interface,
            interface_port,
            None,
            None,
            false,
            String::from("http://127.0.0.1:1111"),
        );

        // without_protocol_with_port_with_mounted_path
        create_and_verify_new_site(
            interface,
            interface_port,
            None,
            Some("localhost/path/to/site"),
            false,
            String::from("http://localhost:1111/path/to/site"),
        );

        // without_protocol_without_port_without_mounted_path
        // Note: no_port_append only works if we define a base_url
        create_and_verify_new_site(
            interface,
            interface_port,
            None,
            Some("example.com"),
            true,
            String::from("http://example.com"),
        );

        // with_protocol_without_port_without_mounted_path
        create_and_verify_new_site(
            interface,
            interface_port,
            None,
            Some("https://example.com"),
            true,
            String::from("https://example.com"),
        );

        // with_protocol_without_port_with_mounted_path
        create_and_verify_new_site(
            interface,
            interface_port,
            None,
            Some("https://example.com/path/to/site"),
            true,
            String::from("https://example.com/path/to/site"),
        );

        // with_protocol_with_port_with_mounted_path
        create_and_verify_new_site(
            interface,
            interface_port,
            None,
            Some("https://example.com/path/to/site"),
            false,
            String::from("https://example.com:1111/path/to/site"),
        );
    }
}
