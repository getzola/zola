use errors::Result;

use crate::minify;

use super::{ContentType, Middleware, MiddlewareContext};

/// Middleware that minifies HTML content
#[derive(Default)]
pub struct MinifyMiddleware;

impl MinifyMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl Middleware for MinifyMiddleware {
    fn process(&self, ctx: &mut MiddlewareContext) -> Result<()> {
        // Only minify HTML content
        if ctx.metadata.content_type != ContentType::Html {
            return Ok(());
        }

        // Only minify if enabled in config
        if !ctx.config.minify_html {
            return Ok(());
        }

        // Take ownership of content, minify it, and put it back
        let content = std::mem::take(&mut ctx.content);
        ctx.content = minify::html(content)?;

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
        let mw = MinifyMiddleware::new();
        let mut ctx =
            create_test_context("<html>  <body>  <p>test</p>  </body>  </html>".to_string(), true);

        let original_len = ctx.content.len();
        mw.process(&mut ctx).unwrap();

        // Minified content should be shorter
        assert!(
            ctx.content.len() < original_len,
            "Minified content should be shorter. Original: {}, Minified: {}",
            original_len,
            ctx.content.len()
        );

        // Should still contain the essential content
        // Note: minifier removes optional closing tags like </body> and </html>
        assert!(ctx.content.contains("test"), "Should contain 'test'");
        assert!(ctx.content.contains("<p>"), "Should contain opening '<p>'");

        // The minifier removes optional closing tags like </body> and </html> but keeps </p>
        assert_eq!(ctx.content, "<html><body><p>test");
    }

    #[test]
    fn test_skip_when_disabled() {
        let mw = MinifyMiddleware::new();
        let mut ctx = create_test_context(
            "<html>  <body>  <p>test</p>  </body>  </html>".to_string(),
            false, // minify disabled
        );

        let original = ctx.content.clone();
        mw.process(&mut ctx).unwrap();

        // Content should be unchanged
        assert_eq!(ctx.content, original);
    }

    #[test]
    fn test_skip_non_html() {
        let mw = MinifyMiddleware::new();
        let mut ctx = create_test_context("<xml>  <item>  test  </item>  </xml>".to_string(), true);
        ctx.metadata.content_type = ContentType::Xml;

        let original = ctx.content.clone();
        mw.process(&mut ctx).unwrap();

        // XML content should be unchanged
        assert_eq!(ctx.content, original);
    }

    fn create_test_context(content: String, minify_html: bool) -> MiddlewareContext {
        let mut config = config::Config::default_for_test();
        config.minify_html = minify_html;

        MiddlewareContext {
            content,
            binary_content: None,
            compressed_extension: None,
            metadata: super::super::ContentMetadata {
                path: PathBuf::from("test.md"),
                components: vec!["test".to_string()],
                filename: "index.html".to_string(),
                template_name: "page.html".to_string(),
                content_type: ContentType::Html,
                language: "en".to_string(),
                permalink: "http://example.com/test/".to_string(),
            },
            config: Arc::new(config),
        }
    }
}
