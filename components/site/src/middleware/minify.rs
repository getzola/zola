use errors::Result;

use crate::minify;

use super::{ContentType, Middleware, OutputPackage};

/// Middleware that minifies HTML content
#[derive(Default)]
pub struct MinifyMiddleware;

impl MinifyMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl Middleware for MinifyMiddleware {
    fn process(&self, package: &mut OutputPackage) -> Result<()> {
        // Only minify if enabled in config
        if !package.config.minify_html {
            return Ok(());
        }

        // Iterate over all outputs and minify HTML text that hasn't been minified yet
        for mut entry in package.outputs.iter_mut() {
            let output = entry.value_mut();

            // Skip if already minified or not HTML
            if output.tags.is_minified || output.content_type != ContentType::Html {
                continue;
            }

            // Minify HTML text content
            if let Some(html) = output.data.as_text_mut() {
                let minified = minify::html(std::mem::take(html))?;
                *html = minified;
                output.tags.is_minified = true;
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "minify"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn test_minify_html() {
        use super::super::OutputKey;

        let mw = MinifyMiddleware::new();
        let mut package =
            create_test_package("<html>  <body>  <p>test</p>  </body>  </html>".to_string(), true);

        let key = OutputKey::new(vec!["test".to_string()], "index.html");
        let original_len = package.outputs.get(&key).unwrap().data.as_text().unwrap().len();

        mw.process(&mut package).unwrap();

        let output = package.outputs.get(&key).unwrap();
        let content = output.data.as_text().unwrap();

        // Minified content should be shorter
        assert!(
            content.len() < original_len,
            "Minified content should be shorter. Original: {}, Minified: {}",
            original_len,
            content.len()
        );

        // Should still contain the essential content
        // Note: minifier removes optional closing tags like </body> and </html>
        assert!(content.contains("test"), "Should contain 'test'");
        assert!(content.contains("<p>"), "Should contain opening '<p>'");

        // The minifier removes optional closing tags like </body> and </html> but keeps </p>
        assert_eq!(content, "<html><body><p>test");

        // Check that is_minified tag is set
        assert!(output.tags.is_minified);
    }

    #[test]
    fn test_skip_when_disabled() {
        use super::super::OutputKey;

        let mw = MinifyMiddleware::new();
        let mut package = create_test_package(
            "<html>  <body>  <p>test</p>  </body>  </html>".to_string(),
            false, // minify disabled
        );

        let key = OutputKey::new(vec!["test".to_string()], "index.html");
        let original = package.outputs.get(&key).unwrap().data.as_text().unwrap().to_string();

        mw.process(&mut package).unwrap();

        let output_ref = package.outputs.get(&key).unwrap();
        let content = output_ref.data.as_text().unwrap();
        // Content should be unchanged
        assert_eq!(content, &original);
    }

    #[test]
    fn test_skip_non_html() {
        use super::super::OutputKey;

        let mw = MinifyMiddleware::new();
        let mut package =
            create_test_package("<xml>  <item>  test  </item>  </xml>".to_string(), true);

        // Change content type to XML
        let key = OutputKey::new(vec!["test".to_string()], "index.html");
        package.outputs.get_mut(&key).unwrap().content_type = ContentType::Xml;

        let original = package.outputs.get(&key).unwrap().data.as_text().unwrap().to_string();
        mw.process(&mut package).unwrap();

        let output_ref = package.outputs.get(&key).unwrap();
        let content = output_ref.data.as_text().unwrap();
        // XML content should be unchanged
        assert_eq!(content, &original);
    }

    fn create_test_package(content: String, minify_html: bool) -> super::super::OutputPackage {
        use super::super::{ContentMetadata, OutputPackage};

        let mut config = config::Config::default_for_test();
        config.minify_html = minify_html;

        let metadata = ContentMetadata {
            path: PathBuf::from("test.md"),
            components: vec!["test".to_string()],
            filename: "index.html".to_string(),
            template_name: "page.html".to_string(),
            content_type: ContentType::Html,
            language: "en".to_string(),
            permalink: "http://example.com/test/".to_string(),
        };

        OutputPackage::new(content, metadata, Arc::new(config))
    }
}
