use std::path::PathBuf;
use std::sync::Arc;

use derive_builder::Builder;

use config::Config;
use errors::Result;

mod livereload;
mod minify;

pub use livereload::LiveReloadMiddleware;
pub use minify::MinifyMiddleware;

/// Trait for middleware that can process rendered content
pub trait Middleware: Send + Sync {
    /// Process the content in the middleware context
    fn process(&self, ctx: &mut MiddlewareContext) -> Result<()>;

    /// Name of the middleware for debugging/logging
    fn name(&self) -> &str;
}

/// Context passed to middleware during processing
pub struct MiddlewareContext {
    /// The rendered content to be processed
    pub content: String,

    /// Metadata about the content being processed
    pub metadata: ContentMetadata,

    /// Site configuration
    pub config: Arc<Config>,
}

/// Metadata about the content being processed
#[derive(Clone, Debug, Builder)]
#[builder(setter(into, strip_option), build_fn(error = "String"))]
pub struct ContentMetadata {
    /// Source file path
    #[builder(default)]
    pub path: PathBuf,

    /// Output path components (e.g., ["blog", "my-post"])
    #[builder(default)]
    pub components: Vec<String>,

    /// Output filename (e.g., "index.html")
    #[builder(default = "String::from(\"index.html\")")]
    pub filename: String,

    /// Template name used to render (e.g., "page.html")
    #[builder(default = "String::from(\"index.html\")", setter(custom))]
    pub template_name: String,

    /// Content type based on template extension
    #[builder(default = "ContentType::Html")]
    pub content_type: ContentType,

    /// Language code
    #[builder(default)]
    pub language: String,

    /// Permalink URL
    #[builder(default)]
    pub permalink: String,
}

impl ContentMetadata {
    /// Create a new builder for ContentMetadata
    pub fn builder() -> ContentMetadataBuilder {
        ContentMetadataBuilder::default()
    }
}

impl ContentMetadataBuilder {
    /// Custom setter for template_name that automatically sets content_type
    pub fn template_name(&mut self, template_name: impl Into<String>) -> &mut Self {
        let name = template_name.into();
        self.content_type = Some(ContentType::from_template(&name));
        self.template_name = Some(name);
        self
    }
}

/// Content type determined from template extension
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContentType {
    Html,
    Xml,
    Text,
    Json,
}

impl ContentType {
    /// Determine content type from template name
    pub fn from_template(template: &str) -> Self {
        let path = std::path::Path::new(template);
        match path.extension().and_then(|s| s.to_str()) {
            Some("html") | Some("htm") => ContentType::Html,
            Some("xml") => ContentType::Xml,
            Some("json") => ContentType::Json,
            Some("txt") | Some("text") => ContentType::Text,
            _ => ContentType::Html, // default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_from_template() {
        assert_eq!(ContentType::from_template("page.html"), ContentType::Html);
        assert_eq!(ContentType::from_template("post.htm"), ContentType::Html);
        assert_eq!(ContentType::from_template("feed.xml"), ContentType::Xml);
        assert_eq!(ContentType::from_template("data.json"), ContentType::Json);
        assert_eq!(ContentType::from_template("robots.txt"), ContentType::Text);
        assert_eq!(ContentType::from_template("unknown.foo"), ContentType::Html);
    }
}
