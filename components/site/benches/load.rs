//! Benchmarking loading/markdown of generated sites of various sizes
use std::env;

use config::{HighlightConfig, Highlighting};
use criterion::{Criterion, criterion_group, criterion_main};
use site::Site;

fn bench_loading_small_blog(c: &mut Criterion) {
    let mut path = env::current_dir().unwrap();
    path.push("benches");
    path.push("small-blog");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();

    c.bench_function("loading_small_blog", |b| b.iter(|| site.load().unwrap()));
}

fn bench_loading_small_blog_with_syntax_highlighting(c: &mut Criterion) {
    let mut path = env::current_dir().unwrap();
    path.push("benches");
    path.push("small-blog");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();

    let mut highlighting = Highlighting {
        error_on_missing_language: false,
        style: Default::default(),
        theme: HighlightConfig::Single { theme: "github-dark".to_string() },
        extra_grammars: vec![],
        extra_themes: vec![],
        registry: Default::default(),
    };
    highlighting.init(std::path::Path::new(".")).unwrap();
    site.config.markdown.highlighting = Some(highlighting);

    c.bench_function("loading_small_blog_with_syntax_highlighting", |b| {
        b.iter(|| site.load().unwrap())
    });
}

fn bench_loading_small_kb(c: &mut Criterion) {
    let mut path = env::current_dir().unwrap();
    path.push("benches");
    path.push("small-kb");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();

    c.bench_function("loading_small_kb", |b| b.iter(|| site.load().unwrap()));
}

fn bench_loading_small_kb_with_syntax_highlighting(c: &mut Criterion) {
    let mut path = env::current_dir().unwrap();
    path.push("benches");
    path.push("small-kb");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();

    let mut highlighting = Highlighting {
        error_on_missing_language: false,
        style: Default::default(),
        theme: HighlightConfig::Single { theme: "github-dark".to_string() },
        extra_grammars: vec![],
        extra_themes: vec![],
        registry: Default::default(),
    };
    highlighting.init(std::path::Path::new(".")).unwrap();
    site.config.markdown.highlighting = Some(highlighting);

    c.bench_function("loading_small_kb_with_syntax_highlighting", |b| {
        b.iter(|| site.load().unwrap())
    });
}

criterion_group!(
    benches,
    bench_loading_small_blog,
    bench_loading_small_blog_with_syntax_highlighting,
    bench_loading_small_kb,
    bench_loading_small_kb_with_syntax_highlighting,
);
criterion_main!(benches);
