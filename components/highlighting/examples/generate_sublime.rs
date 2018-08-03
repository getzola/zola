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
    println!("USAGE: cargo run --example generate_sublime synpack source-dir newlines.packdump nonewlines.packdump\n
              cargo run --example generate_sublime themepack source-dir themepack.themedump");
    ::std::process::exit(2);
}

// Not an example of Gutenberg but is used to generate the theme and syntax dump
// used for syntax highlighting.
// Check README for more details
fn main() {
    let mut args = env::args().skip(1);
    match (args.next(), args.next(), args.next(), args.next()) {
        (Some(ref cmd), Some(ref package_dir), Some(ref packpath_newlines), Some(ref packpath_nonewlines)) if cmd == "synpack" => {
            let mut ps = SyntaxSet::new();
            ps.load_plain_text_syntax();
            ps.load_syntaxes(package_dir, true).unwrap();
            dump_to_file(&ps, packpath_newlines).unwrap();

            ps = SyntaxSet::new();
            ps.load_plain_text_syntax();
            ps.load_syntaxes(package_dir, false).unwrap();
            dump_to_file(&ps, packpath_nonewlines).unwrap();

            for s in ps.syntaxes() {
                if !s.file_extensions.is_empty() {
                    println!("- {} -> {:?}", s.name, s.file_extensions);
                }
            }
        },
        (Some(ref cmd), Some(ref theme_dir), Some(ref packpath), None) if cmd == "themepack" => {
            let ts = ThemeSet::load_from_folder(theme_dir).unwrap();
            for path in ts.themes.keys() {
                println!("{:?}", path);
            }
            dump_to_file(&ts, packpath).unwrap();
        }
        _ => usage_and_exit(),
    }
}
