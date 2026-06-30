use config::{Config, Search};
use content::Library;
use elasticlunr::{IndexBuilder, lang};
use errors::{Result, bail};
use time::format_description::well_known::Rfc3339;

use crate::{IndexItem, clean_and_truncate_body, collect_index_items};

pub const ELASTICLUNR_JS: &str = include_str!("elasticlunr.min.js");

fn build_fields(search_config: &Search, mut index: IndexBuilder) -> IndexBuilder {
    if search_config.include_title {
        index = index.add_field("title");
    }

    if search_config.include_description {
        index = index.add_field("description");
    }

    if search_config.include_date {
        index = index.add_field("date")
    }

    if search_config.include_path {
        index = index.add_field_with_tokenizer("path", Box::new(path_tokenizer));
    }

    if search_config.include_content {
        index = index.add_field("body")
    }

    index
}

fn path_tokenizer(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || c == '-' || c == '/')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_lowercase())
        .collect()
}

fn fill_index(search_config: &Search, item: IndexItem) -> Vec<String> {
    let mut row = vec![];

    if search_config.include_title {
        row.push(item.title.clone().unwrap_or_default());
    }

    if search_config.include_description {
        row.push(item.description.clone().unwrap_or_default());
    }

    if search_config.include_date
        && let Some(date) = item.datetime
        && let Ok(d) = date.format(&Rfc3339)
    {
        row.push(d);
    }

    if search_config.include_path {
        row.push(item.path.to_string());
    }

    if search_config.include_content {
        row.push(clean_and_truncate_body(search_config.truncate_content_length, item.content));
    }
    row
}

/// Returns the generated JSON index with all the documents of the site added using
/// the language given
/// Errors if the language given is not available in Elasticlunr
/// TODO: is making `in_search_index` apply to subsections of a `false` section useful?
pub fn build_index(lang: &str, library: &Library, config: &Config) -> Result<String> {
    let language = match lang::from_code(lang) {
        Some(l) => l,
        None => {
            bail!("Tried to build search index for language {} which is not supported", lang);
        }
    };
    let language_options = &config.languages[lang];
    let mut index = IndexBuilder::with_language(language);
    index = build_fields(&language_options.search, index);
    let mut index = index.build();

    let items = collect_index_items(lang, library);

    for item in items {
        index.add_doc(item.url, fill_index(&language_options.search, item));
    }

    Ok(index.to_json())
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::Config;
    use elasticlunr::IndexBuilder;
    use time::OffsetDateTime;

    #[test]
    fn can_build_fields() {
        let mut config = Config::default();
        let index = build_fields(&config.search, IndexBuilder::new()).build();
        assert_eq!(index.get_fields(), vec!["title", "body"]);

        config.search.include_content = false;
        config.search.include_description = true;
        let index = build_fields(&config.search, IndexBuilder::new()).build();
        assert_eq!(index.get_fields(), vec!["title", "description"]);

        config.search.include_content = true;
        let index = build_fields(&config.search, IndexBuilder::new()).build();
        assert_eq!(index.get_fields(), vec!["title", "description", "body"]);

        config.search.include_title = false;
        let index = build_fields(&config.search, IndexBuilder::new()).build();
        assert_eq!(index.get_fields(), vec!["description", "body"]);
    }

    #[test]
    fn can_fill_index_default() {
        let config = Config::default();
        let title = Some("A title".to_string());
        let description = Some("A description".to_string());
        let path = "/a/page/".to_string();
        let content = "Some content".to_string();

        let item = crate::IndexItem {
            url: "http://example.com",
            title: &title,
            description: &description,
            datetime: &None,
            path: &path,
            content: &content,
        };

        let res = fill_index(&config.search, item);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], title.unwrap());
        assert_eq!(res[1], content);
    }

    #[test]
    fn can_fill_index_description() {
        let mut config = Config::default();
        config.search.include_description = true;
        let title = Some("A title".to_string());
        let description = Some("A description".to_string());
        let path = "/a/page/".to_string();
        let content = "Some content".to_string();

        let item = crate::IndexItem {
            url: "http://example.com",
            title: &title,
            description: &description,
            datetime: &None,
            path: &path,
            content: &content,
        };

        let res = fill_index(&config.search, item);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0], title.unwrap());
        assert_eq!(res[1], description.unwrap());
        assert_eq!(res[2], content);
    }

    #[test]
    fn can_fill_index_truncated_content() {
        let mut config = Config::default();
        config.search.truncate_content_length = Some(5);
        let title = Some("A title".to_string());
        let description = Some("A description".to_string());
        let path = "/a/page/".to_string();
        let content = "Some content".to_string();

        let item = crate::IndexItem {
            url: "http://example.com",
            title: &title,
            description: &description,
            datetime: &None,
            path: &path,
            content: &content,
        };

        let res = fill_index(&config.search, item);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], title.unwrap());
        assert_eq!(res[1], content[..5]);
    }

    #[test]
    fn can_fill_index_date() {
        let mut config = Config::default();
        config.search.include_date = true;
        let title = Some("A title".to_string());
        let description = Some("A description".to_string());
        let path = "/a/page/".to_string();
        let content = "Some content".to_string();
        let datetime = Some(OffsetDateTime::parse("2023-01-31T00:00:00Z", &Rfc3339).unwrap());

        let item = crate::IndexItem {
            url: "http://example.com",
            title: &title,
            description: &description,
            datetime: &datetime,
            path: &path,
            content: &content,
        };

        let res = fill_index(&config.search, item);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0], title.unwrap());
        assert_eq!(res[1], "2023-01-31T00:00:00Z");
        assert_eq!(res[2], content);
    }
}
