use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use ahash::AHashMap;
use base64::engine::{Engine, general_purpose::STANDARD as standard_b64};
use fs_err as fs;
use sha2::{Digest, Sha256, Sha384, Sha512, digest};

use config::Config;
use tera::{Error, Function, Kwargs, State, TeraResult};
use utils::site::resolve_internal_link;

use crate::helpers::search_for_file;

struct DigestWriter<D>(pub D);

impl<D: Digest> std::io::Write for DigestWriter<D> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.update(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn format_hash<D>(hasher: D, as_base64: bool) -> String
where
    D: Digest,
    digest::Output<D>: AsRef<[u8]>,
{
    let result = hasher.finalize();
    let bytes = result.as_ref();
    if as_base64 {
        standard_b64.encode(bytes)
    } else {
        let mut hex = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            write!(hex, "{:02x}", b).unwrap();
        }
        hex
    }
}

fn compute_hash<D>(data: &[u8], as_base64: bool) -> String
where
    D: Digest,
    digest::Output<D>: AsRef<[u8]>,
{
    let mut hasher = D::new();
    hasher.update(data);
    format_hash(hasher, as_base64)
}

fn compute_file_hash<D>(file_path: &Path, as_base64: bool) -> std::io::Result<String>
where
    D: Digest,
    digest::Output<D>: AsRef<[u8]>,
{
    let mut file = fs::File::open(file_path)?;
    let mut writer = DigestWriter(D::new());
    std::io::copy(&mut file, &mut writer)?;
    Ok(format_hash(writer.0, as_base64))
}

