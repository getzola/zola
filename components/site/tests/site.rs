extern crate site;
extern crate front_matter;
extern crate tempdir;

use std::env;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use tempdir::TempDir;
use site::Site;


#[test]
fn can_parse_site() {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();

    // Correct number of pages (sections are pages too)
    assert_eq!(site.pages.len(), 12);
    let posts_path = path.join("content").join("posts");

    // Make sure we remove all the pwd + content from the sections
    let basic = &site.pages[&posts_path.join("simple.md")];
    assert_eq!(basic.file.components, vec!["posts".to_string()]);

    // Make sure the page with a url doesn't have any sections
    let url_post = &site.pages[&posts_path.join("fixed-url.md")];
    assert_eq!(url_post.path, "a-fixed-url/");

    // Make sure the article in a folder with only asset doesn't get counted as a section
    let asset_folder_post = &site.pages[&posts_path.join("with-assets").join("index.md")];
    assert_eq!(asset_folder_post.file.components, vec!["posts".to_string()]);

    // That we have the right number of sections
    assert_eq!(site.sections.len(), 6);

    // And that the sections are correct
    let index_section = &site.sections[&path.join("content").join("_index.md")];
    assert_eq!(index_section.subsections.len(), 2);
    assert_eq!(index_section.pages.len(), 1);

    let posts_section = &site.sections[&posts_path.join("_index.md")];
    assert_eq!(posts_section.subsections.len(), 1);
    assert_eq!(posts_section.pages.len(), 6);

    let tutorials_section = &site.sections[&posts_path.join("tutorials").join("_index.md")];
    assert_eq!(tutorials_section.subsections.len(), 2);
    assert_eq!(tutorials_section.subsections[0].clone().meta.title.unwrap(), "Programming");
    assert_eq!(tutorials_section.subsections[1].clone().meta.title.unwrap(), "DevOps");
    assert_eq!(tutorials_section.pages.len(), 0);

    let devops_section = &site.sections[&posts_path.join("tutorials").join("devops").join("_index.md")];
    assert_eq!(devops_section.subsections.len(), 0);
    assert_eq!(devops_section.pages.len(), 2);

    let prog_section = &site.sections[&posts_path.join("tutorials").join("programming").join("_index.md")];
    assert_eq!(prog_section.subsections.len(), 0);
    assert_eq!(prog_section.pages.len(), 2);
}

// 2 helper macros to make all the build testing more bearable
macro_rules! file_exists {
    ($root: expr, $path: expr) => {
        {
            let mut path = $root.clone();
            for component in $path.split("/") {
                path = path.join(component);
            }
            Path::new(&path).exists()
        }
    }
}

macro_rules! file_contains {
    ($root: expr, $path: expr, $text: expr) => {
        {
            let mut path = $root.clone();
            for component in $path.split("/") {
                path = path.join(component);
            }
            let mut file = File::open(&path).unwrap();
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();
            println!("{}", s);
            s.contains($text)
        }
    }
}

#[test]
fn can_build_site_without_live_reload() {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    let tmp_dir = TempDir::new("example").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());
    assert!(file_exists!(public, "index.html"));
    assert!(file_exists!(public, "sitemap.xml"));
    assert!(file_exists!(public, "robots.txt"));
    assert!(file_exists!(public, "a-fixed-url/index.html"));

    assert!(file_exists!(public, "posts/python/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/nix/index.html"));
    assert!(file_exists!(public, "posts/with-assets/index.html"));
    assert!(file_exists!(public, "posts/no-section/simple/index.html"));

    // Sections
    assert!(file_exists!(public, "posts/index.html"));
    assert!(file_exists!(public, "posts/tutorials/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/index.html"));
    assert!(file_exists!(public, "posts/tutorials/programming/index.html"));
    // Ensure subsection pages are correctly filled
    assert!(file_contains!(public, "posts/tutorials/index.html", "Sub-pages: 2"));
    // TODO: add assertion for syntax highlighting

    // aliases work
    assert!(file_exists!(public, "an-old-url/old-page/index.html"));
    assert!(file_contains!(public, "an-old-url/old-page/index.html", "something-else"));

    // redirect_to works
    assert!(file_exists!(public, "posts/tutorials/devops/index.html"));
    assert!(file_contains!(public, "posts/tutorials/devops/index.html", "docker"));

    // No tags or categories
    assert_eq!(file_exists!(public, "categories/index.html"), false);
    assert_eq!(file_exists!(public, "tags/index.html"), false);

    // Theme files are there
    assert!(file_exists!(public, "sample.css"));
    assert!(file_exists!(public, "some.js"));

    // no live reload code
    assert_eq!(file_contains!(public, "index.html", "/livereload.js?port=1112&mindelay=10"), false);

    // Both pages and sections are in the sitemap
    assert!(file_contains!(public, "sitemap.xml", "<loc>https://replace-this-with-your-url.com/posts/simple/</loc>"));
    assert!(file_contains!(public, "sitemap.xml", "<loc>https://replace-this-with-your-url.com/posts/</loc>"));
    // Drafts are not in the sitemap
    assert!(!file_contains!(public, "sitemap.xml", "draft"));
}

