use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Instant;

use cli::{Cli, Command};
use errors::anyhow;
use tracing;
use tracing_subscriber::EnvFilter;
use utils::net::{get_available_port, port_is_available};

use clap::{CommandFactory, Parser};
use time::UtcOffset;

mod cli;
mod cmd;
mod fs_utils;
mod messages;
mod prompt;

fn get_config_file_path(dir: &Path, config_path: &Path) -> (PathBuf, PathBuf) {
    let root_dir = dir.ancestors().find(|a| a.join(config_path).exists()).unwrap_or_else(|| {
        messages::unravel_errors(
            "",
            &anyhow!(
                "{} not found in current directory or ancestors, current_dir is {}",
                config_path.display(),
                dir.display()
            ),
        );
        std::process::exit(1);
    });

    // if we got here we found root_dir so config file should exist so we could theoretically unwrap safely
    let config_file_uncanonicalized = root_dir.join(config_path);
    let config_file = config_file_uncanonicalized.canonicalize().unwrap_or_else(|e| {
        messages::unravel_errors(
            &format!("Could not find canonical path of {}", config_file_uncanonicalized.display()),
            &e.into(),
        );
        std::process::exit(1);
    });

    (root_dir.to_path_buf(), config_file)
}

// tracing-subscriber prints to stderr, so we detect color configuration by considering the stderr stream (and not stdout)
static SHOULD_COLOR_OUTPUT: LazyLock<anstream::ColorChoice> =
    LazyLock::new(|| anstream::AutoStream::choice(&std::io::stderr()));

fn main() {
    // ensure that logging uses the "info" level for anything in Zola by default
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("zola=info"));

    // Determine if we should use ANSI colors
    let use_ansi = matches!(
        *SHOULD_COLOR_OUTPUT,
        anstream::ColorChoice::Always
            | anstream::ColorChoice::AlwaysAnsi
            | anstream::ColorChoice::Auto
    );

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(use_ansi)
        .with_target(false)
        .with_level(true)
        .without_time()
        .init();

    let cli = Cli::parse();
    let cli_dir: PathBuf = cli.root.canonicalize().unwrap_or_else(|e| {
        messages::unravel_errors(
            &format!("Could not find canonical path of root dir: {}", cli.root.display()),
            &e.into(),
        );
        std::process::exit(1);
    });

    match cli.command {
        Command::Init { name, force } => {
            if let Err(e) = cmd::create_new_project(&name, force) {
                messages::unravel_errors("Failed to create the project", &e);
                std::process::exit(1);
            }
        }
        Command::Build { base_url, output_dir, force, drafts, minify } => {
            tracing::info!("Building site...");
            let start = Instant::now();
            let (root_dir, config_file) = get_config_file_path(&cli_dir, &cli.config);
            match cmd::build(
                &root_dir,
                &config_file,
                base_url.as_deref(),
                output_dir.as_deref(),
                force,
                drafts,
                minify,
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
            force,
            base_url,
            drafts,
            open,
            store_html,
            fast,
            no_port_append,
            extra_watch_path,
            debounce,
        } => {
            if port != 1111 && !port_is_available(interface, port) {
                tracing::error!("The requested port is not available");
                std::process::exit(1);
            }

            if !port_is_available(interface, port) {
                port = get_available_port(interface, 1111).unwrap_or_else(|| {
                    tracing::error!("No port available");
                    std::process::exit(1);
                });
            }

            let (root_dir, config_file) = get_config_file_path(&cli_dir, &cli.config);
            tracing::info!("Building site...");
            if let Err(e) = cmd::serve(
                &root_dir,
                interface,
                port,
                output_dir.as_deref(),
                force,
                base_url.as_deref(),
                &config_file,
                open,
                drafts,
                store_html,
                fast,
                no_port_append,
                UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC),
                extra_watch_path,
                debounce,
            ) {
                messages::unravel_errors("Failed to serve the site", &e);
                std::process::exit(1);
            }
        }
        Command::Check { drafts, skip_external_links } => {
            tracing::info!("Checking site...");
            let start = Instant::now();
            let (root_dir, config_file) = get_config_file_path(&cli_dir, &cli.config);
            match cmd::check(&root_dir, &config_file, None, None, drafts, skip_external_links) {
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
