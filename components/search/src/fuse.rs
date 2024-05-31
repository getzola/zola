use config::Search;
use content::Library;
use errors::Result;
use libs::serde_json;

use crate::clean_and_truncate_body;

/// build index in Fuse.js format.
pub fn build_index(lang: &str, library: &Library, config: &Search) -> Result<String> {
    #[derive(serde::Serialize)]
    struct Item<'a> {
        url: &'a str,
        title: Option<&'a str>,
        description: Option<&'a str>,
        body: Option<String>, // AMMONIA.clean has to allocate anyway
        path: Option<&'a str>,
    }
    let mut items: Vec<Item> = Vec::new();
    for (_, section) in &library.sections {
        if section.lang == lang
            && section.meta.redirect_to.is_none()
            && section.meta.in_search_index
        {
            items.push(Item {
                url: &section.permalink,
                title: match config.include_title {
                    true => Some(&section.meta.title.as_deref().unwrap_or_default()),
                    false => None,
                },
                description: match config.include_description {
                    true => Some(&section.meta.description.as_deref().unwrap_or_default()),
                    false => None,
                },
                body: match config.include_content {
                    true => Some(clean_and_truncate_body(
                        config.truncate_content_length,
                        &section.content,
                    )),
                    false => None,
                },
                path: match config.include_path {
                    true => Some(&section.path),
                    false => None,
                },
            });
            for page in &section.pages {
                let page = &library.pages[page];
                if page.meta.in_search_index {
                    items.push(Item {
                        url: &page.permalink,
                        title: match config.include_title {
                            true => Some(&page.meta.title.as_deref().unwrap_or_default()),
                            false => None,
                        },
                        description: match config.include_description {
                            true => Some(&page.meta.description.as_deref().unwrap_or_default()),
                            false => None,
                        },
                        body: match config.include_content {
                            true => Some(super::clean_and_truncate_body(
                                config.truncate_content_length,
                                &page.content,
                            )),
                            false => None,
                        },
                        path: match config.include_path {
                            true => Some(&page.path),
                            false => None,
                        },
                    })
                }
            }
        }
    }
    Ok(serde_json::to_string(&items)?)
}
