use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use content::{Library, Taxonomy, TaxonomyTerm};
use tera::{Error, Function, Kwargs, State, TeraResult, Value};
use utils::slugs::{SlugifyStrategy, slugify_paths};

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
            taxonomies.insert(format!("{}-{}", taxo.kind.name, taxo.lang), items);
        }
        Self { taxonomies, default_lang: default_lang.to_string(), slugify }
    }
}

impl Function<TeraResult<Value>> for GetTaxonomyUrl {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let kind: String = kwargs.must_get("kind")?;
        let name: String = kwargs.must_get("name")?;
        let lang: String = state
            .get("lang")
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.default_lang.clone());
        let required: bool = kwargs.get("required")?.unwrap_or(true);

        let container = match (self.taxonomies.get(&format!("{}-{}", kind, lang)), required) {
            (Some(c), _) => c,
            (None, false) => return Ok(Value::null()),
            (None, true) => {
                return Err(Error::message(format!(
                    "`get_taxonomy_url` received an unknown taxonomy as kind: {}",
                    kind
                )));
            }
        };

        if let Some(permalink) = container.get(&slugify_paths(&name, self.slugify)) {
            return Ok(Value::from(permalink.as_str()));
        }

        Err(Error::message(format!(
            "`get_taxonomy_url`: couldn't find `{}` in `{}` taxonomy",
            name, kind
        )))
    }

    fn is_safe(&self) -> bool {
        true
    }
}

fn add_lang_to_path(path: &str, lang: &str) -> TeraResult<Cow<'static, str>> {
    match path.rfind('.') {
        Some(period_offset) => {
            let prefix = path.get(0..period_offset);
            let suffix = path.get(period_offset..);
            if prefix.is_none() || suffix.is_none() {
                Err(Error::message(format!("Error adding language code to {}", path)))
            } else {
                Ok(Cow::Owned(format!("{}.{}{}", prefix.unwrap(), lang, suffix.unwrap())))
            }
        }
        None => Ok(Cow::Owned(format!("{}.{}", path, lang))),
    }
}

fn get_path_with_lang<'a>(
    path: &'a str,
    lang: Option<&str>,
    default_lang: &str,
    supported_languages: &[String],
) -> TeraResult<Cow<'a, str>> {
    if supported_languages.contains(&default_lang.to_string()) {
        lang.as_ref().map_or_else(
            || Ok(Cow::Borrowed(path)),
            |&lang_code| match default_lang == lang_code {
                true => Ok(Cow::Borrowed(path)),
                false => {
                    // Need to convert Cow<'static, str> to Cow<'a, str>
                    add_lang_to_path(path, lang_code).map(|c| Cow::Owned(c.into_owned()))
                }
            },
        )
    } else {
        Err(Error::message(format!("Unsupported language {}", default_lang)))
    }
}

#[derive(Debug)]
pub struct GetPage {
    base_path: PathBuf,
    default_lang: String,
    supported_languages: Arc<Vec<String>>,
    library: Arc<RwLock<Library>>,
}

impl GetPage {
    pub fn new(
        base_path: PathBuf,
        default_lang: &str,
        supported_languages: Arc<Vec<String>>,
        library: Arc<RwLock<Library>>,
    ) -> Self {
        Self {
            base_path: base_path.join("content"),
            default_lang: default_lang.to_string(),
            supported_languages,
            library,
        }
    }
}

impl Function<TeraResult<Value>> for GetPage {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let path: String = kwargs.must_get("path")?;
        let lang: Option<String> = state.get("lang").as_str().map(|s| s.to_string());

        get_path_with_lang(&path, lang.as_deref(), &self.default_lang, &self.supported_languages)
            .and_then(|path_with_lang| {
                let full_path = self.base_path.join(path_with_lang.as_ref());
                let library = self.library.read().unwrap();

                match library.pages.get(&full_path) {
                    Some(p) => Ok(Value::from_serializable(&p.serialize(&library))),
                    None => match lang {
                        Some(lang_code) => Err(Error::message(format!(
                            "Page `{}` not found for language `{}`.",
                            path, lang_code
                        ))),
                        None => Err(Error::message(format!("Page `{}` not found.", path))),
                    },
                }
            })
    }
}

#[derive(Debug)]
pub struct GetSection {
    base_path: PathBuf,
    default_lang: String,
    supported_languages: Arc<Vec<String>>,
    library: Arc<RwLock<Library>>,
}

