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
use std::fs::remove_dir_all;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::{Instant, Duration};
use std::thread;

use chrono::prelude::*;
use iron::{Iron, Request, IronResult, Response, status};
use mount::Mount;
use staticfile::Static;
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


fn livereload_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, LIVE_RELOAD.to_string())))
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
    let mut site = Site::new(env::current_dir().unwrap(), config_file)?;

    let base_address = format!("{}:{}", base_url, port);
    let address = format!("{}:{}", interface, port);

    site.config.base_url = if site.config.base_url.ends_with('/') {
        format!("http://{}/", base_address)
    } else {
        format!("http://{}", base_address)
    };

    site.set_output_path(output_dir);
    site.load()?;
    site.enable_live_reload();
    console::notify_site_size(&site);
    console::warn_about_ignored_pages(&site);
    site.build()?;
    Ok((site, address))
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
    watcher.watch("config.toml", RecursiveMode::Recursive)
        .chain_err(|| "Can't watch the `config.toml` file. Does it exist?")?;

    if Path::new("static").exists() {
        watching_static = true;
        watcher.watch("static/", RecursiveMode::Recursive)
            .chain_err(|| "Can't watch the `static` folder. Does it exist?")?;
    }

    // Sass support is optional so don't make it an error to no have a sass folder
    let _ = watcher.watch("sass/", RecursiveMode::Recursive);

    let ws_address = format!("{}:{}", interface, "1112");

    // Start a webserver that serves the `output_dir` directory
    let mut mount = Mount::new();
    mount.mount("/", Static::new(Path::new(output_dir)));
    mount.mount("/livereload.js", livereload_handler);
    // Starts with a _ to not trigger the unused lint
    // we need to assign to a variable otherwise it will block
    let _iron = Iron::new(mount).http(address.as_str())
        .chain_err(|| "Can't start the webserver")?;

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

    let pwd = format!("{}", env::current_dir().unwrap().display());

    let mut watchers = vec!["content", "templates", "config.toml"];
    if watching_static {
        watchers.push("static");
    }
    if site.config.compile_sass {
        watchers.push("sass");
    }

    println!("Listening for changes in {}/{{{}}}", pwd, watchers.join(", "));
    println!("Web server is available at http://{}", address);
    println!("Press Ctrl+C to stop\n");
    // Delete the output folder on ctrl+C
    let output_path = Path::new(output_dir).to_path_buf();
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
                                    rebuild_done_handling(&broadcaster, copy_file(&path, &site.output_path, &site.static_path), &p);
                                }
                            },
                            (ChangeKind::Sass, p) => {
                                console::info(&format!("-> Sass file changed {}", path.display()));
                                rebuild_done_handling(&broadcaster, site.compile_sass(&site.base_path), &p);
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
                    filename.to_str().unwrap().starts_with('#')
                } else {
                    false
                }
            }
        },
        None => {
            path.ends_with(".DS_STORE")
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
    } else if path_str.starts_with("/sass") {
        ChangeKind::Sass
    } else if path_str == "/config.toml" {
        ChangeKind::Config
    } else {
        unreachable!("Got a change in an unexpected path: {}", path_str)
    };

    (change_kind, path_str)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

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
            (
                (ChangeKind::Sass, "/sass/print.scss".to_string()),
                "/home/vincent/site", Path::new("/home/vincent/site/sass/print.scss")
            ),
            (
                (ChangeKind::Config, "/config.toml".to_string()),
                "/home/vincent/site", Path::new("/home/vincent/site/config.toml")
            ),
        ];

        for (expected, pwd, path) in test_cases {
            assert_eq!(expected, detect_change_kind(&pwd, &path));
        }
    }


}
