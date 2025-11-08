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
    http::{HeaderMap, Request, StatusCode, Uri, header},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use mime_guess::from_path as mimetype_from_path;
use time::macros::format_description;
use time::{OffsetDateTime, UtcOffset};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::Level;

use libs::percent_encoding;
use libs::relative_path::{RelativePath, RelativePathBuf};
use libs::serde_json;
use notify_debouncer_full::{new_debouncer, notify::RecursiveMode};

use config::Config;
use errors::{Context, Error, Result, anyhow};
use site::sass::compile_sass;
use site::{BuildMode, ContentData, SITE_CONTENT, Site};
use utils::fs::{clean_site_output_folder, copy_file, create_directory};

use crate::fs_utils::{ChangeKind, SimpleFileSystemEventKind, filter_events};
use crate::messages;
use std::ffi::OsStr;

#[derive(Debug, PartialEq)]
enum WatchMode {
    Required,
    Optional,
    Condition(bool),
}

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

#[derive(Clone)]
struct AppState {
    static_root: PathBuf,
    base_path: String,
    reload_tx: broadcast::Sender<String>,
    config: Arc<Config>,
}

async fn serve_file(
    State(state): State<Arc<AppState>>,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Response<Body>, StatusCode> {
    let path_str = uri.path();

    // Handle base path trimming
    if !path_str.starts_with(&state.base_path) {
        return Err(StatusCode::NOT_FOUND);
    }

    let trimmed_path = &path_str[state.base_path.len() - 1..];

    // Parse path
    let decoded = percent_encoding::percent_decode_str(trimmed_path)
        .decode_utf8()
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let decoded_path = if state.base_path != "/" && decoded.starts_with(&state.base_path) {
        decoded[state.base_path.len()..].to_string()
    } else {
        decoded.to_string()
    };

    let mut path = RelativePathBuf::new();
    for c in decoded_path.split('/') {
        path.push(c);
    }

    // Normalize empty path or directory paths to index.html
    // This ensures we look for index.html.br instead of .br
    let normalized_path = if path.as_str().is_empty() || path.as_str().ends_with('/') {
        let mut index_path = path.clone();
        index_path.push("index.html");
        index_path
    } else {
        path.clone()
    };

    // Try to serve pre-compressed variant if compression is enabled
    if let Some((compressed_path, encoding)) =
        find_compressed_variant(&normalized_path, &headers, &state.config).await
    {
        // Try in-memory first
        if let Some(content_data) = SITE_CONTENT.get(&compressed_path) {
            tracing::debug!(
                "Serving {} (compressed: {}, source: memory)",
                normalized_path.as_str(),
                encoding
            );
            return Ok(build_response(&normalized_path, content_data.value(), Some(encoding)));
        }

        // Try filesystem
        if let Ok(content) = serve_from_filesystem(&state.static_root, &compressed_path).await {
            tracing::debug!(
                "Serving {} (compressed: {}, source: filesystem)",
                compressed_path.as_str(),
                encoding
            );
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    mimetype_from_path(normalized_path.as_str())
                        .first_or_octet_stream()
                        .essence_str(),
                )
                .header(header::CONTENT_ENCODING, encoding)
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(content))
                .unwrap();
            return Ok(response);
        }
    }

    // Serve original file (not compressed)
    // Check SITE_CONTENT (memory) using normalized path
    if let Some(content_data) = SITE_CONTENT.get(&normalized_path) {
        tracing::debug!("Serving {} (source: memory)", normalized_path.as_str());
        return Ok(build_response(&normalized_path, content_data.value(), None));
    }

    // Fallback to filesystem
    // We need to check the filesystem with the same security checks as before
    let mut full_path = state.static_root.clone();
    full_path.push(&decoded[1..]);

    // Resolve and canonicalize to prevent path traversal
    let canonical_path = match tokio::fs::canonicalize(&full_path).await {
        Ok(p) => p,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    // Ensure we're still within the static root
    if !canonical_path.starts_with(&state.static_root) {
        return Err(StatusCode::NOT_FOUND);
    }

    // Check if it's a directory
    let metadata = match tokio::fs::metadata(&canonical_path).await {
        Ok(m) => m,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    let file_path =
        if metadata.is_dir() { canonical_path.join("index.html") } else { canonical_path };

    match tokio::fs::read(&file_path).await {
        Ok(contents) => {
            // Get relative path for logging
            let relative_path = file_path
                .strip_prefix(&state.static_root)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or_else(|| file_path.to_str().unwrap_or("unknown"));
            tracing::debug!("Serving {} (source: filesystem)", relative_path);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    mimetype_from_path(&file_path).first_or_octet_stream().essence_str(),
                )
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(contents))
                .unwrap())
        }
        Err(_) => serve_404(),
    }
}

