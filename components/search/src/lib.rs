use std::collections::{HashMap, HashSet};

use elasticlunr::{Index, Language};
use lazy_static::lazy_static;

use config::Config;
use errors::{bail, Result};
use library::{Library, Section};

pub const ELASTICLUNR_JS: &str = include_str!("elasticlunr.min.js");

lazy_static! {
    static ref AMMONIA: ammonia::Builder<'static> = {
        let mut clean_content = HashSet::new();
        clean_content.insert("script");
        clean_content.insert("style");
        let mut builder = ammonia::Builder::new();
        builder
            .tags(HashSet::new())
            .tag_attributes(HashMap::new())
            .generic_attributes(HashSet::new())
            .link_rel(None)
            .allowed_classes(HashMap::new())
            .clean_content_tags(clean_content);
        builder
    };
}

fn build_fields(config: &Config) -> Vec<String> {
    let mut fields = vec![];
    if config.search.include_title {
        fields.push("title".to_owned());
    }

    if config.search.include_description {
        fields.push("description".to_owned());
    }

    if config.search.include_content {
        fields.push("body".to_owned());
    }

    fields
}

fn fill_index(
    config: &Config,
    title: &Option<String>,
    description: &Option<String>,
    content: &str,
) -> Vec<String> {
    let mut row = vec![];

    if config.search.include_title {
        row.push(title.clone().unwrap_or_default());
    }

    if config.search.include_description {
        row.push(description.clone().unwrap_or_default());
    }

    if config.search.include_content {
        let body = AMMONIA.clean(&content).to_string();
        if let Some(truncate_len) = config.search.truncate_content_length {
            // Not great for unicode
            // TODO: fix it like the truncate in Tera
            match body.char_indices().nth(truncate_len) {
                None => row.push(body),
                Some((idx, _)) => row.push((&body[..idx]).to_string()),
            };
        } else {
            row.push(body);
        };
    }

    row
}

/// Returns the generated JSON index with all the documents of the site added using
/// the language given
/// Errors if the language given is not available in Elasticlunr
/// TODO: is making `in_search_index` apply to subsections of a `false` section useful?
pub fn build_index(lang: &str, library: &Library, config: &Config) -> Result<String> {
    let language = match Language::from_code(lang) {
        Some(l) => l,
        None => {
            bail!("Tried to build search index for language {} which is not supported", lang);
        }
    };

    let mut index = Index::with_language(language, &build_fields(&config));

    for section in library.sections_values() {
        if section.lang == lang {
            add_section_to_index(&mut index, section, library, config);
        }
    }

    Ok(index.to_json())
}

fn add_section_to_index(index: &mut Index, section: &Section, library: &Library, config: &Config) {
    if !section.meta.in_search_index {
        return;
    }

    // Don't index redirecting sections
    if section.meta.redirect_to.is_none() {
        index.add_doc(
            &section.permalink,
            &fill_index(config, &section.meta.title, &section.meta.description, &section.content),
        );
    }

    for key in &section.pages {
        let page = library.get_page_by_key(*key);
        if !page.meta.in_search_index {
            continue;
        }

        index.add_doc(
            &page.permalink,
            &fill_index(config, &page.meta.title, &page.meta.description, &page.content),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use config::Config;

    #[test]
    fn can_build_fields() {
        let mut config = Config::default();
        let fields = build_fields(&config);
        assert_eq!(fields, vec!["title", "body"]);

        config.search.include_content = false;
        config.search.include_description = true;
        let fields = build_fields(&config);
        assert_eq!(fields, vec!["title", "description"]);

        config.search.include_content = true;
        let fields = build_fields(&config);
        assert_eq!(fields, vec!["title", "description", "body"]);

        config.search.include_title = false;
        let fields = build_fields(&config);
        assert_eq!(fields, vec!["description", "body"]);
    }

    #[test]
    fn can_fill_index_default() {
        let config = Config::default();
        let title = Some("A title".to_string());
        let description = Some("A description".to_string());
        let content = "Some content".to_string();

        let res = fill_index(&config, &title, &description, &content);
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
        let content = "Some content".to_string();

        let res = fill_index(&config, &title, &description, &content);
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
        let content = "Some content".to_string();

        let res = fill_index(&config, &title, &description, &content);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], title.unwrap());
        assert_eq!(res[1], content[..5]);
    }
}
