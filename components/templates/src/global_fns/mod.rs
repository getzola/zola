use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::{fs, io, result};

use sha2::{Digest, Sha256, Sha384, Sha512};
use svg_metadata as svg;
use tera::{from_value, to_value, Error, Function as TeraFn, Result, Value};

use config::Config;
use image::GenericImageView;
use library::{Library, Taxonomy};
use utils::site::resolve_internal_link;
use utils::slugs::{slugify_paths, SlugifyStrategy};

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

        let term = self
            .config
            .get_translation(lang, key)
            .map_err(|e| Error::chain("Failed to retrieve term translation", e))?;

        Ok(to_value(term).unwrap())
    }
}

#[derive(Debug)]
pub struct GetUrl {
    config: Config,
    permalinks: HashMap<String, String>,
    search_paths: Vec<PathBuf>,
}
impl GetUrl {
    pub fn new(
        config: Config,
        permalinks: HashMap<String, String>,
        search_paths: Vec<PathBuf>,
    ) -> Self {
        Self { config, permalinks, search_paths }
    }
}

fn make_path_with_lang(path: String, lang: &str, config: &Config) -> Result<String> {
    if lang == config.default_language {
        return Ok(path);
    }

    if !config.languages.iter().any(|x| x.code == lang) {
        return Err(
            format!("`{}` is not an authorized language (check config.languages).", lang).into()
        );
    }

    let mut splitted_path: Vec<String> = path.split('.').map(String::from).collect();
    let ilast = splitted_path.len() - 1;
    splitted_path[ilast] = format!("{}.{}", lang, splitted_path[ilast]);
    Ok(splitted_path.join("."))
}

fn open_file(search_paths: &[PathBuf], url: &str) -> result::Result<fs::File, io::Error> {
    let cleaned_url = url.trim_start_matches("@/").trim_start_matches('/');
    for base_path in search_paths {
        match fs::File::open(base_path.join(cleaned_url)) {
            Ok(f) => return Ok(f),
            Err(_) => continue,
        };
    }
    Err(io::Error::from(io::ErrorKind::NotFound))
}

fn compute_file_sha256(mut file: fs::File) -> result::Result<String, io::Error> {
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn compute_file_sha384(mut file: fs::File) -> result::Result<String, io::Error> {
    let mut hasher = Sha384::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn compute_file_sha512(mut file: fs::File) -> result::Result<String, io::Error> {
    let mut hasher = Sha512::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

fn file_not_found_err(search_paths: &[PathBuf], url: &str) -> Result<Value> {
    Err(format!(
        "file `{}` not found; searched in{}",
        url,
        search_paths.iter().fold(String::new(), |acc, arg| acc + " " + arg.to_str().unwrap())
    )
    .into())
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

        let lang = optional_arg!(String, args.get("lang"), "`get_url`: `lang` must be a string.")
            .unwrap_or_else(|| self.config.default_language.clone());

        if path.starts_with("@/") {
            let path_with_lang = match make_path_with_lang(path, &lang, &self.config) {
                Ok(x) => x,
                Err(e) => return Err(e),
            };

            match resolve_internal_link(&path_with_lang, &self.permalinks) {
                Ok(resolved) => Ok(to_value(resolved.permalink).unwrap()),
                Err(_) => {
                    Err(format!("Could not resolve URL for link `{}` not found.", path_with_lang)
                        .into())
                }
            }
        } else {
            // anything else
            let mut permalink = self.config.make_permalink(&path);
            if !trailing_slash && permalink.ends_with('/') {
                permalink.pop(); // Removes the slash
            }

            if cachebust {
                match open_file(&self.search_paths, &path).and_then(compute_file_sha256) {
                    Ok(hash) => {
                        permalink = format!("{}?h={}", permalink, hash);
                    }
                    Err(_) => return file_not_found_err(&self.search_paths, &path),
                };
            }
            Ok(to_value(permalink).unwrap())
        }
    }
}

#[derive(Debug)]
pub struct GetFileHash {
    search_paths: Vec<PathBuf>,
}
impl GetFileHash {
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self { search_paths }
    }
}

const DEFAULT_SHA_TYPE: u16 = 384;

impl TeraFn for GetFileHash {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`get_file_hash` requires a `path` argument with a string value"
        );
        let sha_type = optional_arg!(
            u16,
            args.get("sha_type"),
            "`get_file_hash`: `sha_type` must be 256, 384 or 512"
        )
        .unwrap_or(DEFAULT_SHA_TYPE);

        let compute_hash_fn = match sha_type {
            256 => compute_file_sha256,
            384 => compute_file_sha384,
            512 => compute_file_sha512,
            _ => return Err("`get_file_hash`: `sha_type` must be 256, 384 or 512".into()),
        };

        let hash = open_file(&self.search_paths, &path).and_then(compute_hash_fn);

        match hash {
            Ok(digest) => Ok(to_value(digest).unwrap()),
            Err(_) => file_not_found_err(&self.search_paths, &path),
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

static DEFAULT_OP: &str = "fill";
static DEFAULT_FMT: &str = "auto";
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
            "`get_image_metadata` requires a `path` argument with a string value"
        );
        let src_path = self.content_path.join(&path);
        if !src_path.exists() {
            return Err(format!("`get_image_metadata`: Cannot find path: {}", path).into());
        }
        let (height, width) = image_dimensions(&src_path)?;
        let mut map = tera::Map::new();
        map.insert(String::from("height"), Value::Number(tera::Number::from(height)));
        map.insert(String::from("width"), Value::Number(tera::Number::from(width)));
        Ok(Value::Object(map))
    }
}

