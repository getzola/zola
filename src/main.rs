// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use] extern crate clap;
#[macro_use] extern crate error_chain;

mod config;
mod errors;
mod cmd;


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
        _ => unreachable!(),
    }
}

