use config::Search;
use content::Library;

/// build index in Fuse.js format.
pub fn build_index(lang: &str, library: &Library, config: &Search) -> super::Result<String> {
    #[derive(serde::Serialize)]
    struct Item {
        title: Option<String>,
        body: Option<String>,
        path: String,
    }
    let mut items: Vec<Item> = Vec::new();
    for (_, section) in &library.sections {
        if section.lang == lang
            && section.meta.redirect_to.is_none()
            && section.meta.in_search_index
        {
            items.push(Item {
                path: section.path.clone(),
                title: if config.include_title {
                    Some(section.meta.title.clone().unwrap_or_default())
                } else {
                    None
                },
                body: if config.include_content {
                    Some(super::AMMONIA.clean(&section.content).to_string())
                } else {
                    None
                },
            });
            for page in &section.pages {
                let page = &library.pages[page];
                if page.meta.in_search_index {
                    items.push(Item {
                        title: if config.include_title {
                            Some(page.meta.title.clone().unwrap_or_default())
                        } else {
                            None
                        },
                        body: if config.include_content {
                            Some(super::AMMONIA.clean(&page.content).to_string())
                        } else {
                            None
                        },
                        path: page.path.clone(),
                    })
                }
            }
        }
    }
    Ok(serde_json::to_string(&items)?)
}