// Try to read the image dimensions for a given image
fn image_dimensions(path: &PathBuf) -> Result<(u32, u32)> {
    if let Some("svg") = path.extension().and_then(OsStr::to_str) {
        let img = svg::Metadata::parse_file(&path)
            .map_err(|e| Error::chain(format!("Failed to process SVG: {}", path.display()), e))?;
        match (img.height(), img.width(), img.view_box()) {
            (Some(h), Some(w), _) => Ok((h as u32, w as u32)),
            (_, _, Some(view_box)) => Ok((view_box.height as u32, view_box.width as u32)),
            _ => Err("Invalid dimensions: SVG width/height and viewbox not set.".into()),
        }
    } else {
        let img = image::open(&path)
            .map_err(|e| Error::chain(format!("Failed to process image: {}", path.display()), e))?;
        Ok((img.height(), img.width()))
    }
}

#[derive(Debug)]
pub struct GetTaxonomyUrl {
    taxonomies: HashMap<String, HashMap<String, String>>,
    default_lang: String,
    slugify: SlugifyStrategy,
}

impl GetTaxonomyUrl {
    pub fn new(default_lang: &str, all_taxonomies: &[Taxonomy], slugify: SlugifyStrategy) -> Self {
        let mut taxonomies = HashMap::new();
        for taxo in all_taxonomies {
            let mut items = HashMap::new();
            for item in &taxo.items {
                items.insert(slugify_paths(&item.name.clone(), slugify), item.permalink.clone());
            }
            taxonomies.insert(format!("{}-{}", taxo.kind.name, taxo.kind.lang), items);
        }
        Self { taxonomies, default_lang: default_lang.to_string(), slugify }
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
        let lang =
            optional_arg!(String, args.get("lang"), "`get_taxonomy`: `lang` must be a string")
                .unwrap_or_else(|| self.default_lang.clone());

        let container = match self.taxonomies.get(&format!("{}-{}", kind, lang)) {
            Some(c) => c,
            None => {
                return Err(format!(
                    "`get_taxonomy_url` received an unknown taxonomy as kind: {}",
                    kind
                )
                .into());
            }
        };

        if let Some(permalink) = container.get(&slugify_paths(&name, self.slugify)) {
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
    default_lang: String,
}
impl GetTaxonomy {
    pub fn new(
        default_lang: &str,
        all_taxonomies: Vec<Taxonomy>,
        library: Arc<RwLock<Library>>,
    ) -> Self {
        let mut taxonomies = HashMap::new();
        for taxo in all_taxonomies {
            taxonomies.insert(format!("{}-{}", taxo.kind.name, taxo.kind.lang), taxo);
        }
        Self { taxonomies, library, default_lang: default_lang.to_string() }
    }
}
impl TeraFn for GetTaxonomy {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let kind = required_arg!(
            String,
            args.get("kind"),
            "`get_taxonomy` requires a `kind` argument with a string value"
        );

        let lang =
            optional_arg!(String, args.get("lang"), "`get_taxonomy`: `lang` must be a string")
                .unwrap_or_else(|| self.default_lang.clone());

        match self.taxonomies.get(&format!("{}-{}", kind, lang)) {
            Some(t) => Ok(to_value(t.to_serialized(&self.library.read().unwrap())).unwrap()),
            None => {
                Err(format!("`get_taxonomy` received an unknown taxonomy as kind: {}", kind).into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GetFileHash, GetTaxonomy, GetTaxonomyUrl, GetUrl, Trans};

    use std::collections::HashMap;
    use std::env::temp_dir;
    use std::fs::remove_dir_all;
    use std::path::PathBuf;
    use std::sync::{Arc, RwLock};

    use lazy_static::lazy_static;

    use tera::{to_value, Function, Value};

    use config::{Config, Taxonomy as TaxonomyConfig};
    use library::{Library, Taxonomy, TaxonomyItem};
    use utils::fs::{create_directory, create_file};
    use utils::slugs::SlugifyStrategy;

    struct TestContext {
        static_path: PathBuf,
    }
    impl TestContext {
        fn setup() -> Self {
            let dir = temp_dir().join("static");
            create_directory(&dir).expect("Could not create test directory");
            create_file(&dir.join("app.css"), "// Hello world!")
                .expect("Could not create test content (app.css)");
            Self { static_path: dir }
        }
    }
    impl Drop for TestContext {
        fn drop(&mut self) {
            remove_dir_all(&self.static_path).expect("Could not free test directory");
        }
    }

    lazy_static! {
        static ref TEST_CONTEXT: TestContext = TestContext::setup();
    }

    #[test]
    fn can_add_cachebust_to_url() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new(), vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css?h=572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840");
    }

    #[test]
    fn can_add_trailing_slashes() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new(), vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css/");
    }

    #[test]
    fn can_add_slashes_and_cachebust() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new(), vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(true).unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css/?h=572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840");
    }

    #[test]
    fn can_link_to_some_static_file() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new(), vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css");
    }

