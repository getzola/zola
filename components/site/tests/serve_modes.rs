//! What each `BuildMode` puts where.
//!
//! `zola serve` keeps rendered HTML in the `SITE_CONTENT` global map, which
//! costs as much memory as the site's entire output — 9.2 GB on a site with
//! 9 GB of HTML, against 0.5 GB to build it (PERF-016). `--store-html` exists
//! to serve from disk instead, and it is only worth anything if the in-memory
//! copy is genuinely not made.
//!
//! `SITE_CONTENT` is a process-wide global, so this file holds exactly one test
//! and drives the modes in sequence. Adding a second `#[test]` here would race
//! it: cargo runs tests within a target on threads of one process.

use fs_err as fs;
use site::{BuildMode, SITE_CONTENT, Site};
use tempfile::tempdir;

fn scaffold(root: &std::path::Path) {
    fs::create_dir_all(root.join("content/blog")).unwrap();
    fs::create_dir_all(root.join("templates")).unwrap();
    fs::write(root.join("config.toml"), "base_url = \"https://example.com\"\n").unwrap();
    fs::write(root.join("templates/index.html"), "<h1>{{ section.title }}</h1>").unwrap();
    fs::write(root.join("templates/section.html"), "<h1>{{ section.title }}</h1>").unwrap();
    fs::write(root.join("templates/page.html"), "<h1>{{ page.title }}</h1>").unwrap();
    fs::write(root.join("content/_index.md"), "+++\n+++\n").unwrap();
    fs::write(root.join("content/blog/_index.md"), "+++\ntitle = \"Blog\"\n+++\n").unwrap();
    fs::write(root.join("content/blog/one.md"), "+++\ntitle = \"One\"\n+++\nBody.\n").unwrap();
}

fn build_in(mode: BuildMode) -> (usize, bool) {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path();
    scaffold(root);

    SITE_CONTENT.write().unwrap().clear();

    let mut site = Site::new(root, root.join("config.toml")).unwrap();
    site.enable_serve_mode(mode);
    site.load().unwrap();
    let public = root.join("public");
    site.set_output_path(&public);
    site.build().expect("build");

    let in_memory = SITE_CONTENT.read().unwrap().len();
    let on_disk = public.join("blog/one/index.html").is_file();
    SITE_CONTENT.write().unwrap().clear();
    (in_memory, on_disk)
}

#[test]
fn disk_mode_writes_the_html_without_also_keeping_it_in_memory() {
    let (memory_held, memory_on_disk) = build_in(BuildMode::Memory);
    assert!(memory_held > 0, "serve's default mode should hold rendered HTML in memory");
    assert!(!memory_on_disk, "serve's default mode should not write HTML to disk");

    let (disk_held, disk_on_disk) = build_in(BuildMode::Disk);
    assert!(disk_on_disk, "disk mode should write the HTML");
    assert_eq!(
        disk_held, 0,
        "disk mode kept {disk_held} entries in SITE_CONTENT — the whole point of \
         --store-html is that the site's output is not also held in memory"
    );

    // `Both` is what `--store-html` used to select: it pays for the write and
    // keeps the copy anyway. Asserted so the distinction stays visible.
    let (both_held, both_on_disk) = build_in(BuildMode::Both);
    assert!(both_on_disk && both_held > 0, "Both should do both");
}
