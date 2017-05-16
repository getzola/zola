
use std::fs::{create_dir};
use std::path::Path;

use gutenberg::errors::Result;
use gutenberg::create_file;


const CONFIG: &'static str = r#"
title = "My site"
# replace the url below with yours
base_url = "https://example.com"

[extra]
# Put all your custom variables here
"#;


pub fn create_new_project(name: &str) -> Result<()> {
    let path = Path::new(name);

    // Better error message than the rust default
    if path.exists() && path.is_dir() {
        bail!("Folder `{}` already exists", path.to_string_lossy().to_string());
    }

    // main folder
    create_dir(path)?;
    create_file(&path.join("config.toml"), CONFIG.trim_left())?;

    // content folder
    create_dir(path.join("content"))?;

    // layouts folder
    create_dir(path.join("templates"))?;

    create_dir(path.join("static"))?;

    Ok(())
}
