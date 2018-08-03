//! Benchmarking writing down on the hard drive sites of various sizes

#![feature(test)]
extern crate test;
extern crate site;
extern crate tempfile;

use std::env;

use site::Site;
use tempfile::tempdir;


#[bench]
fn bench_rendering_small_blog(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push("small-blog");
    let mut site = Site::new(&path, "config.toml").unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.load().unwrap();

    b.iter(|| site.build().unwrap());
}

#[bench]
fn bench_rendering_medium_blog(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push("medium-blog");
    let mut site = Site::new(&path, "config.toml").unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.load().unwrap();

    b.iter(|| site.build().unwrap());
}

//#[bench]
//fn bench_rendering_big_blog(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("big-blog");
//    let mut site = Site::new(&path, "config.toml").unwrap();
//    let tmp_dir = tempdir().expect("create temp dir");
//    let public = &tmp_dir.path().join("public");
//    site.set_output_path(&public);
//    site.load().unwrap();
//
//    b.iter(|| site.build().unwrap());
//}

#[bench]
fn bench_rendering_small_kb(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push("small-kb");
    let mut site = Site::new(&path, "config.toml").unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.load().unwrap();

    b.iter(|| site.build().unwrap());
}

#[bench]
fn bench_rendering_medium_kb(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push("medium-kb");
    let mut site = Site::new(&path, "config.toml").unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.load().unwrap();

    b.iter(|| site.build().unwrap());
}

