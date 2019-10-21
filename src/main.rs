extern crate actix_files;
extern crate actix_web;
extern crate atty;
#[macro_use]
extern crate clap;
extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate ctrlc;
extern crate notify;
extern crate termcolor;
extern crate url;
extern crate ws;

extern crate site;
#[macro_use]
extern crate errors;
extern crate front_matter;
extern crate open;
extern crate rebuild;
extern crate utils;

use std::time::Instant;

use utils::net::{get_available_port, port_is_available};

mod cli;
mod cmd;
mod console;
mod prompt;

fn main() {
    let matches = cli::build_cli().get_matches();

    let config_file = matches.value_of("config").unwrap();

    match matches.subcommand() {
        ("init", Some(matches)) => {
            let name = matches.value_of("name");

            if name.is_none() {
                println!("You have to provide folder name as an argument to init.");
                ::std::process::exit(1);
            }

            let name = name.unwrap();

            match cmd::create_new_project(name) {
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
            let output_dir = matches.value_of("output_dir").unwrap();
            match cmd::build(
                config_file,
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
            let watch_only = matches.is_present("watch_only");
            let open = matches.is_present("open");
            let include_drafts = matches.is_present("drafts");

            // Default one
            if port != 1111 && !watch_only && !port_is_available(port) {
                console::error("The requested port is not available");
                ::std::process::exit(1);
            }

            if !watch_only && !port_is_available(port) {
                port = if let Some(p) = get_available_port(1111) {
                    p
                } else {
                    console::error("No port available.");
                    ::std::process::exit(1);
                }
            }
            let output_dir = matches.value_of("output_dir").unwrap();
            let base_url = matches.value_of("base_url").unwrap();
            console::info("Building site...");
            match cmd::serve(
                interface,
                port,
                output_dir,
                base_url,
                config_file,
                watch_only,
                open,
                include_drafts,
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
                config_file,
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
