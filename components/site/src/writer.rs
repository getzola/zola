use std::path::PathBuf;

use errors::Result;
use libs::relative_path::RelativePathBuf;
use utils::fs::create_file;

use crate::{BuildMode, ContentData, SITE_CONTENT};

/// Handles writing rendered content to disk and/or memory
#[derive(Clone)]
pub struct ContentWriter {
    mode: BuildMode,
    output_path: PathBuf,
}

impl ContentWriter {
    pub fn new(mode: BuildMode, output_path: PathBuf) -> Self {
        Self { mode, output_path }
    }

    /// Write content to the appropriate destination(s) based on build mode
    pub fn write(&self, components: &[String], filename: &str, content: &str) -> Result<()> {
        let mut current_path = self.output_path.clone();
        let mut site_path = RelativePathBuf::new();

        for component in components {
            current_path.push(component);
            site_path.push(component);
        }

        // Write to disk if needed
        match self.mode {
            BuildMode::Disk | BuildMode::Both => {
                let end_path = current_path.join(filename);
                create_file(&end_path, content)?;
            }
            _ => (),
        }

        // Write to memory if needed
        match self.mode {
            BuildMode::Memory | BuildMode::Both => {
                let site_path = site_path.join(filename);

                SITE_CONTENT.insert(site_path, ContentData::Text(content.to_string()));
            }
            _ => (),
        }

        Ok(())
    }

    /// Write binary content to the appropriate destination(s) based on build mode
    pub fn write_binary(
        &self,
        components: &[String],
        filename: &str,
        content: &[u8],
    ) -> Result<()> {
        let mut current_path = self.output_path.clone();
        let mut site_path = RelativePathBuf::new();

        for component in components {
            current_path.push(component);
            site_path.push(component);
        }

        // Write to disk if needed
        match self.mode {
            BuildMode::Disk | BuildMode::Both => {
                let end_path = current_path.join(filename);
                std::fs::create_dir_all(end_path.parent().unwrap())?;
                std::fs::write(&end_path, content)?;
            }
            _ => (),
        }

        // Write to memory if needed
        match self.mode {
            BuildMode::Memory | BuildMode::Both => {
                let site_path = site_path.join(filename);

                SITE_CONTENT.insert(site_path, ContentData::Binary(content.to_vec()));
            }
            _ => (),
        }

        Ok(())
    }
}