#[test]
fn can_build_site_with_live_reload() {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    let tmp_dir = TempDir::new("example").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.enable_live_reload();
    site.build().unwrap();

    assert!(Path::new(&public).exists());

    assert!(file_exists!(public, "index.html"));
    assert!(file_exists!(public, "sitemap.xml"));
    assert!(file_exists!(public, "robots.txt"));
    assert!(file_exists!(public, "a-fixed-url/index.html"));

    assert!(file_exists!(public, "posts/python/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/nix/index.html"));
    assert!(file_exists!(public, "posts/with-assets/index.html"));

    // Sections
    assert!(file_exists!(public, "posts/index.html"));
    assert!(file_exists!(public, "posts/tutorials/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/index.html"));
    assert!(file_exists!(public, "posts/tutorials/programming/index.html"));
    // TODO: add assertion for syntax highlighting

    // No tags or categories
    assert_eq!(file_exists!(public, "categories/index.html"), false);
    assert_eq!(file_exists!(public, "tags/index.html"), false);

    // no live reload code
    assert!(file_contains!(public, "index.html", "/livereload.js?port=1112&mindelay=10"));
}

#[test]
fn can_build_site_with_categories() {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.config.generate_categories_pages = Some(true);
    site.load().unwrap();

    for (i, page) in site.pages.values_mut().enumerate() {
        page.meta.category = if i % 2 == 0 {
            Some("A".to_string())
        } else {
            Some("B".to_string())
        };
    }
    site.populate_tags_and_categories();
    let tmp_dir = TempDir::new("example").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());
    assert_eq!(site.categories.unwrap().len(), 2);

    assert!(file_exists!(public, "index.html"));
    assert!(file_exists!(public, "sitemap.xml"));
    assert!(file_exists!(public, "robots.txt"));
    assert!(file_exists!(public, "a-fixed-url/index.html"));

    assert!(file_exists!(public, "posts/python/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/nix/index.html"));
    assert!(file_exists!(public, "posts/with-assets/index.html"));

    // Sections
    assert!(file_exists!(public, "posts/index.html"));
    assert!(file_exists!(public, "posts/tutorials/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/index.html"));
    assert!(file_exists!(public, "posts/tutorials/programming/index.html"));
    // TODO: add assertion for syntax highlighting

    // Categories are there
    assert!(file_exists!(public, "categories/index.html"));
    assert!(file_exists!(public, "categories/a/index.html"));
    assert!(file_exists!(public, "categories/b/index.html"));
    // Extending from a theme works
    assert!(file_contains!(public, "categories/a/index.html", "EXTENDED"));
    // Tags aren't
    assert_eq!(file_exists!(public, "tags/index.html"), false);

    // Categories are in the sitemap
    assert!(file_contains!(public, "sitemap.xml", "<loc>https://replace-this-with-your-url.com/categories/</loc>"));
    assert!(file_contains!(public, "sitemap.xml", "<loc>https://replace-this-with-your-url.com/categories/a/</loc>"));
}

#[test]
fn can_build_site_with_tags() {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.config.generate_tags_pages = Some(true);
    site.load().unwrap();

    for (i, page) in site.pages.values_mut().enumerate() {
        page.meta.tags = if i % 2 == 0 {
            Some(vec!["tag1".to_string(), "tag2".to_string()])
        } else {
            Some(vec!["tag with space".to_string()])
        };
    }
    site.populate_tags_and_categories();

    let tmp_dir = TempDir::new("example").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());
    assert_eq!(site.tags.unwrap().len(), 3);

    assert!(file_exists!(public, "index.html"));
    assert!(file_exists!(public, "sitemap.xml"));
    assert!(file_exists!(public, "robots.txt"));
    assert!(file_exists!(public, "a-fixed-url/index.html"));
    assert!(file_exists!(public, "posts/python/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/nix/index.html"));
    assert!(file_exists!(public, "posts/with-assets/index.html"));

    // Sections
    assert!(file_exists!(public, "posts/index.html"));
    assert!(file_exists!(public, "posts/tutorials/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/index.html"));
    assert!(file_exists!(public, "posts/tutorials/programming/index.html"));
    // TODO: add assertion for syntax highlighting

    // Tags are there
    assert!(file_exists!(public, "tags/index.html"));
    assert!(file_exists!(public, "tags/tag1/index.html"));
    assert!(file_exists!(public, "tags/tag2/index.html"));
    assert!(file_exists!(public, "tags/tag-with-space/index.html"));
    // Categories aren't
    assert_eq!(file_exists!(public, "categories/index.html"), false);
    // Tags are in the sitemap
    assert!(file_contains!(public, "sitemap.xml", "<loc>https://replace-this-with-your-url.com/tags/</loc>"));
    assert!(file_contains!(public, "sitemap.xml", "<loc>https://replace-this-with-your-url.com/tags/tag-with-space/</loc>"));
}

