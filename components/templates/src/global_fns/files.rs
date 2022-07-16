use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io, result};

use crate::global_fns::helpers::search_for_file;
use config::Config;
use libs::base64::encode as encode_b64;
use libs::sha2::{digest, Sha256, Sha384, Sha512};
use libs::tera::{from_value, to_value, Function as TeraFn, Result, Value};
use libs::url;
use utils::site::resolve_internal_link;

fn compute_file_hash<D: digest::Digest>(
    mut file: fs::File,
    as_base64: bool,
) -> result::Result<String, io::Error>
where
    digest::Output<D>: core::fmt::LowerHex,
    D: std::io::Write,
{
    let mut hasher = D::new();
    io::copy(&mut file, &mut hasher)?;
    if as_base64 {
        Ok(encode_b64(hasher.finalize()))
    } else {
        Ok(format!("{:x}", hasher.finalize()))
    }
}

#[derive(Debug)]
pub struct GetUrl {
    base_path: PathBuf,
    config: Config,
    permalinks: HashMap<String, String>,
    output_path: PathBuf,
}

impl GetUrl {
    pub fn new(
        base_path: PathBuf,
        config: Config,
        permalinks: HashMap<String, String>,
        output_path: PathBuf,
    ) -> Self {
        Self { base_path, config, permalinks, output_path }
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

    let mut split_path: Vec<String> = path.split('.').map(String::from).collect();
    let ilast = split_path.len() - 1;
    split_path[ilast] = format!("{}.{}", lang, split_path[ilast]);
    Ok(split_path.join("."))
}

impl TeraFn for GetUrl {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`get_url` requires a `path` argument with a string value"
        );
        let cachebust = optional_arg!(
            bool,
            args.get("cachebust"),
            "`get_url`: `cachebust` must be a boolean (true or false)"
        )
        .unwrap_or(false);
        let trailing_slash = optional_arg!(
            bool,
            args.get("trailing_slash"),
            "`get_url`: `trailing_slash` must be a boolean (true or false)"
        )
        .unwrap_or(false);
        let lang = optional_arg!(String, args.get("lang"), "`get_url`: `lang` must be a string.")
            .unwrap_or_else(|| self.config.default_language.clone());

