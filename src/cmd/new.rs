
use std::fs::{create_dir};
use std::path::Path;

use errors::Result;
use utils::create_file;


const CONFIG: &'static str = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"
"#;


pub fn create_new_project<P: AsRef<Path>>(name: P) -> Result<()> {
    let path = name.as_ref();
    // Better error message than the rust default
    if path.exists() && path.is_dir() {
        bail!("Folder `{}` already exists", path.to_string_lossy().to_string());
    }

    // main folder
    create_dir(path)?;
    create_file(path.join("config.toml"), CONFIG.trim_left())?;

    // content folder
    create_dir(path.join("content"))?;

    // layouts folder
    create_dir(path.join("layouts"))?;

    create_dir(path.join("static"))?;

    Ok(())
}
