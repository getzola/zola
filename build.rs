#[macro_use]
extern crate clap;

// use clap::Shell;

include!("src/cli.rs");

fn main() {
    // disabled below as it fails in CI
//    let mut app = build_cli();
//    app.gen_completions("gutenberg", Shell::Bash, "completions/");
//    app.gen_completions("gutenberg", Shell::Fish, "completions/");
//    app.gen_completions("gutenberg", Shell::Zsh, "completions/");
//    app.gen_completions("gutenberg", Shell::PowerShell, "completions/");
}
