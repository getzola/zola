use errors::{bail, Result};
use minify_html::{with_friendly_error, Cfg};

pub fn html(html: String) -> Result<String> {
    let cfg = &Cfg { minify_js: false, minify_css: false };
    let mut input_bytes = html.as_bytes().to_vec();

    match with_friendly_error(&mut input_bytes, cfg) {
        Ok(len) => match std::str::from_utf8(&input_bytes) {
            Ok(result) => Ok(result[..len].to_string()),
            Err(err) => bail!("Failed to convert bytes to string : {}", err),
        },
        Err(minify_error) => {
            bail!(
                "Failed to truncate html at character {}: {} \n {}",
                minify_error.position,
                minify_error.message,
                minify_error.code_context
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://github.com/getzola/zola/issues/1292
    #[test]
    fn can_minify_html() {
        let input = r#"
<!doctype html>
<html>
<head>
  <meta charset="utf-8">
</head>
<body>


    <p>Example blog post</p>

  FOO BAR
</body>
</html>
"#;
        let expected = r#"<!doctype html><html><head><meta charset=utf-8><body><p>Example blog post</p> FOO BAR"#;
        let res = html(input.to_owned()).unwrap();
        assert_eq!(res, expected);
    }
}
