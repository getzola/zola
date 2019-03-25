extern crate site;
extern crate tempfile;

use std::env;
use std::path::PathBuf;

use self::site::Site;
use self::tempfile::{tempdir, TempDir};

// 2 helper macros to make all the build testing more bearable
#[macro_export]
macro_rules! file_exists {
    ($root: expr, $path: expr) => {{
        let mut path = $root.clone();
        for component in $path.split("/") {
            path = path.join(component);
        }
        std::path::Path::new(&path).exists()
    }};
}

#[macro_export]
macro_rules! file_contains {
    ($root: expr, $path: expr, $text: expr) => {{
        use std::io::prelude::*;
        let mut path = $root.clone();
        for component in $path.split("/") {
            path = path.join(component);
        }
        let mut file = std::fs::File::open(&path).expect(&format!("Failed to open {:?}", $path));
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        println!("{}", s);
        s.contains($text)
    }};
}

/// We return the tmpdir otherwise it would get out of scope and be deleted
/// The tests can ignore it if they dont need it by prefixing it with a `_`
pub fn build_site(name: &str) -> (Site, TempDir, PathBuf) {
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push(name);
    let mut site = Site::new(&path, "config.toml").unwrap();
    site.load().unwrap();
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().expect("Couldn't build the site");
    (site, tmp_dir, public.clone())
}

/// Same as `build_site` but has a hook to setup some config options
pub fn build_site_with_setup<F>(name: &str, mut setup_cb: F) -> (Site, TempDir, PathBuf)
where
    F: FnMut(Site) -> (Site, bool),
{
    let mut path = env::current_dir().unwrap().parent().unwrap().parent().unwrap().to_path_buf();
    path.push(name);
    let site = Site::new(&path, "config.toml").unwrap();
    let (mut site, needs_loading) = setup_cb(site);
    if needs_loading {
        site.load().unwrap();
    }
    let tmp_dir = tempdir().expect("create temp dir");
    let public = &tmp_dir.path().join("public");
    site.set_output_path(&public);
    site.build().expect("Couldn't build the site");
    (site, tmp_dir, public.clone())
}
