use std::path::{Path, PathBuf};
use std::time::Instant;

use cli::{Cli, Command};
use utils::net::{get_available_port, port_is_available};

use clap::{CommandFactory, Parser};
use time::UtcOffset;

mod cli;
mod cmd;
mod messages;
mod prompt;

fn get_config_file_path(dir: &Path, config_path: &Path) -> (PathBuf, PathBuf) {
    let root_dir = dir
        .ancestors()
        .find(|a| a.join(config_path).exists())
        .unwrap_or_else(|| panic!("could not find directory containing config file"));

    // if we got here we found root_dir so config file should exist so we can unwrap safely
    let config_file = root_dir
        .join(config_path)
        .canonicalize()
        .unwrap_or_else(|_| panic!("could not find directory containing config file"));
    (root_dir.to_path_buf(), config_file)
}

fn main() {
    let cli = Cli::parse();
    let cli_dir: PathBuf = cli.root.canonicalize().unwrap_or_else(|_| {
        panic!("Could not find canonical path of root dir: {}", cli.root.display())
    });

    match cli.command {
        Command::Init { name, force } => {
            if let Err(e) = cmd::create_new_project(&name, force) {
                messages::unravel_errors("Failed to create the project", &e);
                std::process::exit(1);
            }
        }
        Command::Build { base_url, output_dir, force, drafts } => {
            console::info("Building site...");
            let start = Instant::now();
            let (root_dir, config_file) = get_config_file_path(&cli_dir, &cli.config);
            match cmd::build(
                &root_dir,
                &config_file,
                base_url.as_deref(),
                output_dir.as_deref(),
                force,
                drafts,
            ) {
                Ok(()) => messages::report_elapsed_time(start),
                Err(e) => {
                    messages::unravel_errors("Failed to build the site", &e);
                    std::process::exit(1);
                }
            }
        }
        Command::Serve {
            interface,
            mut port,
            output_dir,
            base_url,
            drafts,
            open,
            fast,
            no_port_append,
        } => {
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

            let (root_dir, config_file) = get_config_file_path(&cli_dir, &cli.config);
            console::info("Building site...");
            if let Err(e) = cmd::serve(
                &root_dir,
                &interface,
                port,
                output_dir.as_deref(),
                &base_url,
                &config_file,
                open,
                drafts,
                fast,
                no_port_append,
                UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC),
            ) {
                messages::unravel_errors("Failed to serve the site", &e);
                std::process::exit(1);
            }
        }
        Command::Check { drafts } => {
            console::info("Checking site...");
            let start = Instant::now();
            let (root_dir, config_file) = get_config_file_path(&cli_dir, &cli.config);
            match cmd::check(&root_dir, &config_file, None, None, drafts) {
                Ok(()) => messages::report_elapsed_time(start),
                Err(e) => {
                    messages::unravel_errors("Failed to check the site", &e);
                    std::process::exit(1);
                }
            }
        }
        Command::Completion { shell } => {
            let cmd = &mut Cli::command();
            clap_complete::generate(shell, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
        }
    }
}
