use errors::Result;

use super::{ContentType, Middleware, OutputPackage};

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
    fn process(&self, package: &mut OutputPackage) -> Result<()> {
        let script =
            format!(r#"<script src="/livereload.js?port={}&amp;mindelay=10"></script>"#, self.port);

        // Iterate over all outputs and inject script into HTML text content
        for mut entry in package.outputs.iter_mut() {
            let output = entry.value_mut();

            // Only inject into HTML content
            if output.content_type != ContentType::Html {
                continue;
            }

            // Inject into HTML text content
            if let Some(html) = output.data.as_text_mut() {
                // Insert before </body> if found, otherwise append
                if let Some(index) = html.rfind("</body>") {
                    html.insert_str(index, &script);
                } else {
                    html.push_str(&script);
                }
            }
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
        use super::super::OutputKey;

        let mw = LiveReloadMiddleware::new(1024);
        let mut package = create_test_package("<html><body></body></html>".to_string());

        mw.process(&mut package).unwrap();

        let key = OutputKey::new(vec!["test".to_string()], "index.html");
        let output_ref = package.outputs.get(&key).unwrap();
        let content = output_ref.data.as_text().unwrap();

        assert!(content.contains("livereload.js?port=1024"));
        assert!(content.contains("</body>"));
        // Script should be before </body>
        let script_pos = content.find("livereload.js").unwrap();
        let body_pos = content.find("</body>").unwrap();
        assert!(script_pos < body_pos);
    }

    #[test]
    fn test_inject_livereload_without_body() {
        use super::super::OutputKey;

        let mw = LiveReloadMiddleware::new(1024);
        let mut package = create_test_package("<html></html>".to_string());

        mw.process(&mut package).unwrap();

        let key = OutputKey::new(vec!["test".to_string()], "index.html");
        let output_ref = package.outputs.get(&key).unwrap();
        let content = output_ref.data.as_text().unwrap();

        assert!(content.contains("livereload.js?port=1024"));
        // Script should be at the end
        assert!(content.ends_with("</script>"));
    }

    #[test]
    fn test_skip_non_html() {
        use super::super::OutputKey;

        let mw = LiveReloadMiddleware::new(1024);
        let mut package = create_test_package("<xml></xml>".to_string());

        // Change content type to XML
        let key = OutputKey::new(vec!["test".to_string()], "index.html");
        package.outputs.get_mut(&key).unwrap().content_type = ContentType::Xml;

        let original = package.outputs.get(&key).unwrap().data.as_text().unwrap().to_string();
        mw.process(&mut package).unwrap();

        let output_ref = package.outputs.get(&key).unwrap();
        let content = output_ref.data.as_text().unwrap();
        assert_eq!(content, &original);
        assert!(!content.contains("livereload.js"));
    }

    fn create_test_package(content: String) -> super::super::OutputPackage {
        use super::super::{ContentMetadata, OutputPackage};

        let metadata = ContentMetadata {
            path: PathBuf::from("test.md"),
            components: vec!["test".to_string()],
            filename: "index.html".to_string(),
            template_name: "page.html".to_string(),
            content_type: ContentType::Html,
            language: "en".to_string(),
            permalink: "http://example.com/test/".to_string(),
        };

        OutputPackage::new(content, metadata, Arc::new(config::Config::default_for_test()))
    }
}
