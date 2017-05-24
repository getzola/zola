use std::io::prelude::*;
use std::fs::{File, create_dir};
use std::path::Path;

use errors::{Result, ResultExt};

/// Create a file with the content given
pub fn create_file(path: &Path, content: &str) -> Result<()> {
    let mut file = File::create(&path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Create a directory at the given path if it doesn't exist already
pub fn ensure_directory_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        create_directory(path)?;
    }
    Ok(())
}

/// Very similar to `create_dir` from the std except it checks if the folder
/// exists before creating it
pub fn create_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        create_dir(path)
            .chain_err(|| format!("Was not able to create folder {}", path.display()))?;
    }
    Ok(())
}

/// Return the content of a file, with error handling added
pub fn read_file(path: &Path) -> Result<String> {
    let mut content = String::new();
    File::open(path)
        .chain_err(|| format!("Failed to open '{:?}'", path.display()))?
        .read_to_string(&mut content)?;

    Ok(content)
}
