#![feature(test)]
extern crate test;
extern crate site;
extern crate tempdir;

use std::env;

use tempdir::TempDir;
use site::Site;


fn setup_site(name: &str) -> Site {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push(name);
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    site
}

#[bench]
fn bench_render_aliases(b: &mut test::Bencher) {
    let mut site = setup_site("huge-blog");
    let tmp_dir = TempDir::new("benches").expect("create temp dir");
    let output_dir = &tmp_dir.path().join("public");
    site.set_output_path(&output_dir);
    b.iter(|| site.render_aliases().unwrap());
}

#[bench]
fn bench_render_sitemap(b: &mut test::Bencher) {
    let mut site = setup_site("huge-blog");
    let tmp_dir = TempDir::new("benches").expect("create temp dir");
    let output_dir = &tmp_dir.path().join("public");
    site.set_output_path(&output_dir);
    b.iter(|| site.render_sitemap().unwrap());
}

#[bench]
fn bench_render_rss_feed(b: &mut test::Bencher) {
    let mut site = setup_site("huge-blog");
    let tmp_dir = TempDir::new("benches").expect("create temp dir");
    let output_dir = &tmp_dir.path().join("public");
    site.set_output_path(&output_dir);
    b.iter(|| site.render_rss_feed().unwrap());
}

#[bench]
fn bench_render_categories(b: &mut test::Bencher) {
    let mut site = setup_site("huge-blog");
    let tmp_dir = TempDir::new("benches").expect("create temp dir");
    let output_dir = &tmp_dir.path().join("public");
    site.set_output_path(&output_dir);
    b.iter(|| site.render_categories().unwrap());
}

#[bench]
fn bench_render_paginated(b: &mut test::Bencher) {
    let mut site = setup_site("medium-blog");
    let tmp_dir = TempDir::new("benches").expect("create temp dir");
    let output_dir = &tmp_dir.path().join("public");
    site.set_output_path(&output_dir);
    let section = site.sections.values().collect::<Vec<_>>()[0];

    b.iter(|| site.render_paginated(output_dir, section));
}
