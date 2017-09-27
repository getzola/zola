use clap::App;


pub fn build_cli() -> App<'static, 'static> {
    clap_app!(Gutenberg =>
        (version: crate_version!())
        (author: "Vincent Prouillet")
        (about: "Static site generator")
        (@setting SubcommandRequiredElseHelp)
        (@arg config: -c --config +takes_value "Path to a config file other than config.toml")
        (@subcommand init =>
            (about: "Create a new Gutenberg project")
            (@arg name: +required "Name of the project. Will create a new directory with that name in the current directory")
        )
        (@subcommand build =>
            (about: "Builds the site")
        )
        (@subcommand serve =>
            (about: "Serve the site. Rebuild and reload on change automatically")
            (@arg interface: -i --interface +takes_value "Interface to bind on (default to 127.0.0.1)")
            (@arg port: -p --port +takes_value "Which port to use (default to 1111)")
        )
    )
}
