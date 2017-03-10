#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate gutenberg;
extern crate chrono;

extern crate staticfile;
extern crate iron;
extern crate mount;
extern crate notify;
extern crate ws;


use std::time::Instant;
use chrono::Duration;

mod cmd;


// Print the time elapsed rounded to 1 decimal
fn report_elapsed_time(instant: Instant) {
    let duration_ms = Duration::from_std(instant.elapsed()).unwrap().num_milliseconds() as f64;

    if duration_ms < 1000.0 {
        println!("Done in {}ms.\n", duration_ms);
    } else {
        let duration_sec = duration_ms / 1000.0;
        println!("Done in {:.1}s.\n", ((duration_sec * 10.0).round() / 10.0));
    }
}


fn main() {
    let matches = clap_app!(Gutenberg =>
        (version: crate_version!())
        (author: "Vincent Prouillet")
        (about: "Static site generator")
        (@setting SubcommandRequiredElseHelp)
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

    match matches.subcommand() {
        ("init", Some(matches)) => {
            match cmd::create_new_project(matches.value_of("name").unwrap()) {
                Ok(()) => {
                    println!("Project created");
                },
                Err(e) => {
                    println!("Error: {}", e);
                    ::std::process::exit(1);
                },
            };
        },
        ("build", Some(_)) => {
            println!("Building site");
            let start = Instant::now();
            match cmd::build() {
                Ok(()) => {
                    report_elapsed_time(start);
                },
                Err(e) => {
                    println!("Failed to build the site");
                    println!("Error: {}", e);
                    for e in e.iter().skip(1) {
                        println!("Reason: {}", e)
                    }
                    ::std::process::exit(1);
                },
            };
        },
        ("serve", Some(matches)) => {
            let interface = matches.value_of("interface").unwrap_or("127.0.0.1");
            let port = matches.value_of("port").unwrap_or("1111");
            match cmd::serve(interface, port) {
                Ok(()) => (),
                Err(e) => {
                    println!("Error: {}", e);
                    for e in e.iter().skip(1) {
                        println!("Reason: {}", e)
                    }
                    ::std::process::exit(1);
                },
            };
        },
        _ => unreachable!(),
    }
}

