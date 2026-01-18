use std::collections::HashMap;
use std::sync::Arc;

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

impl Default for GetTaxonomyUrl {
    fn default() -> Self {
        Self {
            taxonomies: HashMap::new(),
            default_lang: String::new(),
            slugify: SlugifyStrategy::default(),
        }
    }
}

impl Function<TeraResult<Value>> for GetTaxonomyUrl {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let kind: String = kwargs.must_get("kind")?;
        let name: String = kwargs.must_get("name")?;
        let lang: String = state.get("lang")?.unwrap_or_else(|| self.default_lang.clone());
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

#[derive(Debug)]
pub struct GetTaxonomy {
    library: Arc<Library>,
    taxonomies: HashMap<String, Taxonomy>,
    default_lang: String,
}

impl GetTaxonomy {
    pub fn new(default_lang: &str, all_taxonomies: Vec<Taxonomy>, library: Arc<Library>) -> Self {
        let mut taxonomies = HashMap::new();
        for taxo in all_taxonomies {
            taxonomies.insert(format!("{}-{}", taxo.kind.name, taxo.lang), taxo);
        }
        Self { taxonomies, library, default_lang: default_lang.to_string() }
    }
}

impl Default for GetTaxonomy {
    fn default() -> Self {
        Self {
            library: Arc::new(Library::default()),
            taxonomies: HashMap::new(),
            default_lang: String::new(),
        }
    }
}

impl Function<TeraResult<Value>> for GetTaxonomy {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let kind: String = kwargs.must_get("kind")?;
        let required: bool = kwargs.get("required")?.unwrap_or(true);
        let lang: String = state.get("lang")?.unwrap_or_else(|| self.default_lang.clone());

        match (self.taxonomies.get(&format!("{}-{}", kind, lang)), required) {
            (Some(t), _) => Ok(Value::from_serializable(&t.to_serialized(&self.library))),
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
    library: Arc<Library>,
    taxonomies: HashMap<String, Taxonomy>,
    default_lang: String,
}

impl GetTaxonomyTerm {
    pub fn new(default_lang: &str, all_taxonomies: Vec<Taxonomy>, library: Arc<Library>) -> Self {
        let mut taxonomies = HashMap::new();
        for taxo in all_taxonomies {
            taxonomies.insert(format!("{}-{}", taxo.kind.name, taxo.lang), taxo);
        }
        Self { taxonomies, library, default_lang: default_lang.to_string() }
    }
}

impl Default for GetTaxonomyTerm {
    fn default() -> Self {
        Self {
            library: Arc::new(Library::default()),
            taxonomies: HashMap::new(),
            default_lang: String::new(),
        }
    }
}

impl Function<TeraResult<Value>> for GetTaxonomyTerm {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let kind: String = kwargs.must_get("kind")?;
        let term: String = kwargs.must_get("term")?;
        let include_pages: bool = kwargs.get("include_pages")?.unwrap_or(true);
        let required: bool = kwargs.get("required")?.unwrap_or(true);
        let lang: String = state.get("lang")?.unwrap_or_else(|| self.default_lang.clone());

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
            Ok(Value::from_serializable(&taxonomy_term.serialize(&self.library)))
        } else {
            Ok(Value::from_serializable(&taxonomy_term.serialize_without_pages(&self.library)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, TaxonomyConfig};
    use content::Library;
    use tera::{Context, Kwargs};
    use utils::slugs::SlugifyStrategy;

    fn make_context_with_lang(lang: &str) -> Context {
        let mut ctx = Context::new();
        ctx.insert("lang", &lang);
        ctx
    }

    #[test]
    fn can_get_taxonomy() {
        let mut config = Config::default_for_test();
        config.slugify.taxonomies = SlugifyStrategy::On;
        let taxo_config = TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        let taxo_config_fr =
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        config.slugify_taxonomies();
        let library = Arc::new(Library::new(&config));
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
        let kwargs = Kwargs::from([("kind", Value::from("tags"))]);
        let ctx = Context::new();
        let res = get_taxonomy.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        let items = res_obj.get(&"items".into()).unwrap().as_vec().unwrap();
        assert_eq!(items.len(), 1);
        let item = items[0].as_map().unwrap();
        assert_eq!(item.get(&"name".into()).unwrap().as_str().unwrap(), "Programming");
        assert_eq!(item.get(&"slug".into()).unwrap().as_str().unwrap(), "programming");

        // Works with other languages as well (lang in context)
        let kwargs = Kwargs::from([("kind", Value::from("tags"))]);
        let ctx = make_context_with_lang("fr");
        let res = get_taxonomy.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        let items = res_obj.get(&"items".into()).unwrap().as_vec().unwrap();
        assert_eq!(items.len(), 1);
        let item = items[0].as_map().unwrap();
        assert_eq!(item.get(&"name".into()).unwrap().as_str().unwrap(), "Programmation");

        // and errors if it can't find it
        let kwargs = Kwargs::from([("kind", Value::from("something-else"))]);
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
            Kwargs::from([("kind", Value::from("tags")), ("name", Value::from("Programming"))]);
        let ctx = Context::new();
        assert_eq!(
            get_taxonomy_url.call(kwargs, &State::new(&ctx)).unwrap().as_str().unwrap(),
            "http://a-website.com/tags/programming/"
        );

        // can find it correctly with inconsistent capitalisation
        let kwargs =
            Kwargs::from([("kind", Value::from("tags")), ("name", Value::from("programming"))]);
        let ctx = Context::new();
        assert_eq!(
            get_taxonomy_url.call(kwargs, &State::new(&ctx)).unwrap().as_str().unwrap(),
            "http://a-website.com/tags/programming/"
        );

        // works with other languages (lang in context)
        let kwargs =
            Kwargs::from([("kind", Value::from("tags")), ("name", Value::from("Programmation"))]);
        let ctx = make_context_with_lang("fr");
        assert_eq!(
            get_taxonomy_url.call(kwargs, &State::new(&ctx)).unwrap().as_str().unwrap(),
            "http://a-website.com/fr/tags/programmation/"
        );

        // and errors if it can't find it
        let kwargs = Kwargs::from([("kind", Value::from("tags")), ("name", Value::from("random"))]);
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
        let library = Arc::new(Library::new(&config));
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
            Kwargs::from([("kind", Value::from("tags")), ("term", Value::from("Programming"))]);
        let ctx = Context::new();
        let res = get_taxonomy_term.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"name".into()).unwrap().as_str().unwrap(), "Programming");
        assert_eq!(res_obj.get(&"slug".into()).unwrap().as_str().unwrap(), "programming");

        // Works with other languages as well (lang in context)
        let kwargs =
            Kwargs::from([("kind", Value::from("tags")), ("term", Value::from("Programmation"))]);
        let ctx = make_context_with_lang("fr");
        let res = get_taxonomy_term.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"name".into()).unwrap().as_str().unwrap(), "Programmation");

        // and errors if it can't find either taxonomy or term
        let kwargs = Kwargs::from([
            ("kind", Value::from("something-else")),
            ("term", Value::from("Programming")),
        ]);
        let ctx = Context::new();
        assert!(get_taxonomy_term.call(kwargs, &State::new(&ctx)).is_err());

        let kwargs =
            Kwargs::from([("kind", Value::from("tags")), ("term", Value::from("something-else"))]);
        let ctx = Context::new();
        assert!(get_taxonomy_term.call(kwargs, &State::new(&ctx)).is_err());
    }
}
