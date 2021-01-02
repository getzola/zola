//! This program is mainly intended for generating the dumps that are compiled in to
//! syntect, not as a helpful example for beginners.
//! Although it is a valid example for serializing syntaxes, you probably won't need
//! to do this yourself unless you want to cache your own compiled grammars.

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::iter::FromIterator;
use std::path::Path;
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
            let base_path = Path::new(&package_dir).to_path_buf();

            // First the official Sublime packages
            let mut default = base_path.clone();
            default.push("Packages");
            match builder.add_from_folder(&default, true) {
                Ok(_) => (),
                Err(e) => println!("Loading error: {:?}", e),
            };

            // and then the ones we add
            let mut extra = base_path.clone();
            extra.push("extra");
            match builder.add_from_folder(&extra, true) {
                Ok(_) => (),
                Err(e) => println!("Loading error: {:?}", e),
            };

            let ss = builder.build();
            dump_to_file(&ss, packpath_newlines).unwrap();
            let mut syntaxes: HashMap<String, HashSet<String>> = HashMap::new();

            for s in ss.syntaxes().iter() {
                syntaxes
                    .entry(s.name.clone())
                    .and_modify(|e| {
                        for ext in &s.file_extensions {
                            e.insert(ext.clone());
                        }
                    })
                    .or_insert_with(|| HashSet::from_iter(s.file_extensions.iter().cloned()));
            }
            let mut keys = syntaxes.keys().collect::<Vec<_>>();
            keys.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            for k in keys {
                if !syntaxes[k].is_empty() {
                    let mut extensions_sorted = syntaxes[k].iter().cloned().collect::<Vec<_>>();
                    extensions_sorted.sort();
                    println!("- {} -> {:?}", k, extensions_sorted);
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