#[derive(Debug, Default)]
pub struct GetUrl {
    base_path: PathBuf,
    config: Config,
    permalinks: HashMap<String, String>,
    output_path: PathBuf,
    colocated_assets: AHashMap<String, (String, String)>,
    result_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl GetUrl {
    pub fn new(
        base_path: PathBuf,
        config: Config,
        permalinks: HashMap<String, String>,
        output_path: PathBuf,
        colocated_assets: AHashMap<String, (String, String)>,
    ) -> Self {
        Self {
            base_path,
            config,
            permalinks,
            output_path,
            colocated_assets,
            result_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn cachebust_hash(&self, file_path: &Path, cache_key: &str) -> Option<String> {
        if let Some(hash) =
            self.result_cache.lock().expect("result cache lock").get(cache_key).cloned()
        {
            return Some(hash);
        }
        let hash = compute_file_hash::<Sha256>(file_path, false).ok()?;
        self.result_cache
            .lock()
            .expect("result cache lock")
            .insert(cache_key.to_string(), hash.clone());
        Some(hash)
    }
}

fn add_cachebust(permalink: &mut String, hash: &str) {
    // 2^-80 chance of false positive
    write!(permalink, "?h={}", &hash[..20]).unwrap();
}

fn make_path_with_lang(path: String, lang: &str, config: &Config) -> TeraResult<String> {
    if lang == config.default_language {
        return Ok(path);
    }

    if !config.other_languages().contains_key(lang) {
        return Err(Error::message(format!(
            "`{}` is not an authorized language (check config.languages).",
            lang
        )));
    }

    let mut split_path: Vec<String> = path.split('.').map(String::from).collect();
    let ilast = split_path.len() - 1;
    split_path[ilast] = format!("{}.{}", lang, split_path[ilast]);
    Ok(split_path.join("."))
}

impl Function<TeraResult<String>> for GetUrl {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<String> {
        let path: String = kwargs.must_get("path")?;
        let cachebust: bool = kwargs.get("cachebust")?.unwrap_or(false);
        let trailing_slash: bool = kwargs.get("trailing_slash")?.unwrap_or(false);
        let explicit_lang: Option<String> = kwargs.get("lang")?;

        // if it starts with @/, resolve it as an internal link
        if let Some(stripped) = path.strip_prefix("@/") {
            let lang: String = explicit_lang
                .or(state.get("lang")?)
                .unwrap_or_else(|| self.config.default_language.clone());

            // Check if it's a colocated asset first
            if let Some((owner_md, rel)) = self.colocated_assets.get(stripped) {
                let owner_with_lang =
                    make_path_with_lang(format!("@/{}", owner_md), &lang, &self.config)?;
                let resolved =
                    resolve_internal_link(&owner_with_lang, &self.permalinks).map_err(|_| {
                        Error::message(format!(
                            "`get_url`: could not resolve URL for asset `{path}` not found."
                        ))
                    })?;
                let mut permalink = format!("{}{rel}", resolved.permalink);
                if cachebust {
                    let maybe_hash =
                        self.result_cache.lock().expect("result cache lock").get(&path).cloned();
                    let hash = match maybe_hash {
                        Some(h) => h,
                        None => {
                            let file_path = self.base_path.join("content").join(stripped);
                            match self.cachebust_hash(&file_path, &path) {
                                Some(hash) => hash,
                                None => {
                                    return Err(Error::message(format!(
                                        "`get_url`: Could not find or open file {path} for cachebusting"
                                    )));
                                }
                            }
                        }
                    };
                    add_cachebust(&mut permalink, &hash);
                }
                return Ok(permalink);
            }

            if cachebust {
                return Err(Error::message(
                    "`get_url`: `cachebust` can only be used with files, not with content",
                ));
            }

            let path_with_lang = make_path_with_lang(path, &lang, &self.config)?;

            match resolve_internal_link(&path_with_lang, &self.permalinks) {
                Ok(resolved) => Ok(resolved.permalink),
                Err(_) => Err(Error::message(
                    "`get_url`: could not resolve URL for link `{path_with_lang}` not found.",
                )),
            }
        } else {
            // anything else
            let mut segments = vec![];

            if let Some(ref lang) = explicit_lang
                && *lang != self.config.default_language
            {
                let p = path.strip_prefix('/').unwrap_or(&path);
                if p != lang.as_str() && !p.starts_with(&format!("{lang}/")) {
                    segments.push(lang.clone());
                }
            }
            if segments.is_empty() {
                segments.push(path.clone());
            } else {
                // We've pushed a {lang}/ so remove the / first
                segments.push(path.trim_start_matches('/').to_string());
            }

            let path_with_lang = segments.join("/");

            let mut permalink = self.config.make_permalink(&path_with_lang);
            if !trailing_slash && permalink.ends_with('/') {
                permalink.pop(); // Removes the slash
            }

            if cachebust {
                let cache_key = path_with_lang.trim_start_matches('/');
                let maybe_hash =
                    self.result_cache.lock().expect("result cache lock").get(cache_key).cloned();

                let hash = match maybe_hash {
                    Some(hash) => hash,
                    None => {
                        match search_for_file(
                            &self.base_path,
                            &path_with_lang,
                            &self.config.theme,
                            &self.output_path,
                        )
                        .map_err(|e| Error::message(format!("`get_url`: {e}")))?
                        .and_then(|(file_path, unified_path)| {
                            self.cachebust_hash(&file_path, &unified_path)
                        }) {
                            Some(hash) => hash,
                            None => {
                                return Err(Error::message(format!(
                                    "`get_url`: Could not find or open file {}",
                                    path_with_lang
                                )));
                            }
                        }
                    }
                };

                add_cachebust(&mut permalink, &hash);
            }

            Ok(permalink)
        }
    }

    fn is_safe(&self) -> bool {
        true
    }
}

#[derive(Debug, Default)]
pub struct GetHash {
    base_path: PathBuf,
    theme: Option<String>,
    output_path: PathBuf,
    result_cache: Mutex<HashMap<(String, u16, bool), String>>,
}

impl GetHash {
    pub fn new(base_path: PathBuf, theme: Option<String>, output_path: PathBuf) -> Self {
        Self { base_path, theme, output_path, result_cache: Mutex::new(HashMap::new()) }
    }
}

impl Function<TeraResult<String>> for GetHash {
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<String> {
        let path: Option<String> = kwargs.get("path")?;
        let literal: Option<String> = kwargs.get("literal")?;
        let sha_type: u16 = kwargs.get("sha_type")?.unwrap_or(384);
        if ![256, 384, 512].contains(&sha_type) {
            return Err(Error::message("`get_hash`: Invalid sha value"));
        }
        let base64: bool = kwargs.get("base64")?.unwrap_or(true);

        match (path, literal) {
            (Some(_), Some(_)) => Err(Error::message(
                "`get_hash`: must have only one of `path` or `literal` argument",
            )),
            (None, None) => Err(Error::message(
                "`get_hash`: must have at least one of `path` or `literal` argument",
            )),
            (Some(path_v), None) => {
                let key = (path_v, sha_type, base64);
                if let Some(hash) =
                    self.result_cache.lock().expect("result cache lock").get(&key).cloned()
                {
                    return Ok(hash);
                }

                let (file_path, _) =
                    search_for_file(&self.base_path, &key.0, &self.theme, &self.output_path)
                        .map_err(|e| Error::message(format!("`get_hash`: {}", e)))?
                        .ok_or_else(|| {
                            Error::message(format!("`get_hash`: Cannot find file: {}", key.0))
                        })?;

                let hash = match sha_type {
                    256 => compute_file_hash::<Sha256>(&file_path, base64),
                    384 => compute_file_hash::<Sha384>(&file_path, base64),
                    512 => compute_file_hash::<Sha512>(&file_path, base64),
                    _ => unreachable!(),
                }
                .map_err(|e| Error::message(format!("File {} could not be read: {}", key.0, e)))?;

                self.result_cache.lock().expect("result cache lock").insert(key, hash.clone());

                Ok(hash)
            }
            (None, Some(literal_v)) => {
                let hash = match sha_type {
                    256 => compute_hash::<Sha256>(literal_v.as_bytes(), base64),
                    384 => compute_hash::<Sha384>(literal_v.as_bytes(), base64),
                    512 => compute_hash::<Sha512>(literal_v.as_bytes(), base64),
                    _ => unreachable!(),
                };

                Ok(hash)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GetHash, GetUrl};

    use ahash::AHashMap;
    use fs_err as fs;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use tempfile::{TempDir, tempdir};
    use tera::{Context, Function, Kwargs, State};

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
        let mut permalinks = HashMap::new();
        permalinks.insert(
            "a_section/an_article/index.md".to_string(),
            "http://a-website.com/an_article/".to_string(),
        );
        let mut colocated_assets = AHashMap::new();
        colocated_assets.insert(
            "a_section/an_article/gutenberg.jpg".to_string(),
            ("a_section/an_article/index.md".to_string(), "gutenberg.jpg".to_string()),
        );
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            permalinks,
            PathBuf::new(),
            colocated_assets,
        );

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("app.css")),
            ("cachebust", tera::Value::from(true)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "http://a-website.com/app.css?h=572e691dc68c3fcd653a"
        );

        // And binary files as well
        fs::copy("gutenberg.jpg", dir.path().join("gutenberg.jpg")).unwrap();
        let kwargs = Kwargs::from([
            ("path", tera::Value::from("gutenberg.jpg")),
            ("cachebust", tera::Value::from(true)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "http://a-website.com/gutenberg.jpg?h=93fff9d0ecde9b119c0c"
        );

        // also works with colocated assets
        fs::create_dir_all(dir.path().join("content/a_section/an_article")).unwrap();
        fs::copy("gutenberg.jpg", dir.path().join("content/a_section/an_article/gutenberg.jpg"))
            .unwrap();
        let kwargs = Kwargs::from([
            ("path", tera::Value::from("@/a_section/an_article/gutenberg.jpg")),
            ("cachebust", tera::Value::from(true)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "http://a-website.com/an_article/gutenberg.jpg?h=93fff9d0ecde9b119c0c"
        );

        // cachebust on a page errors
        let kwargs = Kwargs::from([
            ("path", tera::Value::from("@/a_section/an_article/index.md")),
            ("cachebust", tera::Value::from(true)),
        ]);
        let ctx = Context::new();
        assert!(get_url.call(kwargs, &State::new(&ctx)).is_err());
    }

    #[test]
    fn can_add_trailing_slashes() {
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
            AHashMap::new(),
        );

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("app.css")),
            ("trailing_slash", tera::Value::from(true)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "http://a-website.com/app.css/"
        );
    }

    #[test]
    fn can_add_slashes_and_cachebust() {
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
            AHashMap::new(),
        );

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("app.css")),
            ("trailing_slash", tera::Value::from(true)),
            ("cachebust", tera::Value::from(true)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "http://a-website.com/app.css/?h=572e691dc68c3fcd653a"
        );
    }

    #[test]
    fn can_link_to_some_static_file() {
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
            AHashMap::new(),
        );

        let kwargs = Kwargs::from([("path", tera::Value::from("app.css"))]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "http://a-website.com/app.css"
        );

        let kwargs = Kwargs::from([("path", tera::Value::from("/app.css"))]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "http://a-website.com/app.css"
        );
    }

    #[test]
    fn can_link_to_file_in_output_path() {
        let dir = create_temp_dir();
        let public = dir.path().join("public");
        fs::create_dir(&public).expect("Failed to create output directory");
        create_file(&public.join("style.css"), "// Hello world")
            .expect("Failed to create file in output directory");

        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            public,
            AHashMap::new(),
        );

        let kwargs = Kwargs::from([("path", tera::Value::from("style.css"))]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "http://a-website.com/style.css"
        );
    }

