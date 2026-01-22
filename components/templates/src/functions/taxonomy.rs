use std::sync::Arc;

use render::RenderCache;
use tera::{Error, Function, Kwargs, State, TeraResult, Value};
use utils::slugs::{SlugifyStrategy, slugify_paths};

#[derive(Debug)]
pub struct GetTaxonomyUrl {
    cache: Arc<RenderCache>,
    default_lang: String,
    slugify: SlugifyStrategy,
}

impl GetTaxonomyUrl {
    pub fn new(default_lang: &str, cache: Arc<RenderCache>, slugify: SlugifyStrategy) -> Self {
        Self { cache, default_lang: default_lang.to_string(), slugify }
    }
}

impl Default for GetTaxonomyUrl {
    fn default() -> Self {
        Self {
            cache: Arc::new(RenderCache::default()),
            default_lang: String::new(),
            slugify: SlugifyStrategy::default(),
        }
    }
}

impl Function<TeraResult<Value>> for GetTaxonomyUrl {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let kind: &str = kwargs.must_get("kind")?;
        let term: &str = match (kwargs.get::<&str>("term")?, kwargs.get::<&str>("name")?) {
            (Some(t), _) => t,
            (None, Some(n)) => {
                log::warn!("`get_taxonomy_url`: `name` argument is deprecated, use `term` instead");
                n
            }
            (None, None) => {
                return Err(Error::message("`get_taxonomy_url` requires a `term` argument"));
            }
        };
        let lang: String = state.get("lang")?.unwrap_or_else(|| self.default_lang.clone());
        let required: bool = kwargs.get("required")?.unwrap_or(true);

        let cached = match (self.cache.get_taxonomy(&lang, &kind), required) {
            (Some(c), _) => c,
            (None, false) => return Ok(Value::null()),
            (None, true) => {
                return Err(Error::message(format!(
                    "`get_taxonomy_url` received an unknown taxonomy as kind: {}",
                    kind
                )));
            }
        };

        let slug = slugify_paths(&term, self.slugify);
        if let Some(t) = cached.terms.get(&slug) {
            if let Some(map) = t.as_map() {
                if let Some(permalink) = map.get(&"permalink".into()).and_then(|v| v.as_str()) {
                    return Ok(Value::from(permalink));
                }
            }
        }

        Err(Error::message(format!(
            "`get_taxonomy_url`: couldn't find `{}` in `{}` taxonomy",
            term, kind
        )))
    }

    fn is_safe(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct GetTaxonomy {
    cache: Arc<RenderCache>,
    default_lang: String,
}

impl GetTaxonomy {
    pub fn new(default_lang: &str, cache: Arc<RenderCache>) -> Self {
        Self { cache, default_lang: default_lang.to_string() }
    }
}

impl Default for GetTaxonomy {
    fn default() -> Self {
        Self { cache: Arc::new(RenderCache::default()), default_lang: String::new() }
    }
}

impl Function<TeraResult<Value>> for GetTaxonomy {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let kind: &str = kwargs.must_get("kind")?;
        let required: bool = kwargs.get("required")?.unwrap_or(true);
        let lang: String = state.get("lang")?.unwrap_or_else(|| self.default_lang.clone());

        match self.cache.get_taxonomy(&lang, &kind) {
            Some(cached) => Ok(cached.value.clone()),
            None if !required => Ok(Value::null()),
            None => Err(Error::message(format!(
                "`get_taxonomy` received an unknown taxonomy as kind: {}",
                kind
            ))),
        }
    }
}

#[derive(Debug)]
pub struct GetTaxonomyTerm {
    cache: Arc<RenderCache>,
    default_lang: String,
    slugify: SlugifyStrategy,
}

impl GetTaxonomyTerm {
    pub fn new(default_lang: &str, cache: Arc<RenderCache>, slugify: SlugifyStrategy) -> Self {
        Self { cache, default_lang: default_lang.to_string(), slugify }
    }
}

impl Default for GetTaxonomyTerm {
    fn default() -> Self {
        Self {
            cache: Arc::new(RenderCache::default()),
            default_lang: String::new(),
            slugify: SlugifyStrategy::default(),
        }
    }
}

impl Function<TeraResult<Value>> for GetTaxonomyTerm {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let kind: &str = kwargs.must_get("kind")?;
        let term: &str = kwargs.must_get("term")?;
        let required: bool = kwargs.get("required")?.unwrap_or(true);
        let lang: String = state.get("lang")?.unwrap_or_else(|| self.default_lang.clone());

        let cached = match (self.cache.get_taxonomy(&lang, &kind), required) {
            (Some(c), _) => c,
            (None, false) => return Ok(Value::null()),
            (None, true) => {
                return Err(Error::message(format!(
                    "`get_taxonomy_term` received an unknown taxonomy as kind: {}",
                    kind
                )));
            }
        };

        let slug = slugify_paths(&term, self.slugify);
        match (cached.terms.get(&slug).cloned(), required) {
            (Some(t), _) => Ok(t),
            (None, false) => Ok(Value::null()),
            (None, true) => Err(Error::message(format!(
                "`get_taxonomy_term` received an unknown term: {}",
                term
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, TaxonomyConfig};
    use content::{Library, Taxonomy, TaxonomyTerm};
    use render::RenderCache;
    use tera::{Context, Kwargs, Tera};
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
        let library = Library::new(&config);
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
        let tera = Tera::default();
        let cache = Arc::new(RenderCache::build(&config, &library, &taxonomies, &tera));
        let get_taxonomy = GetTaxonomy::new(&config.default_language, cache);

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
        let mut config = Config::default_for_test();
        config.slugify.taxonomies = SlugifyStrategy::On;
        let taxo_config = TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        let taxo_config_fr =
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        config.slugify_taxonomies();
        let library = Library::new(&config);
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
        let tera = Tera::default();
        let cache = Arc::new(RenderCache::build(&config, &library, &taxonomies, &tera));
        let get_taxonomy_url =
            GetTaxonomyUrl::new(&config.default_language, cache, config.slugify.taxonomies);

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
        let library = Library::new(&config);
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
        let tera = Tera::default();
        let cache = Arc::new(RenderCache::build(&config, &library, &taxonomies, &tera));
        let get_taxonomy_term =
            GetTaxonomyTerm::new(&config.default_language, cache, config.slugify.taxonomies);

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
