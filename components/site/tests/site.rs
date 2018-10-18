extern crate site;
extern crate tempfile;

use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

use tempfile::tempdir;
use site::Site;


#[test]
fn can_parse_site() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();

    // Correct number of pages (sections are pages too)
    assert_eq!(site.library.pages().len(), 15);
    let posts_path = path.join("content").join("posts");

    // Make sure the page with a url doesn't have any sections
    let url_post = site.library.get_page(&posts_path.join("fixed-url.md")).unwrap();
    assert_eq!(url_post.path, "a-fixed-url/");

    // Make sure the article in a folder with only asset doesn't get counted as a section
    let asset_folder_post = site.library.get_page(&posts_path.join("with-assets").join("index.md")).unwrap();
    assert_eq!(asset_folder_post.file.components, vec!["posts".to_string()]);

    // That we have the right number of sections
    assert_eq!(site.library.sections().len(), 7);

    // And that the sections are correct
    let index_section = site.library.get_section(&path.join("content").join("_index.md")).unwrap();
    assert_eq!(index_section.subsections.len(), 3);
    assert_eq!(index_section.pages.len(), 1);
    assert!(index_section.ancestors.is_empty());

    let posts_section = site.library.get_section(&posts_path.join("_index.md")).unwrap();
    assert_eq!(posts_section.subsections.len(), 1);
    assert_eq!(posts_section.pages.len(), 7);
    assert_eq!(posts_section.ancestors, vec![*site.library.get_section_key(&index_section.file.path).unwrap()]);

    // Make sure we remove all the pwd + content from the sections
    let basic = site.library.get_page(&posts_path.join("simple.md")).unwrap();
    assert_eq!(basic.file.components, vec!["posts".to_string()]);
    assert_eq!(
        basic.ancestors,
        vec![
            *site.library.get_section_key(&index_section.file.path).unwrap(),
            *site.library.get_section_key(&posts_section.file.path).unwrap(),
        ]
    );

    let tutorials_section = site.library.get_section(&posts_path.join("tutorials").join("_index.md")).unwrap();
    assert_eq!(tutorials_section.subsections.len(), 2);
    let sub1 = site.library.get_section_by_key(tutorials_section.subsections[0]);
    let sub2 = site.library.get_section_by_key(tutorials_section.subsections[1]);
    assert_eq!(sub1.clone().meta.title.unwrap(), "Programming");
    assert_eq!(sub2.clone().meta.title.unwrap(), "DevOps");
    assert_eq!(tutorials_section.pages.len(), 0);

    let devops_section = site.library.get_section(&posts_path.join("tutorials").join("devops").join("_index.md")).unwrap();
    assert_eq!(devops_section.subsections.len(), 0);
    assert_eq!(devops_section.pages.len(), 2);
    assert_eq!(
        devops_section.ancestors,
        vec![
            *site.library.get_section_key(&index_section.file.path).unwrap(),
            *site.library.get_section_key(&posts_section.file.path).unwrap(),
            *site.library.get_section_key(&tutorials_section.file.path).unwrap(),
        ]
    );

    let prog_section = site.library.get_section(&posts_path.join("tutorials").join("programming").join("_index.md")).unwrap();
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
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(&public.exists());
    assert!(file_exists!(public, "index.html"));
    assert!(file_exists!(public, "sitemap.xml"));
    assert!(file_exists!(public, "robots.txt"));
    assert!(file_exists!(public, "a-fixed-url/index.html"));

    assert!(file_exists!(public, "posts/python/index.html"));
    // Shortcodes work
    assert!(file_contains!(public, "posts/python/index.html", "Basic shortcode"));
    assert!(file_contains!(public, "posts/python/index.html", "Arrrh Bob"));
    assert!(file_contains!(public, "posts/python/index.html", "Arrrh Bob_Sponge"));
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

    // Pages and section get their relative path
    assert!(file_contains!(public, "posts/tutorials/index.html", "posts/tutorials/_index.md"));
    assert!(file_contains!(public, "posts/tutorials/devops/nix/index.html", "posts/tutorials/devops/nix.md"));

    // aliases work
    assert!(file_exists!(public, "an-old-url/old-page/index.html"));
    assert!(file_contains!(public, "an-old-url/old-page/index.html", "something-else"));

    // html aliases work
    assert!(file_exists!(public, "an-old-url/an-old-alias.html"));
    assert!(file_contains!(public, "an-old-url/an-old-alias.html", "something-else"));

    // redirect_to works
    assert!(file_exists!(public, "posts/tutorials/devops/index.html"));
    assert!(file_contains!(public, "posts/tutorials/devops/index.html", "docker"));

    // No tags or categories
    assert_eq!(file_exists!(public, "categories/index.html"), false);
    assert_eq!(file_exists!(public, "tags/index.html"), false);

    // Theme files are there
    assert!(file_exists!(public, "sample.css"));
    assert!(file_exists!(public, "some.js"));

    // SASS and SCSS files compile correctly
    assert!(file_exists!(public, "blog.css"));
    assert!(file_contains!(public, "blog.css", "red"));
    assert!(file_contains!(public, "blog.css", "blue"));
    assert!(!file_contains!(public, "blog.css", "@import \"included\""));
    assert!(file_contains!(public, "blog.css", "2rem")); // check include
    assert!(!file_exists!(public, "_included.css"));
    assert!(file_exists!(public, "scss.css"));
    assert!(file_exists!(public, "sass.css"));
    assert!(file_exists!(public, "nested_sass/sass.css"));
    assert!(file_exists!(public, "nested_sass/scss.css"));

    // no live reload code
    assert_eq!(file_contains!(public, "index.html", "/livereload.js?port=1112&mindelay=10"), false);

    // Both pages and sections are in the sitemap
    assert!(file_contains!(public, "sitemap.xml", "<loc>https://replace-this-with-your-url.com/posts/simple/</loc>"));
    assert!(file_contains!(public, "sitemap.xml", "<loc>https://replace-this-with-your-url.com/posts/</loc>"));
    // Drafts are not in the sitemap
    assert!(!file_contains!(public, "sitemap.xml", "draft"));

    // robots.txt has been rendered from the template
    assert!(file_contains!(public, "robots.txt", "User-agent: zola"));
    assert!(file_contains!(public, "robots.txt", "Sitemap: https://replace-this-with-your-url.com/sitemap.xml"));
}

