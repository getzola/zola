#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;
extern crate walkdir;
extern crate pulldown_cmark;
extern crate regex;
extern crate tera;
extern crate glob;
extern crate syntect;
extern crate slug;

extern crate staticfile;
extern crate iron;
extern crate mount;
extern crate notify;
extern crate ws;


use std::time::Instant;

mod utils;
mod config;
mod errors;
mod cmd;
mod page;
mod front_matter;
mod site;


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
            let start = Instant::now();
            match cmd::build() {
                Ok(()) => {
                    let duration = start.elapsed();
                    println!("Site built in {}s.", duration.as_secs());
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
                    ::std::process::exit(1);
                },
            };
        },
        _ => unreachable!(),
    }
}

