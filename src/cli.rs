use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    App::new("zola")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg(
            Arg::with_name("root")
                .short("r")
                .long("root")
                .takes_value(true)
                .default_value(".")
                .help("Directory to use as root of project")
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .help("Path to a config file other than config.toml in the root of project")
        )
        .subcommands(vec![
            SubCommand::with_name("init")
                .about("Create a new Zola project")
                .args(&[
                    Arg::with_name("name")
                        .default_value(".")
                        .help("Name of the project. Will create a new directory with that name in the current directory"),
                    Arg::with_name("force")
                        .short("f")
                        .takes_value(false)
                        .help("Force creation of project even if directory is non-empty")
                ]),
            SubCommand::with_name("build")
                .about("Deletes the output directory if there is one and builds the site")
                .args(&[
                    Arg::with_name("base_url")
                        .short("u")
                        .long("base-url")
                        .takes_value(true)
                        .help("Force the base URL to be that value (default to the one in config.toml)"),
                    Arg::with_name("output_dir")
                        .short("o")
                        .long("output-dir")
                        .takes_value(true)
                        .help("Outputs the generated site in the given path"),
                    Arg::with_name("drafts")
                        .long("drafts")
                        .takes_value(false)
                        .help("Include drafts when loading the site"),
                ]),
            SubCommand::with_name("serve")
                .about("Serve the site. Rebuild and reload on change automatically")
                .args(&[
                    Arg::with_name("interface")
                        .short("i")
                        .long("interface")
                        .default_value("127.0.0.1")
                        .help("Interface to bind on"),
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .default_value("1111")
                        .help("Which port to use"),
                    Arg::with_name("output_dir")
                        .short("o")
                        .long("output-dir")
                        .takes_value(true)
                        .help("Outputs the generated site in the given path"),
                    Arg::with_name("base_url")
                        .short("u")
                        .long("base-url")
                        .default_value("127.0.0.1")
                        .takes_value(true)
                        .help("Changes the base_url"),
                    Arg::with_name("drafts")
                        .long("drafts")
                        .takes_value(false)
                        .help("Include drafts when loading the site"),
                    Arg::with_name("open")
                        .short("O")
                        .long("open")
                        .takes_value(false)
                        .help("Open site in the default browser"),
                    Arg::with_name("fast")
                        .short("f")
                        .long("fast")
                        .takes_value(false)
                        .help("Only rebuild the minimum on change - useful when working on a specific page/section"),
                ]),
            SubCommand::with_name("check")
                .about("Try building the project without rendering it. Checks links")
                .args(&[
                    Arg::with_name("drafts")
                        .long("drafts")
                        .takes_value(false)
                        .help("Include drafts when loading the site"),
                ])
        ])
}