async fn find_compressed_variant(
    path: &RelativePathBuf,
    headers: &HeaderMap,
    config: &Config,
) -> Option<(RelativePathBuf, &'static str)> {
    // Only if compression was enabled during build
    if !config.compress {
        return None;
    }

    let accept_encoding = headers.get(header::ACCEPT_ENCODING).and_then(|v| v.to_str().ok())?;

    // Try brotli first (better compression)
    if accept_encoding.contains("br") {
        // Create path like "index.html.br" by appending .br to the filename
        let br_path_str = format!("{}.br", path.as_str());
        let br_path = RelativePathBuf::from(br_path_str);

        if check_exists(&br_path).await {
            return Some((br_path, "br"));
        }
    }

    // Try gzip
    if accept_encoding.contains("gzip") {
        // Create path like "index.html.gz" by appending .gz to the filename
        let gz_path_str = format!("{}.gz", path.as_str());
        let gz_path = RelativePathBuf::from(gz_path_str);

        if check_exists(&gz_path).await {
            return Some((gz_path, "gzip"));
        }
    }

    None
}

async fn check_exists(path: &RelativePathBuf) -> bool {
    SITE_CONTENT.contains_key(path)
}

async fn serve_livereload_js() -> Html<&'static str> {
    Html(LIVE_RELOAD)
}

fn build_response(
    path: &RelativePathBuf,
    content_data: &ContentData,
    content_encoding: Option<&str>,
) -> Response<Body> {
    let mut builder =
        Response::builder().status(StatusCode::OK).header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");

    // Handle content type
    match content_data {
        ContentData::Text(_) => {
            let content_type = match path.extension() {
                Some(ext) => match ext {
                    "xml" => "text/xml",
                    "json" => "application/json",
                    "txt" => "text/plain",
                    _ => "text/html",
                },
                None => "text/html",
            };
            builder = builder.header(header::CONTENT_TYPE, content_type);
        }
        ContentData::Binary(_) => {
            let mime_type = mimetype_from_path(path.as_str()).first_or_octet_stream();
            builder = builder.header(header::CONTENT_TYPE, mime_type.essence_str());
        }
    }

    if let Some(encoding) = content_encoding {
        builder = builder.header(header::CONTENT_ENCODING, encoding);
    }

    let body = match content_data {
        ContentData::Text(s) => Body::from(s.clone()),
        ContentData::Binary(b) => Body::from(b.clone()),
    };

    builder.body(body).unwrap()
}

async fn serve_from_filesystem(
    static_root: &Path,
    path: &RelativePathBuf,
) -> Result<Vec<u8>, std::io::Error> {
    let full_path = static_root.join(path.as_str());
    tokio::fs::read(full_path).await
}

fn serve_404() -> Result<Response<Body>, StatusCode> {
    let not_found_path = RelativePath::new("404.html");

    if let Some(content_data) = SITE_CONTENT.get(not_found_path) {
        let mut response =
            build_response(&RelativePathBuf::from("404.html"), content_data.value(), None);
        *response.status_mut() = StatusCode::NOT_FOUND;
        return Ok(response);
    }

    Err(StatusCode::NOT_FOUND)
}

/// Inserts build error message boxes into HTML responses when needed.
async fn error_injection_middleware(request: Request<Body>, next: Next) -> Response {
    let response = next.run(request).await;

    // Only inject errors into HTML responses
    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.contains("text/html"))
        .unwrap_or(false);

    if !is_html {
        return response;
    }

    // Check if we have errors
    let has_error = SERVE_ERROR.lock().unwrap().get_mut().is_some();
    if !has_error {
        return response;
    }

    // Inject errors into HTML
    inject_errors_into_response(response).await
}

