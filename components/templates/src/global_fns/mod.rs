use std::borrow::Cow;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io, result};

use base64::encode as encode_b64;
use sha2::{digest, Sha256, Sha384, Sha512};
use tera::{from_value, to_value, Function as TeraFn, Result, Value};

use config::Config;
use utils::site::resolve_internal_link;

#[macro_use]
mod macros;

mod content;
mod i18n;
mod images;
mod load_data;

pub use self::content::{GetPage, GetSection, GetTaxonomy, GetTaxonomyUrl};
pub use self::i18n::Trans;
pub use self::images::{GetImageMetadata, ResizeImage};
pub use self::load_data::LoadData;

/// This is used by a few Tera functions to search for files on the filesystem.
/// This does try to find the file in 3 different spots:
/// 1. base_path + path
/// 2. base_path + static + path
/// 3. base_path + content + path
pub fn search_for_file(base_path: &Path, path: &str) -> Option<PathBuf> {
    let search_paths = [base_path.join("static"), base_path.join("content")];
    let actual_path = if path.starts_with("@/") {
        Cow::Owned(path.replace("@/", "content/"))
    } else {
        Cow::Borrowed(path)
    };
    let mut file_path = base_path.join(&*actual_path);
    let mut file_exists = file_path.exists();

    if !file_exists {
        // we need to search in both search folders now
        for dir in &search_paths {
            let p = dir.join(&*actual_path);
            if p.exists() {
                file_path = p;
                file_exists = true;
                break;
            }
        }
    }

    if file_exists {
        Some(file_path)
    } else {
        None
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

    if !config.other_languages().contains_key(lang) {
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

fn compute_file_hash<D: digest::Digest>(
    mut file: fs::File,
    base64: bool,
) -> result::Result<String, io::Error>
where
    digest::Output<D>: core::fmt::LowerHex,
    D: std::io::Write,
{
    let mut hasher = D::new();
    io::copy(&mut file, &mut hasher)?;
    if base64 {
        Ok(encode_b64(hasher.finalize()))
    } else {
        Ok(format!("{:x}", hasher.finalize()))
    }
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
            let mut segments = vec![];

            if lang != self.config.default_language {
                segments.push(lang);
            };

            segments.push(path);

            let path_with_lang = segments.join("/");

            let mut permalink = self.config.make_permalink(&path_with_lang);
            if !trailing_slash && permalink.ends_with('/') {
                permalink.pop(); // Removes the slash
            }

            if cachebust {
                match open_file(&self.search_paths, &path_with_lang)
                    .and_then(|f| compute_file_hash::<Sha256>(f, false))
                {
                    Ok(hash) => {
                        permalink = format!("{}?h={}", permalink, hash);
                    }
                    Err(_) => return file_not_found_err(&self.search_paths, &path_with_lang),
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
const DEFAULT_BASE64: bool = false;

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
        let base64 = optional_arg!(
            bool,
            args.get("base64"),
            "`get_file_hash`: `base64` must be true or false"
        )
        .unwrap_or(DEFAULT_BASE64);

        let f = match open_file(&self.search_paths, &path) {
            Ok(f) => f,
            Err(e) => {
                return Err(format!(
                    "File {} could not be open: {} (searched in {:?})",
                    path, e, self.search_paths
                )
                .into());
            }
        };

        let hash = match sha_type {
            256 => compute_file_hash::<Sha256>(f, base64),
            384 => compute_file_hash::<Sha384>(f, base64),
            512 => compute_file_hash::<Sha512>(f, base64),
            _ => return Err("`get_file_hash`: Invalid sha value".into()),
        };

        match hash {
            Ok(digest) => Ok(to_value(digest).unwrap()),
            Err(_) => file_not_found_err(&self.search_paths, &path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GetFileHash, GetUrl};

    use std::collections::HashMap;

    use tempfile::{tempdir, TempDir};
    use tera::{to_value, Function};

    use config::Config;
    use utils::fs::create_file;

    fn create_temp_dir() -> TempDir {
        let dir = tempdir().unwrap();
        create_file(&dir.path().join("app.css"), "// Hello world!").expect("Failed to create file");
        dir
    }

    const CONFIG_DATA: &str = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr"

[translations]
title = "Un titre"

[languages.en]
[languages.en.translations]
title = "A title"
"#;

    #[test]
    fn can_add_cachebust_to_url() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new(), vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css?h=572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840");
    }

    #[test]
    fn can_add_trailing_slashes() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new(), vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css/");
    }

    #[test]
    fn can_add_slashes_and_cachebust() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new(), vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(true).unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css/?h=572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840");
    }

    #[test]
    fn can_link_to_some_static_file() {
        let config = Config::default();
        let static_fn = GetUrl::new(config, HashMap::new(), vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css");
    }

    #[test]
    fn error_when_language_not_available() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let static_fn = GetUrl::new(config, HashMap::new(), vec![create_temp_dir().into_path()]);
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
        let config = Config::parse(CONFIG_DATA).unwrap();
        let mut permalinks = HashMap::new();
        permalinks.insert(
            "a_section/a_page.md".to_string(),
            "https://remplace-par-ton-url.fr/a_section/a_page/".to_string(),
        );
        permalinks.insert(
            "a_section/a_page.en.md".to_string(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page/".to_string(),
        );
        let static_fn = GetUrl::new(config, permalinks, vec![create_temp_dir().into_path()]);
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
        let config = Config::parse(CONFIG_DATA).unwrap();
        let mut permalinks = HashMap::new();
        permalinks.insert(
            "a_section/a_page.md".to_string(),
            "https://remplace-par-ton-url.fr/a_section/a_page/".to_string(),
        );
        permalinks.insert(
            "a_section/a_page.en.md".to_string(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page/".to_string(),
        );
        let static_fn = GetUrl::new(config, permalinks, vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("@/a_section/a_page.md").unwrap());
        args.insert("lang".to_string(), to_value("en").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page/"
        );
    }

    #[test]
    fn can_get_feed_url_with_default_language() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let static_fn =
            GetUrl::new(config.clone(), HashMap::new(), vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value(config.feed_filename).unwrap());
        args.insert("lang".to_string(), to_value("fr").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "https://remplace-par-ton-url.fr/atom.xml");
    }

    #[test]
    fn can_get_feed_url_with_other_language() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let static_fn =
            GetUrl::new(config.clone(), HashMap::new(), vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value(config.feed_filename).unwrap());
        args.insert("lang".to_string(), to_value("en").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "https://remplace-par-ton-url.fr/en/atom.xml");
    }

    #[test]
    fn can_get_file_hash_sha256() {
        let static_fn = GetFileHash::new(vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("sha_type".to_string(), to_value(256).unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840"
        );
    }

    #[test]
    fn can_get_file_hash_sha256_base64() {
        let static_fn = GetFileHash::new(vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("sha_type".to_string(), to_value(256).unwrap());
        args.insert("base64".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "Vy5pHcaMP81lOuRjJhvbOPNdxvAXFdnOaHmTGd0ViEA=");
    }

    #[test]
    fn can_get_file_hash_sha384() {
        let static_fn = GetFileHash::new(vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "141c09bd28899773b772bbe064d8b718fa1d6f2852b7eafd5ed6689d26b74883b79e2e814cd69d5b52ab476aa284c414"
            );
    }

    #[test]
    fn can_get_file_hash_sha384_base64() {
        let static_fn = GetFileHash::new(vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("base64".to_string(), to_value(true).unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "FBwJvSiJl3O3crvgZNi3GPodbyhSt+r9XtZonSa3SIO3ni6BTNadW1KrR2qihMQU"
        );
    }

    #[test]
    fn can_get_file_hash_sha512() {
        let static_fn = GetFileHash::new(vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("sha_type".to_string(), to_value(512).unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "379dfab35123b9159d9e4e92dc90e2be44cf3c2f7f09b2e2df80a1b219b461de3556c93e1a9ceb3008e999e2d6a54b4f1d65ee9be9be63fa45ec88931623372f"
        );
    }

    #[test]
    fn can_get_file_hash_sha512_base64() {
        let static_fn = GetFileHash::new(vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("sha_type".to_string(), to_value(512).unwrap());
        args.insert("base64".to_string(), to_value(true).unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "N536s1EjuRWdnk6S3JDivkTPPC9/CbLi34Chshm0Yd41Vsk+GpzrMAjpmeLWpUtPHWXum+m+Y/pF7IiTFiM3Lw=="
        );
    }

    #[test]
    fn error_when_file_not_found_for_hash() {
        let static_fn = GetFileHash::new(vec![create_temp_dir().into_path()]);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("doesnt-exist").unwrap());
        let err = format!("{}", static_fn.call(&args).unwrap_err());
        println!("{:?}", err);

        assert!(err.contains("File doesnt-exist could not be open"));
    }
}
