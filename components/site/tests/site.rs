mod common;

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use common::{build_site, build_site_with_setup};
use config::TaxonomyConfig;
use content::Page;
use libs::ahash::AHashMap;
use site::sitemap;
use site::Site;

#[test]
fn can_parse_site() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();
    site.load().unwrap();
    let library = site.library.read().unwrap();

    // Correct number of pages (sections do not count as pages, draft are ignored)
    assert_eq!(library.pages.len(), 35);
    let posts_path = path.join("content").join("posts");

    // Make sure the page with a url doesn't have any sections
    let url_post = library.pages.get(&posts_path.join("fixed-url.md")).unwrap();
    assert_eq!(url_post.path, "/a-fixed-url/");

    // Make sure the article in a folder with only asset doesn't get counted as a section
    let asset_folder_post =
        library.pages.get(&posts_path.join("with-assets").join("index.md")).unwrap();
    assert_eq!(asset_folder_post.file.components, vec!["posts".to_string()]);

    // That we have the right number of sections
    assert_eq!(library.sections.len(), 13);

    // And that the sections are correct
    let index_section = library.sections.get(&path.join("content").join("_index.md")).unwrap();
    assert_eq!(index_section.subsections.len(), 5);
    assert_eq!(index_section.pages.len(), 5);
    assert!(index_section.ancestors.is_empty());

    let posts_section = library.sections.get(&posts_path.join("_index.md")).unwrap();
    assert_eq!(posts_section.subsections.len(), 2);
    assert_eq!(posts_section.pages.len(), 10); // 11 with 1 draft == 10
    assert_eq!(posts_section.ancestors, vec![index_section.file.relative.clone()]);

    // Make sure we remove all the pwd + content from the sections
    let basic = library.pages.get(&posts_path.join("simple.md")).unwrap();
    assert_eq!(basic.file.components, vec!["posts".to_string()]);
    assert_eq!(
        basic.ancestors,
        vec![index_section.file.relative.clone(), posts_section.file.relative.clone(),]
    );

    let tutorials_section =
        library.sections.get(&posts_path.join("tutorials").join("_index.md")).unwrap();
    assert_eq!(tutorials_section.subsections.len(), 2);
    let sub1 = &library.sections[&tutorials_section.subsections[0]];
    let sub2 = &library.sections[&tutorials_section.subsections[1]];
    assert_eq!(sub1.clone().meta.title.unwrap(), "Programming");
    assert_eq!(sub2.clone().meta.title.unwrap(), "DevOps");
    assert_eq!(tutorials_section.pages.len(), 0);

    let devops_section = library
        .sections
        .get(&posts_path.join("tutorials").join("devops").join("_index.md"))
        .unwrap();
    assert_eq!(devops_section.subsections.len(), 0);
    assert_eq!(devops_section.pages.len(), 2);
    assert_eq!(
        devops_section.ancestors,
        vec![
            index_section.file.relative.clone(),
            posts_section.file.relative.clone(),
            tutorials_section.file.relative.clone(),
        ]
    );

    let prog_section = library
        .sections
        .get(&posts_path.join("tutorials").join("programming").join("_index.md"))
        .unwrap();
    assert_eq!(prog_section.subsections.len(), 0);
    assert_eq!(prog_section.pages.len(), 2);

    // Testing extra variables in sections & sitemaps
    // Regression test for #https://github.com/getzola/zola/issues/842
    assert_eq!(
        prog_section.meta.extra.get("we_have_extra").and_then(|s| s.as_str()),
        Some("variables")
    );
    let sitemap_entries = sitemap::find_entries(&library, &site.taxonomies[..], &site.config);
    let sitemap_entry = sitemap_entries
        .iter()
        .find(|e| e.permalink.ends_with("tutorials/programming/"))
        .expect("expected to find programming section in sitemap");
    assert_eq!(Some(&prog_section.meta.extra), sitemap_entry.extra);
}

