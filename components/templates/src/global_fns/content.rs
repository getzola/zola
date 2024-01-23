use content::{Library, Taxonomy, TaxonomyTerm};
use libs::tera::{from_value, to_value, Function as TeraFn, Result, Value};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use utils::slugs::{slugify_paths, SlugifyStrategy};

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
        let required = optional_arg!(
            bool,
            args.get("required"),
            "`get_taxonomy_url`: `required` must be a boolean (true or false)"
        )
        .unwrap_or(true);

        let container = match (self.taxonomies.get(&format!("{}-{}", kind, lang)), required) {
            (Some(c), _) => c,
            (None, false) => return Ok(Value::Null),
            (None, true) => {
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

    fn is_safe(&self) -> bool {
        true
    }
}

fn add_lang_to_path<'a>(path: &str, lang: &str) -> Result<Cow<'a, String>> {
    match path.rfind('.') {
        Some(period_offset) => {
            let prefix = path.get(0..period_offset);
            let suffix = path.get(period_offset..);
            if prefix.is_none() || suffix.is_none() {
                Err(format!("Error adding language code to {}", path).into())
            } else {
                Ok(Cow::Owned(format!("{}.{}{}", prefix.unwrap(), lang, suffix.unwrap())))
            }
        }
        None => Ok(Cow::Owned(format!("{}.{}", path, lang))),
    }
}

fn get_path_with_lang<'a>(
    path: &'a String,
    lang: &Option<String>,
    default_lang: &str,
    supported_languages: &[String],
) -> Result<Cow<'a, String>> {
    if supported_languages.contains(&default_lang.to_string()) {
        lang.as_ref().map_or_else(
            || Ok(Cow::Borrowed(path)),
            |lang_code| match default_lang == lang_code {
                true => Ok(Cow::Borrowed(path)),
                false => add_lang_to_path(path, lang_code),
            },
        )
    } else {
        Err(format!("Unsupported language {}", default_lang).into())
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
impl TeraFn for GetPage {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`get_page` requires a `path` argument with a string value"
        );

        let lang =
            optional_arg!(String, args.get("lang"), "`get_section`: `lang` must be a string");

        get_path_with_lang(&path, &lang, &self.default_lang, &self.supported_languages).and_then(
            |path_with_lang| {
                let full_path = self.base_path.join(path_with_lang.as_ref());
                let library = self.library.read().unwrap();

                match library.pages.get(&full_path) {
                    Some(p) => Ok(to_value(p.serialize(&library)).unwrap()),
                    None => match lang {
                        Some(lang_code) => {
                            Err(format!("Page `{}` not found for language `{}`.", path, lang_code)
                                .into())
                        }
                        None => Err(format!("Page `{}` not found.", path).into()),
                    },
                }
            },
        )
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

        let lang =
            optional_arg!(String, args.get("lang"), "`get_section`: `lang` must be a string");

        get_path_with_lang(&path, &lang, self.default_lang.as_str(), &self.supported_languages)
            .and_then(|path_with_lang| {
                let full_path = self.base_path.join(path_with_lang.as_ref());
                let library = self.library.read().unwrap();

                match library.sections.get(&full_path) {
                    Some(s) => {
                        if metadata_only {
                            Ok(to_value(s.serialize_basic(&library)).unwrap())
                        } else {
                            Ok(to_value(s.serialize(&library)).unwrap())
                        }
                    }
                    None => match lang {
                        Some(lang_code) => Err(format!(
                            "Section `{}` not found for language `{}`.",
                            path, lang_code
                        )
                        .into()),
                        None => Err(format!("Section `{}` not found.", path).into()),
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
impl TeraFn for GetTaxonomy {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let kind = required_arg!(
            String,
            args.get("kind"),
            "`get_taxonomy` requires a `kind` argument with a string value"
        );
        let required = optional_arg!(
            bool,
            args.get("required"),
            "`get_taxonomy`: `required` must be a boolean (true or false)"
        )
        .unwrap_or(true);

        let lang =
            optional_arg!(String, args.get("lang"), "`get_taxonomy`: `lang` must be a string")
                .unwrap_or_else(|| self.default_lang.clone());

        match (self.taxonomies.get(&format!("{}-{}", kind, lang)), required) {
            (Some(t), _) => Ok(to_value(t.to_serialized(&self.library.read().unwrap())).unwrap()),
            (None, false) => Ok(Value::Null),
            (None, true) => {
                Err(format!("`get_taxonomy` received an unknown taxonomy as kind: {}", kind).into())
            }
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
impl TeraFn for GetTaxonomyTerm {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let kind = required_arg!(
            String,
            args.get("kind"),
            "`get_taxonomy_term` requires a `kind` argument with a string value"
        );
        let term = required_arg!(
            String,
            args.get("term"),
            "`get_taxonomy_term` requires a `term` argument with a string value"
        );
        let include_pages = optional_arg!(
            bool,
            args.get("include_pages"),
            "`get_taxonomy_term`: `include_pages` must be a boolean (true or false)"
        )
        .unwrap_or(true);
        let required = optional_arg!(
            bool,
            args.get("required"),
            "`get_taxonomy_term`: `required` must be a boolean (true or false)"
        )
        .unwrap_or(true);

        let lang = optional_arg!(
            String,
            args.get("lang"),
            "`get_taxonomy_term_by_name`: `lang` must be a string"
        )
        .unwrap_or_else(|| self.default_lang.clone());

        let tax: &Taxonomy = match (self.taxonomies.get(&format!("{}-{}", kind, lang)), required) {
            (Some(t), _) => t,
            (None, false) => {
                return Ok(Value::Null);
            }
            (None, true) => {
                return Err(format!(
                    "`get_taxonomy_term_by_name` received an unknown taxonomy as kind: {}",
                    kind
                )
                .into());
            }
        };

        let term: &TaxonomyTerm = match (tax.items.iter().find(|i| i.name == term), required) {
            (Some(t), _) => t,
            (None, false) => {
                return Ok(Value::Null);
            }
            (None, true) => {
                return Err(format!(
                    "`get_taxonomy_term_by_name` received an unknown taxonomy as kind: {}",
                    kind
                )
                .into());
            }
        };

        if include_pages {
            Ok(to_value(term.serialize(&self.library.read().unwrap())).unwrap())
        } else {
            Ok(to_value(term.serialize_without_pages(&self.library.read().unwrap())).unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, TaxonomyConfig};
    use content::{FileInfo, Library, Page, Section, SortBy, TaxonomyTerm};
    use std::path::Path;
    use std::sync::{Arc, RwLock};

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

        let static_fn =
            GetPage::new(base_path, "en", Arc::new(lang_list), Arc::new(RwLock::new(library)));

        // Find with lang argument
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("wiki/recipes.md").unwrap());
        args.insert("lang".to_string(), to_value("fr").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["title"], to_value("Recettes").unwrap());

        // Find with lang in path for legacy support
        args = HashMap::new();
        args.insert("path".to_string(), to_value("wiki/recipes.fr.md").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["title"], to_value("Recettes").unwrap());

        // Find with default lang
        args = HashMap::new();
        args.insert("path".to_string(), to_value("wiki/recipes.md").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["title"], to_value("Recipes").unwrap());

        // Find with default lang when default lang passed
        args = HashMap::new();
        args.insert("path".to_string(), to_value("wiki/recipes.md").unwrap());
        args.insert("lang".to_string(), to_value("en").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["title"], to_value("Recipes").unwrap());
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

        let static_fn =
            GetSection::new(base_path, "en", Arc::new(lang_list), Arc::new(RwLock::new(library)));

        // Find with lang argument
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("wiki/recipes/_index.md").unwrap());
        args.insert("lang".to_string(), to_value("fr").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["title"], to_value("Recettes").unwrap());

        // Find with lang in path for legacy support
        args = HashMap::new();
        args.insert("path".to_string(), to_value("wiki/recipes/_index.fr.md").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["title"], to_value("Recettes").unwrap());

        // Find with default lang
        args = HashMap::new();
        args.insert("path".to_string(), to_value("wiki/recipes/_index.md").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["title"], to_value("Recipes").unwrap());

        // Find with default lang when default lang passed
        args = HashMap::new();
        args.insert("path".to_string(), to_value("wiki/recipes/_index.md").unwrap());
        args.insert("lang".to_string(), to_value("en").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["title"], to_value("Recipes").unwrap());
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
        let static_fn = GetTaxonomy::new(&config.default_language, taxonomies, library);
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
        let static_fn = GetTaxonomyTerm::new(&config.default_language, taxonomies, library);
        // can find it correctly
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("term".to_string(), to_value("Programming").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["name"], Value::String("Programming".to_string()));
        assert_eq!(res_obj["slug"], Value::String("programming".to_string()));
        assert_eq!(
            res_obj["permalink"],
            Value::String("http://a-website.com/tags/programming/".to_string())
        );
        assert_eq!(res_obj["pages"], Value::Array(vec![]));
        // Works with other languages as well
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("term".to_string(), to_value("Programmation").unwrap());
        args.insert("lang".to_string(), to_value("fr").unwrap());
        let res = static_fn.call(&args).unwrap();
        let res_obj = res.as_object().unwrap();
        assert_eq!(res_obj["name"], Value::String("Programmation".to_string()));

        // and errors if it can't find either taxonomy or term
        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("something-else").unwrap());
        args.insert("term".to_string(), to_value("Programming").unwrap());
        assert!(static_fn.call(&args).is_err());

        let mut args = HashMap::new();
        args.insert("kind".to_string(), to_value("tags").unwrap());
        args.insert("kind".to_string(), to_value("something-else").unwrap());
        assert!(static_fn.call(&args).is_err());
    }
}
