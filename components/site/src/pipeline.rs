use std::sync::{Arc, RwLock};

use config::Config;
use content::Library;
use errors::Result;
use libs::rayon::prelude::*;
use libs::tera::Tera;

use super::middleware::{Middleware, OutputData, OutputPackage};
use super::renderable::Renderable;
use super::writer::ContentWriter;

/// Pipeline for processing rendered content through middleware
pub struct Pipeline {
    tera: Arc<Tera>,
    config: Arc<Config>,
    library: Arc<RwLock<Library>>,
    middleware: Vec<Box<dyn Middleware>>,
    writer: Arc<ContentWriter>,
}

impl Pipeline {
    pub fn new(
        tera: Arc<Tera>,
        config: Arc<Config>,
        library: Arc<RwLock<Library>>,
        middleware: Vec<Box<dyn Middleware>>,
        writer: Arc<ContentWriter>,
    ) -> Self {
        Self { tera, config, library, middleware, writer }
    }

    /// Process a renderable through the pipeline
    pub fn process(&self, renderable: &dyn Renderable) -> Result<ProcessedContent> {
        // Step 1: Render the content
        let content = renderable.render(&self.tera, &self.config, &self.library.read().unwrap())?;

        // Step 2: Get metadata
        let metadata = renderable.metadata();

        // Step 3: Create output package with primary output
        let mut package = OutputPackage::new(content, metadata, self.config.clone());

        // Step 4: Apply middleware chain
        for mw in &self.middleware {
            mw.process(&mut package)?;
        }

        // Step 5: Extract outputs from package
        let outputs = package.outputs.into_iter().collect();

        // Step 6: Return processed content with write capability
        Ok(ProcessedContent { outputs, writer: self.writer.clone() })
    }

    /// Replace the middleware chain
    pub fn with_middleware(mut self, middleware: Vec<Box<dyn Middleware>>) -> Self {
        self.middleware = middleware;
        self
    }
}

/// Processed content ready to be written
pub struct ProcessedContent {
    outputs: Vec<(super::middleware::OutputKey, super::middleware::Output)>,
    writer: Arc<ContentWriter>,
}

impl ProcessedContent {
    /// Write all outputs to their destinations
    /// Skips virtual outputs (those with is_virtual=true)
    pub fn write(self) -> Result<()> {
        self.outputs
            .into_par_iter()
            .filter(|(_, output)| !output.tags.is_virtual)
            .map(|(key, output)| match output.data {
                OutputData::Text(content) => {
                    self.writer.write(&key.components, &key.filename, &content)
                }
                OutputData::Binary(content) => {
                    self.writer.write_binary(&key.components, &key.filename, &content)
                }
            })
            .collect()
    }

    /// Get the primary output content without writing (for testing/inspection)
    pub fn primary_content(&self) -> Option<String> {
        self.outputs
            .iter()
            .find(|(_, output)| output.tags.is_primary)
            .and_then(|(_, output)| output.data.as_text().map(|s| s.to_string()))
    }

    /// Get all output keys (for testing/inspection)
    #[cfg(test)]
    pub fn output_keys(&self) -> Vec<super::middleware::OutputKey> {
        self.outputs.iter().map(|(key, _)| key.clone()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::{
        ContentMetadata, ContentType, Output, OutputData, OutputPackage, OutputTags,
    };
    use std::path::PathBuf;

    #[test]
    fn test_virtual_outputs_not_written() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().to_path_buf();

        // Create a simple output package with a virtual output
        let metadata = ContentMetadata {
            path: PathBuf::from("test.md"),
            components: vec!["test".to_string()],
            filename: "index.html".to_string(),
            template_name: "page.html".to_string(),
            content_type: ContentType::Html,
            language: "en".to_string(),
            permalink: "http://example.com/test/".to_string(),
        };

        let package = OutputPackage::new(
            "Primary content".to_string(),
            metadata,
            Arc::new(config::Config::default_for_test()),
        );

        // Add a regular output
        package.add(
            "extra.html",
            Output {
                data: OutputData::Text("Extra content".to_string()),
                content_type: ContentType::Html,
                tags: OutputTags { is_derived: true, ..Default::default() },
            },
        );

        // Add a virtual output
        package.add_virtual(
            "virtual.json",
            Output {
                data: OutputData::Text("Virtual content".to_string()),
                content_type: ContentType::Json,
                tags: OutputTags::default(),
            },
        );

        // Create writer and processed content
        let writer = Arc::new(ContentWriter::new(crate::BuildMode::Disk, output_path.clone()));
        let outputs: Vec<_> = package.outputs.into_iter().collect();
        let processed = ProcessedContent { outputs, writer };

        // Write outputs
        processed.write().unwrap();

        // Verify primary output was written
        assert!(output_path.join("test").join("index.html").exists());

        // Verify regular derived output was written
        assert!(output_path.join("test").join("extra.html").exists());

        // Verify virtual output was NOT written
        assert!(!output_path.join("test").join("virtual.json").exists());
    }
}
