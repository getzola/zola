extern crate site;

use std::env;

use site::Site;

#[test]
fn can_parse_multilingual_site() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site_i18n");
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();

    assert_eq!(site.library.pages().len(), 9);
    assert_eq!(site.library.sections().len(), 4);

    // default index sections
    let default_index_section = site.library.get_section(&path.join("content").join("_index.md")).unwrap();
    assert_eq!(default_index_section.pages.len(), 1);
    assert!(default_index_section.ancestors.is_empty());

    let fr_index_section = site.library.get_section(&path.join("content").join("_index.fr.md")).unwrap();
    assert_eq!(fr_index_section.pages.len(), 1);
    assert!(fr_index_section.ancestors.is_empty());

    // blog sections get only their own language pages
    let blog_path = path.join("content").join("blog");

    let default_blog = site.library.get_section(&blog_path.join("_index.md")).unwrap();
    assert_eq!(default_blog.subsections.len(), 0);
    assert_eq!(default_blog.pages.len(), 4);
    assert_eq!(default_blog.ancestors, vec![*site.library.get_section_key(&default_index_section.file.path).unwrap()]);
    for key in &default_blog.pages {
        let page = site.library.get_page_by_key(*key);
        assert_eq!(page.lang, None);
    }

    let fr_blog = site.library.get_section(&blog_path.join("_index.fr.md")).unwrap();
    assert_eq!(fr_blog.subsections.len(), 0);
    assert_eq!(fr_blog.pages.len(), 3);
    assert_eq!(fr_blog.ancestors, vec![*site.library.get_section_key(&fr_index_section.file.path).unwrap()]);
    for key in &fr_blog.pages {
        let page = site.library.get_page_by_key(*key);
        assert_eq!(page.lang, Some("fr".to_string()));
    }
}
