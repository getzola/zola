extern crate rebuild;
extern crate site;
extern crate tempfile;
extern crate fs_extra;
 
use std::env;
use std::fs::{remove_dir_all, File};
use std::io::prelude::*;

use fs_extra::dir;
use tempfile::tempdir;
use site::Site;

use rebuild::after_content_change;

// Loads the test_site in a tempdir and build it there
// Returns (site_path_in_tempdir, site)
macro_rules! load_and_build_site {
    ($tmp_dir: expr) => {
        {
            let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
            path.push("test_site");
            let mut options = dir::CopyOptions::new();
            options.copy_inside = true;
            dir::copy(&path, &$tmp_dir, &options).unwrap();

            let site_path = $tmp_dir.path().join("test_site");
            // delete useless sections for those tests
            remove_dir_all(site_path.join("content").join("paginated")).unwrap();
            remove_dir_all(site_path.join("content").join("posts")).unwrap();

            let mut site = Site::new(&site_path, "config.toml").unwrap();
            site.load().unwrap();
            let public = &site_path.join("public");
            site.set_output_path(&public);
            site.build().unwrap();

            (site_path, site)
        }
    }
}

/// Replace the file at the path (starting from root) by the given content
/// and return the file path that was modified
macro_rules! edit_file {
    ($site_path: expr, $path: expr, $content: expr) => {
        {
            let mut t = $site_path.clone();
            for c in $path.split('/') {
                t.push(c);
            }
            let mut file = File::create(&t).expect("Could not open/create file");
            file.write_all($content).expect("Could not write to the file");
            t
        }
    }
}

macro_rules! file_contains {
    ($site_path: expr, $path: expr, $text: expr) => {
        {
            let mut path = $site_path.clone();
            for component in $path.split("/") {
                path.push(component);
            }
            let mut file = File::open(&path).unwrap();
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            println!("{:?} -> {}", path, s);
            s.contains($text)
        }
    }
}

#[test]
fn can_rebuild_after_simple_change_to_page_content() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir);
    let file_path = edit_file!(site_path, "content/rebuild/first.md", br#"
+++
title = "first"
weight = 1
date = 2017-01-01
+++

Some content"#);

    let res = after_content_change(&mut site, &file_path);
    assert!(res.is_ok());
    assert!(file_contains!(site_path, "public/rebuild/first/index.html", "<p>Some content</p>"));
}

#[test]
fn can_rebuild_after_title_change_page_global_func_usage() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir);
    let file_path = edit_file!(site_path, "content/rebuild/first.md", br#"
+++
title = "Premier"
weight = 10
date = 2017-01-01
+++

# A title"#);

    let res = after_content_change(&mut site, &file_path);
    assert!(res.is_ok());
    assert!(file_contains!(site_path, "public/rebuild/index.html", "<h1>Premier</h1>"));
}

#[test]
fn can_rebuild_after_sort_change_in_section() {
    let tmp_dir = tempdir().expect("create temp dir");
    let (site_path, mut site) = load_and_build_site!(tmp_dir);
    let file_path = edit_file!(site_path, "content/rebuild/_index.md", br#"
+++
paginate_by = 1
sort_by = "weight"
template = "rebuild.html"
+++
"#);

    let res = after_content_change(&mut site, &file_path);
    assert!(res.is_ok());
    assert!(file_contains!(site_path, "public/rebuild/index.html", "<h1>first</h1><h1>second</h1>"));
}
