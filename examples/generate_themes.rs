//! This program is mainly intended for generating the dumps that are compiled in to
//! syntect, not as a helpful example for beginners.
//! Although it is a valid example for serializing syntaxes, you probably won't need
//! to do this yourself unless you want to cache your own compiled grammars.
extern crate syntect;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use syntect::dumps::*;
use std::env;

fn usage_and_exit() -> ! {
    println!("USAGE: gendata synpack source-dir newlines.packdump nonewlines.packdump\n
              gendata themepack source-dir themepack.themedump");
    ::std::process::exit(2);
}

fn main() {

    let mut a = env::args().skip(1);
    match (a.next(), a.next(), a.next(), a.next()) {
        (Some(ref cmd),
         Some(ref package_dir),
         Some(ref packpath_newlines),
         Some(ref packpath_nonewlines)) if cmd == "synpack" => {
            let mut ps = SyntaxSet::new();
            ps.load_plain_text_syntax();
            ps.load_syntaxes(package_dir, true).unwrap();
            dump_to_file(&ps, packpath_newlines).unwrap();

            ps = SyntaxSet::new();
            ps.load_plain_text_syntax();
            ps.load_syntaxes(package_dir, false).unwrap();
            dump_to_file(&ps, packpath_nonewlines).unwrap();

        }
        (Some(ref s), Some(ref theme_dir), Some(ref packpath), None) if s == "themepack" => {
            let ts = ThemeSet::load_from_folder(theme_dir).unwrap();
            for path in ts.themes.keys() {
                println!("{:?}", path);
            }
            dump_to_file(&ts, packpath).unwrap();
        }
        _ => usage_and_exit(),
    }
}