async fn inject_errors_into_response(response: Response) -> Response {
    let (parts, body) = response.into_parts();

    // Read the body
    let bytes_result = axum::body::to_bytes(body, usize::MAX).await;
    let mut bytes = match bytes_result {
        Ok(b) => b.to_vec(),
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };

    if let Some((msg, error)) = SERVE_ERROR.lock().unwrap().get_mut() {
        // Generate an error message similar to the CLI version
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

        // Push the error message (wrapped in an HTML dialog box) to the end of the HTML body
        let html_error = format!(
            r#"<div style="all:revert;position:fixed;display:flex;align-items:center;justify-content:center;background-color:rgb(0,0,0,0.5);top:0;right:0;bottom:0;left:0;"><div style="background-color:white;padding:0.5rem;border-radius:0.375rem;filter:drop-shadow(0,25px,25px,rgb(0,0,0/0.15));overflow-x:auto;"><p style="font-weight:700;color:black;font-size:1.25rem;margin:0;margin-bottom:0.5rem;">Zola Build Error:</p><pre style="padding:0.5rem;margin:0;border-radius:0.375rem;background-color:#363636;color:#CE4A2F;font-weight:700;">{error_str}</pre></div></div>"#
        );
        bytes.extend(html_error.as_bytes());
    }

    Response::from_parts(parts, Body::from(bytes))
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let reload_tx = state.reload_tx.clone();
    ws.on_upgrade(move |socket| handle_websocket(socket, reload_tx))
}

async fn handle_websocket(mut socket: WebSocket, reload_tx: broadcast::Sender<String>) {
    let mut rx = reload_tx.subscribe();

    loop {
        tokio::select! {
            // Send reload messages to client
            Ok(msg) = rx.recv() => {
                if (socket.send(Message::Text(msg)).await).is_err() {
                    break;
                }
            }
            // Handle incoming messages (livereload protocol)
            msg_result = socket.recv() => {
                match msg_result {
                    Some(Ok(msg)) => {
                        // Handle "hello" message from client
                        if let Message::Text(text) = msg
                            && text.contains("\"hello\"") {
                                let hello_response = r#"{
                            "command": "hello",
                            "protocols": [ "http://livereload.com/protocols/official-7" ],
                            "serverName": "Zola"
                        }"#;

                                if (socket.send(Message::Text(hello_response.to_string())).await).is_err() {
                                    break;
                                }
                            }
                    }
                    Some(Err(_)) | None => {
                        break;
                    }
                }
            }
        }
    }
}

