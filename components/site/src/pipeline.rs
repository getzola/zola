use std::sync::{Arc, RwLock};

use config::Config;
use content::Library;
use errors::Result;
use libs::tera::Tera;

use super::middleware::{Middleware, MiddlewareContext};
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

        // Step 3: Create middleware context
        let mut ctx = MiddlewareContext {
            content,
            binary_content: None,
            compressed_extension: None,
            metadata: metadata.clone(),
            config: self.config.clone(),
        };

        // Step 4: Apply middleware chain
        for mw in &self.middleware {
            mw.process(&mut ctx)?;
        }

        // Step 5: Return processed content with write capability
        Ok(ProcessedContent {
            content: ctx.content,
            binary_content: ctx.binary_content,
            compressed_extension: ctx.compressed_extension,
            components: ctx.metadata.components,
            filename: ctx.metadata.filename,
            writer: self.writer.clone(),
        })
    }

    /// Replace the middleware chain
    pub fn with_middleware(mut self, middleware: Vec<Box<dyn Middleware>>) -> Self {
        self.middleware = middleware;
        self
    }
}

/// Processed content ready to be written
pub struct ProcessedContent {
    content: String,
    binary_content: Option<Vec<u8>>,
    compressed_extension: Option<String>,
    components: Vec<String>,
    filename: String,
    writer: Arc<ContentWriter>,
}

impl ProcessedContent {
    /// Write to the default destination (from metadata)
    pub fn write(self) -> Result<()> {
        let components_refs: Vec<String> = self.components.clone();
        // Write original content
        self.writer.write(&components_refs, &self.filename, &self.content)?;

        // Write compressed version if present
        if let (Some(binary_content), Some(extension)) = (self.binary_content, self.compressed_extension) {
            let compressed_filename = format!("{}{}", self.filename, extension);
            self.writer.write_binary(&components_refs, &compressed_filename, &binary_content)?;
        }

        Ok(())
    }

    /// Write to a custom destination
    pub fn write_to(self, components: &[&str], filename: &str) -> Result<()> {
        let components_owned: Vec<String> = components.iter().map(|s| s.to_string()).collect();
        // Write original content
        self.writer.write(&components_owned, filename, &self.content)?;

        // Write compressed version if present
        if let (Some(binary_content), Some(extension)) = (self.binary_content, self.compressed_extension) {
            let compressed_filename = format!("{}{}", filename, extension);
            self.writer.write_binary(&components_owned, &compressed_filename, &binary_content)?;
        }

        Ok(())
    }

    /// Get the content without writing
    pub fn into_string(self) -> String {
        self.content
    }
}
