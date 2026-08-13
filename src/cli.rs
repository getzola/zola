use std::net::IpAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[clap(
    version,
    author,
    about,
    after_help = "License: EUPL-1.2 <https://eupl.eu>, MIT for code existing before 0.22"
)]
pub struct Cli {
    /// Directory to use as root of project
    #[clap(short = 'r', long, default_value = ".")]
    pub root: PathBuf,

    /// Path to a config file other than zola.toml or config.toml in the root of project
    #[clap(short = 'c', long)]
    pub config: Option<PathBuf>,

    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new Zola project
    Init {
        /// Name of the project. Will create a new directory with that name in the current directory
        #[clap(default_value = ".")]
        name: String,

        /// Force creation of project even if directory is non-empty
        #[clap(short = 'f', long)]
        force: bool,
    },

    /// Deletes the output directory if there is one and builds the site
    Build {
        /// Force the base URL to be that value (defaults to the one in the config file)
        #[clap(short = 'u', long)]
        base_url: Option<String>,

        /// Outputs the generated site in the given path (by default 'public' dir in project root)
        #[clap(short = 'o', long)]
        output_dir: Option<PathBuf>,

        /// Force building the site even if output directory is non-empty
        #[clap(short = 'f', long)]
        force: bool,

        /// Include drafts when loading the site
        #[clap(long)]
        drafts: bool,

        /// Minify generated HTML files
        #[clap(long)]
        minify: bool,

        /// Print a breakdown of how long each build phase took (developer diagnostic)
        #[clap(long)]
        timings: bool,
    },

    /// Serve the site. Rebuild and reload on change automatically
    Serve {
        /// Interface to bind on
        #[clap(short = 'i', long, default_value = "127.0.0.1")]
        interface: IpAddr,

        /// Which port to use
        #[clap(short = 'p', long, default_value_t = 1111)]
        port: u16,

        /// Outputs assets of the generated site in the given path (by default 'public' dir in project root).
        /// HTML/XML will be stored in memory.
        #[clap(short = 'o', long)]
        output_dir: Option<PathBuf>,

        /// Force use of the directory for serving the site even if output directory is non-empty
        #[clap(long)]
        force: bool,

        /// Changes the base_url
        #[clap(short = 'u', long)]
        base_url: Option<String>,

        /// Include drafts when loading the site
        #[clap(long)]
        drafts: bool,

        /// Open site in the default browser
        #[clap(short = 'O', long)]
        open: bool,

        /// Write HTML to the output folder and serve it from there instead of keeping it in
        /// memory. Needed for large sites: serving from memory costs as much RAM as the site's
        /// total output, which on a 9 GB site is 9 GB against 0.5 GB from disk. The cost is that
        /// a full rebuild pays for writing the files.
        #[clap(long)]
        store_html: bool,

        /// Only rebuild the minimum on change - useful when working on a specific page/section
        #[clap(short = 'f', long)]
        fast: bool,

        /// Default append port to the base url.
        #[clap(long)]
        no_port_append: bool,

        /// Extra path to watch for changes, relative to the project root.
        #[clap(long)]
        extra_watch_path: Vec<String>,

        /// Debounce time in milliseconds for the file watcher (at least 1ms)
        #[clap(short = 'd', long, default_value_t = 1000, value_parser = clap::value_parser!(u64).range(1..))]
        debounce: u64,
    },

    /// Try to build the project without rendering it. Checks links
    Check {
        /// Include drafts when loading the site
        #[clap(long)]
        drafts: bool,
        /// Skip external links
        #[clap(long)]
        skip_external_links: bool,
    },

    /// Generate shell completion
    Completion {
        /// Shell to generate completion for
        #[clap(value_enum)]
        shell: Shell,
    },
}