    #[test]
    fn error_when_language_not_available() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            config,
            HashMap::new(),
            PathBuf::new(),
            AHashMap::new(),
        );

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("@/a_section/a_page.md")),
            ("lang", tera::Value::from("it")),
        ]);
        let ctx = Context::new();
        let err = get_url.call(kwargs, &State::new(&ctx)).unwrap_err();
        assert!(err.to_string().contains("is not an authorized language"));
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
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            config.clone(),
            permalinks.clone(),
            PathBuf::new(),
            AHashMap::new(),
        );

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("@/a_section/a_page.md")),
            ("lang", tera::Value::from("fr")),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
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
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            config,
            permalinks,
            PathBuf::new(),
            AHashMap::new(),
        );

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("@/a_section/a_page.md")),
            ("lang", tera::Value::from("en")),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page/"
        );
    }

    // Colocated assets resolve the owning page/section for the requested language and append the
    // asset's relative path. fr/en use different slugs to prove the lang prefix isn't blindly added.
    #[test]
    fn can_get_colocated_asset_url() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let mut permalinks = HashMap::new();
        permalinks.insert(
            "a_section/an_article/index.md".to_string(),
            "https://remplace-par-ton-url.fr/a_section/mon_article/".to_string(),
        );
        permalinks.insert(
            "a_section/an_article/index.en.md".to_string(),
            "https://remplace-par-ton-url.fr/en/a_section/the_article/".to_string(),
        );
        let mut colocated_assets = AHashMap::new();
        colocated_assets.insert(
            "a_section/an_article/graph.jpg".to_string(),
            ("a_section/an_article/index.md".to_string(), "graph.jpg".to_string()),
        );

        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            config,
            permalinks,
            PathBuf::new(),
            colocated_assets,
        );
        let resolve = |path: &str, lang: &str| {
            let kwargs = Kwargs::from([
                ("path", tera::Value::from(path)),
                ("lang", tera::Value::from(lang)),
            ]);
            let ctx = Context::new();
            get_url.call(kwargs, &State::new(&ctx))
        };

        // Default language, then another language following that page's own slug
        assert_eq!(
            resolve("@/a_section/an_article/graph.jpg", "fr").unwrap(),
            "https://remplace-par-ton-url.fr/a_section/mon_article/graph.jpg"
        );
        assert_eq!(
            resolve("@/a_section/an_article/graph.jpg", "en").unwrap(),
            "https://remplace-par-ton-url.fr/en/a_section/the_article/graph.jpg"
        );
        // Unknown asset errors
        assert!(resolve("@/a_section/my-article/missing.jpg", "fr").is_err());
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
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            config,
            permalinks,
            PathBuf::new(),
            AHashMap::new(),
        );

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("/en/a_section/a_page/")),
            ("lang", tera::Value::from("en")),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "https://remplace-par-ton-url.fr/en/a_section/a_page"
        );
    }

    #[test]
    fn can_get_feed_urls_with_default_language() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            config.clone(),
            HashMap::new(),
            PathBuf::new(),
            AHashMap::new(),
        );

        for feed_filename in &config.feed_filenames {
            let kwargs = Kwargs::from([
                ("path", tera::Value::from(feed_filename.as_str())),
                ("lang", tera::Value::from("fr")),
            ]);
            let ctx = Context::new();
            assert_eq!(
                get_url.call(kwargs, &State::new(&ctx)).unwrap(),
                "https://remplace-par-ton-url.fr/atom.xml"
            );
        }
    }

    #[test]
    fn can_get_feed_urls_with_other_language() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            config.clone(),
            HashMap::new(),
            PathBuf::new(),
            AHashMap::new(),
        );

        for feed_filename in &config.feed_filenames {
            let kwargs = Kwargs::from([
                ("path", tera::Value::from(feed_filename.as_str())),
                ("lang", tera::Value::from("en")),
            ]);
            let ctx = Context::new();
            assert_eq!(
                get_url.call(kwargs, &State::new(&ctx)).unwrap(),
                "https://remplace-par-ton-url.fr/en/atom.xml"
            );
        }
    }

    #[test]
    fn can_get_file_hash_sha256_no_base64() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("app.css")),
            ("sha_type", tera::Value::from(256)),
            ("base64", tera::Value::from(false)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840"
        );
    }

    #[test]
    fn can_get_file_hash_sha256_base64() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("app.css")),
            ("sha_type", tera::Value::from(256)),
            ("base64", tera::Value::from(true)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "Vy5pHcaMP81lOuRjJhvbOPNdxvAXFdnOaHmTGd0ViEA="
        );
    }

    #[test]
    fn can_get_file_hash_sha384_no_base64() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("app.css")),
            ("base64", tera::Value::from(false)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "141c09bd28899773b772bbe064d8b718fa1d6f2852b7eafd5ed6689d26b74883b79e2e814cd69d5b52ab476aa284c414"
        );
    }

    #[test]
    fn can_get_file_hash_sha384() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([("path", tera::Value::from("app.css"))]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "FBwJvSiJl3O3crvgZNi3GPodbyhSt+r9XtZonSa3SIO3ni6BTNadW1KrR2qihMQU"
        );
    }

    #[test]
    fn can_get_file_hash_sha512_no_base64() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("app.css")),
            ("sha_type", tera::Value::from(512)),
            ("base64", tera::Value::from(false)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "379dfab35123b9159d9e4e92dc90e2be44cf3c2f7f09b2e2df80a1b219b461de3556c93e1a9ceb3008e999e2d6a54b4f1d65ee9be9be63fa45ec88931623372f"
        );
    }

    #[test]
    fn can_get_file_hash_sha512() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("app.css")),
            ("sha_type", tera::Value::from(512)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "N536s1EjuRWdnk6S3JDivkTPPC9/CbLi34Chshm0Yd41Vsk+GpzrMAjpmeLWpUtPHWXum+m+Y/pF7IiTFiM3Lw=="
        );
    }

    #[test]
    fn can_get_hash_sha256_no_base64() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([
            ("literal", tera::Value::from("Hello World")),
            ("sha_type", tera::Value::from(256)),
            ("base64", tera::Value::from(false)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "a591a6d40bf420404a011733cfb7b190d62c65bf0bcda32b57b277d9ad9f146e"
        );
    }

    #[test]
    fn can_get_hash_sha256_base64() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([
            ("literal", tera::Value::from("Hello World")),
            ("sha_type", tera::Value::from(256)),
            ("base64", tera::Value::from(true)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "pZGm1Av0IEBKARczz7exkNYsZb8LzaMrV7J32a2fFG4="
        );
    }

    #[test]
    fn can_get_hash_sha384_no_base64() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([
            ("literal", tera::Value::from("Hello World")),
            ("base64", tera::Value::from(false)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "99514329186b2f6ae4a1329e7ee6c610a729636335174ac6b740f9028396fcc803d0e93863a7c3d90f86beee782f4f3f"
        );
    }

    #[test]
    fn can_get_hash_sha384() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([("literal", tera::Value::from("Hello World"))]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "mVFDKRhrL2rkoTKefubGEKcpY2M1F0rGt0D5AoOW/MgD0Ok4Y6fD2Q+Gvu54L08/"
        );
    }

    #[test]
    fn can_get_hash_sha512_no_base64() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([
            ("literal", tera::Value::from("Hello World")),
            ("sha_type", tera::Value::from(512)),
            ("base64", tera::Value::from(false)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "2c74fd17edafd80e8447b0d46741ee243b7eb74dd2149a0ab1b9246fb30382f27e853d8585719e0e67cbda0daa8f51671064615d645ae27acb15bfb1447f459b"
        );
    }

    #[test]
    fn can_get_hash_sha512() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([
            ("literal", tera::Value::from("Hello World")),
            ("sha_type", tera::Value::from(512)),
        ]);
        let ctx = Context::new();
        assert_eq!(
            get_hash.call(kwargs, &State::new(&ctx)).unwrap(),
            "LHT9F+2v2A6ER7DUZ0HuJDt+t03SFJoKsbkkb7MDgvJ+hT2FhXGeDmfL2g2qj1FnEGRhXWRa4nrLFb+xRH9Fmw=="
        );
    }

    #[test]
    fn static_file_does_not_get_lang_prefix_from_context() {
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            config,
            HashMap::new(),
            PathBuf::new(),
            AHashMap::new(),
        );

        // lang="en" (non-default) comes from template context, not from kwarg
        let kwargs = Kwargs::from([("path", tera::Value::from("app.css"))]);
        let mut ctx = Context::new();
        ctx.insert("lang", "en");
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "https://remplace-par-ton-url.fr/app.css"
        );
    }

    #[test]
    fn error_when_file_not_found_for_hash() {
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([("path", tera::Value::from("doesnt-exist"))]);
        let ctx = Context::new();
        let err = get_hash.call(kwargs, &State::new(&ctx)).unwrap_err();
        assert!(err.to_string().contains("Cannot find file"));
    }

    // https://github.com/getzola/zola/issues/2621
    #[test]
    fn does_not_drop_lang() {
        let dir = create_temp_dir();
        let config = Config::parse(CONFIG_DATA).unwrap();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            config,
            HashMap::new(),
            PathBuf::new(),
            AHashMap::new(),
        );
        let kwargs = Kwargs::from([
            ("path", tera::Value::from("genres")),
            ("lang", tera::Value::from("en")),
        ]);
        let ctx = Context::new();
        let url = get_url.call(kwargs, &State::new(&ctx)).unwrap();
        assert_eq!(url, "https://remplace-par-ton-url.fr/en/genres");

        // https://github.com/getzola/zola/issues/2216
        let kwargs =
            Kwargs::from([("path", tera::Value::from("/")), ("lang", tera::Value::from("en"))]);
        let url = get_url.call(kwargs, &State::new(&ctx)).unwrap();
        assert_eq!(url, "https://remplace-par-ton-url.fr/en");

        let kwargs = Kwargs::from([
            ("path", tera::Value::from("/")),
            ("lang", tera::Value::from("en")),
            ("trailing_slash", tera::Value::from(true)),
        ]);
        let url = get_url.call(kwargs, &State::new(&ctx)).unwrap();
        assert_eq!(url, "https://remplace-par-ton-url.fr/en/");
    }

    #[test]
    fn repeated_calls_use_cache_without_filesystem_lookup() {
        let dir = create_temp_dir();
        let app_css = dir.path().join("app.css");
        assert!(app_css.exists());

        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
            AHashMap::new(),
        );

        let ctx = Context::new();
        let kwargs = Kwargs::from([
            ("path", tera::Value::from("app.css")),
            ("cachebust", tera::Value::from(true)),
        ]);
        let url1 = get_url.call(kwargs.clone(), &State::new(&ctx)).unwrap();
        assert_eq!(url1, "http://a-website.com/app.css?h=572e691dc68c3fcd653a");

        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());
        let hash_kwargs = Kwargs::from([
            ("path", tera::Value::from("app.css")),
            ("sha_type", tera::Value::from(256)),
            ("base64", tera::Value::from(false)),
        ]);
        let hash1 = get_hash.call(hash_kwargs.clone(), &State::new(&ctx)).unwrap();
        assert_eq!(hash1, "572e691dc68c3fcd653ae463261bdb38f35dc6f01715d9ce68799319dd158840");

        // Remove the file from disk: subsequent calls must succeed from cache without filesystem access
        fs::remove_file(&app_css).unwrap();
        assert!(!app_css.exists());

        let url2 = get_url.call(kwargs, &State::new(&ctx)).unwrap();
        assert_eq!(url2, url1);

        let hash2 = get_hash.call(hash_kwargs, &State::new(&ctx)).unwrap();
        assert_eq!(hash2, hash1);
    }
}
