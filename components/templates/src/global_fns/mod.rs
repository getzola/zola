use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tera::{from_value, to_value, GlobalFn, Result, Value};

use config::Config;
use library::{Library, Taxonomy};
use utils::site::resolve_internal_link;

use imageproc;

#[macro_use]
mod macros;

mod load_data;

pub use self::load_data::make_load_data;

pub fn make_trans(config: Config) -> GlobalFn {
    let translations_config = config.translations;
    let default_lang = config.default_language.clone();

    Box::new(move |args| -> Result<Value> {
        let key = required_arg!(String, args.get("key"), "`trans` requires a `key` argument.");
        let lang = optional_arg!(String, args.get("lang"), "`trans`: `lang` must be a string.")
            .unwrap_or_else(|| default_lang.clone());
        let translations = &translations_config[lang.as_str()];
        Ok(to_value(&translations[key.as_str()]).unwrap())
    })
}

pub fn make_get_page(library: &Library) -> GlobalFn {
    let mut pages = HashMap::new();
    for page in library.pages_values() {
        pages.insert(
            page.file.relative.clone(),
            to_value(library.get_page(&page.file.path).unwrap().to_serialized(library)).unwrap(),
        );
    }

    Box::new(move |args| -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`get_page` requires a `path` argument with a string value"
        );
        match pages.get(&path) {
            Some(p) => Ok(p.clone()),
            None => Err(format!("Page `{}` not found.", path).into()),
        }
    })
}

pub fn make_get_section(library: &Library) -> GlobalFn {
    let mut sections = HashMap::new();
    let mut sections_basic = HashMap::new();
    for section in library.sections_values() {
        sections.insert(
            section.file.relative.clone(),
            to_value(library.get_section(&section.file.path).unwrap().to_serialized(library))
                .unwrap(),
        );

        sections_basic.insert(
            section.file.relative.clone(),
            to_value(library.get_section(&section.file.path).unwrap().to_serialized_basic(library))
                .unwrap(),
        );
    }

    Box::new(move |args| -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`get_section` requires a `path` argument with a string value"
        );

        let metadata_only = args
            .get("metadata_only")
            .map_or(false, |c| from_value::<bool>(c.clone()).unwrap_or(false));

        let container = if metadata_only { &sections_basic } else { &sections };

        match container.get(&path) {
            Some(p) => Ok(p.clone()),
            None => Err(format!("Section `{}` not found.", path).into()),
        }
    })
}

pub fn make_get_url(permalinks: HashMap<String, String>, config: Config) -> GlobalFn {
    Box::new(move |args| -> Result<Value> {
        let cachebust =
            args.get("cachebust").map_or(false, |c| from_value::<bool>(c.clone()).unwrap_or(false));

        let trailing_slash = args
            .get("trailing_slash")
            .map_or(false, |c| from_value::<bool>(c.clone()).unwrap_or(false));

        let path = required_arg!(
            String,
            args.get("path"),
            "`get_url` requires a `path` argument with a string value"
        );
        if path.starts_with("./") {
            match resolve_internal_link(&path, &permalinks) {
                Ok(url) => Ok(to_value(url).unwrap()),
                Err(_) => {
                    Err(format!("Could not resolve URL for link `{}` not found.", path).into())
                }
            }
        } else {
            // anything else
            let mut permalink = config.make_permalink(&path);
            if !trailing_slash && permalink.ends_with('/') {
                permalink.pop(); // Removes the slash
            }

            if cachebust {
                permalink = format!("{}?t={}", permalink, config.build_timestamp.unwrap());
            }
            Ok(to_value(permalink).unwrap())
        }
    })
}

pub fn make_get_taxonomy(all_taxonomies: &[Taxonomy], library: &Library) -> GlobalFn {
    let mut taxonomies = HashMap::new();
    for taxonomy in all_taxonomies {
        taxonomies
            .insert(taxonomy.kind.name.clone(), to_value(taxonomy.to_serialized(library)).unwrap());
    }

    Box::new(move |args| -> Result<Value> {
        let kind = required_arg!(
            String,
            args.get("kind"),
            "`get_taxonomy` requires a `kind` argument with a string value"
        );
        let container = match taxonomies.get(&kind) {
            Some(c) => c,
            None => {
                return Err(format!(
                    "`get_taxonomy` received an unknown taxonomy as kind: {}",
                    kind
                )
                .into());
            }
        };

        Ok(to_value(container).unwrap())
    })
}

pub fn make_get_taxonomy_url(all_taxonomies: &[Taxonomy]) -> GlobalFn {
    let mut taxonomies = HashMap::new();
    for taxonomy in all_taxonomies {
        let mut items = HashMap::new();
        for item in &taxonomy.items {
            items.insert(item.name.clone(), item.permalink.clone());
        }
        taxonomies.insert(taxonomy.kind.name.clone(), items);
    }

    Box::new(move |args| -> Result<Value> {
        let kind = required_arg!(
            String,
            args.get("kind"),
            "`get_taxonomy_url` requires a `kind` argument with a string value"
        );
        let name = required_arg!(
            String,
            args.get("name"),
            "`get_taxonomy_url` requires a `name` argument with a string value"
        );
        let container = match taxonomies.get(&kind) {
            Some(c) => c,
            None => {
                return Err(format!(
                    "`get_taxonomy_url` received an unknown taxonomy as kind: {}",
                    kind
                )
                .into());
            }
        };

        if let Some(permalink) = container.get(&name) {
            return Ok(to_value(permalink).unwrap());
        }

        Err(format!("`get_taxonomy_url`: couldn't find `{}` in `{}` taxonomy", name, kind).into())
    })
}