#[test]
fn errors_on_unknown_taxonomies() {
    let (mut site, _, _) = build_site("test_site");
    let mut page = Page::default();
    page.file.path = PathBuf::from("unknown/taxo.md");
    page.meta.taxonomies.insert("wrong".to_string(), vec![]);
    let res = site.add_page(page, false);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(
        err.to_string(),
        "Page `unknown/taxo.md` has taxonomy `wrong` which is not defined in config.toml"
    );
}

#[test]
fn can_build_site_without_live_reload() {
    let (_, _tmp_dir, public) = build_site("test_site");

    assert!(&public.exists());
    assert!(file_exists!(public, "index.html"));
    assert!(file_exists!(public, "sitemap.xml"));
    assert!(file_exists!(public, "robots.txt"));
    assert!(file_exists!(public, "a-fixed-url/index.html"));
    // the config.title is there
    assert!(file_contains!(public, "index.html", "My Integration Testing site"));

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
    assert!(file_contains!(
        public,
        "posts/tutorials/devops/nix/index.html",
        "posts/tutorials/devops/nix.md"
    ));

    // aliases work
    assert!(file_exists!(public, "an-old-url/old-page/index.html"));
    assert!(file_contains!(public, "an-old-url/old-page/index.html", "something-else"));
    assert!(file_contains!(public, "another-old-url/index.html", "posts/"));

    // html aliases work
    assert!(file_exists!(public, "an-old-url/an-old-alias.html"));
    assert!(file_contains!(public, "an-old-url/an-old-alias.html", "something-else"));

    // redirect_to works
    assert!(file_exists!(public, "posts/tutorials/devops/index.html"));
    assert!(file_contains!(public, "posts/tutorials/devops/index.html", "docker"));

    // We do have categories
    assert!(file_exists!(public, "categories/index.html"));
    assert!(file_exists!(public, "categories/a-category/index.html"));
    assert!(file_exists!(public, "categories/a-category/atom.xml"));
    // and podcast_authors (https://github.com/getzola/zola/issues/1177)
    assert!(file_exists!(public, "podcast-authors/index.html"));
    assert!(file_exists!(public, "podcast-authors/some-person/index.html"));
    assert!(file_exists!(public, "podcast-authors/some-person/atom.xml"));
    // But no tags
    assert!(!file_exists!(public, "tags/index.html"));

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

    assert!(!file_exists!(public, "secret_section/index.html"));
    assert!(!file_exists!(public, "secret_section/page.html"));
    assert!(!file_exists!(public, "secret_section/secret_sub_section/hello.html"));
    // no live reload code
    assert!(!file_contains!(public, "index.html", "/livereload.js?port=1112&amp;mindelay=10"),);

    // Both pages and sections are in the sitemap
    assert!(file_contains!(
        public,
        "sitemap.xml",
        "<loc>https://replace-this-with-your-url.com/posts/simple/</loc>"
    ));
    assert!(file_contains!(
        public,
        "sitemap.xml",
        "<loc>https://replace-this-with-your-url.com/posts/</loc>"
    ));
    // Drafts are not in the sitemap
    assert!(!file_contains!(public, "sitemap.xml", "draft"));
    // render: false sections are not in the sitemap either
    assert!(!file_contains!(public, "sitemap.xml", "posts/2018/</loc>"));

    // robots.txt has been rendered from the template
    assert!(file_contains!(public, "robots.txt", "User-agent: zola"));
    assert!(file_contains!(
        public,
        "robots.txt",
        "Sitemap: https://replace-this-with-your-url.com/sitemap.xml"
    ));

    // And
    assert!(file_contains!(
        public,
        "colocated-assets/index.html",
        "Assets in root content directory"
    ));
}

