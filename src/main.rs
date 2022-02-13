use std::path::PathBuf;
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
    let cli_dir: PathBuf =
        cli.root.canonicalize().unwrap_or_else(|_| panic!("Could not find canonical path of root dir: {}", cli.root.display()));

    let root_dir = cli_dir
        .ancestors()
        .find_map(|a| if a.join(&cli.config).exists() { Some(a) } else { None })
        .unwrap_or_else(|| panic!("could not find directory containing config file"));

    match cli.command {
        Command::Init { name, force } => {
            if let Err(e) = cmd::create_new_project(&name, force) {
                console::unravel_errors("Failed to create the proejct", &e);
                std::process::exit(1);
            }
        }
        Command::Build { base_url, output_dir, drafts } => {
            console::info("Building site...");
            let start = Instant::now();
            match cmd::build(
                root_dir,
                &cli.config,
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
            if let Err(e) = cmd::serve(
                root_dir,
                &interface,
                port,
                output_dir.as_deref(),
                &base_url,
                &cli.config,
                open,
                drafts,
                fast,
            ) {
                console::unravel_errors("Failed to serve the site", &e);
                std::process::exit(1);
            }
        }
        Command::Check { drafts } => {
            console::info("Checking site...");
            let start = Instant::now();
            match cmd::check(root_dir, &cli.config, None, None, drafts) {
                Ok(()) => console::report_elapsed_time(start),
                Err(e) => {
                    console::unravel_errors("Failed to check the site", &e);
                    std::process::exit(1);
                }
            }
        }
    }
}
