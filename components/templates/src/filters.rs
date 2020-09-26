use std::collections::HashMap;
use std::hash::BuildHasher;

use base64::{decode, encode};
use tera::Tera;
use tera::{to_value, try_get_value, Result as TeraResult, Value};

use config::Config;
use front_matter::InsertAnchor;
use rendering::{render_content, RenderContext};

pub fn markdown<S: BuildHasher>(
    value: &Value,
    args: &HashMap<String, Value, S>,
) -> TeraResult<Value> {
    let s = try_get_value!("markdown", "value", String, value);
    let inline = match args.get("inline") {
        Some(val) => try_get_value!("markdown", "inline", bool, val),
        None => false,
    };
    let highlight = match args.get("highlight") {
        Some(val) => try_get_value!("markdown", "highlight", bool, val),
        None => false,
    };
    let highlight_theme = match args.get("highlight_theme") {
        Some(val) => try_get_value!("markdown", "highlight_theme", String, val),
        None => "base16-ocean-dark".to_string(),
    };

    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default();
    config.highlight_code = highlight;
    config.highlight_theme = highlight_theme;
    let context = RenderContext::new(&tera_ctx, &config, "", &permalinks_ctx, InsertAnchor::None);
    // Use the zola rendering module instead of build in one.
    let res = render_content(&s, &context).unwrap();
    let html: String;
    if inline {
        html = res.body.trim_start_matches("<p>").trim_end_matches("</p>\n").to_string();
    } else {
        html = res.body
    }

    Ok(to_value(&html).unwrap())
}

pub fn base64_encode<S: BuildHasher>(
    value: &Value,
    _: &HashMap<String, Value, S>,
) -> TeraResult<Value> {
    let s = try_get_value!("base64_encode", "value", String, value);
    Ok(to_value(&encode(s.as_bytes())).unwrap())
}

pub fn base64_decode<S: BuildHasher>(
    value: &Value,
    _: &HashMap<String, Value, S>,
) -> TeraResult<Value> {
    let s = try_get_value!("base64_decode", "value", String, value);
    Ok(to_value(&String::from_utf8(decode(s.as_bytes()).unwrap()).unwrap()).unwrap())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use tera::to_value;

    use super::{base64_decode, base64_encode, markdown};

    #[test]
    fn markdown_filter() {
        let result = markdown(&to_value(&"# Hey").unwrap(), &HashMap::new());
        assert!(result.is_ok());
        // zola render adds an id to the h1 tag
        assert_eq!(result.unwrap(), to_value(&"<h1 id=\"hey\">Hey</h1>\n").unwrap());
    }

    #[test]
    fn markdown_filter_highlight() {
        let mut args = HashMap::new();
        args.insert("highlight".to_string(), to_value(true).unwrap());
        let result = markdown(&to_value(&"```\n$ gutenberg server\n$ ping\n```").unwrap(), &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(&"<pre style=\"background-color:#2b303b;\">\n<code><span style=\"color:#c0c5ce;\">$ gutenberg server\n$ ping\n</span></code></pre>").unwrap());
    }

    #[test]
    fn markdown_filter_inline() {
        let mut args = HashMap::new();
        args.insert("inline".to_string(), to_value(true).unwrap());
        let result = markdown(
            &to_value(&"Using `map`, `filter`, and `fold` instead of `for`").unwrap(),
            &args,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(&"Using <code>map</code>, <code>filter</code>, and <code>fold</code> instead of <code>for</code>").unwrap());
    }

    #[test]
    fn markdown_filter_inline_highlight_theme() {
        let mut args = HashMap::new();
        args.insert("inline".to_string(), to_value(true).unwrap());
        args.insert("highlight".to_string(), to_value(true).unwrap());
        args.insert("highlight_theme".to_string(), to_value("gruvbox-light".to_string()).unwrap());
        let result = markdown(&to_value(&"```\n$ gutenberg server\n$ ping\n```").unwrap(), &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(&"<pre style=\"background-color:#fcf0ca;\">\n<code><span style=\"color:#282828aa;\">$ gutenberg server\n$ ping\n</span></code></pre>").unwrap());
    }

    // https://github.com/Keats/gutenberg/issues/417
    #[test]
    fn markdown_filter_inline_tables() {
        let mut args = HashMap::new();
        args.insert("inline".to_string(), to_value(true).unwrap());
        let result = markdown(
            &to_value(
                &r#"
|id|author_id|       timestamp_created|title                 |content           |
|-:|--------:|-----------------------:|:---------------------|:-----------------|
| 1|        1|2018-09-05 08:03:43.141Z|How to train your ORM |Badly written blog|
| 2|        1|2018-08-22 13:11:50.050Z|How to bake a nice pie|Badly written blog|
        "#,
            )
            .unwrap(),
            &args,
        );
        assert!(result.is_ok());
        assert!(result.unwrap().as_str().unwrap().contains("<table>"));
    }

    #[test]
    fn base64_encode_filter() {
        // from https://tools.ietf.org/html/rfc4648#section-10
        let tests = vec![
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ];
        for (input, expected) in tests {
            let args = HashMap::new();
            let result = base64_encode(&to_value(input).unwrap(), &args);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), to_value(expected).unwrap());
        }
    }

    #[test]
    fn base64_decode_filter() {
        let tests = vec![
            ("", ""),
            ("Zg==", "f"),
            ("Zm8=", "fo"),
            ("Zm9v", "foo"),
            ("Zm9vYg==", "foob"),
            ("Zm9vYmE=", "fooba"),
            ("Zm9vYmFy", "foobar"),
        ];
        for (input, expected) in tests {
            let args = HashMap::new();
            let result = base64_decode(&to_value(input).unwrap(), &args);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), to_value(expected).unwrap());
        }
    }
}