fn create_router(
    static_root: PathBuf,
    base_path: String,
    reload_tx: broadcast::Sender<String>,
    config: Arc<Config>,
    verbose: bool,
) -> Router {
    let app_state = AppState { static_root, base_path, reload_tx, config };

    let mut app = Router::new()
        .route("/livereload.js", get(serve_livereload_js))
        .route("/livereload", get(ws_handler))
        .fallback(serve_file)
        .layer(CorsLayer::permissive());

    if verbose {
        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(|request: &Request<Body>| {
                let accept_encoding = request
                    .headers()
                    .get(header::ACCEPT_ENCODING)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("");

                tracing::info_span!(
                    "request",
                    method = %request.method(),
                    uri = %request.uri(),
                    accept_encoding = %accept_encoding,
                )
            })
            .on_response(
                |response: &Response, latency: std::time::Duration, _span: &tracing::Span| {
                    let content_encoding = response
                        .headers()
                        .get(header::CONTENT_ENCODING)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("none");

                    let content_type = response
                        .headers()
                        .get(header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("unknown");

                    tracing::info!(
                        status = %response.status(),
                        content_type = %content_type,
                        content_encoding = %content_encoding,
                        latency_ms = %latency.as_millis(),
                        "response"
                    );
                },
            );
        app = app.layer(trace_layer);
    }

    app.layer(middleware::from_fn(error_injection_middleware)).with_state(Arc::new(app_state))
}

fn rebuild_done_handling(
    broadcaster: &broadcast::Sender<String>,
    res: Result<()>,
    reload_path: &str,
) {
    match res {
        Ok(_) => {
            clear_serve_error();
            let reload_msg = format!(
                r#"{{
                    "command": "reload",
                    "path": {},
                    "originalPath": "",
                    "liveCSS": true,
                    "liveImg": true,
                    "protocol": ["http://livereload.com/protocols/official-7"]
                }}"#,
                serde_json::to_string(&reload_path).unwrap()
            );
            let _ = broadcaster.send(reload_msg);
        }
        Err(e) => {
            let msg = "Failed to build the site";
            messages::unravel_errors(msg, &e);
            set_serve_error(msg, e);
        }
    }
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
    ws_port: Option<u16>,
) -> Result<(Site, SocketAddr, String)> {
    SITE_CONTENT.clear();

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
    if let Some(p) = ws_port {
        site.enable_live_reload_with_port(p);
    } else {
        site.enable_live_reload(interface, interface_port);
    }
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
    verbose: bool,
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
        Some(interface_port), // WebSocket is on the same server as HTTP in Axum
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

    // An array of (path, WatchMode, RecursiveMode)
    let mut watch_this = vec![
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

    // static_root needs to be canonicalized
    let static_root = std::fs::canonicalize(&output_path).unwrap();

    // Create broadcast channel for WebSocket live reload
    let (reload_tx, _) = broadcast::channel::<String>(100);
    let reload_tx_clone = reload_tx.clone();

    // Setup tracing if verbose
    if verbose {
        tracing_subscriber::fmt().with_max_level(Level::DEBUG).init();
    }

    let app = create_router(
        static_root.clone(),
        base_path.clone(),
        reload_tx.clone(),
        Arc::new(site.config.clone()),
        verbose,
    );

    // Start Axum server in a separate thread
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Could not build tokio runtime");

        rt.block_on(async {
            let listener =
                tokio::net::TcpListener::bind(bind_address).await.expect("Cannot bind to address");

            let local_addr = listener.local_addr().expect("Could not get local address");

            println!(
                "Web server is available at {} (bound to {})\n",
                &constructed_base_url.replace(&bind_address.to_string(), &local_addr.to_string()),
                &local_addr
            );

            if open && let Err(err) = open::that(&constructed_base_url) {
                eprintln!("Failed to open URL in your browser: {err}");
            }

            axum::serve(listener, app).await.expect("Could not start web server");
        });
    });

    let broadcaster = reload_tx_clone;

    let config_name =
        config_path.file_name().unwrap().to_str().expect("Config name is not valid UTF-8.");
    let watch_list = watchers
        .iter()
        .map(|w| if w == root_dir_str { config_name } else { w })
        .collect::<Vec<&str>>()
        .join(",");
    println!("Listening for changes in {}{}{{{}}}", root_dir.display(), MAIN_SEPARATOR, watch_list);

    let preserve_dotfiles_in_output = site.config.preserve_dotfiles_in_output;

    println!("Press Ctrl+C to stop\n");
    // Clean the output folder on ctrl+C
    ctrlc::set_handler(move || {
        match clean_site_output_folder(&output_path, preserve_dotfiles_in_output) {
            Ok(()) => (),
            Err(e) => println!("Errored while cleaning output folder: {e}"),
        }
        ::std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let reload_sass = |site: &Site, paths: &Vec<&PathBuf>| {
        let combined_paths =
            paths.iter().map(|p| p.display().to_string()).collect::<Vec<String>>().join(", ");
        let msg = format!("-> Sass file(s) changed {combined_paths}");
        console::info(&msg);
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

    let ws_port = site.live_reload;
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
        ws_port,
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
                        println!("Change detected @ {time_str}");
                    } else {
                        println!("Change detected");
                    };

                    let start = Instant::now();
                    match change_kind {
                        ChangeKind::Content => {
                            for (_, full_path, event_kind) in change_group.iter() {
                                console::info(&format!(
                                    "-> Content changed {}",
                                    full_path.display()
                                ));

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
                                    } else if let Some(s) = recreate_site() {
                                        site = s;
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
                            let msg = format!("-> Template file(s) changed {combined_paths}");
                            console::info(&msg);

                            let shortcodes_updated = partial_paths
                                .iter()
                                .any(|p| p.starts_with("/templates/shortcodes"));
                            if shortcodes_updated {
                                if let Some(s) = recreate_site() {
                                    site = s;
                                }
                            } else {
                                println!("Reloading only template");
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
                            console::info("-> Themes changed.");

                            if let Some(s) = recreate_site() {
                                site = s;
                            }
                        }
                        ChangeKind::Config => {
                            console::info(
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
                            console::info(&format!(
                                "-> {combined_paths} changed. Recreating whole site."
                            ));

                            if let Some(s) = recreate_site() {
                                site = s;
                            }
                        }
                    };
                    messages::report_elapsed_time(start);
                }
            }
            Ok(Err(e)) => console::error(&format!("File system event errors: {e:?}")),
            Err(e) => console::error(&format!("File system event receiver errors: {e:?}")),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::{construct_url, create_new_site};
    use crate::get_config_file_path;
    use libs::url::Url;
    use std::net::{IpAddr, SocketAddr};
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

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
        ws_port: Option<u16>,
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
            ws_port,
        )
        .unwrap();

        assert_eq!(bind_address, SocketAddr::new(interface, interface_port));
        assert_eq!(constructed_base_url, expected_base_url);
        assert!(site.base_path.exists());
        assert_eq!(site.base_path, root_dir);
        assert_eq!(site.config.base_url, constructed_base_url);
        assert_ne!(site.live_reload, None);
        assert_ne!(site.live_reload, Some(1111));
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
    fn test_create_new_site_without_protocol_with_port_without_mounted_path() {
        let interface = IpAddr::from_str("127.0.0.1").unwrap();
        let interface_port = 1111;
        let output_dir: Option<PathBuf> = None;
        let base_url: Option<String> = None;
        let no_port_append = false;
        let ws_port: Option<u16> = None;
        let expected_base_url = String::from("http://127.0.0.1:1111");

        create_and_verify_new_site(
            interface,
            interface_port,
            output_dir.as_deref(),
            base_url.as_deref(),
            no_port_append,
            ws_port,
            expected_base_url,
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn test_create_new_site_without_protocol_with_port_with_mounted_path() {
        let interface = IpAddr::from_str("127.0.0.1").unwrap();
        let interface_port = 1111;
        let output_dir: Option<PathBuf> = None;
        let base_url: Option<String> = Some(String::from("localhost/path/to/site"));
        let no_port_append = false;
        let ws_port: Option<u16> = None;
        let expected_base_url = String::from("http://localhost:1111/path/to/site");

        create_and_verify_new_site(
            interface,
            interface_port,
            output_dir.as_deref(),
            base_url.as_deref(),
            no_port_append,
            ws_port,
            expected_base_url,
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn test_create_new_site_without_protocol_without_port_without_mounted_path() {
        let interface = IpAddr::from_str("127.0.0.1").unwrap();
        let interface_port = 1111;
        let output_dir: Option<PathBuf> = None;
        let base_url: Option<String> = Some(String::from("example.com"));
        let no_port_append = true;
        let ws_port: Option<u16> = None;
        let expected_base_url = String::from("http://example.com");

        create_and_verify_new_site(
            interface,
            interface_port,
            output_dir.as_deref(),
            base_url.as_deref(),
            no_port_append,
            ws_port,
            expected_base_url,
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn test_create_new_site_with_protocol_without_port_without_mounted_path() {
        let interface = IpAddr::from_str("127.0.0.1").unwrap();
        let interface_port = 1111;
        let output_dir: Option<PathBuf> = None;
        let base_url: Option<String> = Some(String::from("https://example.com"));
        let no_port_append = true;
        let ws_port: Option<u16> = None;
        let expected_base_url = String::from("https://example.com");

        create_and_verify_new_site(
            interface,
            interface_port,
            output_dir.as_deref(),
            base_url.as_deref(),
            no_port_append,
            ws_port,
            expected_base_url,
        );
    }

    #[test]
    fn test_create_new_site_with_protocol_without_port_with_mounted_path() {
        let interface = IpAddr::from_str("127.0.0.1").unwrap();
        let interface_port = 1111;
        let output_dir: Option<PathBuf> = None;
        let base_url: Option<String> = Some(String::from("https://example.com/path/to/site"));
        let no_port_append = true;
        let ws_port: Option<u16> = None;
        let expected_base_url = String::from("https://example.com/path/to/site");

        create_and_verify_new_site(
            interface,
            interface_port,
            output_dir.as_deref(),
            base_url.as_deref(),
            no_port_append,
            ws_port,
            expected_base_url,
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn test_create_new_site_with_protocol_with_port_with_mounted_path() {
        let interface = IpAddr::from_str("127.0.0.1").unwrap();
        let interface_port = 1111;
        let output_dir: Option<PathBuf> = None;
        let base_url: Option<String> = Some(String::from("https://example.com/path/to/site"));
        let no_port_append = false;
        let ws_port: Option<u16> = None;
        let expected_base_url = String::from("https://example.com:1111/path/to/site");

        create_and_verify_new_site(
            interface,
            interface_port,
            output_dir.as_deref(),
            base_url.as_deref(),
            no_port_append,
            ws_port,
            expected_base_url,
        );
    }
}
