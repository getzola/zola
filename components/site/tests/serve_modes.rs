//! What each `BuildMode` puts where.
//!
//! `zola serve` keeps rendered HTML in the `SITE_CONTENT` global map, which used
//! to cost as much memory as the site's entire output — 9.4 GB on a site with
//! 9 GB of HTML, against 0.5 GB to build it (PERF-016). Two things address that
//! and both need holding to: the map compresses what it stores, and
//! `--store-html` does not populate it at all. Each is only worth something if
//! the bytes genuinely are not held.
//!
//! `SITE_CONTENT` is a process-wide global, so this file holds exactly one test
//! and drives the modes in sequence. Adding a second `#[test]` here would race
//! it: cargo runs tests within a target on threads of one process.

use fs_err as fs;
use site::{BuildMode, Site, site_content_clear, site_content_stats};
use tempfile::tempdir;

fn scaffold(root: &std::path::Path) {
    fs::create_dir_all(root.join("content/blog")).unwrap();
    fs::create_dir_all(root.join("templates")).unwrap();
    fs::write(root.join("config.toml"), "base_url = \"https://example.com\"\n").unwrap();
    fs::write(root.join("templates/index.html"), "<h1>{{ section.title }}</h1>").unwrap();
    fs::write(root.join("templates/section.html"), "<h1>{{ section.title }}</h1>").unwrap();
    // A page whose body repeats, which is what a real site's navigation is:
    // the reference site's nav is 88% of every page, and that self-similarity is
    // where the compression ratio comes from.
    fs::write(
        root.join("templates/page.html"),
        "<h1>{{ page.title }}</h1>{% for i in range(end=400) %}\
         <li class=\"nav-item\"><a href=\"/somewhere/or/other/\">A navigation entry</a></li>\
         {% endfor %}",
    )
    .unwrap();
    fs::write(root.join("content/_index.md"), "+++\n+++\n").unwrap();
    fs::write(root.join("content/blog/_index.md"), "+++\ntitle = \"Blog\"\n+++\n").unwrap();
    fs::write(root.join("content/blog/one.md"), "+++\ntitle = \"One\"\n+++\nBody.\n").unwrap();
}

/// Returns (entries held, bytes held, whether the HTML reached disk).
fn build_in(mode: BuildMode) -> (usize, usize, bool) {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path();
    scaffold(root);

    site_content_clear();

    let mut site = Site::new(root, root.join("config.toml")).unwrap();
    site.enable_serve_mode(mode);
    site.load().unwrap();
    let public = root.join("public");
    site.set_output_path(&public);
    site.build().expect("build");

    let (in_memory, bytes) = site_content_stats();
    let on_disk = public.join("blog/one/index.html").is_file();
    let raw =
        fs::read_to_string(public.join("blog/one/index.html")).map(|s| s.len()).unwrap_or_default();
    site_content_clear();
    (in_memory, if on_disk { bytes * 1000 / raw.max(1) } else { bytes }, on_disk)
}

#[test]
fn each_build_mode_holds_only_what_it_should() {
    let (memory_held, memory_bytes, memory_on_disk) = build_in(BuildMode::Memory);
    assert!(memory_held > 0, "serve's default mode should hold rendered HTML in memory");
    assert!(!memory_on_disk, "serve's default mode should not write HTML to disk");

    let (disk_held, _, disk_on_disk) = build_in(BuildMode::Disk);
    assert!(disk_on_disk, "disk mode should write the HTML");
    assert_eq!(
        disk_held, 0,
        "disk mode kept {disk_held} entries in SITE_CONTENT — the whole point of \
         --store-html is that the site's output is not also held in memory"
    );

    // `Both` is what `--store-html` used to select: it pays for the write and
    // keeps the copy anyway. Asserted so the distinction stays visible.
    let (both_held, both_bytes, both_on_disk) = build_in(BuildMode::Both);
    assert!(both_on_disk && both_held > 0, "Both should do both");

    // What is held must be compressed. `both_bytes` is the held size scaled
    // against the same page on disk, per mille: anything near 1000 would mean
    // the map is storing the HTML verbatim.
    assert!(
        both_bytes < 500,
        "held bytes are {}‰ of the uncompressed page — SITE_CONTENT is not \
         compressing, and serving a large site will cost its whole output in RAM",
        both_bytes
    );
    assert!(memory_bytes > 0, "memory mode should hold something");
}
