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

include!("src/cli.rs");

fn generate_man_pages() {
    use clap::CommandFactory;

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let cmd = Cli::command();
    clap_mangen::generate_to(cmd, out_dir).unwrap();
}

fn main() {
    if std::env::var("PROFILE").unwrap() != "release" {
        return;
    }
    if cfg!(windows) {
        generate_pe_header();
    }
    if cfg!(unix) {
        generate_man_pages();
    }
}
