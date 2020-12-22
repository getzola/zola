use std::env;
use std::path::{Path, PathBuf};
use std::time::Instant;

use utils::net::{get_available_port, port_is_available};

mod cli;
mod cmd;
mod console;
mod prompt;

fn main() {
    let matches = cli::build_cli().get_matches();

    let root_dir = match matches.value_of("root").unwrap() {
        "." => env::current_dir().unwrap(),
        path => PathBuf::from(path)
            .canonicalize()
            .unwrap_or_else(|_| panic!("Cannot find root directory: {}", path)),
    };
    let config_file = match matches.value_of("config") {
        Some(path) => PathBuf::from(path),
        None => root_dir.join("config.toml"),
    };

    match matches.subcommand() {
        ("init", Some(matches)) => {
            let force = matches.is_present("force");
            match cmd::create_new_project(matches.value_of("name").unwrap(), force) {
                Ok(()) => (),
                Err(e) => {
                    console::unravel_errors("Failed to create the project", &e);
                    ::std::process::exit(1);
                }
            };
        }
        ("build", Some(matches)) => {
            console::info("Building site...");
            let start = Instant::now();
            let output_dir = matches.value_of("output_dir").map(|output_dir| Path::new(output_dir));
            match cmd::build(
                &root_dir,
                &config_file,
                matches.value_of("base_url"),
                output_dir,
                matches.is_present("drafts"),
            ) {
                Ok(()) => console::report_elapsed_time(start),
                Err(e) => {
                    console::unravel_errors("Failed to build the site", &e);
                    ::std::process::exit(1);
                }
            };
        }
        ("serve", Some(matches)) => {
            let interface = matches.value_of("interface").unwrap_or("127.0.0.1");
            let mut port: u16 = match matches.value_of("port").unwrap().parse() {
                Ok(x) => x,
                Err(_) => {
                    console::error("The request port needs to be an integer");
                    ::std::process::exit(1);
                }
            };
            let open = matches.is_present("open");
            let include_drafts = matches.is_present("drafts");
            let fast = matches.is_present("fast");

            // Default one
            if port != 1111 && !port_is_available(port) {
                console::error("The requested port is not available");
                ::std::process::exit(1);
            }

            if !port_is_available(port) {
                port = if let Some(p) = get_available_port(1111) {
                    p
                } else {
                    console::error("No port available.");
                    ::std::process::exit(1);
                }
            }
            let output_dir = matches.value_of("output_dir").map(|output_dir| Path::new(output_dir));
            let base_url = matches.value_of("base_url").unwrap();
            console::info("Building site...");
            match cmd::serve(
                &root_dir,
                interface,
                port,
                output_dir,
                base_url,
                &config_file,
                open,
                include_drafts,
                fast,
            ) {
                Ok(()) => (),
                Err(e) => {
                    console::unravel_errors("", &e);
                    ::std::process::exit(1);
                }
            };
        }
        ("check", Some(matches)) => {
            console::info("Checking site...");
            let start = Instant::now();
            match cmd::check(
                &root_dir,
                &config_file,
                matches.value_of("base_path"),
                matches.value_of("base_url"),
                matches.is_present("drafts"),
            ) {
                Ok(()) => console::report_elapsed_time(start),
                Err(e) => {
                    console::unravel_errors("Failed to check the site", &e);
                    ::std::process::exit(1);
                }
            };
        }
        _ => unreachable!(),
    }
}
