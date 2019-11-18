//! This program is mainly intended for generating the dumps that are compiled in to
//! syntect, not as a helpful example for beginners.
//! Although it is a valid example for serializing syntaxes, you probably won't need
//! to do this yourself unless you want to cache your own compiled grammars.
extern crate syntect;

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use syntect::dumps::*;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxReference, SyntaxSetBuilder};

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
            let mut syntaxes: HashMap<String, SyntaxReference> = HashMap::new();
            for s in ss.syntaxes().iter() {
                if !syntaxes.contains_key(&s.name) {
                    syntaxes.insert(s.name.clone(), s.clone());
                } else {
                    if let Some(some_s) = syntaxes.get_mut(&s.name) {
                        let mut extensions: HashSet<String> =
                            some_s.file_extensions.iter().cloned().collect();
                        for fe in s.clone().file_extensions.iter() {
                            extensions.insert(fe.clone());
                        }
                        let mut extensions_merged = extensions.iter().cloned().collect::<Vec<_>>();
                        extensions_merged.sort();
                        some_s.file_extensions.clear();
                        some_s.file_extensions.extend(extensions_merged);
                    }
                }
            }
            let mut keys = syntaxes.keys().collect::<Vec<_>>();
            keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            for k in keys {
                if let Some(s) = syntaxes.get(k) {
                    if !s.file_extensions.is_empty() {
                        println!("- {} -> {:?}", s.name, s.file_extensions);
                    }
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
