#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate gutenberg;
extern crate chrono;
extern crate term_painter;

extern crate staticfile;
extern crate iron;
extern crate mount;
extern crate notify;
extern crate ws;


use std::time::Instant;

use chrono::Duration;
use gutenberg::errors::Error;

mod cmd;
mod console;


/// Print the time elapsed rounded to 1 decimal
fn report_elapsed_time(instant: Instant) {
    let duration_ms = Duration::from_std(instant.elapsed()).unwrap().num_milliseconds() as f64;

    if duration_ms < 1000.0 {
        console::success(&format!("Done in {}ms.\n", duration_ms));
    } else {
        let duration_sec = duration_ms / 1000.0;
        console::success(&format!("Done in {:.1}s.\n", ((duration_sec * 10.0).round() / 10.0)));
    }
}

////Display an error message, the actual error and then exits if requested
fn unravel_errors(message: &str, error: &Error, exit: bool) {
        console::error(message);
        console::error(&format!("Error: {}", error));
        for e in error.iter().skip(1) {
            console::error(&format!("Reason: {}", e));
        }
        if exit {
            ::std::process::exit(1);
        }
}


fn main() {
    let matches = clap_app!(Gutenberg =>
        (version: crate_version!())
        (author: "Vincent Prouillet")
        (about: "Static site generator")
        (@setting SubcommandRequiredElseHelp)
        (@arg config: -c --config +takes_value "Path to a config file other than config.toml")
        (@subcommand init =>
            (about: "Create a new Gutenberg project")
            (@arg name: +required "Name of the project. Will create a directory with that name in the current directory")
        )
        (@subcommand build =>
            (about: "Builds the site")
        )
        (@subcommand serve =>
            (about: "Serve the site. Rebuild and reload on change automatically")
            (@arg interface: "Interface to bind on (default to 127.0.0.1)")
            (@arg port: "Which port to use (default to 1111)")
        )
    ).get_matches();

    let config_file = matches.value_of("config").unwrap_or("config.toml");

    match matches.subcommand() {
        ("init", Some(matches)) => {
            match cmd::create_new_project(matches.value_of("name").unwrap()) {
                Ok(()) => console::success("Project created"),
                Err(e) => unravel_errors("Failed to create the project", &e, true),
            };
        },
        ("build", Some(_)) => {
            console::info("Building site...");
            let start = Instant::now();
            match cmd::build(config_file) {
                Ok(()) => report_elapsed_time(start),
                Err(e) => unravel_errors("Failed to build the site", &e, true),
            };
        },
        ("serve", Some(matches)) => {
            let interface = matches.value_of("interface").unwrap_or("127.0.0.1");
            let port = matches.value_of("port").unwrap_or("1111");
            console::info("Building site...");
            match cmd::serve(interface, port, config_file) {
                Ok(()) => (),
                Err(e) => unravel_errors("Failed to build the site", &e, true),
            };
        },
        _ => unreachable!(),
    }
}