        // if it starts with @/, resolve it as an internal link
        if path.starts_with("@/") {
            let path_with_lang = match make_path_with_lang(path, &lang, &self.config) {
                Ok(x) => x,
                Err(e) => return Err(e),
            };

            match resolve_internal_link(&path_with_lang, &self.permalinks) {
                Ok(resolved) => Ok(to_value(resolved.permalink).unwrap()),
                Err(_) => Err(format!(
                    "`get_url`: could not resolve URL for link `{}` not found.",
                    path_with_lang
                )
                .into()),
            }
        } else {
            // anything else
            let mut segments = vec![];

            if lang != self.config.default_language {
                if path.is_empty() || !path[1..].starts_with(&lang) {
                    segments.push(lang);
                }
            }

            segments.push(path);

            let path_with_lang = segments.join("/");

            let mut permalink = self.config.make_permalink(&path_with_lang);
            if !trailing_slash && permalink.ends_with('/') {
                permalink.pop(); // Removes the slash
            }

            if cachebust {
                match search_for_file(
                    &self.base_path,
                    &path_with_lang,
                    &self.config.theme,
                    &self.output_path,
                )
                .map_err(|e| format!("`get_url`: {}", e))?
                .and_then(|(p, _)| fs::File::open(&p).ok())
                .and_then(|f| compute_file_hash::<Sha256>(f, false).ok())
                {
                    Some(hash) => {
                        permalink = format!("{}?h={}", permalink, hash);
                    }
                    None => {
                        return Err(format!(
                            "`get_url`: Could not find or open file {}",
                            path_with_lang
                        )
                        .into())
                    }
                };
            }

            if cfg!(target_os = "windows") {
                permalink = match url::Url::parse(&permalink) {
                    Ok(parsed) => parsed.into(),
                    Err(_) => {
                        return Err(format!(
                            "`get_url`: Could not parse link `{}` as a valid URL",
                            permalink
                        )
                        .into())
                    }
                };
            }

            Ok(to_value(permalink).unwrap())
        }
    }

    fn is_safe(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct GetFileHash {
    base_path: PathBuf,
    theme: Option<String>,
    output_path: PathBuf,
}
impl GetFileHash {
    pub fn new(base_path: PathBuf, theme: Option<String>, output_path: PathBuf) -> Self {
        Self { base_path, theme, output_path }
    }
}

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
        .unwrap_or(384);
        let base64 = optional_arg!(
            bool,
            args.get("base64"),
            "`get_file_hash`: `base64` must be true or false"
        )
        .unwrap_or(true);

        let file_path =
            match search_for_file(&self.base_path, &path, &self.theme, &self.output_path)
                .map_err(|e| format!("`get_file_hash`: {}", e))?
            {
                Some((f, _)) => f,
                None => {
                    return Err(format!("`get_file_hash`: Cannot find file: {}", path).into());
                }
            };

        let f = match std::fs::File::open(file_path) {
            Ok(f) => f,
            Err(e) => {
                return Err(format!("File {} could not be open: {}", path, e).into());
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
            Err(_) => Err("`get_file_hash`: could no compute hash".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GetFileHash, GetUrl};

    use std::collections::HashMap;
    use std::fs::create_dir;
    use std::path::PathBuf;

    use libs::tera::{to_value, Function};
    use tempfile::{tempdir, TempDir};

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
        let dir = create_temp_dir();
        let static_fn = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
        );
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css?h=572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840");
    }

    #[test]
    fn can_add_trailing_slashes() {
        let dir = create_temp_dir();
        let static_fn = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
        );
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css/");
    }

    #[test]
    fn can_add_slashes_and_cachebust() {
        let dir = create_temp_dir();
        let static_fn = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
        );
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(true).unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css/?h=572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840");
    }

    #[test]
    fn can_link_to_some_static_file() {
        let dir = create_temp_dir();
        let static_fn = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
        );
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css");

        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("/app.css").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/app.css");
    }

    #[test]
    fn can_link_to_file_in_output_path() {
        let dir = create_temp_dir();
        let public = dir.path().join("public");
        create_dir(&public).expect("Failed to create output directory");
        create_file(&public.join("style.css"), "// Hello world")
            .expect("Failed to create file in output directory");

        let static_fn =
            GetUrl::new(dir.path().to_path_buf(), Config::default(), HashMap::new(), public);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("style.css").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "http://a-website.com/style.css");
    }

    #[test]
    fn error_when_language_not_available() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let static_fn =
            GetUrl::new(dir.path().to_path_buf(), config, HashMap::new(), PathBuf::new());
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
        let mut permalinks = HashMap::new();
        permalinks.insert(
            "a_section/a_page.md".to_string(),
            "https://remplace-par-ton-url.fr/a_section/a_page/".to_string(),
        );
        permalinks.insert(
            "a_section/a_page.en.md".to_string(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page/".to_string(),
        );
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let static_fn = GetUrl::new(
            dir.path().to_path_buf(),
            config.clone(),
            permalinks.clone(),
            PathBuf::new(),
        );
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
        let dir = create_temp_dir();
        let static_fn = GetUrl::new(dir.path().to_path_buf(), config, permalinks, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("@/a_section/a_page.md").unwrap());
        args.insert("lang".to_string(), to_value("en").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page/"
        );
    }

    #[test]
    fn does_not_duplicate_lang() {
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
        let dir = create_temp_dir();
        let static_fn = GetUrl::new(dir.path().to_path_buf(), config, permalinks, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("/en/a_section/a_page/").unwrap());
        args.insert("lang".to_string(), to_value("en").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page"
        );
    }

    #[test]
    fn can_get_feed_url_with_default_language() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let static_fn =
            GetUrl::new(dir.path().to_path_buf(), config.clone(), HashMap::new(), PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value(config.feed_filename).unwrap());
        args.insert("lang".to_string(), to_value("fr").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "https://remplace-par-ton-url.fr/atom.xml");
    }

    #[test]
    fn can_get_feed_url_with_other_language() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let static_fn =
            GetUrl::new(dir.path().to_path_buf(), config.clone(), HashMap::new(), PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value(config.feed_filename).unwrap());
        args.insert("lang".to_string(), to_value("en").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "https://remplace-par-ton-url.fr/en/atom.xml");
    }

    #[test]
    fn can_get_file_hash_sha256_no_base64() {
        let dir = create_temp_dir();
        let static_fn = GetFileHash::new(dir.into_path(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("sha_type".to_string(), to_value(256).unwrap());
        args.insert("base64".to_string(), to_value(false).unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840"
        );
    }

    #[test]
    fn can_get_file_hash_sha256_base64() {
        let dir = create_temp_dir();
        let static_fn = GetFileHash::new(dir.into_path(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("sha_type".to_string(), to_value(256).unwrap());
        args.insert("base64".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "Vy5pHcaMP81lOuRjJhvbOPNdxvAXFdnOaHmTGd0ViEA=");
    }

    #[test]
    fn can_get_file_hash_sha384_no_base64() {
        let dir = create_temp_dir();
        let static_fn = GetFileHash::new(dir.into_path(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("base64".to_string(), to_value(false).unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "141c09bd28899773b772bbe064d8b718fa1d6f2852b7eafd5ed6689d26b74883b79e2e814cd69d5b52ab476aa284c414"
            );
    }

    #[test]
    fn can_get_file_hash_sha384() {
        let dir = create_temp_dir();
        let static_fn = GetFileHash::new(dir.into_path(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "FBwJvSiJl3O3crvgZNi3GPodbyhSt+r9XtZonSa3SIO3ni6BTNadW1KrR2qihMQU"
        );
    }

    #[test]
    fn can_get_file_hash_sha512_no_base64() {
        let dir = create_temp_dir();
        let static_fn = GetFileHash::new(dir.into_path(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("sha_type".to_string(), to_value(512).unwrap());
        args.insert("base64".to_string(), to_value(false).unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "379dfab35123b9159d9e4e92dc90e2be44cf3c2f7f09b2e2df80a1b219b461de3556c93e1a9ceb3008e999e2d6a54b4f1d65ee9be9be63fa45ec88931623372f"
        );
    }

    #[test]
    fn can_get_file_hash_sha512() {
        let dir = create_temp_dir();
        let static_fn = GetFileHash::new(dir.into_path(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("sha_type".to_string(), to_value(512).unwrap());
        assert_eq!(
            static_fn.call(&args).unwrap(),
            "N536s1EjuRWdnk6S3JDivkTPPC9/CbLi34Chshm0Yd41Vsk+GpzrMAjpmeLWpUtPHWXum+m+Y/pF7IiTFiM3Lw=="
        );
    }

    #[test]
    fn can_resolve_asset_path_to_valid_url() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let static_fn =
            GetUrl::new(dir.path().to_path_buf(), config, HashMap::new(), PathBuf::new());
        let mut args = HashMap::new();
        args.insert(
            "path".to_string(),
            to_value(dir.path().join("app.css").strip_prefix(std::env::temp_dir()).unwrap())
                .unwrap(),
        );
        assert_eq!(
            static_fn.call(&args).unwrap(),
            format!(
                "https://remplace-par-ton-url.fr/{}/app.css",
                dir.path().file_stem().unwrap().to_string_lossy()
            )
        )
    }

    #[test]
    fn error_when_file_not_found_for_hash() {
        let dir = create_temp_dir();
        let static_fn = GetFileHash::new(dir.into_path(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("doesnt-exist").unwrap());
        let err = format!("{}", static_fn.call(&args).unwrap_err());

        assert!(err.contains("Cannot find file"));
    }
}
