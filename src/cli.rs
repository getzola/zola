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
                .default_value("config.toml")
                .takes_value(true)
                .help("Path to a config file other than config.toml")
        )
        .subcommands(vec![
            SubCommand::with_name("init")
                .about("Create a new Zola project")
                .arg(
                    Arg::with_name("name")
                        .default_value(".")
                        .help("Name of the project. Will create a new directory with that name in the current directory")
                ),
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
                        .default_value("public")
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
                        .default_value("public")
                        .takes_value(true)
                        .help("Outputs the generated site in the given path"),
                    Arg::with_name("base_url")
                        .short("u")
                        .long("base-url")
                        .default_value("127.0.0.1")
                        .takes_value(true)
                        .help("Changes the base_url"),
                    Arg::with_name("watch_only")
                        .long("watch-only")
                        .takes_value(false)
                        .help("Do not start a server, just re-build project on changes"),
                    Arg::with_name("drafts")
                        .long("drafts")
                        .takes_value(false)
                        .help("Include drafts when loading the site"),
                    Arg::with_name("open")
                        .short("O")
                        .long("open")
                        .takes_value(false)
                        .help("Open site in the default browser"),
                ]),
            SubCommand::with_name("check")
                .about("Try building the project without rendering it. Checks links")
                .args(&[
                    Arg::with_name("drafts")
                        .long("drafts")
                        .takes_value(false)
                        .help("Include drafts when loading the site"),
                ]),
            SubCommand::with_name("index")
                .about("Create a search index as a stand-alone task, and with additional options")
                .args({
                    let drafts = Arg::with_name("drafts") .long("drafts")
                        .takes_value(false)
                        .help("Include drafts when loading the site");

                        let index_type = Arg::with_name("index_type")
                        .long("index-type")
                        .short("t")
                        .takes_value(true)
                        .possible_values(&["elasticlunr", "tantivy"])
                        .required(true)
                        .help("what kind of search index to build");
                    let output_dir = Arg::with_name("output_dir")
                        .short("o")
                        .long("output-dir")
                        .default_value("public")
                        .takes_value(true)
                        .help("Outputs the generated search index files into the provided dir. \
                               Note: Tantivy indexing produces a directory instead of a file, \
                               which will be located at output-dir/tantivy-index");
                    &[drafts, index_type, output_dir]
                }),
        ])
}
