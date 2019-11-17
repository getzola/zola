//! This program is mainly intended for generating the dumps that are compiled in to
//! syntect, not as a helpful example for beginners.
//! Although it is a valid example for serializing syntaxes, you probably won't need
//! to do this yourself unless you want to cache your own compiled grammars.
extern crate itertools;
extern crate syntect;

use itertools::Itertools;
use std::env;
use syntect::dumps::*;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSetBuilder;

fn usage_and_exit() -> ! {
    println!("USAGE: cargo run --example generate_sublime synpack source-dir newlines.packdump nonewlines.packdump\n
              cargo run --example generate_sublime themepack source-dir themepack.themedump");
    ::std::process::exit(2);
}

// Not an example of zola but is used to generate the theme and syntax dump
// used for syntax highlighting.
// Check README for more details
fn main() {
    let mut args = env::args().skip(1);
    match (args.next(), args.next(), args.next()) {
        (Some(ref cmd), Some(ref package_dir), Some(ref packpath_newlines)) if cmd == "synpack" => {
            let mut builder = SyntaxSetBuilder::new();
            builder.add_plain_text_syntax();
            builder.add_from_folder(package_dir, true).unwrap();
            let ss = builder.build();
            dump_to_file(&ss, packpath_newlines).unwrap();
            let syntaxes_sorted = ss
                .syntaxes()
                .iter()
                .sorted_by(|a, b| {
                    Ord::cmp(&a.name.to_ascii_lowercase(), &b.name.to_ascii_lowercase())
                })
                .coalesce(|x, y| {
                    if x.name == y.name {
                        if x.file_extensions.len() > y.file_extensions.len() {
                            Ok(x)
                        } else {
                            Ok(y)
                        }
                    } else {
                        Err((x, y))
                    }
                });
            for s in syntaxes_sorted {
                if !s.file_extensions.is_empty() {
                    println!("- {} -> {:?}", s.name, s.file_extensions);
                }
            }
        }
        (Some(ref cmd), Some(ref theme_dir), Some(ref packpath)) if cmd == "themepack" => {
            let ts = ThemeSet::load_from_folder(theme_dir).unwrap();
            for path in ts.themes.keys() {
                println!("{:?}", path);
            }
            dump_to_file(&ts, packpath).unwrap();
        }
        _ => usage_and_exit(),
    }
}
