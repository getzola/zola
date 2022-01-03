use std::time::Instant;

use cli::{Cli, Command};
use utils::net::{get_available_port, port_is_available};

use clap::Parser;

mod cli;
mod cmd;
mod console;
mod prompt;

fn main() {
    let cli = Cli::parse();
    let root_dir = cli
        .root
        .canonicalize()
        .unwrap_or_else(|_| panic!("Cannot find root directory: {}", cli.root.display()));
    let config_file = cli
        .config
        .map(|path| {
            path.canonicalize()
                .unwrap_or_else(|_| panic!("Cannot find config file: {}", path.display()))
        })
        .unwrap_or_else(|| root_dir.join("config.toml"));

    match cli.command {
        Command::Init { name, force } => match cmd::create_new_project(&name, force) {
            Ok(()) => (),
            Err(e) => {
                console::unravel_errors("Failed to create the proejct", &e);
                std::process::exit(1);
            }
        },
        Command::Build { base_url, output_dir, drafts } => {
            console::info("Building site...");
            let start = Instant::now();
            match cmd::build(
                &root_dir,
                &config_file,
                base_url.as_deref(),
                output_dir.as_deref(),
                drafts,
            ) {
                Ok(()) => console::report_elapsed_time(start),
                Err(e) => {
                    console::unravel_errors("Failed to build the site", &e);
                    std::process::exit(1);
                }
            }
        }
        Command::Serve { interface, mut port, output_dir, base_url, drafts, open, fast } => {
            if port != 1111 && !port_is_available(port) {
                console::error("The requested port is not available");
                std::process::exit(1);
            }

            if !port_is_available(port) {
                port = get_available_port(1111).unwrap_or_else(|| {
                    console::error("No port available");
                    std::process::exit(1);
                });
            }

            console::info("Building site...");
            match cmd::serve(
                &root_dir,
                &interface,
                port,
                output_dir.as_deref(),
                &base_url,
                &config_file,
                open,
                drafts,
                fast,
            ) {
                Ok(()) => (),
                Err(e) => {
                    console::unravel_errors("Failed to serve the site", &e);
                    std::process::exit(1);
                }
            }
        }
        Command::Check { drafts } => {
            console::info("Checking site...");
            let start = Instant::now();
            match cmd::check(&root_dir, &config_file, None, None, drafts) {
                Ok(()) => console::report_elapsed_time(start),
                Err(e) => {
                    console::unravel_errors("Failed to check the site", &e);
                    std::process::exit(1);
                }
            }
        }
    }
}