    #[test]
    fn can_get_taxonomy() {
        let mut config = Config::default();
        config.slugify.taxonomies = SlugifyStrategy::On;
        let taxo_config = TaxonomyConfig {
            name: "tags".to_string(),
            lang: config.default_language.clone(),
            ..TaxonomyConfig::default()
        };
        let taxo_config_fr = TaxonomyConfig {
            name: "tags".to_string(),
            lang: "fr".to_string(),
            ..TaxonomyConfig::default()
        };
        let library = Arc::new(RwLock::new(Library::new(0, 0, false)));
        let tag = TaxonomyItem::new(
            "Programming",
            &taxo_config,
            "tags",
            &config,
            vec![],
            &library.read().unwrap(),
        );
        let tag_fr = TaxonomyItem::new(
            "Programmation",
            &taxo_config_fr,
            "tags",
            &config,
            vec![],
            &library.read().unwrap(),
        );
        let tags = Taxonomy { kind: taxo_config, slug: "tags".to_string(), items: vec![tag] };
        let tags_fr =
            Taxonomy { kind: taxo_config_fr, slug: "tags".to_string(), items: vec![tag_fr] };

        let taxonomies = vec![tags.clone(), tags_fr.clone()];
        let static_fn =
            GetTaxonomy::new(&config.default_language, taxonomies.clone(), library.clone());
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
        // Works with other languages as well
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("lang".to_string(), to_value("fr").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["kind"], to_value(tags_fr.kind).unwrap());
        assert_eq!(res_obj["items"].clone().as_array().unwrap().len(), 1);
        assert_eq!(
            res_obj["items"].clone().as_array().unwrap()[0].clone().as_object().unwrap()["name"],
            Value::String("Programmation".to_string())
        );

        // and errors if it can't find it
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("something-else").unwrap());
        assert!(static_fn.call(&args).is_err());
    }

    #[test]
    fn can_get_taxonomy_url() {
        let mut config = Config::default();
        config.slugify.taxonomies = SlugifyStrategy::On;
        let taxo_config = TaxonomyConfig {
            name: "tags".to_string(),
            lang: config.default_language.clone(),
            ..TaxonomyConfig::default()
        };
        let taxo_config_fr = TaxonomyConfig {
            name: "tags".to_string(),
            lang: "fr".to_string(),
            ..TaxonomyConfig::default()
        };
        let library = Library::new(0, 0, false);
        let tag = TaxonomyItem::new("Programming", &taxo_config, "tags", &config, vec![], &library);
        let tag_fr =
            TaxonomyItem::new("Programmation", &taxo_config_fr, "tags", &config, vec![], &library);
        let tags = Taxonomy { kind: taxo_config, slug: "tags".to_string(), items: vec![tag] };
        let tags_fr =
            Taxonomy { kind: taxo_config_fr, slug: "tags".to_string(), items: vec![tag_fr] };

        let taxonomies = vec![tags.clone(), tags_fr.clone()];
        let static_fn =
            GetTaxonomyUrl::new(&config.default_language, &taxonomies, config.slugify.taxonomies);

        // can find it correctly
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("name".to_string(), to_value("Programming").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            to_value("http://a-website.com/tags/programming/").unwrap()
        );

        // can find it correctly with inconsistent capitalisation
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("name".to_string(), to_value("programming").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            to_value("http://a-website.com/tags/programming/").unwrap()
        );

        // works with other languages
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("name".to_string(), to_value("Programmation").unwrap());
        args.insert("lang".to_string(), to_value("fr").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            to_value("http://a-website.com/fr/tags/programmation/").unwrap()
        );

        // and errors if it can't find it
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("name".to_string(), to_value("random").unwrap());
        assert!(static_fn.call(&args).is_err());
    }

    const TRANS_CONFIG: &str = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr"
