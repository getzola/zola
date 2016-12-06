
use std::io::prelude::*;
use std::fs::{create_dir, File};
use std::path::Path;

use errors::{Result, ErrorKind};


const CONFIG: &'static str = r#"
title = "My site"
base_url = "https://replace-this-with-your-url.com"
"#;


pub fn create_new_project<P: AsRef<Path>>(name: P) -> Result<()> {
    let path = name.as_ref();
    // Better error message than the rust default
    if path.exists() && path.is_dir() {
        return Err(ErrorKind::FolderExists(path.to_string_lossy().to_string()).into());
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

fn create_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
