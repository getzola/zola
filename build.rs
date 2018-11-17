#[macro_use]
extern crate clap;

// use clap::Shell;

include!("src/cli.rs");

fn main() {
    // disabled below as it fails in CI
    //    let mut app = build_cli();
    //    app.gen_completions("zola", Shell::Bash, "completions/");
    //    app.gen_completions("zola", Shell::Fish, "completions/");
    //    app.gen_completions("zola", Shell::Zsh, "completions/");
    //    app.gen_completions("zola", Shell::PowerShell, "completions/");
}
