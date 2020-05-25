//! Benchmarking loading/markdown rendering of generated sites of various sizes
#![feature(test)]
extern crate test;

use std::env;

use site::Site;

#[bench]
fn bench_loading_small_blog(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push("small-blog");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();

    b.iter(|| site.load().unwrap());
}

#[bench]
fn bench_loading_small_blog_with_syntax_highlighting(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push("small-blog");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();
    site.config.highlight_code = true;

    b.iter(|| site.load().unwrap());
}

//#[bench]
//fn bench_loading_medium_blog(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("medium-blog");
//    let config_file = path.join("config.toml");
//    let mut site = Site::new(&path, &config_file).unwrap();
//
//    b.iter(|| site.load().unwrap());
//}
//
//#[bench]
//fn bench_loading_medium_blog_with_syntax_highlighting(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("medium-blog");
//    let config_file = path.join("config.toml");
//    let mut site = Site::new(&path, &config_file).unwrap();
//    site.config.highlight_code = true;
//
//    b.iter(|| site.load().unwrap());
//}
//
//#[bench]
//fn bench_loading_big_blog(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("big-blog");
//    let config_file = path.join("config.toml");
//    let mut site = Site::new(&path, &config_file).unwrap();
//
//    b.iter(|| site.load().unwrap());
//}
//
//#[bench]
//fn bench_loading_big_blog_with_syntax_highlighting(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("big-blog");
//    let config_file = path.join("config.toml");
//    let mut site = Site::new(&path, &config_file).unwrap();
//    site.config.highlight_code = true;
//
//    b.iter(|| site.load().unwrap());
//}

//#[bench]
//fn bench_loading_huge_blog(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("huge-blog");
//    let config_file = path.join("config.toml");
//    let mut site = Site::new(&path, &config_file).unwrap();
//
//    b.iter(|| site.load().unwrap());
//}
//
//#[bench]
//fn bench_loading_huge_blog_with_syntax_highlighting(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("huge-blog");
//    let config_file = path.join("config.toml");
//    let mut site = Site::new(&path, &config_file).unwrap();
//    site.config.highlight_code = true;
//
//    b.iter(|| site.load().unwrap());
//}

#[bench]
fn bench_loading_small_kb(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push("small-kb");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();

    b.iter(|| site.load().unwrap());
}

#[bench]
fn bench_loading_small_kb_with_syntax_highlighting(b: &mut test::Bencher) {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("benches");
    path.push("small-kb");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();
    site.config.highlight_code = true;

    b.iter(|| site.load().unwrap());
}

//#[bench]
//fn bench_loading_medium_kb(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("medium-kb");
//    let config_file = path.join("config.toml");
//    let mut site = Site::new(&path, &config_file).unwrap();
//
//    b.iter(|| site.load().unwrap());
//}
//
//#[bench]
//fn bench_loading_medium_kb_with_syntax_highlighting(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("medium-kb");
//    let config_file = path.join("config.toml");
//    let mut site = Site::new(&path, &config_file).unwrap();
//    site.config.highlight_code = Some(true);
//
//    b.iter(|| site.load().unwrap());
//}

//#[bench]
//fn bench_loading_huge_kb(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("huge-kb");
//    let config_file = path.join("config.toml");
//    let mut site = Site::new(&path, &config_file).unwrap();
//
//    b.iter(|| site.load().unwrap());
//}
//
//#[bench]
//fn bench_loading_huge_kb_with_syntax_highlighting(b: &mut test::Bencher) {
//    let mut path = env::current_dir().unwrap().to_path_buf();
//    path.push("benches");
//    path.push("huge-kb");
//    let config_file = path.join("config.toml");
//    let mut site = Site::new(&path, &config_file).unwrap();
//    site.config.highlight_code = Some(true);
//
//    b.iter(|| site.load().unwrap());
//}