pub fn make_resize_image(imageproc: Arc<Mutex<imageproc::Processor>>) -> GlobalFn {
    static DEFAULT_OP: &'static str = "fill";
    static DEFAULT_FMT: &'static str = "auto";
    const DEFAULT_Q: u8 = 75;

    Box::new(move |args| -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`resize_image` requires a `path` argument with a string value"
        );
        let width = optional_arg!(
            u32,
            args.get("width"),
            "`resize_image`: `width` must be a non-negative integer"
        );
        let height = optional_arg!(
            u32,
            args.get("height"),
            "`resize_image`: `height` must be a non-negative integer"
        );
        let op = optional_arg!(String, args.get("op"), "`resize_image`: `op` must be a string")
            .unwrap_or_else(|| DEFAULT_OP.to_string());

        let format =
            optional_arg!(String, args.get("format"), "`resize_image`: `format` must be a string")
                .unwrap_or_else(|| DEFAULT_FMT.to_string());

        let quality =
            optional_arg!(u8, args.get("quality"), "`resize_image`: `quality` must be a number")
                .unwrap_or(DEFAULT_Q);
        if quality == 0 || quality > 100 {
            return Err("`resize_image`: `quality` must be in range 1-100".to_string().into());
        }

        let mut imageproc = imageproc.lock().unwrap();
        if !imageproc.source_exists(&path) {
            return Err(format!("`resize_image`: Cannot find path: {}", path).into());
        }

        let imageop = imageproc::ImageOp::from_args(path, &op, width, height, &format, quality)
            .map_err(|e| format!("`resize_image`: {}", e))?;
        let url = imageproc.insert(imageop);

        to_value(url).map_err(|err| err.into())
    })
}

#[cfg(test)]
mod tests {
    use super::{make_get_taxonomy, make_get_taxonomy_url, make_get_url, make_trans};

    use std::collections::HashMap;

    use tera::{to_value, Value};

    use config::{Config, Taxonomy as TaxonomyConfig};
    use library::{Library, Taxonomy, TaxonomyItem};

    #[test]
    fn can_add_cachebust_to_url() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css?t=1");
    }

    #[test]
    fn can_add_trailing_slashes() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css/");
    }

    #[test]
    fn can_add_slashes_and_cachebust() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(true).unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css/?t=1");
    }

    #[test]
    fn can_link_to_some_static_file() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css");
    }

    #[test]
    fn can_get_taxonomy() {
        let taxo_config = TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        let library = Library::new(0, 0, false);
        let tag = TaxonomyItem::new("Programming", &taxo_config, &Config::default(), vec![], &library);
        let tags = Taxonomy { kind: taxo_config, items: vec![tag] };

        let taxonomies = vec![tags.clone()];
        let static_fn = make_get_taxonomy(&taxonomies, &library);
        // can find it correctly
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        let res = static_fn(args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["kind"], to_value(tags.kind).unwrap());
        assert_eq!(res_obj["items"].clone().as_array().unwrap().len(), 1);
        assert_eq!(
            res_obj["items"].clone().as_array().unwrap()[0].clone().as_object().unwrap()["name"],
            Value::String("Programming".to_string())
        );
        assert_eq!(
            res_obj["items"].clone().as_array().unwrap()[0].clone().as_object().unwrap()["slug"],
            Value::String("programming".to_string())
        );
        assert_eq!(
            res_obj["items"].clone().as_array().unwrap()[0].clone().as_object().unwrap()
                ["permalink"],
            Value::String("http://a-website.com/tags/programming/".to_string())
        );
        assert_eq!(
            res_obj["items"].clone().as_array().unwrap()[0].clone().as_object().unwrap()["pages"],
            Value::Array(vec![])
        );
        // and errors if it can't find it
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("something-else").unwrap());
        assert!(static_fn(args).is_err());
    }

    #[test]
    fn can_get_taxonomy_url() {
        let taxo_config = TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        let library = Library::new(0, 0, false);
        let tag = TaxonomyItem::new("Programming", &taxo_config, &Config::default(), vec![], &library);
        let tags = Taxonomy { kind: taxo_config, items: vec![tag] };

        let taxonomies = vec![tags.clone()];
        let static_fn = make_get_taxonomy_url(&taxonomies);
        // can find it correctly
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("name".to_string(), to_value("Programming").unwrap());
        assert_eq!(
            static_fn(args).unwrap(),
            to_value("http://a-website.com/tags/programming/").unwrap()
        );
        // and errors if it can't find it
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("name".to_string(), to_value("random").unwrap());
        assert!(static_fn(args).is_err());
    }

    #[test]
    fn can_translate_a_string() {
        let trans_config = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr"

[translations]
[translations.fr]
title = "Un titre"

[translations.en]
title = "A title"

        "#;

        let config = Config::parse(trans_config).unwrap();
        let static_fn = make_trans(config);
        let mut args = HashMap::new();

        args.insert("key".to_string(), to_value("title").unwrap());
        assert_eq!(static_fn(args.clone()).unwrap(), "Un titre");

        args.insert("lang".to_string(), to_value("en").unwrap());
        assert_eq!(static_fn(args.clone()).unwrap(), "A title");

        args.insert("lang".to_string(), to_value("fr").unwrap());
        assert_eq!(static_fn(args.clone()).unwrap(), "Un titre");
    }
}
