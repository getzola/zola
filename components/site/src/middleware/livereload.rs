use errors::Result;

use super::{ContentType, Middleware, MiddlewareContext};

/// Middleware that injects live reload script into HTML content
pub struct LiveReloadMiddleware {
    port: u16,
}

impl LiveReloadMiddleware {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

impl Middleware for LiveReloadMiddleware {
    fn process(&self, ctx: &mut MiddlewareContext) -> Result<()> {
        // Only inject into HTML content
        if ctx.metadata.content_type != ContentType::Html {
            return Ok(());
        }

        let script =
            format!(r#"<script src="/livereload.js?port={}&amp;mindelay=10"></script>"#, self.port);

        // Insert before </body> if found, otherwise append
        if let Some(index) = ctx.content.rfind("</body>") {
            ctx.content.insert_str(index, &script);
        } else {
            ctx.content.push_str(&script);
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "livereload"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn test_inject_livereload_with_body() {
        let mw = LiveReloadMiddleware::new(1024);
        let mut ctx = create_test_context("<html><body></body></html>".to_string());

        mw.process(&mut ctx).unwrap();

        assert!(ctx.content.contains("livereload.js?port=1024"));
        assert!(ctx.content.contains("</body>"));
        // Script should be before </body>
        let script_pos = ctx.content.find("livereload.js").unwrap();
        let body_pos = ctx.content.find("</body>").unwrap();
        assert!(script_pos < body_pos);
    }

    #[test]
    fn test_inject_livereload_without_body() {
        let mw = LiveReloadMiddleware::new(1024);
        let mut ctx = create_test_context("<html></html>".to_string());

        mw.process(&mut ctx).unwrap();

        assert!(ctx.content.contains("livereload.js?port=1024"));
        // Script should be at the end
        assert!(ctx.content.ends_with("</script>"));
    }

    #[test]
    fn test_skip_non_html() {
        let mw = LiveReloadMiddleware::new(1024);
        let mut ctx = create_test_context("<xml></xml>".to_string());
        ctx.metadata.content_type = ContentType::Xml;

        let original = ctx.content.clone();
        mw.process(&mut ctx).unwrap();

        assert_eq!(ctx.content, original);
        assert!(!ctx.content.contains("livereload.js"));
    }

    fn create_test_context(content: String) -> MiddlewareContext {
        MiddlewareContext {
            content,
            metadata: super::super::ContentMetadata {
                path: PathBuf::from("test.md"),
                components: vec!["test".to_string()],
                filename: "index.html".to_string(),
                template_name: "page.html".to_string(),
                content_type: ContentType::Html,
                language: "en".to_string(),
                permalink: "http://example.com/test/".to_string(),
            },
            config: Arc::new(config::Config::default_for_test()),
        }
    }
}
