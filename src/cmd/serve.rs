use std::env;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Instant, Duration};
use std::thread;

use iron::{Iron, Request, IronResult, Response, status};
use mount::Mount;
use staticfile::Static;
use notify::{Watcher, RecursiveMode, watcher};
use ws::{WebSocket};
use gutenberg::Site;
use gutenberg::errors::{Result};


use ::time_elapsed;


#[derive(Debug, PartialEq)]
enum ChangeKind {
    Content,
    Templates,
    StaticFiles,
}

const LIVE_RELOAD: &'static str = include_str!("livereload.js");


fn livereload_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, LIVE_RELOAD.to_string())))
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
    // Starts with a _ to not trigger the unused lint
    // we need to assign to a variable otherwise it will block
    let _iron = Iron::new(mount).http(address.clone()).unwrap();
    println!("Web server is available at http://{}", address);

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
    let pwd = format!("{}", env::current_dir().unwrap().display());
    println!("Listening for changes in {}/{{content, static, templates}}", pwd);
    println!("Press CTRL+C to stop");

    use notify::DebouncedEvent::*;

    loop {
        // See https://github.com/spf13/hugo/blob/master/commands/hugo.go
        // for a more complete version of that
        match rx.recv() {
            Ok(event) => {
                match event {
                    Create(path) |
                    Write(path) |
                    Remove(path) |
                    Rename(_, path) => {
                        if is_temp_file(&path) {
                            continue;
                        }

                        println!("Change detected, rebuilding site");
                        let what_changed = detect_change_kind(&pwd, &path);
                        let mut reload_path = String::new();
                        match what_changed {
                            (ChangeKind::Content, _) => println!("Content changed {}", path.display()),
                            (ChangeKind::Templates, _) => println!("Template changed {}", path.display()),
                            (ChangeKind::StaticFiles, p) => {
                                reload_path = p;
                                println!("Static file changes detected {}", path.display());
                            },
                        };
                        println!("Reloading {}", reload_path);
                        let start = Instant::now();
                        match site.rebuild() {
                            Ok(_) => {
                                println!("Done in {:.1}s.\n", time_elapsed(start));
                                broadcaster.send(format!(r#"
                            {{
                                "command": "reload",
                                "path": "{}",
                                "originalPath": "",
                                "liveCSS": true,
                                "liveImg": true,
                                "protocol": ["http://livereload.com/protocols/official-7"]
                            }}"#, reload_path)).unwrap();
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
                    _ => {}
                }
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
            // vim
            x if x.ends_with("~") => true,
            _ => {
                if let Some(filename) = path.file_stem() {
                    // emacs
                    filename.to_str().unwrap().starts_with("#")
                } else {
                    false
                }
            }
        },
        None => {
            if path.ends_with(".DS_STORE") {
                true
            } else {
                false
            }
        },
    }
}


/// Detect what changed from the given path so we have an idea what needs
/// to be reloaded
fn detect_change_kind(pwd: &str, path: &Path) -> (ChangeKind, String) {
    let path_str = format!("{}", path.display())
        .replace(pwd, "")
        .replace("\\", "/");
    let change_kind = if path_str.starts_with("/templates") {
        ChangeKind::Templates
    } else if path_str.starts_with("/content") {
        ChangeKind::Content
    } else if path_str.starts_with("/static") {
        ChangeKind::StaticFiles
    } else {
        panic!("Got a change in an unexpected path: {}", path_str);
    };

    (change_kind, path_str)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{is_temp_file, detect_change_kind, ChangeKind};

    #[test]
    fn test_can_recognize_temp_files() {
        let testcases = vec![
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

        for t in testcases {
            println!("{:?}", t.display());
            assert!(is_temp_file(&t));
        }
    }

    #[test]
    fn test_can_detect_kind_of_changes() {
        let testcases = vec![
            (
                (ChangeKind::Templates, "/templates/hello.html".to_string()),
                "/home/vincent/site", Path::new("/home/vincent/site/templates/hello.html")
            ),
            (
                (ChangeKind::StaticFiles, "/static/site.css".to_string()),
                "/home/vincent/site", Path::new("/home/vincent/site/static/site.css")
            ),
            (
                (ChangeKind::Content, "/content/posts/hello.md".to_string()),
                "/home/vincent/site", Path::new("/home/vincent/site/content/posts/hello.md")
            ),
        ];

        for (expected, pwd, path) in testcases {
            println!("{:?}", path.display());
            assert_eq!(expected, detect_change_kind(&pwd, &path));
        }
    }


}