impl GetSection {
    pub fn new(
        base_path: PathBuf,
        default_lang: &str,
        supported_languages: Arc<Vec<String>>,
        library: Arc<RwLock<Library>>,
    ) -> Self {
        Self {
            base_path: base_path.join("content"),
            default_lang: default_lang.to_string(),
            supported_languages,
            library,
        }
    }
}

impl Function<TeraResult<Value>> for GetSection {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let path: String = kwargs.must_get("path")?;
        let metadata_only: bool = kwargs.get("metadata_only")?.unwrap_or(false);
        let lang: Option<String> = state.get("lang").as_str().map(|s| s.to_string());

        get_path_with_lang(&path, lang.as_deref(), &self.default_lang, &self.supported_languages)
            .and_then(|path_with_lang| {
                let full_path = self.base_path.join(path_with_lang.as_ref());
                let library = self.library.read().unwrap();

                match library.sections.get(&full_path) {
                    Some(s) => {
                        if metadata_only {
                            Ok(Value::from_serializable(&s.serialize_basic(&library)))
                        } else {
                            Ok(Value::from_serializable(&s.serialize(&library)))
                        }
                    }
                    None => match lang {
                        Some(lang_code) => Err(Error::message(format!(
                            "Section `{}` not found for language `{}`.",
                            path, lang_code
                        ))),
                        None => Err(Error::message(format!("Section `{}` not found.", path))),
                    },
                }
            })
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
            taxonomies.insert(format!("{}-{}", taxo.kind.name, taxo.lang), taxo);
        }
        Self { taxonomies, library, default_lang: default_lang.to_string() }
    }
}

impl Function<TeraResult<Value>> for GetTaxonomy {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let kind: String = kwargs.must_get("kind")?;
        let required: bool = kwargs.get("required")?.unwrap_or(true);
        let lang: String = state
            .get("lang")
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.default_lang.clone());

        match (self.taxonomies.get(&format!("{}-{}", kind, lang)), required) {
            (Some(t), _) => {
                Ok(Value::from_serializable(&t.to_serialized(&self.library.read().unwrap())))
            }
            (None, false) => Ok(Value::null()),
            (None, true) => Err(Error::message(format!(
                "`get_taxonomy` received an unknown taxonomy as kind: {}",
                kind
            ))),
        }
    }
}

#[derive(Debug)]
pub struct GetTaxonomyTerm {
    library: Arc<RwLock<Library>>,
    taxonomies: HashMap<String, Taxonomy>,
    default_lang: String,
}

impl GetTaxonomyTerm {
    pub fn new(
        default_lang: &str,
        all_taxonomies: Vec<Taxonomy>,
        library: Arc<RwLock<Library>>,
    ) -> Self {
        let mut taxonomies = HashMap::new();
        for taxo in all_taxonomies {
            taxonomies.insert(format!("{}-{}", taxo.kind.name, taxo.lang), taxo);
        }
        Self { taxonomies, library, default_lang: default_lang.to_string() }
    }
}