#[test]
fn can_build_site_with_live_reload() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
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
    assert!(file_contains!(public, "index.html", "/livereload.js"));

    // the summary anchor link has been created
    assert!(file_contains!(public, "posts/python/index.html", r#"<a name="continue-reading"></a>"#));
    assert!(file_contains!(public, "posts/draft/index.html", r#"THEME_SHORTCODE"#));
}

#[test]
fn can_build_site_with_taxonomies() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();

    for (i, (_, page)) in site.library.pages_mut().iter_mut().enumerate() {
        page.meta.taxonomies = {
            let mut taxonomies = HashMap::new();
            taxonomies.insert("categories".to_string(), vec![if i % 2 == 0 { "A" } else { "B" }.to_string()]);
            taxonomies
        };
    }
    site.populate_taxonomies().unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());
    assert_eq!(site.taxonomies.len(), 1);

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

    // Categories are there
    assert!(file_exists!(public, "categories/index.html"));
    assert!(file_exists!(public, "categories/a/index.html"));
    assert!(file_exists!(public, "categories/b/index.html"));
    assert!(file_exists!(public, "categories/a/rss.xml"));
    assert!(file_contains!(public, "categories/a/rss.xml", "https://replace-this-with-your-url.com/categories/a/rss.xml"));
    // Extending from a theme works
    assert!(file_contains!(public, "categories/a/index.html", "EXTENDED"));
    // Tags aren't
    assert_eq!(file_exists!(public, "tags/index.html"), false);

    // Categories are in the sitemap
    assert!(file_contains!(public, "sitemap.xml", "<loc>https://replace-this-with-your-url.com/categories/</loc>"));
    assert!(file_contains!(public, "sitemap.xml", "<loc>https://replace-this-with-your-url.com/categories/a/</loc>"));
}

#[test]
fn can_build_site_and_insert_anchor_links() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();

    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());
    // anchor link inserted
    assert!(file_contains!(public, "posts/something-else/index.html", "<h1 id=\"title\"><a class=\"zola-anchor\" href=\"#title\""));
}

