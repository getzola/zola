use content::{Library, Taxonomy};
use libs::tera::{from_value, to_value, Function as TeraFn, Result, Value};
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
        match library.pages.get(&full_path) {
            Some(p) => Ok(to_value(p.serialize(&library)).unwrap()),
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

        match library.sections.get(&full_path) {
            Some(s) => {
                if metadata_only {
                    Ok(to_value(s.serialize_basic(&library)).unwrap())
                } else {
                    Ok(to_value(s.serialize(&library)).unwrap())
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

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, TaxonomyConfig};
    use content::TaxonomyItem;

    #[test]
    fn can_get_taxonomy() {
        let mut config = Config::default();
        config.slugify.taxonomies = SlugifyStrategy::On;
        let taxo_config = TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        let taxo_config_fr =
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() };
        let library = Arc::new(RwLock::new(Library::new()));
        let tag = TaxonomyItem::new("Programming", &config.default_language, "tags", &[], &config);
        let tag_fr = TaxonomyItem::new("Programmation", "fr", "tags", &[], &config);
        let tags = Taxonomy {
            kind: taxo_config,
            lang: config.default_language.clone(),
            slug: "tags".to_string(),
            permalink: "/tags/".to_string(),
            items: vec![tag],
        };
        let tags_fr = Taxonomy {
            kind: taxo_config_fr,
            lang: "fr".to_owned(),
            slug: "tags".to_string(),
            permalink: "/fr/tags/".to_string(),
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
        let tag = TaxonomyItem::new("Programming", &config.default_language, "tags", &[], &config);
        let tag_fr = TaxonomyItem::new("Programmation", "fr", "tags", &[], &config);
        let tags = Taxonomy {
            kind: taxo_config,
            lang: config.default_language.clone(),
            slug: "tags".to_string(),
            permalink: "/tags/".to_string(),
            items: vec![tag],
        };
        let tags_fr = Taxonomy {
            kind: taxo_config_fr,
            lang: "fr".to_owned(),
            slug: "tags".to_string(),
            permalink: "/fr/tags/".to_string(),
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
}