languages = [
    { code = "en" },
]

[translations]
[translations.fr]
title = "Un titre"

[translations.en]
title = "A title"
        "#;

    #[test]
    fn can_translate_a_string() {
        let config = Config::parse(TRANS_CONFIG).unwrap();
        let static_fn = Trans::new(config);
        let mut args = HashMap::new();

        args.insert("key".to_string(), to_value("title").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "Un titre");

        args.insert("lang".to_string(), to_value("en").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "A title");

        args.insert("lang".to_string(), to_value("fr").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "Un titre");
    }

    #[test]
    fn error_on_absent_translation_lang() {
        let mut args = HashMap::new();
        args.insert("lang".to_string(), to_value("absent").unwrap());
        args.insert("key".to_string(), to_value("title").unwrap());

        let config = Config::parse(TRANS_CONFIG).unwrap();
        let error = Trans::new(config).call(&args).unwrap_err();
        assert_eq!("Failed to retrieve term translation", format!("{}", error));
    }

    #[test]
    fn error_on_absent_translation_key() {
        let mut args = HashMap::new();
        args.insert("lang".to_string(), to_value("en").unwrap());
        args.insert("key".to_string(), to_value("absent").unwrap());

        let config = Config::parse(TRANS_CONFIG).unwrap();
        let error = Trans::new(config).call(&args).unwrap_err();
        assert_eq!("Failed to retrieve term translation", format!("{}", error));
    }

    #[test]
    fn error_when_language_not_available() {
        let config = Config::parse(TRANS_CONFIG).unwrap();
        let static_fn = GetUrl::new(config, HashMap::new(), vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("@/a_section/a_page.md").unwrap());
        args.insert("lang".to_string(), to_value("it").unwrap());
        let err = static_fn.call(&args).unwrap_err();
        assert_eq!(
            "`it` is not an authorized language (check config.languages).",
            format!("{}", err)
        );
    }

    #[test]
    fn can_get_url_with_default_language() {
        let config = Config::parse(TRANS_CONFIG).unwrap();
        let mut permalinks = HashMap::new();
        permalinks.insert(
            "a_section/a_page.md".to_string(),
            "https://remplace-par-ton-url.fr/a_section/a_page/".to_string(),
        );
        permalinks.insert(
            "a_section/a_page.en.md".to_string(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page/".to_string(),
        );
        let static_fn = GetUrl::new(config, permalinks, vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("@/a_section/a_page.md").unwrap());
        args.insert("lang".to_string(), to_value("fr").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "https://remplace-par-ton-url.fr/a_section/a_page/"
        );
    }

    #[test]
    fn can_get_url_with_other_language() {
        let config = Config::parse(TRANS_CONFIG).unwrap();
        let mut permalinks = HashMap::new();
        permalinks.insert(
            "a_section/a_page.md".to_string(),
            "https://remplace-par-ton-url.fr/a_section/a_page/".to_string(),
        );
        permalinks.insert(
            "a_section/a_page.en.md".to_string(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page/".to_string(),
        );
        let static_fn = GetUrl::new(config, permalinks, vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("@/a_section/a_page.md").unwrap());
        args.insert("lang".to_string(), to_value("en").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page/"
        );
    }

    #[test]
    fn can_get_file_hash_sha256() {
        let static_fn = GetFileHash::new(vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("sha_type".to_string(), to_value(256).unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840"
        );
    }

    #[test]
    fn can_get_file_hash_sha384() {
        let static_fn = GetFileHash::new(vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "141c09bd28899773b772bbe064d8b718fa1d6f2852b7eafd5ed6689d26b74883b79e2e814cd69d5b52ab476aa284c414");
    }

    #[test]
    fn can_get_file_hash_sha512() {
        let static_fn = GetFileHash::new(vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("sha_type".to_string(), to_value(512).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "379dfab35123b9159d9e4e92dc90e2be44cf3c2f7f09b2e2df80a1b219b461de3556c93e1a9ceb3008e999e2d6a54b4f1d65ee9be9be63fa45ec88931623372f");
    }

    #[test]
    fn error_when_file_not_found_for_hash() {
        let static_fn = GetFileHash::new(vec![TEST_CONTEXT.static_path.clone()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("doesnt-exist").unwrap());
        assert_eq!(
            format!(
                "file `doesnt-exist` not found; searched in {}",
                TEST_CONTEXT.static_path.to_str().unwrap()
            ),
            format!("{}", static_fn.call(&args).unwrap_err())
        );
    }
}
