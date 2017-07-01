//! Benchmarking separate functions of Gutenberg

#![feature(test)]
extern crate test;
extern crate gutenberg;
extern crate tempdir;

use std::env;

use tempdir::TempDir;
use gutenberg::{Site, sort_pages, SortBy};

fn setup_site(name: &str) -> Site {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push(name);
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    site
}

#[bench]
fn bench_sort_pages_medium_blog(b: &mut test::Bencher) {
    let mut site = setup_site("huge-blog");
    let section = site.sections.values().next().unwrap().clone();
    b.iter(|| sort_pages(section.pages.clone(), SortBy::None));
}
