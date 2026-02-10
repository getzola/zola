use std::collections::HashMap;

use config::Config;
use markdown::{MarkdownContext, render_content};
use tera::{Error, Filter, Kwargs, State, TeraResult, Value};
use utils::types::InsertAnchor;

#[derive(Debug)]
pub struct MarkdownFilter {
    config: Config,
    permalinks: HashMap<String, String>,
    tera: tera::Tera,
}

impl MarkdownFilter {
    pub fn new(config: Config, permalinks: HashMap<String, String>, tera: tera::Tera) -> Self {
        Self { config, permalinks, tera }
    }
}

impl Filter<&str, TeraResult<String>> for MarkdownFilter {
    fn call(&self, value: &str, kwargs: Kwargs, state: &State) -> TeraResult<String> {
        let inline: bool = kwargs.get("inline")?.unwrap_or_default();

        // Extract page/section context from Tera state when available
        let page_or_section = state
            .get::<Value>("page")
            .ok()
            .flatten()
            .or_else(|| state.get::<Value>("section").ok().flatten());

        let (current_path, current_permalink, lang) = page_or_section
            .as_ref()
            .and_then(|v| v.as_map())
            .map(|map| {
                let path = map
                    .get(&"relative_path".into())
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let permalink =
                    map.get(&"permalink".into()).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let lang = state
                    .get::<String>("lang")
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| self.config.default_language.clone());
                (path, permalink, lang)
            })
            .unwrap_or_else(|| {
                (String::new(), String::new(), self.config.default_language.clone())
            });

        let context = MarkdownContext {
            tera: &self.tera,
            config: &self.config,
            permalinks: &self.permalinks,
            lang: &lang,
            current_permalink: &current_permalink,
            current_path: &current_path,
            insert_anchor: InsertAnchor::None,
        };

        let mut html = render_content(value, &context)
            .map_err(|e| Error::message(format!("Failed to render markdown filter: {:?}", e)))?
            .body;

        if inline {
            html = html
                .trim_start_matches("<p>")
                // pulldown_cmark finishes a paragraph with `</p>\n`
                .trim_end_matches("</p>\n")
                .to_string();
        }

        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use config::{Config, HighlightConfig, HighlightStyle, Highlighting, Registry};
    use giallo::DataAttrPosition;
    use tera::{Context, Filter, Kwargs, State, Value};

    use super::MarkdownFilter;

    fn get_test_registry() -> Registry {
        let mut registry = Registry::builtin().unwrap();
        registry.link_grammars();
        registry
    }

    #[test]
    fn markdown_filter() {
        let ctx = Context::new();
        let state = State::new(&ctx);
        let kwargs = Kwargs::from([]);

        let result = MarkdownFilter::new(Config::default(), HashMap::new(), tera::Tera::default())
            .call("# Hey", kwargs, &state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<h1 id=\"hey\">Hey</h1>\n");
    }

    #[test]
    fn markdown_filter_inline() {
        let ctx = Context::new();
        let state = State::new(&ctx);
        let kwargs = Kwargs::from([("inline", Value::from(true))]);

        let result = MarkdownFilter::new(Config::default(), HashMap::new(), tera::Tera::default())
            .call("Using `map`, `filter`, and `fold` instead of `for`", kwargs, &state);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Using <code>map</code>, <code>filter</code>, and <code>fold</code> instead of <code>for</code>"
        );
    }

    // https://github.com/Keats/gutenberg/issues/417
    #[test]
    fn markdown_filter_inline_tables() {
        let ctx = Context::new();
        let state = State::new(&ctx);
        let kwargs = Kwargs::from([("inline", Value::from(true))]);

        let result = MarkdownFilter::new(Config::default(), HashMap::new(), tera::Tera::default())
            .call(
                r#"
|id|author_id|       timestamp_created|title                 |content           |
|-:|--------:|-----------------------:|:---------------------|:-----------------|
| 1|        1|2018-09-05 08:03:43.141Z|How to train your ORM |Badly written blog|
| 2|        1|2018-08-22 13:11:50.050Z|How to bake a nice pie|Badly written blog|
        "#,
                kwargs,
                &state,
            );
        assert!(result.is_ok());
        assert!(result.unwrap().contains("<table>"));
    }

    #[test]
    fn markdown_filter_use_config_options() {
        let mut config = Config::default();
        config.markdown.highlighting = Some(Highlighting {
            error_on_missing_language: false,
            style: HighlightStyle::Inline,
            theme: HighlightConfig::Single { theme: "github-dark".to_string() },
            extra_grammars: vec![],
            extra_themes: vec![],
            data_attr_position: DataAttrPosition::default(),
            registry: get_test_registry(),
        });
        config.markdown.smart_punctuation = true;
        config.markdown.render_emoji = true;
        config.markdown.external_links_target_blank = true;

        let ctx = Context::new();
        let state = State::new(&ctx);

        let md = "Hello <https://google.com> :smile: ...";
        let kwargs = Kwargs::from([]);
        let result = MarkdownFilter::new(config.clone(), HashMap::new(), tera::Tera::default())
            .call(md, kwargs, &state);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "<p>Hello <a rel=\"noopener external\" target=\"_blank\" href=\"https://google.com\">https://google.com</a> 😄 …</p>\n"
        );

        let md = "```py\ni=0\n```";
        let kwargs = Kwargs::from([]);
        let result = MarkdownFilter::new(config, HashMap::new(), tera::Tera::default())
            .call(md, kwargs, &state);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("style"));
    }

    #[test]
    fn markdown_filter_can_use_internal_links() {
        let mut permalinks = HashMap::new();
        permalinks.insert("blog/_index.md".to_string(), "/foo/blog".to_string());

        let ctx = Context::new();
        let state = State::new(&ctx);
        let kwargs = Kwargs::from([]);

        let md = "Hello. Check out [my blog](@/blog/_index.md)!";
        let result = MarkdownFilter::new(Config::default(), permalinks, tera::Tera::default())
            .call(md, kwargs, &state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<p>Hello. Check out <a href=\"/foo/blog\">my blog</a>!</p>\n");
    }
}
