use std::env;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use std::thread;

use iron::{Iron, Request, IronResult, Response, status};
use mount::Mount;
use staticfile::Static;
use notify::{Watcher, RecursiveMode, watcher};
use ws::{WebSocket};

use site::Site;
use errors::{Result};

const LIVE_RELOAD: &'static [u8; 37809] = include_bytes!("livereload.js");


fn livereload_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, String::from_utf8(LIVE_RELOAD.to_vec()).unwrap())))
}


// Most of it taken from mdbook
pub fn serve(interface: &str, port: &str) -> Result<()> {
    let mut site = Site::new(true)?;
    site.build()?;

    let address = format!("{}:{}", interface, port);
    let ws_address = format!("{}:{}", interface, "1112");

    // Start a webserver that serves the `public` directory
    let mut mount = Mount::new();
    mount.mount("/", Static::new(Path::new("public/")));
    mount.mount("/livereload.js", livereload_handler);
    let server = Iron::new(mount).http(address.clone()).unwrap();
    println!("Web server is available at http://{}", address);
    println!("Press CTRL+C to stop");

    // The websocket for livereload
    let ws_server = WebSocket::new(|_| {
        |_| {
            Ok(())
        }
    }).unwrap();
    let broadcaster = ws_server.broadcaster();
    thread::spawn(move || {
        ws_server.listen(&*ws_address).unwrap();
    });

    // And finally watching/reacting on file changes
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    watcher.watch("content/", RecursiveMode::Recursive).unwrap();
    watcher.watch("static/", RecursiveMode::Recursive).unwrap();
    watcher.watch("templates/", RecursiveMode::Recursive).unwrap();
    let pwd = env::current_dir().unwrap();
    println!("Listening for changes in {}/{{content, static, templates}}", pwd.display());

    use notify::DebouncedEvent::*;

    loop {
        // See https://github.com/spf13/hugo/blob/master/commands/hugo.go
        // for a more complete version of that
        match rx.recv() {
            Ok(event) => match event {
                NoticeWrite(path) |
                NoticeRemove(path) |
                Create(path) |
                Write(path) |
                Remove(path) |
                Rename(_, path) => {
                    if !is_temp_file(&path) {
                        println!("Change detected in {}", path.display());
                        match site.rebuild() {
                            Ok(_) => {
                                println!("Site rebuilt");
                                broadcaster.send(r#"
                                {
                                    "command": "reload",
                                    "path": "",
                                    "originalPath": "",
                                    "liveCSS": true,
                                    "liveImg": true,
                                    "protocol": ["http://livereload.com/protocols/official-7"]
                                }"#).unwrap();
                            },
                            Err(e) => {
                                println!("Failed to build the site");
                                println!("Error: {}", e);
                                for e in e.iter().skip(1) {
                                    println!("Reason: {}", e)
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Err(e) => println!("Watch error: {:?}", e),
        };
    }
}


/// Returns whether the path we received corresponds to a temp file create
/// by an editor
fn is_temp_file(path: &Path) -> bool {
    let ext = path.extension();
    match ext {
        Some(ex) => match ex.to_str().unwrap() {
            "swp" | "swx" | "tmp" | ".DS_STORE" => true,
            // jetbrains IDE
            x if x.ends_with("jb_old___") => true,
            x if x.ends_with("jb_tmp___") => true,
            x if x.ends_with("jb_bak___") => true,
            // byword
            x if x.starts_with("sb-") => true,
            // gnome
            x if x.starts_with(".gooutputstream") => true,
            _ => {
                if let Some(filename) = path.file_stem() {
                    // emacs
                    filename.to_str().unwrap().starts_with("#")
                } else {
                    false
                }
            }
        },
        None => false,
    }
}
