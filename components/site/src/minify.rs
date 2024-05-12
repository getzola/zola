use errors::{bail, Result};
use libs::minify_html::{minify, Cfg};

pub fn html(html: String) -> Result<String> {
    let mut cfg = Cfg::spec_compliant();
    cfg.keep_html_and_head_opening_tags = true;
    cfg.minify_css = true;
    cfg.minify_js = true;

    let minified = minify(html.as_bytes(), &cfg);
    match std::str::from_utf8(&minified) {
        Ok(result) => Ok(result.to_string()),
        Err(err) => bail!("Failed to convert bytes to string : {}", err),
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

    // https://github.com/getzola/zola/issues/1304
    #[test]
    fn can_minify_multibyte_characters() {
        let input = r#"
俺が好きなのはキツネの…ケツねｗ
ー丁寧なインタネット生活の人より
"#;
        let expected = r#"俺が好きなのはキツネの…ケツねｗ ー丁寧なインタネット生活の人より"#;
        let res = html(input.to_owned()).unwrap();
        assert_eq!(res, expected);
    }

    // https://github.com/getzola/zola/issues/1300
    #[test]
    fn can_minify_and_preserve_whitespace_in_pre_elements() {
        let input = r#"
<!doctype html>
<html>
<head>
  <meta charset="utf-8">
</head>
<body>
  <pre><code>fn main() {
    println!("Hello, world!");
    <span>loop {
      println!("Hello, world!");
    }</span>
  }
  </code></pre>
</body>
</html>
"#;
        let expected = r#"<!doctype html><html><head><meta charset=utf-8><body><pre><code>fn main() {
    println!("Hello, world!");
    <span>loop {
      println!("Hello, world!");
    }</span>
  }
  </code></pre>"#;
        let res = html(input.to_owned()).unwrap();
        assert_eq!(res, expected);
    }

    // https://github.com/getzola/zola/issues/1765
    #[test]
    fn can_minify_css() {
        let input = r#"
<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    p {
      color: white;
      margin-left: 10000px;
    }
  </style>
</head>
<body>


    <p>Example blog post</p>

  FOO BAR
</body>
</html>
"#;
        let expected = r#"<!doctype html><html><head><meta charset=utf-8><style>p{color:#fff;margin-left:10000px}</style><body><p>Example blog post</p> FOO BAR"#;
        let res = html(input.to_owned()).unwrap();
        assert_eq!(res, expected);
    }

    // https://github.com/getzola/zola/issues/1765
    #[test]
    fn can_minify_js() {
        let input = r#"
<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <script>
    alert("Hello World!");
    console.log("Some information: %o", information);
  </script>
</head>
<body>


    <p>Example blog post</p>

  FOO BAR
</body>
</html>
"#;
        let expected = r#"<!doctype html><html><head><meta charset=utf-8><script>alert(`Hello World!`);console.log(`Some information: %o`,information)</script><body><p>Example blog post</p> FOO BAR"#;
        let res = html(input.to_owned()).unwrap();
        assert_eq!(res, expected);
    }
}
