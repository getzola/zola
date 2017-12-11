#[macro_use]
extern crate clap;
extern crate chrono;
extern crate term_painter;
extern crate staticfile;
extern crate iron;
extern crate mount;
extern crate notify;
extern crate url;
extern crate ws;

extern crate site;
#[macro_use]
extern crate errors;
extern crate content;
extern crate front_matter;
extern crate utils;

use std::time::Instant;

mod cmd;
mod console;
mod rebuild;
mod cli;
mod prompt;


fn main() {
    let matches = cli::build_cli().get_matches();

    let config_file = matches.value_of("config").unwrap_or("config.toml");

    match matches.subcommand() {
        ("init", Some(matches)) => {
            match cmd::create_new_project(matches.value_of("name").unwrap()) {
                Ok(()) => (),
                Err(e) => {
                    console::unravel_errors("Failed to create the project", &e);
                    ::std::process::exit(1);
                },
            };
        },
        ("build", Some(matches)) => {
            console::info("Building site...");
            let start = Instant::now();
            let output_dir = matches.value_of("output_dir").unwrap();
            match cmd::build(config_file, matches.value_of("base_url"), output_dir) {
                Ok(()) => console::report_elapsed_time(start),
                Err(e) => {
                    console::unravel_errors("Failed to build the site", &e);
                    ::std::process::exit(1);
                },
            };
        },
        ("serve", Some(matches)) => {
            let interface = matches.value_of("interface").unwrap();
            let port = matches.value_of("port").unwrap();
            let output_dir = matches.value_of("output_dir").unwrap();
            console::info("Building site...");
            match cmd::serve(interface, port, output_dir, config_file) {
                Ok(()) => (),
                Err(e) => {
                    console::unravel_errors("", &e);
                    ::std::process::exit(1);
                },
            };
        },
        _ => unreachable!(),
    }
}

