#![feature(test)]
extern crate test;

use std::env;

use library::Paginator;
use site::Site;
use tempfile::tempdir;

fn setup_site(name: &str) -> Site {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push(name);
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();
    site.load().unwrap();
    site
}

#[bench]
fn bench_render_aliases(b: &mut test::Bencher) {
    let mut site = setup_site("big-blog");
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    b.iter(|| site.render_aliases().unwrap());
}

#[bench]
fn bench_render_sitemap(b: &mut test::Bencher) {
    let mut site = setup_site("big-blog");
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    b.iter(|| site.render_sitemap().unwrap());
}

#[bench]
fn bench_render_feed(b: &mut test::Bencher) {
    let mut site = setup_site("big-blog");
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    b.iter(|| {
        site.render_feed(
            site.library.read().unwrap().pages_values(),
            None,
            &site.config.default_language,
            |c| c,
        )
        .unwrap();
    });
}

#[bench]
fn bench_render_taxonomies(b: &mut test::Bencher) {
    let mut site = setup_site("small-blog");
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    b.iter(|| site.render_taxonomies().unwrap());
}

#[bench]
fn bench_render_paginated(b: &mut test::Bencher) {
    let mut site = setup_site("small-blog");
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    let library = site.library.read().unwrap();
    let section = library.sections_values()[0];
    let paginator = Paginator::from_section(&section, &library);

    b.iter(|| site.render_paginated(Vec::new(), &paginator));
}

#[bench]
fn bench_populate_sections_medium_blog(b: &mut test::Bencher) {
    let mut site = setup_site("medium-blog");
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);

    b.iter(|| site.populate_sections());
}

#[bench]
fn bench_populate_sections_medium_kb(b: &mut test::Bencher) {
    let mut site = setup_site("medium-kb");
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);

    b.iter(|| site.populate_sections());
}

#[bench]
fn bench_render_markdown_small_blog(b: &mut test::Bencher) {
    let mut site = setup_site("small-blog");
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);

    b.iter(|| site.render_markdown());
}

#[bench]
fn bench_render_markdown_small_kb(b: &mut test::Bencher) {
    let mut site = setup_site("small-kb");
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);

    b.iter(|| site.render_markdown());
}
