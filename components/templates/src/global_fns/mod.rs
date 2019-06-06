use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};

use tera::{from_value, to_value, Error, Function as TeraFn, Result, Value};

use config::Config;
use image;
use image::GenericImageView;
use library::{Library, Taxonomy};
use utils::site::resolve_internal_link;

use imageproc;

#[macro_use]
mod macros;

mod load_data;

pub use self::load_data::LoadData;

#[derive(Debug)]
pub struct Trans {
    config: Config,
}
impl Trans {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}
impl TeraFn for Trans {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let key = required_arg!(String, args.get("key"), "`trans` requires a `key` argument.");
        let lang = optional_arg!(String, args.get("lang"), "`trans`: `lang` must be a string.")
            .unwrap_or_else(|| self.config.default_language.clone());
        let translations = &self.config.translations[lang.as_str()];
        Ok(to_value(&translations[key.as_str()]).unwrap())
    }
}

#[derive(Debug)]
pub struct GetUrl {
    config: Config,
    permalinks: HashMap<String, String>,
}
impl GetUrl {
    pub fn new(config: Config, permalinks: HashMap<String, String>) -> Self {
        Self { config, permalinks }
    }
}
impl TeraFn for GetUrl {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
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
        if path.starts_with("@/") {
            match resolve_internal_link(&path, &self.permalinks) {
                Ok(resolved) => Ok(to_value(resolved.permalink).unwrap()),
                Err(_) => {
                    Err(format!("Could not resolve URL for link `{}` not found.", path).into())
                }
            }
        } else {
            // anything else
            let mut permalink = self.config.make_permalink(&path);
            if !trailing_slash && permalink.ends_with('/') {
                permalink.pop(); // Removes the slash
            }

            if cachebust {
                permalink = format!("{}?t={}", permalink, self.config.build_timestamp.unwrap());
            }
            Ok(to_value(permalink).unwrap())
        }
    }
}

#[derive(Debug)]
pub struct ResizeImage {
    imageproc: Arc<Mutex<imageproc::Processor>>,
}
impl ResizeImage {
    pub fn new(imageproc: Arc<Mutex<imageproc::Processor>>) -> Self {
        Self { imageproc }
    }
}

static DEFAULT_OP: &'static str = "fill";
static DEFAULT_FMT: &'static str = "auto";
const DEFAULT_Q: u8 = 75;

impl TeraFn for ResizeImage {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
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

        let mut imageproc = self.imageproc.lock().unwrap();
        if !imageproc.source_exists(&path) {
            return Err(format!("`resize_image`: Cannot find path: {}", path).into());
        }

        let imageop = imageproc::ImageOp::from_args(path, &op, width, height, &format, quality)
            .map_err(|e| format!("`resize_image`: {}", e))?;
        let url = imageproc.insert(imageop);

        to_value(url).map_err(|err| err.into())
    }
}

#[derive(Debug)]
pub struct GetImageMeta {
    content_path: PathBuf,
}

impl GetImageMeta {
    pub fn new(content_path: PathBuf) -> Self {
        Self { content_path }
    }
}

impl TeraFn for GetImageMeta {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`get_image_meta` requires a `path` argument with a string value"
        );
        let src_path = self.content_path.join(&path);
        if !src_path.exists() {
            return Err(format!("`get_image_meta`: Cannot find path: {}", path).into());
        }
        let img = image::open(&src_path)
            .map_err(|e| Error::chain(format!("Failed to process image: {}", path), e))?;
        let mut map = tera::Map::new();
        map.insert(String::from("height"), Value::Number(tera::Number::from(img.height())));
        map.insert(String::from("width"), Value::Number(tera::Number::from(img.width())));
        Ok(Value::Object(map))
    }
}

#[derive(Debug)]
pub struct GetTaxonomyUrl {
    taxonomies: HashMap<String, HashMap<String, String>>,
}
impl GetTaxonomyUrl {
    pub fn new(all_taxonomies: &[Taxonomy]) -> Self {
        let mut taxonomies = HashMap::new();
        for taxonomy in all_taxonomies {
            let mut items = HashMap::new();
            for item in &taxonomy.items {
                items.insert(item.name.clone(), item.permalink.clone());
            }
            taxonomies.insert(taxonomy.kind.name.clone(), items);
        }
        Self { taxonomies }
    }
}
impl TeraFn for GetTaxonomyUrl {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
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
        let container = match self.taxonomies.get(&kind) {
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
    }
}

#[derive(Debug)]
pub struct GetPage {
    base_path: PathBuf,
    library: Arc<RwLock<Library>>,
}
impl GetPage {
    pub fn new(base_path: PathBuf, library: Arc<RwLock<Library>>) -> Self {
        Self { base_path: base_path.join("content"), library }
    }
}
impl TeraFn for GetPage {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`get_page` requires a `path` argument with a string value"
        );
        let full_path = self.base_path.join(&path);
        let library = self.library.read().unwrap();
        match library.get_page(&full_path) {
            Some(p) => Ok(to_value(p.to_serialized(&library)).unwrap()),
            None => Err(format!("Page `{}` not found.", path).into()),
        }
    }
}

