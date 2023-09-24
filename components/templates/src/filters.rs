use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::sync::{Arc, Mutex};

use config::Config;

use libs::base64::engine::{general_purpose::STANDARD as standard_b64, Engine};
use libs::regex::Regex;
use libs::tera::{
    to_value, try_get_value, Error as TeraError, Filter as TeraFilter, Result as TeraResult, Tera,
    Value,
};
use markdown::{render_content, RenderContext};

#[derive(Debug)]
pub struct MarkdownFilter {
    config: Config,
    permalinks: HashMap<String, String>,
    tera: Tera,
}

impl MarkdownFilter {
    pub fn new(config: Config, permalinks: HashMap<String, String>, tera: Tera) -> Self {
        Self { config, permalinks, tera }
    }
}

impl TeraFilter for MarkdownFilter {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
        // NOTE: RenderContext below is not aware of the current language
        // However, it should not be a problem because the surrounding tera
        // template has language context, and will most likely call a piece of
        // markdown respecting language preferences.
        let mut context = RenderContext::from_config(&self.config);
        context.permalinks = Cow::Borrowed(&self.permalinks);
        context.tera = Cow::Borrowed(&self.tera);
        let def = utils::templates::get_shortcodes(&self.tera);
        context.set_shortcode_definitions(&def);

        let s = try_get_value!("markdown", "value", String, value);
        let inline = match args.get("inline") {
            Some(val) => try_get_value!("markdown", "inline", bool, val),
            None => false,
        };
        let mut html = match render_content(&s, &context) {
            Ok(res) => res.body,
            Err(e) => return Err(format!("Failed to render markdown filter: {:?}", e).into()),
        };

        if inline {
            html = html
                .trim_start_matches("<p>")
                // pulldown_cmark finishes a paragraph with `</p>\n`
                .trim_end_matches("</p>\n")
                .to_string();
        }

        Ok(to_value(&html).unwrap())
    }
}

pub fn base64_encode<S: BuildHasher>(
    value: &Value,
    _: &HashMap<String, Value, S>,
) -> TeraResult<Value> {
    let s = try_get_value!("base64_encode", "value", String, value);
    Ok(to_value(standard_b64.encode(s.as_bytes())).unwrap())
}

pub fn base64_decode<S: BuildHasher>(
    value: &Value,
    _: &HashMap<String, Value, S>,
) -> TeraResult<Value> {
    let s = try_get_value!("base64_decode", "value", String, value);
    let decoded = standard_b64
        .decode(s.as_bytes())
        .map_err(|e| format!("`base64_decode`: failed to decode: {}", e))?;
    let as_str =
        String::from_utf8(decoded).map_err(|e| format!("`base64_decode`: invalid utf-8: {}", e))?;
    Ok(to_value(as_str).unwrap())
}

#[derive(Debug)]
pub struct RegexReplaceFilter {
    re_cache: Arc<Mutex<HashMap<String, Regex>>>,
}