#[test]
fn can_build_site_with_live_reload_and_drafts() {
    let (site, _tmp_dir, public) = build_site_with_setup("test_site", |mut site| {
        site.enable_live_reload(1000);
        site.include_drafts();
        (site, true)
    });

    assert!(&public.exists());

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

    // We do have categories
    assert!(file_exists!(public, "categories/index.html"));
    assert!(file_exists!(public, "categories/a-category/index.html"));
    assert!(file_exists!(public, "categories/a-category/atom.xml"));
    // But no tags
    assert!(!file_exists!(public, "tags/index.html"));

    // no live reload code
    assert!(file_contains!(public, "index.html", "/livereload.js"));

    // the summary target has been created
    assert!(file_contains!(
        public,
        "posts/python/index.html",
        r#"<span id="continue-reading"></span>"#
    ));

    // Drafts are included
    assert!(file_exists!(public, "posts/draft/index.html"));
    assert!(file_contains!(public, "sitemap.xml", "draft"));

    // drafted sections are included
    let library = site.library.read().unwrap();
    assert_eq!(library.sections.len(), 15);

    assert!(file_exists!(public, "secret_section/index.html"));
    assert!(file_exists!(public, "secret_section/draft-page/index.html"));
    assert!(file_exists!(public, "secret_section/page/index.html"));
    assert!(file_exists!(public, "secret_section/secret_sub_section/hello/index.html"));
}

#[test]
fn can_build_site_with_taxonomies() {
    let (site, _tmp_dir, public) = build_site_with_setup("test_site", |mut site| {
        site.load().unwrap();
        {
            let library = &mut *site.library.write().unwrap();
            let mut pages = vec![];

            let pages_data = std::mem::replace(&mut library.pages, AHashMap::new());
            for (i, (_, mut page)) in pages_data.into_iter().enumerate() {
                page.meta.taxonomies = {
                    let mut taxonomies = HashMap::new();
                    taxonomies.insert(
                        "categories".to_string(),
                        vec![if i % 2 == 0 { "A" } else { "B" }.to_string()],
                    );
                    taxonomies
                };
                pages.push(page);
            }
            for p in pages {
                library.insert_page(p);
            }
        }
        site.populate_taxonomies().unwrap();
        (site, false)
    });

    assert!(&public.exists());
    assert_eq!(site.taxonomies.len(), 2);

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
    assert!(file_exists!(public, "categories/a/atom.xml"));
    assert!(file_contains!(
        public,
        "categories/a/atom.xml",
        "https://replace-this-with-your-url.com/categories/a/atom.xml"
    ));
    // Extending from a theme works
    assert!(file_contains!(public, "categories/a/index.html", "EXTENDED"));
    // Tags aren't
    assert!(!file_exists!(public, "tags/index.html"));

    // Categories are in the sitemap
    assert!(file_contains!(
        public,
        "sitemap.xml",
        "<loc>https://replace-this-with-your-url.com/categories/</loc>"
    ));
    assert!(file_contains!(
        public,
        "sitemap.xml",
        "<loc>https://replace-this-with-your-url.com/categories/a/</loc>"
    ));
}

#[test]
fn can_build_site_and_insert_anchor_links() {
    let (_, _tmp_dir, public) = build_site("test_site");

    assert!(Path::new(&public).exists());
    // anchor link inserted
    assert!(file_contains!(
        public,
        "posts/something-else/index.html",
        "<h1 id=\"title\"><a class=\"zola-anchor\" href=\"#title\""
    ));
}

