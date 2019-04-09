extern crate fs_extra;
extern crate rebuild;
extern crate site;
extern crate tempfile;

use std::env;
use std::fs::{self, File};
use std::io::prelude::*;

use fs_extra::dir;
use site::Site;
use tempfile::tempdir;

use rebuild::{after_content_change, after_content_rename};

// Loads the test_site in a tempdir and build it there
// Returns (site_path_in_tempdir, site)
macro_rules! load_and_build_site {
    ($tmp_dir: expr, $site: expr) => {{
        let mut path =
            env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
        path.push($site);
        let mut options = dir::CopyOptions::new();
        options.copy_inside = true;
        dir::copy(&path, &$tmp_dir, &options).unwrap();

        let site_path = $tmp_dir.path().join($site);
        let mut site = Site::new(&site_path, "config.toml").unwrap();
        site.load().unwrap();
        let public = &site_path.join("public");
        site.set_output_path(&public);
        site.build().unwrap();

        (site_path, site)
    }};
}

/// Replace the file at the path (starting from root) by the given content
/// and return the file path that was modified
macro_rules! edit_file {
    ($site_path: expr, $path: expr, $content: expr) => {{
        let mut t = $site_path.clone();
        for c in $path.split('/') {
            t.push(c);
        }
        let mut file = File::create(&t).expect("Could not open/create file");
        file.write_all($content).expect("Could not write to the file");
        t
    }};
}

macro_rules! file_contains {
    ($site_path: expr, $path: expr, $text: expr) => {{
        let mut path = $site_path.clone();
        for component in $path.split("/") {
            path.push(component);
        }
        let mut file = File::open(&path).unwrap();
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        println!("{:?} -> {}", path, s);
        s.contains($text)
    }};
}

/// Rename a file or a folder to the new given name
macro_rules! rename {
    ($site_path: expr, $path: expr, $new_name: expr) => {{
        let mut t = $site_path.clone();
        for c in $path.split('/') {
            t.push(c);
        }
        let mut new_path = t.parent().unwrap().to_path_buf();
        new_path.push($new_name);
        fs::rename(&t, &new_path).unwrap();
        println!("Renamed {:?} to {:?}", t, new_path);
        (t, new_path)
    }};
}

#[test]
fn can_rebuild_after_simple_change_to_page_content() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site");
    let file_path = edit_file!(
        site_path,
        "content/rebuild/first.md",
        br#"
+++
title = "first"
weight = 1
date = 2017-01-01
+++

Some content"#
    );

    let res = after_content_change(&mut site, &file_path);
    assert!(res.is_ok());
    assert!(file_contains!(site_path, "public/rebuild/first/index.html", "<p>Some content</p>"));
}

#[test]
fn can_rebuild_after_title_change_page_global_func_usage() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site");
    let file_path = edit_file!(
        site_path,
        "content/rebuild/first.md",
        br#"
+++
title = "Premier"
weight = 10
date = 2017-01-01
+++

# A title"#
    );

    let res = after_content_change(&mut site, &file_path);
    assert!(res.is_ok());
    assert!(file_contains!(site_path, "public/rebuild/index.html", "<h1>Premier</h1>"));
}

#[test]
fn can_rebuild_after_sort_change_in_section() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site");
    let file_path = edit_file!(
        site_path,
        "content/rebuild/_index.md",
        br#"
+++
paginate_by = 1
sort_by = "weight"
template = "rebuild.html"
+++
"#
    );

    let res = after_content_change(&mut site, &file_path);
    assert!(res.is_ok());
    assert!(file_contains!(
        site_path,
        "public/rebuild/index.html",
        "<h1>first</h1><h1>second</h1>"
    ));
}

#[test]
fn can_rebuild_after_transparent_change() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site");
    let file_path = edit_file!(
        site_path,
        "content/posts/2018/_index.md",
        br#"
+++
transparent = false
render = false
+++
"#
    );
    // Also remove pagination from posts section so we check whether the transparent page title
    // is there or not without dealing with pagination
    edit_file!(
        site_path,
        "content/posts/_index.md",
        br#"
+++
template = "section.html"
insert_anchor_links = "left"
+++
"#
    );

    let res = after_content_change(&mut site, &file_path);
    assert!(res.is_ok());
    assert!(!file_contains!(site_path, "public/posts/index.html", "A transparent page"));
}

#[test]
fn can_rebuild_after_renaming_page() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site");
    let (old_path, new_path) = rename!(site_path, "content/posts/simple.md", "hard.md");

    let res = after_content_rename(&mut site, &old_path, &new_path);
    println!("{:?}", res);
    assert!(res.is_ok());
    assert!(file_contains!(site_path, "public/posts/hard/index.html", "A simple page"));
}

// https://github.com/Keats/gutenberg/issues/385
#[test]
fn can_rebuild_after_renaming_colocated_asset_folder() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site");
    let (old_path, new_path) =
        rename!(site_path, "content/posts/with-assets", "with-assets-updated");
    assert!(file_contains!(site_path, "content/posts/with-assets-updated/index.md", "Hello"));

    let res = after_content_rename(&mut site, &old_path, &new_path);
    println!("{:?}", res);
    assert!(res.is_ok());
    assert!(file_contains!(
        site_path,
        "public/posts/with-assets-updated/index.html",
        "Hello world"
    ));
}

// https://github.com/Keats/gutenberg/issues/385
#[test]
fn can_rebuild_after_renaming_section_folder() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site");
    let (old_path, new_path) = rename!(site_path, "content/posts", "new-posts");
    assert!(file_contains!(site_path, "content/new-posts/simple.md", "simple"));

    let res = after_content_rename(&mut site, &old_path, &new_path);
    assert!(res.is_ok());

    assert!(file_contains!(site_path, "public/new-posts/simple/index.html", "simple"));
}

#[test]
fn can_rebuild_after_renaming_non_md_asset_in_colocated_folder() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site");
    let (old_path, new_path) =
        rename!(site_path, "content/posts/with-assets/zola.png", "gutenberg.png");

    // Testing that we don't try to load some images as markdown or something
    let res = after_content_rename(&mut site, &old_path, &new_path);
    assert!(res.is_ok());
}

#[test]
fn can_rebuild_after_deleting_file() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site");
    let path = site_path.join("content").join("posts").join("fixed-slug.md");
    fs::remove_file(&path).unwrap();

    let res = after_content_change(&mut site, &path);
    println!("{:?}", res);
    assert!(res.is_ok());
}

#[test]
fn can_rebuild_after_editing_in_colocated_asset_folder_with_language() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site_i18n");
    let file_path = edit_file!(
        site_path,
        "content/blog/with-assets/index.fr.md",
        br#"
+++
date = 2018-11-11
+++

Edite
"#
    );

    let res = after_content_change(&mut site, &file_path);
    println!("{:?}", res);
    assert!(res.is_ok());
    assert!(file_contains!(site_path, "public/fr/blog/with-assets/index.html", "Edite"));
}

// https://github.com/getzola/zola/issues/620
#[test]
fn can_rebuild_after_renaming_section_and_deleting_file() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir, "test_site");
    let (old_path, new_path) = rename!(site_path, "content/posts/", "post/");
    let res = after_content_rename(&mut site, &old_path, &new_path);
    assert!(res.is_ok());

    let path = site_path.join("content").join("_index.md");
    fs::remove_file(&path).unwrap();

    let res = after_content_change(&mut site, &path);
    println!("{:?}", res);
    assert!(res.is_ok());
}