impl RegexReplaceFilter {
    pub fn new() -> Self {
        Self { re_cache: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl TeraFilter for RegexReplaceFilter {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let text = try_get_value!("regex_replace", "value", String, value);
        let pattern = match args.get("pattern") {
            Some(val) => try_get_value!("regex_replace", "pattern", String, val),
            None => {
                return Err(TeraError::msg(
                    "Filter `regex_replace` expected an arg called `pattern`",
                ))
            }
        };
        let rep = match args.get("rep") {
            Some(val) => try_get_value!("regex_replace", "rep", String, val),
            None => {
                return Err(TeraError::msg("Filter `regex_replace` expected an arg called `rep`"))
            }
        };

        let mut cache = self.re_cache.lock().expect("re_cache lock");
        let replaced = {
            match cache.get(&pattern) {
                Some(pat) => pat.replace_all(&text, &rep),
                None => {
                    let pat = Regex::new(&pattern)
                        .map_err(|e| format!("`regex_replace`: failed to compile regex: {}", e))?;
                    let replaced = pat.replace_all(&text, &rep);
                    cache.insert(pattern, pat);
                    replaced
                }
            }
        };

        Ok(to_value(replaced).unwrap())
    }
}

#[derive(Debug)]
pub struct NumFormatFilter {
    default_language: String,
}

impl NumFormatFilter {
    pub fn new<S: Into<String>>(default_language: S) -> Self {
        Self { default_language: default_language.into() }
    }
}

impl TeraFilter for NumFormatFilter {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> TeraResult<Value> {
        use libs::num_format::{Locale, ToFormattedString};

        let num = try_get_value!("num_format", "value", i64, value);
        let locale = match args.get("locale") {
            Some(locale) => try_get_value!("num_format", "locale", String, locale),
            None => self.default_language.clone(),
        };
        let locale = Locale::from_name(&locale).map_err(|_| {
            TeraError::msg(format!(
                "Filter `num_format` was called with an invalid `locale` argument: `{}`.",
                locale
            ))
        })?;
        Ok(to_value(num.to_formatted_string(&locale)).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use libs::tera::{to_value, Filter, Tera};

    use super::{
        base64_decode, base64_encode, MarkdownFilter, NumFormatFilter, RegexReplaceFilter,
    };
    use config::Config;

    #[test]
    fn markdown_filter() {
        let result = MarkdownFilter::new(Config::default(), HashMap::new(), Tera::default())
            .filter(&to_value(&"# Hey").unwrap(), &HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(&"<h1 id=\"hey\">Hey</h1>\n").unwrap());
    }

    #[test]
    fn markdown_filter_override_lang() {
        // We're checking that we can use a workaround to explicitly provide `lang` in markdown filter from tera,
        // because otherwise markdown filter shortcodes are not aware of the current language
        // NOTE: This should also work for `nth` although i don't see a reason to do that
        let args = HashMap::new();
        let config = Config::default();
        let permalinks = HashMap::new();
        let mut tera = Tera::default();
        tera.add_raw_template("shortcodes/explicitlang.html", "a{{ lang }}a").unwrap();
        let filter = MarkdownFilter { config, permalinks, tera };
        let result = filter.filter(&to_value(&"{{ explicitlang(lang='jp') }}").unwrap(), &args);
        println!("{:?}", result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(&"ajpa").unwrap());
    }

    #[test]
    fn markdown_filter_inline() {
        let mut args = HashMap::new();
        args.insert("inline".to_string(), to_value(true).unwrap());
        let result = MarkdownFilter::new(Config::default(), HashMap::new(), Tera::default())
            .filter(
                &to_value(&"Using `map`, `filter`, and `fold` instead of `for`").unwrap(),
                &args,
            );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(&"Using <code>map</code>, <code>filter</code>, and <code>fold</code> instead of <code>for</code>").unwrap());
    }

    // https://github.com/Keats/gutenberg/issues/417
    #[test]
    fn markdown_filter_inline_tables() {
        let mut args = HashMap::new();
        args.insert("inline".to_string(), to_value(true).unwrap());
        let result = MarkdownFilter::new(Config::default(), HashMap::new(), Tera::default())
            .filter(
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
    fn markdown_filter_use_config_options() {
        let mut config = Config::default();
        config.markdown.highlight_code = true;
        config.markdown.smart_punctuation = true;
        config.markdown.render_emoji = true;
        config.markdown.external_links_target_blank = true;

        let md = "Hello <https://google.com> :smile: ...";
        let result = MarkdownFilter::new(config.clone(), HashMap::new(), Tera::default())
            .filter(&to_value(&md).unwrap(), &HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(&"<p>Hello <a rel=\"noopener\" target=\"_blank\" href=\"https://google.com\">https://google.com</a> 😄 …</p>\n").unwrap());

        let md = "```py\ni=0\n```";
        let result = MarkdownFilter::new(config, HashMap::new(), Tera::default())
            .filter(&to_value(&md).unwrap(), &HashMap::new());
        assert!(result.is_ok());
        assert!(result.unwrap().as_str().unwrap().contains("style"));
    }

    #[test]
    fn mardown_filter_can_use_internal_links() {
        let mut permalinks = HashMap::new();
        permalinks.insert("blog/_index.md".to_string(), "/foo/blog".to_string());
        let md = "Hello. Check out [my blog](@/blog/_index.md)!";
        let result = MarkdownFilter::new(Config::default(), permalinks, Tera::default())
            .filter(&to_value(&md).unwrap(), &HashMap::new());
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            to_value(&"<p>Hello. Check out <a href=\"/foo/blog\">my blog</a>!</p>\n").unwrap()
        );
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

    #[test]
    fn regex_replace_filter() {
        let value = "Springsteen, Bruce";
        let expected = "Bruce Springsteen";
        let pattern = r"(?P<last>[^,\s]+),\s+(?P<first>\S+)";
        let rep = "$first $last";
        let mut args = HashMap::new();
        args.insert("pattern".to_string(), to_value(pattern).unwrap());
        args.insert("rep".to_string(), to_value(rep).unwrap());
        let regex_replace = RegexReplaceFilter::new();
        let result = regex_replace.filter(&to_value(value).unwrap(), &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), to_value(expected).unwrap());
        assert!(regex_replace.re_cache.lock().unwrap().contains_key(pattern));
    }

    #[test]
    fn num_format_filter() {
        let tests = vec![
            (100, "100"),
            (1_000, "1,000"),
            (10_000, "10,000"),
            (100_000, "100,000"),
            (1_000_000, "1,000,000"),
        ];

        for (input, expected) in tests {
            let args = HashMap::new();
            let result = NumFormatFilter::new("en").filter(&to_value(input).unwrap(), &args);
            let result = dbg!(result);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), to_value(expected).unwrap());
        }
    }

    #[test]
    fn num_format_filter_with_locale() {
        let tests = vec![
            ("en", 1_000_000, "1,000,000"),
            ("en-IN", 1_000_000, "10,00,000"),
            // Note:
            // U+202F is the "NARROW NO-BREAK SPACE" code point.
            // When displayed to the screen, it looks like a space.
            ("fr", 1_000_000, "1\u{202f}000\u{202f}000"),
        ];

        for (locale, input, expected) in tests {
            let mut args = HashMap::new();
            args.insert("locale".to_string(), to_value(locale).unwrap());
            let result = NumFormatFilter::new("en").filter(&to_value(input).unwrap(), &args);
            let result = dbg!(result);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), to_value(expected).unwrap());
        }
    }
}
