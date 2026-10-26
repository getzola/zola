use std::path::Path;

use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc_minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

pub fn css(source_text: &str) -> Option<String> {
    let popt = PrinterOptions { minify: true, ..Default::default() };
    let mut sty = StyleSheet::parse(source_text, ParserOptions::default()).ok()?;
    sty.minify(MinifyOptions::default()).ok()?;
    let out = sty.to_css(popt).ok()?;
    if out.code.len() < source_text.len() { Some(out.code) } else { None }
}

pub fn js(source_text: &str) -> Option<String> {
    let allocator = Allocator::default();
    let mut parser_ret = Parser::new(&allocator, source_text, SourceType::mjs()).parse();
    if parser_ret.panicked || !parser_ret.errors.is_empty() {
        let cjs_ret = Parser::new(&allocator, source_text, SourceType::cjs()).parse();
        if cjs_ret.panicked || !cjs_ret.errors.is_empty() {
            return None;
        }
        parser_ret = cjs_ret;
    }
    let mut program = parser_ret.program;
    let _ =
        Minifier::new(MinifierOptions { mangle: None, compress: Some(CompressOptions::safest()) })
            .minify(&allocator, &mut program);
    let codegen_options = CodegenOptions {
        minify: true,
        comments: CommentOptions::disabled(),
        ..CodegenOptions::default()
    };
    let minified = Codegen::new().with_options(codegen_options).build(&program).code;
    if minified.len() < source_text.len() { Some(minified) } else { None }
}

/// Returns minified bytes for a CSS or JS asset if it is valid UTF-8 and compressible.
/// If `path` is not a `.css` or `.js` file, or ends with `.min.css`/`.min.js`, or cannot
/// be minified further, returns `None`.
pub fn minify_asset_bytes(path: &Path, content: &[u8]) -> Option<Vec<u8>> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let is_css = name.ends_with(".css") && !name.ends_with(".min.css");
    let is_js = name.ends_with(".js") && !name.ends_with(".min.js");
    if !is_css && !is_js {
        return None;
    }
    let source_str = std::str::from_utf8(content).ok()?;
    let minified = if is_css { css(source_str) } else { js(source_str) };
    minified.map(|s| s.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_minify_comment_only_js() {
        assert_eq!(js("// test content\n"), Some("".to_string()));
    }

    #[test]
    fn can_minify_standalone_css() {
        let input = r#"
        .nav-item {
            color: #ffffff;
            margin: 10px 20px;
            &:hover {
                color: red;
            }
        }
        "#;
        let res = css(input).unwrap();
        assert!(res.len() < input.len());
        assert!(!res.contains("\n        "));
    }

    #[test]
    fn can_minify_standalone_js() {
        let input = r#"
        function greet(name) {
            const message = "Hello, " + name;
            console.log(message);
            return message;
        }
        "#;
        let res = js(input).unwrap();
        assert!(res.len() < input.len());
        assert!(!res.contains("const message"));
        assert!(!res.contains("function greet ("));
    }

    #[test]
    fn can_minify_standalone_js_preserves_globals() {
        let input = r#"
        function topLevelHandler(a, b) {
            var x = a + b;
            return x;
        }
        "#;
        let res = js(input).unwrap();
        assert!(res.len() < input.len());
        assert!(res.contains("topLevelHandler"));
    }

    #[test]
    fn test_minify_asset_bytes() {
        let css_bytes = b"body {\n  color: red;\n}";
        let min_css = minify_asset_bytes(Path::new("app.css"), css_bytes);
        assert!(min_css.is_some());
        assert_eq!(min_css.unwrap(), b"body{color:red}");

        let min_already = b"body{color:red}";
        assert!(minify_asset_bytes(Path::new("app.css"), min_already).is_none());

        let min_file = minify_asset_bytes(Path::new("app.min.css"), css_bytes);
        assert!(min_file.is_none());

        let js_bytes = b"function test() {\n  return 42;\n}";
        let min_js = minify_asset_bytes(Path::new("app.js"), js_bytes);
        assert!(min_js.is_some());

        let other = minify_asset_bytes(Path::new("image.png"), b"not code");
        assert!(other.is_none());
    }
}
