use std::io::prelude::*;
use std::fs::{File};
use std::path::Path;

use errors::Result;


pub fn create_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
