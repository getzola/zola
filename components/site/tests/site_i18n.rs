mod common;

use std::env;

use common::build_site;
use site::Site;

#[test]
fn can_parse_multilingual_site() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site_i18n");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();
    site.load().unwrap();

    let library = site.library.read().unwrap();
    assert_eq!(library.pages().len(), 10);
    assert_eq!(library.sections().len(), 6);

    // default index sections
    let default_index_section =
        library.get_section(&path.join("content").join("_index.md")).unwrap();
    assert_eq!(default_index_section.pages.len(), 1);
    assert!(default_index_section.ancestors.is_empty());

    let fr_index_section = library.get_section(&path.join("content").join("_index.fr.md")).unwrap();
    assert_eq!(fr_index_section.pages.len(), 1);
    assert!(fr_index_section.ancestors.is_empty());

    // blog sections get only their own language pages
    let blog_path = path.join("content").join("blog");

    let default_blog = library.get_section(&blog_path.join("_index.md")).unwrap();
    assert_eq!(default_blog.subsections.len(), 0);
    assert_eq!(default_blog.pages.len(), 4);
    assert_eq!(
        default_blog.ancestors,
        vec![*library.get_section_key(&default_index_section.file.path).unwrap()]
    );
    for key in &default_blog.pages {
        let page = library.get_page_by_key(*key);
        assert_eq!(page.lang, "en");
    }

    let fr_blog = library.get_section(&blog_path.join("_index.fr.md")).unwrap();
    assert_eq!(fr_blog.subsections.len(), 0);
    assert_eq!(fr_blog.pages.len(), 3);
    assert_eq!(
        fr_blog.ancestors,
        vec![*library.get_section_key(&fr_index_section.file.path).unwrap()]
    );
    for key in &fr_blog.pages {
        let page = library.get_page_by_key(*key);
        assert_eq!(page.lang, "fr");
    }
}

#[test]
fn can_build_multilingual_site() {
    let (_, _tmp_dir, public) = build_site("test_site_i18n");

    assert!(public.exists());

    // Index pages
    assert!(file_exists!(public, "index.html"));
    assert!(file_exists!(public, "fr/index.html"));
    assert!(file_contains!(public, "fr/index.html", "Une page"));
    assert!(file_contains!(public, "fr/index.html", "Language: fr"));

    assert!(file_exists!(public, "base/index.html"));
    assert!(file_exists!(public, "fr/base/index.html"));

    // Sections are there as well, with translations info
    assert!(file_exists!(public, "blog/index.html"));
    assert!(file_contains!(
        public,
        "blog/index.html",
        "Translated in fr: Mon blog https://example.com/fr/blog/"
    ));
    assert!(file_contains!(
        public,
        "blog/index.html",
        "Translated in it: Il mio blog https://example.com/it/blog/"
    ));
    assert!(file_exists!(public, "fr/blog/index.html"));
    assert!(file_contains!(public, "fr/blog/index.html", "Language: fr"));
    assert!(file_contains!(
        public,
        "fr/blog/index.html",
        "Translated in en: My blog https://example.com/blog/"
    ));
    assert!(file_contains!(
        public,
        "fr/blog/index.html",
        "Translated in it: Il mio blog https://example.com/it/blog/"
    ));

    // Normal pages are there with the translations
    assert!(file_exists!(public, "blog/something/index.html"));
    assert!(file_contains!(
        public,
        "blog/something/index.html",
        "Translated in fr: Quelque chose https://example.com/fr/blog/something/"
    ));
    assert!(file_exists!(public, "fr/blog/something/index.html"));
    assert!(file_contains!(public, "fr/blog/something/index.html", "Language: fr"));
    assert!(file_contains!(
        public,
        "fr/blog/something/index.html",
        "Translated in en: Something https://example.com/blog/something/"
    ));

    // sitemap contains all languages
    assert!(file_exists!(public, "sitemap.xml"));
    assert!(file_contains!(public, "sitemap.xml", "https://example.com/blog/something-else/"));
    assert!(file_contains!(public, "sitemap.xml", "https://example.com/fr/blog/something-else/"));
    assert!(file_contains!(public, "sitemap.xml", "https://example.com/it/blog/something-else/"));

    // one feed per language
    assert!(file_exists!(public, "atom.xml"));
    assert!(file_contains!(public, "atom.xml", "https://example.com/blog/something-else/"));
    assert!(!file_contains!(public, "atom.xml", "https://example.com/fr/blog/something-else/"));
    assert!(file_contains!(
        public,
        "atom.xml",
        r#"<feed xmlns="http://www.w3.org/2005/Atom" xml:lang="en">"#
    ));
    assert!(file_exists!(public, "fr/atom.xml"));
    assert!(!file_contains!(public, "fr/atom.xml", "https://example.com/blog/something-else/"));
    assert!(file_contains!(public, "fr/atom.xml", "https://example.com/fr/blog/something-else/"));
    assert!(file_contains!(
        public,
        "fr/atom.xml",
        r#"<feed xmlns="http://www.w3.org/2005/Atom" xml:lang="fr">"#
    ));
    // Italian doesn't have feed enabled
    assert!(!file_exists!(public, "it/atom.xml"));

    // Taxonomies are per-language
    // English
    assert!(file_exists!(public, "authors/index.html"));
    assert!(file_contains!(public, "authors/index.html", "Queen"));
    assert!(!file_contains!(public, "authors/index.html", "Vincent"));
    assert!(!file_exists!(public, "auteurs/index.html"));
    assert!(file_exists!(public, "authors/queen-elizabeth/atom.xml"));
    assert!(file_contains!(
        public,
        "authors/queen-elizabeth/atom.xml",
        r#"<feed xmlns="http://www.w3.org/2005/Atom" xml:lang="en">"#
    ));
    assert!(file_contains!(
        public,
        "authors/queen-elizabeth/atom.xml",
        r#"<title> - Queen Elizabeth</title>"#
    ));

    assert!(file_exists!(public, "tags/index.html"));
    assert!(file_contains!(public, "tags/index.html", "hello"));
    assert!(!file_contains!(public, "tags/index.html", "bonjour"));

    // French
    assert!(!file_exists!(public, "fr/authors/index.html"));
    assert!(file_exists!(public, "fr/auteurs/index.html"));
    assert!(!file_contains!(public, "fr/auteurs/index.html", "Queen"));
    assert!(file_contains!(public, "fr/auteurs/index.html", "Vincent"));
    assert!(!file_exists!(public, "fr/auteurs/vincent-prouillet/atom.xml"));

    assert!(file_exists!(public, "fr/tags/index.html"));
    assert!(file_contains!(public, "fr/tags/index.html", "bonjour"));
    assert!(!file_contains!(public, "fr/tags/index.html", "hello"));

    // one lang index per language
    assert!(file_exists!(public, "search_index.en.js"));
    assert!(file_exists!(public, "search_index.it.js"));
    assert!(!file_exists!(public, "search_index.fr.js"));
}