#[test]
fn can_build_site_with_pagination_for_section() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    for (_, section) in site.library.sections_mut() {
        if section.is_index() {
            continue;
        }
        section.meta.paginate_by = Some(2);
        section.meta.template = Some("section_paginated.html".to_string());
    }
    let tmp_dir = tempdir().expect("create temp dir");
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
    assert!(file_contains!(public, "posts/index.html", "Num pagers: 4"));
    assert!(file_contains!(public, "posts/index.html", "Page size: 2"));
    assert!(file_contains!(public, "posts/index.html", "Current index: 1"));
    assert!(!file_contains!(public, "posts/index.html", "has_prev"));
    assert!(file_contains!(public, "posts/index.html", "has_next"));
    assert!(file_contains!(public, "posts/index.html", "First: https://replace-this-with-your-url.com/posts/"));
    assert!(file_contains!(public, "posts/index.html", "Last: https://replace-this-with-your-url.com/posts/page/4/"));
    assert_eq!(file_contains!(public, "posts/index.html", "has_prev"), false);

    assert!(file_exists!(public, "posts/page/2/index.html"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Num pagers: 4"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Page size: 2"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Current index: 2"));
    assert!(file_contains!(public, "posts/page/2/index.html", "has_prev"));
    assert!(file_contains!(public, "posts/page/2/index.html", "has_next"));
    assert!(file_contains!(public, "posts/page/2/index.html", "First: https://replace-this-with-your-url.com/posts/"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Last: https://replace-this-with-your-url.com/posts/page/4/"));

    assert!(file_exists!(public, "posts/page/3/index.html"));
    assert!(file_contains!(public, "posts/page/3/index.html", "Num pagers: 4"));
    assert!(file_contains!(public, "posts/page/3/index.html", "Page size: 2"));
    assert!(file_contains!(public, "posts/page/3/index.html", "Current index: 3"));
    assert!(file_contains!(public, "posts/page/3/index.html", "has_prev"));
    assert!(file_contains!(public, "posts/page/3/index.html", "has_next"));
    assert!(file_contains!(public, "posts/page/3/index.html", "First: https://replace-this-with-your-url.com/posts/"));
    assert!(file_contains!(public, "posts/page/3/index.html", "Last: https://replace-this-with-your-url.com/posts/page/4/"));

    assert!(file_exists!(public, "posts/page/4/index.html"));
    assert!(file_contains!(public, "posts/page/4/index.html", "Num pagers: 4"));
    assert!(file_contains!(public, "posts/page/4/index.html", "Page size: 2"));
    assert!(file_contains!(public, "posts/page/4/index.html", "Current index: 4"));
    assert!(file_contains!(public, "posts/page/4/index.html", "has_prev"));
    assert!(!file_contains!(public, "posts/page/4/index.html", "has_next"));
    assert!(file_contains!(public, "posts/page/4/index.html", "First: https://replace-this-with-your-url.com/posts/"));
    assert!(file_contains!(public, "posts/page/4/index.html", "Last: https://replace-this-with-your-url.com/posts/page/4/"));
}

#[test]
fn can_build_site_with_pagination_for_index() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    {
        let index = site.library.get_section_mut(&path.join("content").join("_index.md")).unwrap();
        index.meta.paginate_by = Some(2);
        index.meta.template = Some("index_paginated.html".to_string());
    }
    let tmp_dir = tempdir().expect("create temp dir");
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
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());
    assert!(file_exists!(public, "rss.xml"));
    // latest article is posts/extra-syntax.md
    assert!(file_contains!(public, "rss.xml", "Extra Syntax"));
    // Next is posts/simple.md
    assert!(file_contains!(public, "rss.xml", "Simple article with shortcodes"));
}


#[test]
fn can_build_search_index() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    site.config.build_search_index = true;
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(Path::new(&public).exists());
    assert!(file_exists!(public, "elasticlunr.min.js"));
    assert!(file_exists!(public, "search_index.en.js"));
}

#[test]
fn can_build_with_extra_syntaxes() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().unwrap();

    assert!(&public.exists());
    assert!(file_exists!(public, "posts/extra-syntax/index.html"));
    assert!(file_contains!(public, "posts/extra-syntax/index.html",
        r#"<span style="color:#d08770;">test</span>"#));
}
