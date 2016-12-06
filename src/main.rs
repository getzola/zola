// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use] extern crate clap;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate lazy_static;
extern crate toml;
extern crate walkdir;
extern crate pulldown_cmark;
extern crate regex;

mod config;
mod errors;
mod cmd;
mod page;

use config::Config;


// Get and parse the config.
// If it doesn't succeed, exit
fn get_config() -> Config {
    match Config::from_file("config.toml") {
        Ok(c) => c,
        Err(e) => {
            println!("Error: {}", e);
            ::std::process::exit(1);
        }
    }
}


fn main() {
    let matches = clap_app!(myapp =>
        (version: crate_version!())
        (author: "Vincent Prouillet")
        (about: "Static site generator")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand new =>
            (about: "Create a new Gutenberg project")
            (@arg name: +required "Name of the project. Will create a directory with that name in the current directory")
        )
        (@subcommand build =>
            (about: "Builds the site")
        )
    ).get_matches();

    match matches.subcommand() {
        ("new", Some(matches)) => {
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
        ("build", None) => {
            match cmd::build(get_config()) {
                Ok(()) => {
                    println!("Project built");
                },
                Err(e) => {
                    println!("Error: {}", e);
                    ::std::process::exit(1);
                },
            };
        },
        _ => unreachable!(),
    }
}