impl Function<TeraResult<Value>> for GetTaxonomyTerm {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let kind: String = kwargs.must_get("kind")?;
        let term: String = kwargs.must_get("term")?;
        let include_pages: bool = kwargs.get("include_pages")?.unwrap_or(true);
        let required: bool = kwargs.get("required")?.unwrap_or(true);
        let lang: String = state
            .get("lang")
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.default_lang.clone());

        let tax: &Taxonomy = match (self.taxonomies.get(&format!("{}-{}", kind, lang)), required) {
            (Some(t), _) => t,
            (None, false) => return Ok(Value::null()),
            (None, true) => {
                return Err(Error::message(format!(
                    "`get_taxonomy_term` received an unknown taxonomy as kind: {}",
                    kind
                )));
            }
        };

        let taxonomy_term: &TaxonomyTerm =
            match (tax.items.iter().find(|i| i.name == term), required) {
                (Some(t), _) => t,
                (None, false) => return Ok(Value::null()),
                (None, true) => {
                    return Err(Error::message(format!(
                        "`get_taxonomy_term` received an unknown term: {}",
                        term
                    )));
                }
            };

        if include_pages {
            Ok(Value::from_serializable(&taxonomy_term.serialize(&self.library.read().unwrap())))
        } else {
            Ok(Value::from_serializable(
                &taxonomy_term.serialize_without_pages(&self.library.read().unwrap()),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, TaxonomyConfig};
    use content::{FileInfo, Library, Page, Section, SortBy};
    use std::path::Path;
    use std::sync::Arc;
    use tera::{Context, Map};

    fn create_page(title: &str, file_path: &str, lang: &str) -> Page {
        let mut page = Page { lang: lang.to_owned(), ..Page::default() };
        page.file = FileInfo::new_page(
            Path::new(format!("/test/base/path/{}", file_path).as_str()),
            &PathBuf::new(),
        );
        page.meta.title = Some(title.to_string());
        page.meta.weight = Some(1);
        page.file.find_language("en", &["fr"]).unwrap();
        page
    }

    fn make_kwargs(args: Vec<(&str, Value)>) -> Kwargs {
        let mut map = Map::new();
        for (k, v) in args {
            map.insert(k.into(), v);
        }
        Kwargs::new(Arc::new(map))
    }

    fn make_context_with_lang(lang: &str) -> Context {
        let mut ctx = Context::new();
        ctx.insert("lang", &lang);
        ctx
    }

    #[test]
    fn can_get_page() {
        let mut library = Library::default();
        let pages = vec![
            ("Homepage", "content/homepage.md", "en"),
            ("Page D'Accueil", "content/homepage.fr.md", "fr"),
            ("Blog", "content/blog.md", "en"),
            ("Wiki", "content/wiki.md", "en"),
            ("Wiki", "content/wiki.fr.md", "fr"),
            ("Recipes", "content/wiki/recipes.md", "en"),
            ("Recettes", "content/wiki/recipes.fr.md", "fr"),
            ("Programming", "content/wiki/programming.md", "en"),
            ("La Programmation", "content/wiki/programming.fr.md", "fr"),
            ("Novels", "content/novels.md", "en"),
            ("Des Romans", "content/novels.fr.md", "fr"),
        ];
        for (t, f, l) in pages.clone() {
            library.insert_page(create_page(t, f, l));
        }
        let base_path = "/test/base/path".into();
        let lang_list = vec!["en".to_string(), "fr".to_string()];

        let get_page =
            GetPage::new(base_path, "en", Arc::new(lang_list), Arc::new(RwLock::new(library)));

        // Find with lang in context
        let kwargs = make_kwargs(vec![("path", Value::from("wiki/recipes.md"))]);
        let ctx = make_context_with_lang("fr");
        let res = get_page.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recettes");

        // Find with lang in path for legacy support
        let kwargs = make_kwargs(vec![("path", Value::from("wiki/recipes.fr.md"))]);
        let ctx = Context::new();
        let res = get_page.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recettes");

        // Find with default lang (no lang in context)
        let kwargs = make_kwargs(vec![("path", Value::from("wiki/recipes.md"))]);
        let ctx = Context::new();
        let res = get_page.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recipes");

        // Find with default lang when default lang in context
        let kwargs = make_kwargs(vec![("path", Value::from("wiki/recipes.md"))]);
        let ctx = make_context_with_lang("en");
        let res = get_page.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recipes");
    }

    fn create_section(title: &str, file_path: &str, lang: &str) -> Section {
        let mut section = Section { lang: lang.to_owned(), ..Section::default() };
        section.file = FileInfo::new_section(
            Path::new(format!("/test/base/path/{}", file_path).as_str()),
            &PathBuf::new(),
        );
        section.meta.title = Some(title.to_string());
        section.meta.weight = 1;
        section.meta.transparent = false;
        section.meta.sort_by = SortBy::None;
        section.meta.page_template = Some("new_page.html".to_owned());
        section.file.find_language("en", &["fr"]).unwrap();
        section
    }

    #[test]
    fn can_get_section() {
        let mut library = Library::default();
        let sections = vec![
            ("Homepage", "content/_index.md", "en"),
            ("Page D'Accueil", "content/_index.fr.md", "fr"),
            ("Blog", "content/blog/_index.md", "en"),
            ("Wiki", "content/wiki/_index.md", "en"),
            ("Wiki", "content/wiki/_index.fr.md", "fr"),
            ("Recipes", "content/wiki/recipes/_index.md", "en"),
            ("Recettes", "content/wiki/recipes/_index.fr.md", "fr"),
            ("Programming", "content/wiki/programming/_index.md", "en"),
            ("La Programmation", "content/wiki/programming/_index.fr.md", "fr"),
            ("Novels", "content/novels/_index.md", "en"),
            ("Des Romans", "content/novels/_index.fr.md", "fr"),
        ];
        for (t, f, l) in sections.clone() {
            library.insert_section(create_section(t, f, l));
        }
        let base_path = "/test/base/path".into();
        let lang_list = vec!["en".to_string(), "fr".to_string()];

        let get_section =
            GetSection::new(base_path, "en", Arc::new(lang_list), Arc::new(RwLock::new(library)));

        // Find with lang in context
        let kwargs = make_kwargs(vec![("path", Value::from("wiki/recipes/_index.md"))]);
        let ctx = make_context_with_lang("fr");
        let res = get_section.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recettes");

        // Find with lang in path for legacy support
        let kwargs = make_kwargs(vec![("path", Value::from("wiki/recipes/_index.fr.md"))]);
        let ctx = Context::new();
        let res = get_section.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recettes");

        // Find with default lang (no lang in context)
        let kwargs = make_kwargs(vec![("path", Value::from("wiki/recipes/_index.md"))]);
        let ctx = Context::new();
        let res = get_section.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recipes");

        // Find with default lang when default lang in context
        let kwargs = make_kwargs(vec![("path", Value::from("wiki/recipes/_index.md"))]);
        let ctx = make_context_with_lang("en");
        let res = get_section.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recipes");
    }

    #[test]
    fn can_get_taxonomy() {
        let mut config = Config::default_for_test();
        config.slugify.taxonomies = SlugifyStrategy::On;
        let taxo_config = TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        let taxo_config_fr =
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        config.slugify_taxonomies();
        let library = Arc::new(RwLock::new(Library::new(&config)));
        let tag = TaxonomyTerm::new("Programming", &config.default_language, "tags", &[], &config);
        let tag_fr = TaxonomyTerm::new("Programmation", "fr", "tags", &[], &config);
        let tags = Taxonomy {
            kind: taxo_config,
            lang: config.default_language.clone(),
            slug: "tags".to_string(),
            path: "/tags/".to_string(),
            permalink: "https://vincent.is/tags/".to_string(),
            items: vec![tag],
        };
        let tags_fr = Taxonomy {
            kind: taxo_config_fr,
            lang: "fr".to_owned(),
            slug: "tags".to_string(),
            path: "/fr/tags/".to_string(),
            permalink: "https://vincent.is/fr/tags/".to_string(),
            items: vec![tag_fr],
        };

        let taxonomies = vec![tags.clone(), tags_fr.clone()];
        let get_taxonomy = GetTaxonomy::new(&config.default_language, taxonomies, library);

        // can find it correctly (default lang)
        let kwargs = make_kwargs(vec![("kind", Value::from("tags"))]);
        let ctx = Context::new();
        let res = get_taxonomy.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        let items = res_obj.get(&"items".into()).unwrap().as_vec().unwrap();
        assert_eq!(items.len(), 1);
        let item = items[0].as_map().unwrap();
        assert_eq!(item.get(&"name".into()).unwrap().as_str().unwrap(), "Programming");
        assert_eq!(item.get(&"slug".into()).unwrap().as_str().unwrap(), "programming");

        // Works with other languages as well (lang in context)
        let kwargs = make_kwargs(vec![("kind", Value::from("tags"))]);
        let ctx = make_context_with_lang("fr");
        let res = get_taxonomy.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        let items = res_obj.get(&"items".into()).unwrap().as_vec().unwrap();
        assert_eq!(items.len(), 1);
        let item = items[0].as_map().unwrap();
        assert_eq!(item.get(&"name".into()).unwrap().as_str().unwrap(), "Programmation");

        // and errors if it can't find it
        let kwargs = make_kwargs(vec![("kind", Value::from("something-else"))]);
        let ctx = Context::new();
        assert!(get_taxonomy.call(kwargs, &State::new(&ctx)).is_err());
    }

    #[test]
    fn can_get_taxonomy_url() {
        let mut config = Config::default();
        config.slugify.taxonomies = SlugifyStrategy::On;
        let taxo_config = TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        let taxo_config_fr =
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        let tag = TaxonomyTerm::new("Programming", &config.default_language, "tags", &[], &config);
        let tag_fr = TaxonomyTerm::new("Programmation", "fr", "tags", &[], &config);
        let tags = Taxonomy {
            kind: taxo_config,
            lang: config.default_language.clone(),
            slug: "tags".to_string(),
            path: "/tags/".to_string(),
            permalink: "https://vincent.is/tags/".to_string(),
            items: vec![tag],
        };
        let tags_fr = Taxonomy {
            kind: taxo_config_fr,
            lang: "fr".to_owned(),
            slug: "tags".to_string(),
            path: "/fr/tags/".to_string(),
            permalink: "https://vincent.is/fr/tags/".to_string(),
            items: vec![tag_fr],
        };

        let taxonomies = vec![tags, tags_fr];
        let get_taxonomy_url =
            GetTaxonomyUrl::new(&config.default_language, &taxonomies, config.slugify.taxonomies);

        // can find it correctly (default lang)
        let kwargs =
            make_kwargs(vec![("kind", Value::from("tags")), ("name", Value::from("Programming"))]);
        let ctx = Context::new();
        assert_eq!(
            get_taxonomy_url.call(kwargs, &State::new(&ctx)).unwrap().as_str().unwrap(),
            "http://a-website.com/tags/programming/"
        );

        // can find it correctly with inconsistent capitalisation
        let kwargs =
            make_kwargs(vec![("kind", Value::from("tags")), ("name", Value::from("programming"))]);
        let ctx = Context::new();
        assert_eq!(
            get_taxonomy_url.call(kwargs, &State::new(&ctx)).unwrap().as_str().unwrap(),
            "http://a-website.com/tags/programming/"
        );

        // works with other languages (lang in context)
        let kwargs = make_kwargs(vec![
            ("kind", Value::from("tags")),
            ("name", Value::from("Programmation")),
        ]);
        let ctx = make_context_with_lang("fr");
        assert_eq!(
            get_taxonomy_url.call(kwargs, &State::new(&ctx)).unwrap().as_str().unwrap(),
            "http://a-website.com/fr/tags/programmation/"
        );

        // and errors if it can't find it
        let kwargs =
            make_kwargs(vec![("kind", Value::from("tags")), ("name", Value::from("random"))]);
        let ctx = Context::new();
        assert!(get_taxonomy_url.call(kwargs, &State::new(&ctx)).is_err());
    }

    #[test]
    fn can_get_taxonomy_term() {
        let mut config = Config::default_for_test();
        config.slugify.taxonomies = SlugifyStrategy::On;
        let taxo_config = TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        let taxo_config_fr =
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        config.slugify_taxonomies();
        let library = Arc::new(RwLock::new(Library::new(&config)));
        let tag = TaxonomyTerm::new("Programming", &config.default_language, "tags", &[], &config);
        let tag_fr = TaxonomyTerm::new("Programmation", "fr", "tags", &[], &config);
        let tags = Taxonomy {
            kind: taxo_config,
            lang: config.default_language.clone(),
            slug: "tags".to_string(),
            path: "/tags/".to_string(),
            permalink: "https://vincent.is/tags/".to_string(),
            items: vec![tag],
        };
        let tags_fr = Taxonomy {
            kind: taxo_config_fr,
            lang: "fr".to_owned(),
            slug: "tags".to_string(),
            path: "/fr/tags/".to_string(),
            permalink: "https://vincent.is/fr/tags/".to_string(),
            items: vec![tag_fr],
        };

        let taxonomies = vec![tags.clone(), tags_fr.clone()];
        let get_taxonomy_term = GetTaxonomyTerm::new(&config.default_language, taxonomies, library);

        // can find it correctly (default lang)
        let kwargs =
            make_kwargs(vec![("kind", Value::from("tags")), ("term", Value::from("Programming"))]);
        let ctx = Context::new();
        let res = get_taxonomy_term.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"name".into()).unwrap().as_str().unwrap(), "Programming");
        assert_eq!(res_obj.get(&"slug".into()).unwrap().as_str().unwrap(), "programming");

        // Works with other languages as well (lang in context)
        let kwargs = make_kwargs(vec![
            ("kind", Value::from("tags")),
            ("term", Value::from("Programmation")),
        ]);
        let ctx = make_context_with_lang("fr");
        let res = get_taxonomy_term.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"name".into()).unwrap().as_str().unwrap(), "Programmation");

        // and errors if it can't find either taxonomy or term
        let kwargs = make_kwargs(vec![
            ("kind", Value::from("something-else")),
            ("term", Value::from("Programming")),
        ]);
        let ctx = Context::new();
        assert!(get_taxonomy_term.call(kwargs, &State::new(&ctx)).is_err());

        let kwargs = make_kwargs(vec![
            ("kind", Value::from("tags")),
            ("term", Value::from("something-else")),
        ]);
        let ctx = Context::new();
        assert!(get_taxonomy_term.call(kwargs, &State::new(&ctx)).is_err());
    }
}
