mod common;

use std::env;

use common::build_site;
use site::Site;

#[test]
fn can_parse_gemini_site() {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push("test_site_gemini");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();
    site.load().unwrap();
    let library = site.library.read().unwrap();

    assert_eq!(library.pages().len(), 3);
    assert_eq!(library.sections().len(), 2);
}

#[test]
fn can_build_gemini_site() {
    let (_, _tmp_dir, public) = build_site("test_site_gemini");
    assert!(&public.exists());
    assert!(file_exists!(public, "index.gemini"));
    assert!(file_exists!(public, "hidden/orphan/index.gemini"));
    assert!(file_exists!(public, "posts/index.gemini"));
    // TODO: These fail; investigate!
    assert!(file_exists!(public, "posts/first/index.gemini"));
    assert!(file_exists!(public, "posts/another/index.gemini"));

    // orphan page is templated correctly
    assert!(file_contains!(public, "hidden/orphan/index.gemini", "back to site root"));
    assert!(file_contains!(public, "hidden/orphan/index.gemini", "# Orphan page"));

    // post list works
    assert!(file_contains!(public, "posts/index.gemini",
        "=> gemini://test.invalid/posts/first/ 2020-01-01 First post"));
    assert!(file_contains!(public, "posts/index.gemini",
        "=> gemini://test.invalid/posts/another/ 2020-02-01 Another one"));
}