#[test]
fn can_build_site_with_pagination_for_section() {
    let (_, _tmp_dir, public) = build_site_with_setup("test_site", |mut site| {
        site.load().unwrap();
        {
            let mut library = site.library.write().unwrap();
            for (_, section) in library.sections.iter_mut() {
                if section.is_index() {
                    continue;
                }
                section.meta.paginate_by = Some(2);
                section.meta.template = Some("section_paginated.html".to_string());
            }
        }
        (site, false)
    });

    assert!(&public.exists());

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
        "http-equiv=\"refresh\" content=\"0; url=https://replace-this-with-your-url.com/posts/\""
    ));
    assert!(file_contains!(public, "posts/index.html", "Num pagers: 5"));
    assert!(file_contains!(public, "posts/index.html", "Page size: 2"));
    assert!(file_contains!(public, "posts/index.html", "Current index: 1"));
    assert!(!file_contains!(public, "posts/index.html", "has_prev"));
    assert!(file_contains!(public, "posts/index.html", "has_next"));
    assert!(file_contains!(
        public,
        "posts/index.html",
        "First: https://replace-this-with-your-url.com/posts/"
    ));
    assert!(file_contains!(
        public,
        "posts/index.html",
        "Last: https://replace-this-with-your-url.com/posts/page/5/"
    ));
    assert!(!file_contains!(public, "posts/index.html", "has_prev"));

    assert!(file_exists!(public, "posts/page/2/index.html"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Num pagers: 5"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Page size: 2"));
    assert!(file_contains!(public, "posts/page/2/index.html", "Current index: 2"));
    assert!(file_contains!(public, "posts/page/2/index.html", "has_prev"));
    assert!(file_contains!(public, "posts/page/2/index.html", "has_next"));
    assert!(file_contains!(
        public,
        "posts/page/2/index.html",
        "First: https://replace-this-with-your-url.com/posts/"
    ));
    assert!(file_contains!(
        public,
        "posts/page/2/index.html",
        "Last: https://replace-this-with-your-url.com/posts/page/5/"
    ));

    assert!(file_exists!(public, "posts/page/3/index.html"));
    assert!(file_contains!(public, "posts/page/3/index.html", "Num pagers: 5"));
    assert!(file_contains!(public, "posts/page/3/index.html", "Page size: 2"));
    assert!(file_contains!(public, "posts/page/3/index.html", "Current index: 3"));
    assert!(file_contains!(public, "posts/page/3/index.html", "has_prev"));
    assert!(file_contains!(public, "posts/page/3/index.html", "has_next"));
    assert!(file_contains!(
        public,
        "posts/page/3/index.html",
        "First: https://replace-this-with-your-url.com/posts/"
    ));
    assert!(file_contains!(
        public,
        "posts/page/3/index.html",
        "Last: https://replace-this-with-your-url.com/posts/page/5/"
    ));

    assert!(file_exists!(public, "posts/page/4/index.html"));
    assert!(file_contains!(public, "posts/page/4/index.html", "Num pagers: 5"));
    assert!(file_contains!(public, "posts/page/4/index.html", "Page size: 2"));
    assert!(file_contains!(public, "posts/page/4/index.html", "Current index: 4"));
    assert!(file_contains!(public, "posts/page/4/index.html", "has_prev"));
    assert!(file_contains!(public, "posts/page/4/index.html", "has_next"));
    assert!(file_contains!(
        public,
        "posts/page/4/index.html",
        "First: https://replace-this-with-your-url.com/posts/"
    ));
    assert!(file_contains!(
        public,
        "posts/page/4/index.html",
        "Last: https://replace-this-with-your-url.com/posts/page/5/"
    ));

    // sitemap contains the pager pages
    assert!(file_contains!(
        public,
        "sitemap.xml",
        "<loc>https://replace-this-with-your-url.com/posts/page/4/</loc>"
    ));

    // current_path
    assert!(file_contains!(public, "posts/index.html", &current_path("/posts/")));
    assert!(file_contains!(public, "posts/page/2/index.html", &current_path("/posts/page/2/")));
    assert!(file_contains!(public, "posts/python/index.html", &current_path("/posts/python/")));
    assert!(file_contains!(
        public,
        "posts/tutorials/index.html",
        &current_path("/posts/tutorials/")
    ));
}

#[test]
fn can_build_site_with_pagination_for_index() {
    let (_, _tmp_dir, public) = build_site_with_setup("test_site", |mut site| {
        site.load().unwrap();
        {
            let mut library = site.library.write().unwrap();
            {
                let index = library
                    .sections
                    .get_mut(&site.base_path.join("content").join("_index.md"))
                    .unwrap();
                index.meta.paginate_by = Some(2);
                index.meta.template = Some("index_paginated.html".to_string());
            }
        }
        (site, false)
    });

    assert!(&public.exists());

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
        "http-equiv=\"refresh\" content=\"0; url=https://replace-this-with-your-url.com/\""
    ));
    assert!(file_contains!(public, "page/1/index.html", "<title>Redirect</title>"));
    assert!(file_contains!(
        public,
        "page/1/index.html",
        "<a href=\"https://replace-this-with-your-url.com/\">Click here</a>"
    ));
    assert!(file_contains!(public, "index.html", "Num pages: 3"));
    assert!(file_contains!(public, "index.html", "Current index: 1"));
    assert!(file_contains!(public, "index.html", "First: https://replace-this-with-your-url.com/"));
    assert!(file_contains!(
        public,
        "index.html",
        "Last: https://replace-this-with-your-url.com/page/3/"
    ));
    assert!(!file_contains!(public, "index.html", "has_prev"));
    assert!(file_contains!(public, "index.html", "has_next"));

    // sitemap contains the pager pages
    assert!(file_contains!(
        public,
        "sitemap.xml",
        "<loc>https://replace-this-with-your-url.com/page/1/</loc>"
    ));

    // current_path
    assert!(file_contains!(public, "index.html", &current_path("/")));
    assert!(file_contains!(public, "page/2/index.html", &current_path("/page/2/")));
    assert!(file_contains!(public, "paginated/index.html", &current_path("/paginated/")));
}

