//! Benchmarking separate functions of Gutenberg

#![feature(test)]
extern crate test;
extern crate gutenberg;
extern crate tempdir;

use std::env;

use tempdir::TempDir;
use gutenberg::Site;


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
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    b.iter(|| site.render_aliases().unwrap());
}
//
//#[bench]
//fn bench_render_sections_one_huge(b: &mut test::Bencher) {
//    let mut site = setup_site("big-blog");
//    let tmp_dir = TempDir::new("benches").expect("create temp dir");
//    let public = &tmp_dir.path().join("public");
//    site.set_output_path(&public);
//    b.iter(|| site.render_sections().unwrap());
//}

#[bench]
fn bench_render_small_section_with_pages_and_pagination(b: &mut test::Bencher) {
    let mut site = setup_site("small-blog");
    let tmp_dir = TempDir::new("benches").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    b.iter(|| site.render_section(site.sections.values().next().unwrap(), true).unwrap());
}

#[bench]
fn bench_render_small_section_with_pages_and_no_pagination(b: &mut test::Bencher) {
    let mut site = setup_site("small-blog");
    let tmp_dir = TempDir::new("benches").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    let mut section = site.sections.values().next().unwrap().clone();
    section.meta.paginate_by = None;
    section.meta.template = None;
    b.iter(|| site.render_section(&section, true).unwrap());
}
