// use clap::Shell;

include!("src/cli.rs");

fn generate_pe_header() {
    use time::OffsetDateTime;

    let today = OffsetDateTime::now_utc();
    let copyright = format!("Copyright © 2017-{} Vincent Prouillet", today.year());
    let mut res = winres::WindowsResource::new();
    // needed for MinGW cross-compiling
    if cfg!(unix) {
        res.set_windres_path("x86_64-w64-mingw32-windres");
    }
    res.set_icon("docs/static/favicon.ico");
    res.set("LegalCopyright", &copyright);
    res.compile().expect("Failed to compile Windows resources!");
}

fn main() {
    // disabled below as it fails in CI
    //    let mut app = build_cli();
    //    app.gen_completions("zola", Shell::Bash, "completions/");
    //    app.gen_completions("zola", Shell::Fish, "completions/");
    //    app.gen_completions("zola", Shell::Zsh, "completions/");
    //    app.gen_completions("zola", Shell::PowerShell, "completions/");
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() != "windows"
        && std::env::var("PROFILE").unwrap() != "release"
    {
        return;
    }
    if cfg!(windows) {
        generate_pe_header();
    }
}
