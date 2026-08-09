mod common;

use common::test_site_path;
use site::Site;

#[test]
fn errors_on_index_md_page_in_section() {
    let path = test_site_path("test_sites_invalid").join("indexmd");
    let config_file = path.join("config.toml");
    let mut site = Site::new(&path, &config_file).unwrap();
    let res = site.load();
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        format!("{:?}", err).contains(
            "We can't have a page called `index.md` in the same folder as an index section"
        )
    );
}