#[test]
fn can_build_site_with_pagination_for_taxonomy() {
    let (_, _tmp_dir, public) = build_site_with_setup("test_site", |mut site| {
        site.config.languages.get_mut("en").unwrap().taxonomies.push(TaxonomyConfig {
            name: "tags".to_string(),
            slug: "tags".to_string(),
            paginate_by: Some(2),
            paginate_path: None,
            render: true,
            feed: true,
        });
        site.load().unwrap();
        {
            let library = &mut *site.library.write().unwrap();
            let mut pages = vec![];

            let pages_data = std::mem::replace(&mut library.pages, AHashMap::new());
            for (i, (_, mut page)) in pages_data.into_iter().enumerate() {
                page.meta.taxonomies = {
                    let mut taxonomies = HashMap::new();
                    taxonomies.insert(
                        "tags".to_string(),
                        vec![if i % 2 == 0 { "A" } else { "B" }.to_string()],
                    );
                    taxonomies
                };
                pages.push(page);
            }
            for p in pages {
                library.insert_page(p);
            }
        }
        site.populate_taxonomies().unwrap();
        (site, false)
    });

    assert!(&public.exists());

    assert!(file_exists!(public, "index.html"));
    assert!(file_exists!(public, "sitemap.xml"));
    assert!(file_exists!(public, "robots.txt"));
    assert!(file_exists!(public, "a-fixed-url/index.html"));
    assert!(file_exists!(public, "posts/python/index.html"));
    assert!(file_exists!(public, "posts/tutorials/devops/nix/index.html"));
    assert!(file_exists!(public, "posts/with-assets/index.html"));

    // Tags
    assert!(file_exists!(public, "tags/index.html"));
    // With Atom
    assert!(file_exists!(public, "tags/a/atom.xml"));
    assert!(file_exists!(public, "tags/b/atom.xml"));
    // And pagination!
    assert!(file_exists!(public, "tags/a/page/1/index.html"));
    assert!(file_exists!(public, "tags/b/page/1/index.html"));
    assert!(file_exists!(public, "tags/a/page/2/index.html"));
    assert!(file_exists!(public, "tags/b/page/2/index.html"));

    // should redirect to posts/
    assert!(file_contains!(
        public,
        "tags/a/page/1/index.html",
        "http-equiv=\"refresh\" content=\"0; url=https://replace-this-with-your-url.com/tags/a/\""
    ));
    assert!(file_contains!(public, "tags/a/index.html", "Num pagers: 9"));
    assert!(file_contains!(public, "tags/a/index.html", "Page size: 2"));
    assert!(file_contains!(public, "tags/a/index.html", "Current index: 1"));
    assert!(!file_contains!(public, "tags/a/index.html", "has_prev"));
    assert!(file_contains!(public, "tags/a/index.html", "has_next"));
    assert!(file_contains!(
        public,
        "tags/a/index.html",
        "First: https://replace-this-with-your-url.com/tags/a/"
    ));
    assert!(file_contains!(
        public,
        "tags/a/index.html",
        "Last: https://replace-this-with-your-url.com/tags/a/page/9/"
    ));
    assert!(!file_contains!(public, "tags/a/index.html", "has_prev"));

    // sitemap contains the pager pages
    assert!(file_contains!(
        public,
        "sitemap.xml",
        "<loc>https://replace-this-with-your-url.com/tags/a/page/8/</loc>"
    ));

    // current_path
    assert!(file_contains!(public, "tags/index.html", &current_path("/tags/")));
    assert!(file_contains!(public, "tags/a/index.html", &current_path("/tags/a/")));
    assert!(file_contains!(public, "tags/a/page/2/index.html", &current_path("/tags/a/page/2/")));
}

