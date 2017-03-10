use std::io::prelude::*;
use std::fs::{File, create_dir};
use std::path::Path;

use errors::{Result, ResultExt};


pub fn create_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Very similar to `create_dir` from the std except it checks if the folder
/// exists before creating it
pub fn create_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();
    if !path.exists() {
        create_dir(path)
            .chain_err(|| format!("Was not able to create folder {}", path.display()))?;
    }
    Ok(())
}