#[test]
fn can_build_site_and_insert_anchor_links() {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();

    let tmp_dir = TempDir::new("example").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());
    // anchor link inserted
    assert!(file_contains!(public, "posts/something-else/index.html", "<h1 id=\"title\"><a class=\"gutenberg-anchor\" href=\"#title\""));
}

#[test]
fn can_build_site_with_pagination_for_section() {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    for section in site.sections.values_mut(){
        if section.is_index() {
            continue;
        }
        section.meta.paginate_by = Some(2);
        section.meta.template = Some("section_paginated.html".to_string());
    }
    let tmp_dir = TempDir::new("example").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());

    assert!(file_exists!(public, "index.html"));
    assert!(file_exists!(public, "sitemap.xml"));
    assert!(file_exists!(public, "robots.txt"));
    assert!(file_exists!(public, "a-fixed-url/index.html"));
    assert!(file_exists!(public, "posts/python/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/nix/index.html"));
    assert!(file_exists!(public, "posts/with-assets/index.html"));

    // Sections
    assert!(file_exists!(public, "posts/index.html"));
    // And pagination!
    assert!(file_exists!(public, "posts/page/1/index.html"));
    // even if there is no pages, only the section!
    assert!(file_exists!(public, "paginated/page/1/index.html"));
    assert!(file_exists!(public, "paginated/index.html"));
    // should redirect to posts/
    assert!(file_contains!(
        public,
        "posts/page/1/index.html",
        "http-equiv=\"refresh\" content=\"0;url=https://replace-this-with-your-url.com/posts/\""
    ));
    assert!(file_contains!(public, "posts/index.html", "Num pagers: 3"));
    assert!(file_contains!(public, "posts/index.html", "Page size: 2"));
    assert!(file_contains!(public, "posts/index.html", "Current index: 1"));
    assert!(file_contains!(public, "posts/index.html", "has_next"));
    assert!(file_contains!(public, "posts/index.html", "First: https://replace-this-with-your-url.com/posts/"));
    assert!(file_contains!(public, "posts/index.html", "Last: https://replace-this-with-your-url.com/posts/page/3/"));
    assert_eq!(file_contains!(public, "posts/index.html", "has_prev"), false);

    assert!(file_exists!(public, "posts/page/2/index.html"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Num pagers: 3"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Page size: 2"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Current index: 2"));
    assert!(file_contains!(public, "posts/page/2/index.html", "has_prev"));
    assert!(file_contains!(public, "posts/page/2/index.html", "has_next"));
    assert!(file_contains!(public, "posts/page/2/index.html", "First: https://replace-this-with-your-url.com/posts/"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Last: https://replace-this-with-your-url.com/posts/page/3/"));
}

#[test]
fn can_build_site_with_pagination_for_index() {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    {
        let index = site.sections.get_mut(&path.join("content").join("_index.md")).unwrap();
        index.meta.paginate_by = Some(2);
        index.meta.template = Some("index_paginated.html".to_string());
    }
    let tmp_dir = TempDir::new("example").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());

    assert!(file_exists!(public, "index.html"));
    assert!(file_exists!(public, "sitemap.xml"));
    assert!(file_exists!(public, "robots.txt"));
    assert!(file_exists!(public, "a-fixed-url/index.html"));
    assert!(file_exists!(public, "posts/python/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/nix/index.html"));
    assert!(file_exists!(public, "posts/with-assets/index.html"));

    // And pagination!
    assert!(file_exists!(public, "page/1/index.html"));
    // even if there is no pages, only the section!
    assert!(file_exists!(public, "paginated/page/1/index.html"));
    assert!(file_exists!(public, "paginated/index.html"));
    // should redirect to index
    assert!(file_contains!(
        public,
        "page/1/index.html",
        "http-equiv=\"refresh\" content=\"0;url=https://replace-this-with-your-url.com/\""
    ));
    assert!(file_contains!(public, "index.html", "Num pages: 1"));
    assert!(file_contains!(public, "index.html", "Current index: 1"));
    assert!(file_contains!(public, "index.html", "First: https://replace-this-with-your-url.com/"));
    assert!(file_contains!(public, "index.html", "Last: https://replace-this-with-your-url.com/"));
    assert_eq!(file_contains!(public, "index.html", "has_prev"), false);
    assert_eq!(file_contains!(public, "index.html", "has_next"), false);
}

#[test]
fn can_build_rss_feed() {
    let mut path = env::current_dir().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    let tmp_dir = TempDir::new("example").expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());
    assert!(file_exists!(public, "rss.xml"));
    // latest article is posts/simple.md
    assert!(file_contains!(public, "rss.xml", "Simple article with shortcodes"));
    // Next is posts/python.md
    assert!(file_contains!(public, "rss.xml", "Python in posts"));
}
