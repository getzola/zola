use std::collections::HashMap;
use std::path::{PathBuf};

use tera::{GlobalFn, Value, from_value, to_value, Result};

use content::{Page, Section};
use config::Config;
use utils::site::resolve_internal_link;
use taxonomies::Taxonomy;


macro_rules! required_string_arg {
    ($e: expr, $err: expr) => {
        match $e {
            Some(v) => match from_value::<String>(v.clone()) {
                Ok(u) => u,
                Err(_) => return Err($err.into())
            },
            None => return Err($err.into())
        };
    };
}

pub fn make_get_page(all_pages: &HashMap<PathBuf, Page>) -> GlobalFn {
    let mut pages = HashMap::new();
    for page in all_pages.values() {
        pages.insert(page.file.relative.clone(), page.clone());
    }

    Box::new(move |args| -> Result<Value> {
        let path = required_string_arg!(args.get("path"), "`get_page` requires a `path` argument with a string value");
        match pages.get(&path) {
            Some(p) => Ok(to_value(p).unwrap()),
            None => Err(format!("Page `{}` not found.", path).into())
        }
    })
}

pub fn make_get_section(all_sections: &HashMap<PathBuf, Section>) -> GlobalFn {
    let mut sections = HashMap::new();
    for section in all_sections.values() {
        sections.insert(section.file.relative.clone(), section.clone());
    }

    Box::new(move |args| -> Result<Value> {
        let path = required_string_arg!(args.get("path"), "`get_section` requires a `path` argument with a string value");
        match sections.get(&path) {
            Some(p) => Ok(to_value(p).unwrap()),
            None => Err(format!("Section `{}` not found.", path).into())
        }
    })
}

pub fn make_get_url(permalinks: HashMap<String, String>, config: Config) -> GlobalFn {
    Box::new(move |args| -> Result<Value> {
        let cachebust = args
            .get("cachebust")
            .map_or(false, |c| {
                from_value::<bool>(c.clone()).unwrap_or(false)
            });

        let trailing_slash = args
            .get("trailing_slash")
            .map_or(true, |c| {
                from_value::<bool>(c.clone()).unwrap_or(true)
            });

        let path = required_string_arg!(args.get("path"), "`get_url` requires a `path` argument with a string value");
        if path.starts_with("./") {
            match resolve_internal_link(&path, &permalinks) {
                Ok(url) => Ok(to_value(url).unwrap()),
                Err(_) => Err(format!("Could not resolve URL for link `{}` not found.", path).into())
            }
        } else {
            // anything else
            let mut permalink = config.make_permalink(&path);
            if !trailing_slash && permalink.ends_with("/") {
                permalink.pop(); // Removes the slash
            }

            if cachebust {
                permalink = format!("{}?t={}", permalink, config.build_timestamp.unwrap());
            }
            Ok(to_value(permalink).unwrap())
        }
    })
}

pub fn make_get_taxonomy_url(tags: Option<Taxonomy>, categories: Option<Taxonomy>) -> GlobalFn {
    Box::new(move |args| -> Result<Value> {
        let kind = required_string_arg!(args.get("kind"), "`get_taxonomy_url` requires a `kind` argument with a string value");
        let name = required_string_arg!(args.get("name"), "`get_taxonomy_url` requires a `name` argument with a string value");
        let container = match kind.as_ref() {
            "tag" => &tags,
            "category" => &categories,
            _ => return Err("`get_taxonomy_url` can only get `tag` or `category` for the `kind` argument".into()),
        };

        if let Some(ref c) = *container {
            for item in &c.items {
                if item.name == name {
                    return Ok(to_value(item.permalink.clone()).unwrap());
                }
            }
            bail!("`get_taxonomy_url`: couldn't find `{}` in `{}` taxonomy", name, kind);
        } else {
            bail!("`get_taxonomy_url` tried to get a taxonomy of kind `{}` but there isn't any", kind);
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{make_get_url, make_get_taxonomy_url};

    use std::collections::HashMap;

    use tera::to_value;

    use config::Config;
    use taxonomies::{Taxonomy, TaxonomyKind, TaxonomyItem};


    #[test]
    fn can_add_cachebust_to_url() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css/?t=1");
    }

    #[test]
    fn can_remove_trailing_slashes() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(false).unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css");
    }

    #[test]
    fn can_remove_slashes_and_cachebust() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(false).unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css?t=1");
    }

    #[test]
    fn can_link_to_some_static_file() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css/");
    }

    #[test]
    fn can_get_tag_url() {
        let tag = TaxonomyItem::new(
            "Prog amming",
            TaxonomyKind::Tags,
            &Config::default(),
            vec![],
        );
        let tags = Taxonomy {
            kind: TaxonomyKind::Tags,
            items: vec![tag],
        };

        let static_fn = make_get_taxonomy_url(Some(tags), None);
        // can find it correctly
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tag").unwrap());
        args.insert("name".to_string(), to_value("Prog amming").unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/tags/prog-amming/");
        // and errors if it can't find it
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tag").unwrap());
        args.insert("name".to_string(), to_value("random").unwrap());
        assert!(static_fn(args).is_err());
    }
}