#[test]
fn can_build_feeds() {
    let (_, _tmp_dir, public) = build_site("test_site");

    assert!(&public.exists());
    assert!(file_exists!(public, "atom.xml"));
    // latest article is posts/extra-syntax.md
    assert!(file_contains!(public, "atom.xml", "Extra Syntax"));
    // Next is posts/simple.md
    assert!(file_contains!(public, "atom.xml", "Simple article with shortcodes"));

    // Test section feeds
    assert!(file_exists!(public, "posts/tutorials/programming/atom.xml"));
    // It contains both sections articles
    assert!(file_contains!(public, "posts/tutorials/programming/atom.xml", "Python tutorial"));
    assert!(file_contains!(public, "posts/tutorials/programming/atom.xml", "Rust"));
    // It doesn't contain articles from other sections
    assert!(!file_contains!(public, "posts/tutorials/programming/atom.xml", "Extra Syntax"));

    // Test Atom feed entry with 3 authors
    assert!(file_contains!(public, "posts/tutorials/programming/atom.xml", "Foo Doe"));
    assert!(file_contains!(public, "posts/tutorials/programming/atom.xml", "Bar Doe"));
    assert!(file_contains!(public, "posts/tutorials/programming/atom.xml", "Baz Doe"));
}

#[test]
fn can_build_search_index() {
    let (_, _tmp_dir, public) = build_site_with_setup("test_site", |mut site| {
        site.config.build_search_index = true;
        (site, true)
    });

    assert!(Path::new(&public).exists());
    assert!(file_exists!(public, "elasticlunr.min.js"));
    assert!(file_exists!(public, "search_index.en.js"));
}

