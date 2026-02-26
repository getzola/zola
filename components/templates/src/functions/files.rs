use std::io::Read;
use std::path::PathBuf;

use fs_err as fs;

use ahash::AHashMap as HashMap;
use base64::engine::{Engine, general_purpose::STANDARD as standard_b64};
use sha2::{Sha256, Sha384, Sha512, digest};

use config::Config;
use tera::{Error, Function, Kwargs, State, TeraResult};
use utils::site::resolve_internal_link;

use crate::helpers::search_for_file;

fn compute_hash<D>(data: &[u8], as_base64: bool) -> String
where
    D: std::io::Write,
    D: digest::Digest,
    digest::Output<D>: core::fmt::LowerHex,
{
    let mut hasher = D::new();
    hasher.update(data);
    if as_base64 {
        standard_b64.encode(hasher.finalize())
    } else {
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Default)]
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
        let lang: String = kwargs
            .get("lang")?
            .or(state.get("lang")?)
            .unwrap_or_else(|| self.config.default_language.clone());

        // if it starts with @/, resolve it as an internal link
        if path.starts_with("@/") {
            let path_with_lang = make_path_with_lang(path, &lang, &self.config)?;

            match resolve_internal_link(&path_with_lang, &self.permalinks) {
                Ok(resolved) => Ok(resolved.permalink),
                Err(_) => Err(Error::message(format!(
                    "`get_url`: could not resolve URL for link `{}` not found.",
                    path_with_lang
                ))),
            }
        } else {
            // anything else
            let mut segments = vec![];

            if lang != self.config.default_language
                && (path.is_empty() || !path[1..].starts_with(&lang))
            {
                segments.push(lang.clone());
            }

            segments.push(path.clone());

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
                .map_err(|e| Error::message(format!("`get_url`: {}", e)))?
                .and_then(|(p, _)| fs::File::open(p).ok())
                .and_then(|mut f| {
                    let mut contents = Vec::new();
                    f.read_to_end(&mut contents).ok()?;
                    Some(compute_hash::<Sha256>(&contents, false))
                }) {
                    Some(hash) => {
                        let shorthash = &hash[..20]; // 2^-80 chance of false positive
                        permalink = format!("{}?h={}", permalink, shorthash);
                    }
                    None => {
                        return Err(Error::message(format!(
                            "`get_url`: Could not find or open file {}",
                            path_with_lang
                        )));
                    }
                };
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
}

impl GetHash {
    pub fn new(base_path: PathBuf, theme: Option<String>, output_path: PathBuf) -> Self {
        Self { base_path, theme, output_path }
    }
}

impl Function<TeraResult<String>> for GetHash {
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<String> {
        let path: Option<String> = kwargs.get("path")?;
        let literal: Option<String> = kwargs.get("literal")?;

        let contents = match (path, literal) {
            (Some(_), Some(_)) => {
                return Err(Error::message(
                    "`get_hash`: must have only one of `path` or `literal` argument",
                ));
            }
            (None, None) => {
                return Err(Error::message(
                    "`get_hash`: must have at least one of `path` or `literal` argument",
                ));
            }
            (Some(path_v), None) => {
                let file_path =
                    match search_for_file(&self.base_path, &path_v, &self.theme, &self.output_path)
                        .map_err(|e| Error::message(format!("`get_hash`: {}", e)))?
                    {
                        Some((f, _)) => f,
                        None => {
                            return Err(Error::message(format!(
                                "`get_hash`: Cannot find file: {}",
                                path_v
                            )));
                        }
                    };

                let mut f = fs::File::open(&file_path).map_err(|e| {
                    Error::message(format!("File {} could not be open: {}", path_v, e))
                })?;

                let mut contents = Vec::new();
                f.read_to_end(&mut contents).map_err(|e| {
                    Error::message(format!("File {} could not be read: {}", path_v, e))
                })?;

                contents
            }
            (None, Some(literal_v)) => literal_v.into_bytes(),
        };

        let sha_type: u16 = kwargs.get("sha_type")?.unwrap_or(384);
        let base64: bool = kwargs.get("base64")?.unwrap_or(true);

        let hash = match sha_type {
            256 => compute_hash::<Sha256>(&contents, base64),
            384 => compute_hash::<Sha384>(&contents, base64),
            512 => compute_hash::<Sha512>(&contents, base64),
            _ => return Err(Error::message("`get_hash`: Invalid sha value")),
        };

        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::{GetHash, GetUrl, HashMap};

    use std::path::PathBuf;

    use fs_err as fs;
    use tempfile::{TempDir, tempdir};
    use tera::{Context, Kwargs, State};

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
        use tera::Function;
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
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
    }

    #[test]
    fn can_add_trailing_slashes() {
        use tera::Function;
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
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
        use tera::Function;
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
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
        use tera::Function;
        let dir = create_temp_dir();
        let get_url = GetUrl::new(
            dir.path().to_path_buf(),
            Config::default(),
            HashMap::new(),
            PathBuf::new(),
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
        use tera::Function;
        let dir = create_temp_dir();
        let public = dir.path().join("public");
        fs::create_dir(&public).expect("Failed to create output directory");
        create_file(&public.join("style.css"), "// Hello world")
            .expect("Failed to create file in output directory");

        let get_url =
            GetUrl::new(dir.path().to_path_buf(), Config::default(), HashMap::new(), public);

        let kwargs = Kwargs::from([("path", tera::Value::from("style.css"))]);
        let ctx = Context::new();
        assert_eq!(
            get_url.call(kwargs, &State::new(&ctx)).unwrap(),
            "http://a-website.com/style.css"
        );
    }

    #[test]
    fn error_when_language_not_available() {
        use tera::Function;
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let get_url = GetUrl::new(dir.path().to_path_buf(), config, HashMap::new(), PathBuf::new());

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
        use tera::Function;
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
        use tera::Function;
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
        let get_url = GetUrl::new(dir.path().to_path_buf(), config, permalinks, PathBuf::new());

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

    #[test]
    fn does_not_duplicate_lang() {
        use tera::Function;
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
        let get_url = GetUrl::new(dir.path().to_path_buf(), config, permalinks, PathBuf::new());

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
        use tera::Function;
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let get_url =
            GetUrl::new(dir.path().to_path_buf(), config.clone(), HashMap::new(), PathBuf::new());

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
        use tera::Function;
        let config = Config::parse(CONFIG_DATA).unwrap();
        let dir = create_temp_dir();
        let get_url =
            GetUrl::new(dir.path().to_path_buf(), config.clone(), HashMap::new(), PathBuf::new());

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
        use tera::Function;
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
        use tera::Function;
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
        use tera::Function;
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
        use tera::Function;
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
        use tera::Function;
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
        use tera::Function;
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
        use tera::Function;
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
        use tera::Function;
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
        use tera::Function;
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
        use tera::Function;
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
        use tera::Function;
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
        use tera::Function;
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
    fn error_when_file_not_found_for_hash() {
        use tera::Function;
        let dir = create_temp_dir();
        let get_hash = GetHash::new(dir.path().to_path_buf(), None, PathBuf::new());

        let kwargs = Kwargs::from([("path", tera::Value::from("doesnt-exist"))]);
        let ctx = Context::new();
        let err = get_hash.call(kwargs, &State::new(&ctx)).unwrap_err();
        assert!(err.to_string().contains("Cannot find file"));
    }
}
