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


mod utils;
mod config;
mod errors;
mod cmd;
mod page;
mod front_matter;


use config::Config;


// Get and parse the config.
// If it doesn't succeed, exit
fn get_config() -> Config {
    match Config::from_file("config.toml") {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to load config.toml");
            println!("Error: {}", e);
            for e in e.iter().skip(1) {
                println!("Reason: {}", e)
            }
            ::std::process::exit(1);
        }
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
    ).get_matches();

    match matches.subcommand() {
        ("init", Some(matches)) => {
            match cmd::create_new_project(matches.value_of("name").unwrap()) {
                Ok(()) => {
                    println!("Project created");
                    println!("You will now need to set a theme in `config.toml`");
                },
                Err(e) => {
                    println!("Error: {}", e);
                    ::std::process::exit(1);
                },
            };
        },
        ("build", Some(_)) => {
            match cmd::build(get_config()) {
                Ok(()) => {
                    println!("Project built.");
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
        _ => unreachable!(),
    }
}