#[derive(Debug)]
pub struct GetSection {
    base_path: PathBuf,
    library: Arc<RwLock<Library>>,
}
impl GetSection {
    pub fn new(base_path: PathBuf, library: Arc<RwLock<Library>>) -> Self {
        Self { base_path: base_path.join("content"), library }
    }
}
impl TeraFn for GetSection {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`get_section` requires a `path` argument with a string value"
        );

        let metadata_only = args
            .get("metadata_only")
            .map_or(false, |c| from_value::<bool>(c.clone()).unwrap_or(false));

        let full_path = self.base_path.join(&path);
        let library = self.library.read().unwrap();

        match library.get_section(&full_path) {
            Some(s) => {
                if metadata_only {
                    Ok(to_value(s.to_serialized_basic(&library)).unwrap())
                } else {
                    Ok(to_value(s.to_serialized(&library)).unwrap())
                }
            }
            None => Err(format!("Section `{}` not found.", path).into()),
        }
    }
}

#[derive(Debug)]
pub struct GetTaxonomy {
    library: Arc<RwLock<Library>>,
    taxonomies: HashMap<String, Taxonomy>,
}
impl GetTaxonomy {
    pub fn new(all_taxonomies: Vec<Taxonomy>, library: Arc<RwLock<Library>>) -> Self {
        let mut taxonomies = HashMap::new();
        for taxo in all_taxonomies {
            taxonomies.insert(taxo.kind.name.clone(), taxo);
        }
        Self { taxonomies, library }
    }
}
impl TeraFn for GetTaxonomy {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let kind = required_arg!(
            String,
            args.get("kind"),
            "`get_taxonomy` requires a `kind` argument with a string value"
        );

        match self.taxonomies.get(&kind) {
            Some(t) => Ok(to_value(t.to_serialized(&self.library.read().unwrap())).unwrap()),
            None => {
                Err(format!("`get_taxonomy` received an unknown taxonomy as kind: {}", kind).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GetTaxonomy, GetTaxonomyUrl, GetUrl, Trans};

    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    use tera::{to_value, Function, Value};

    use config::{Config, Taxonomy as TaxonomyConfig};
    use library::{Library, Taxonomy, TaxonomyItem};

    #[test]
    fn can_add_cachebust_to_url() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css?t=1");
    }

    #[test]
    fn can_add_trailing_slashes() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css/");
    }

    #[test]
    fn can_add_slashes_and_cachebust() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(true).unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css/?t=1");
    }

    #[test]
    fn can_link_to_some_static_file() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css");
    }

    #[test]
    fn can_get_taxonomy() {
        let config = Config::default();
        let taxo_config = TaxonomyConfig {
            name: "tags".to_string(),
            lang: config.default_language.clone(),
            ..TaxonomyConfig::default()
        };
        let library = Arc::new(RwLock::new(Library::new(0, 0, false)));
        let tag = TaxonomyItem::new(
            "Programming",
            &taxo_config,
            &config,
            vec![],
            &library.read().unwrap(),
        );
        let tags = Taxonomy { kind: taxo_config, items: vec![tag] };

        let taxonomies = vec![tags.clone()];
        let static_fn = GetTaxonomy::new(taxonomies.clone(), library.clone());
        // can find it correctly
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        let res = static_fn.call(&args).unwrap();
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
        assert!(static_fn.call(&args).is_err());
    }

    #[test]
    fn can_get_taxonomy_url() {
        let config = Config::default();
        let taxo_config = TaxonomyConfig {
            name: "tags".to_string(),
            lang: config.default_language.clone(),
            ..TaxonomyConfig::default()
        };
        let library = Library::new(0, 0, false);
        let tag = TaxonomyItem::new("Programming", &taxo_config, &config, vec![], &library);
        let tags = Taxonomy { kind: taxo_config, items: vec![tag] };

        let taxonomies = vec![tags.clone()];
        let static_fn = GetTaxonomyUrl::new(&taxonomies);
        // can find it correctly
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("name".to_string(), to_value("Programming").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            to_value("http://a-website.com/tags/programming/").unwrap()
        );
        // and errors if it can't find it
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("name".to_string(), to_value("random").unwrap());
        assert!(static_fn.call(&args).is_err());
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
        let static_fn = Trans::new(config);
        let mut args = HashMap::new();

        args.insert("key".to_string(), to_value("title").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "Un titre");

        args.insert("lang".to_string(), to_value("en").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "A title");

        args.insert("lang".to_string(), to_value("fr").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "Un titre");
    }
}