#[test]
fn can_build_with_extra_syntaxes() {
    let (_, _tmp_dir, public) = build_site("test_site");

    assert!(&public.exists());
    assert!(file_exists!(public, "posts/extra-syntax/index.html"));
    assert!(file_contains!(public, "posts/extra-syntax/index.html", r#"<span style="color:"#));
}

#[test]
fn can_apply_page_templates() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();
    site.load().unwrap();

    let template_path = path.join("content").join("applying_page_template");
    let library = site.library.read().unwrap();

    let template_section = library.sections.get(&template_path.join("_index.md")).unwrap();
    assert_eq!(template_section.subsections.len(), 2);
    assert_eq!(template_section.pages.len(), 2);

    let from_section_config = &library.pages[&template_section.pages[0]];
    assert_eq!(from_section_config.meta.template, Some("page_template.html".into()));
    assert_eq!(from_section_config.meta.title, Some("From section config".into()));

    let override_page_template = &library.pages[&template_section.pages[1]];
    assert_eq!(override_page_template.meta.template, Some("page_template_override.html".into()));
    assert_eq!(override_page_template.meta.title, Some("Override".into()));

    // It should have applied recursively as well
    let another_section =
        library.sections.get(&template_path.join("another_section").join("_index.md")).unwrap();
    assert_eq!(another_section.subsections.len(), 0);
    assert_eq!(another_section.pages.len(), 1);

    let changed_recursively = &library.pages[&another_section.pages[0]];
    assert_eq!(changed_recursively.meta.template, Some("page_template.html".into()));
    assert_eq!(changed_recursively.meta.title, Some("Changed recursively".into()));

    // But it should not have override a children page_template
    let yet_another_section =
        library.sections.get(&template_path.join("yet_another_section").join("_index.md")).unwrap();
    assert_eq!(yet_another_section.subsections.len(), 0);
    assert_eq!(yet_another_section.pages.len(), 1);

    let child = &library.pages[&yet_another_section.pages[0]];
    assert_eq!(child.meta.template, Some("page_template_child.html".into()));
    assert_eq!(child.meta.title, Some("Local section override".into()));
}

// https://github.com/getzola/zola/issues/571
#[test]
fn can_build_site_custom_builtins_from_theme() {
    let (_, _tmp_dir, public) = build_site("test_site");

    assert!(&public.exists());
    // 404.html is a theme template.
    assert!(file_exists!(public, "404.html"));
    assert!(file_contains!(public, "404.html", "Oops"));
}

#[test]
fn can_build_site_with_html_minified() {
    let (_, _tmp_dir, public) = build_site_with_setup("test_site", |mut site| {
        site.config.minify_html = true;
        (site, true)
    });

    assert!(&public.exists());
    assert!(file_exists!(public, "index.html"));
    assert!(file_contains!(
        public,
        "index.html",
        "<!doctype html><html lang=en><head><meta charset=UTF-8>"
    ));
}

#[test]
fn can_ignore_markdown_content() {
    let (_, _tmp_dir, public) = build_site("test_site");
    assert!(!file_exists!(public, "posts/ignored/index.html"));
}

#[test]
fn can_cachebust_static_files() {
    let (_, _tmp_dir, public) = build_site("test_site");
    assert!(file_contains!(public, "index.html",
        "<link href=\"https://replace-this-with-your-url.com/site.css?h=83bd983e8899946ee33d\" rel=\"stylesheet\">"));
}

#[test]
fn can_get_hash_for_static_files() {
    let (_, _tmp_dir, public) = build_site("test_site");
    assert!(file_contains!(
        public,
        "index.html",
        "src=\"https://replace-this-with-your-url.com/scripts/hello.js\""
    ));
    assert!(file_contains!(
        public,
        "index.html",
        "integrity=\"sha384-AUIvMeqnIabErIxvoJon3ZJZ4N/PPHWT14ENkSqd5covWC35eFN7zRD3aJbbYfu5\""
    ));
}

#[test]
fn can_check_site() {
    let (mut site, _tmp_dir, _public) = build_site("test_site");

    assert_eq!(
        site.config.link_checker.skip_anchor_prefixes,
        vec!["https://github.com/rust-lang/rust/blob/"]
    );
    assert_eq!(
        site.config.link_checker.skip_prefixes,
        vec!["http://[2001:db8::]/", "http://invaliddomain"]
    );

    site.config.enable_check_mode();
    site.load().expect("link check test_site");
}

#[test]
#[should_panic]
fn panics_on_invalid_external_domain() {
    let (mut site, _tmp_dir, _public) = build_site("test_site");

    // remove the invalid domain skip prefix
    let i = site
        .config
        .link_checker
        .skip_prefixes
        .iter()
        .position(|prefix| prefix == "http://invaliddomain")
        .unwrap();
    site.config.link_checker.skip_prefixes.remove(i);

    // confirm the invalid domain skip prefix was removed
    assert_eq!(site.config.link_checker.skip_prefixes, vec!["http://[2001:db8::]/"]);

    // check the test site, this time without the invalid domain skip prefix, which should cause a
    // panic
    site.config.enable_check_mode();
    site.load().expect("link check test_site");
}

#[test]
fn can_find_site_and_page_authors() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, config_file).unwrap();
    site.load().unwrap();
    let library = site.library.read().unwrap();

    // The config has a global default author set.
    let author = site.config.author;
    assert_eq!(Some("config@example.com (Config Author)".to_string()), author);

    let posts_path = path.join("content").join("posts");
    let posts_section = library.sections.get(&posts_path.join("_index.md")).unwrap();

    let p1 = &library.pages[&posts_section.pages[0]];
    let p2 = &library.pages[&posts_section.pages[1]];

    // Only the first page has had an author added.
    assert_eq!(1, p1.meta.authors.len());
    assert_eq!("page@example.com (Page Author)", p1.meta.authors.get(0).unwrap());
    assert_eq!(0, p2.meta.authors.len());
}

// Follows test_site/themes/sample/templates/current_path.html
fn current_path(path: &str) -> String {
    format!("[current_path]({})", path)
}
