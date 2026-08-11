use std::path::Path;

use errors::{Context, Result, bail};
use tera::{Map, Value};

use crate::front_matter::PageFrontMatter;
use crate::front_matter::extra::toml_value_to_tera;
use crate::front_matter::split::RawFrontMatter;

/// Front-matter keys never inherited from the default-language page.
const NEVER_INHERITED: [&str; 3] = ["slug", "path", "aliases"];

/// Parses a raw front matter into a tera map; TOML datetimes become strings.
fn raw_to_map(raw: &RawFrontMatter) -> Result<Map> {
    match raw.deserialize()? {
        toml::Value::Table(table) => {
            let mut map = Map::new();
            for (key, value) in table {
                map.insert(key.into(), toml_value_to_tera(value));
            }
            Ok(map)
        }
        _ => bail!("Front matter is not a table"),
    }
}

/// Recursive table merge: `overlay` wins; arrays and scalars replace wholly.
fn merge_maps(base: &Map, overlay: &Map) -> Map {
    let mut merged = base.clone();
    for (key, value) in overlay {
        let value = match (merged.get(key), value) {
            (Some(base_value), overlay_value)
                if base_value.as_map().is_some() && overlay_value.as_map().is_some() =>
            {
                Value::from(merge_maps(
                    base_value.as_map().unwrap(),
                    overlay_value.as_map().unwrap(),
                ))
            }
            _ => value.clone(),
        };
        merged.insert(key.clone(), value);
    }
    merged
}

/// Merge the translation's raw front matter over the default-language one and parse the result.
pub fn merge_inherited_raw(
    default: &RawFrontMatter,
    translated: &RawFrontMatter,
    translated_path: &Path,
) -> Result<PageFrontMatter> {
    let context = || {
        format!(
            "Error when parsing inherited front matter of page `{}`",
            translated_path.to_string_lossy()
        )
    };
    let mut base = raw_to_map(default).with_context(context)?;
    base.retain(|key, _| !NEVER_INHERITED.contains(&key.as_str().unwrap_or_default()));
    let merged = merge_maps(&base, &raw_to_map(translated).with_context(context)?);
    PageFrontMatter::parse_value(Value::from(merged)).with_context(context)
}

#[cfg(test)]
mod tests {
    use super::merge_inherited_raw;
    use crate::front_matter::split::RawFrontMatter;
    use std::path::Path;
    use time::macros::datetime;

    const P: &str = "content/blog/post.fr.md";

    #[test]
    fn missing_keys_inherit_translated_win() {
        let default = RawFrontMatter::Toml("title = \"Default\"\nweight = 10\n");
        let translated = RawFrontMatter::Toml("title = \"Traduit\"\n");
        let meta = merge_inherited_raw(&default, &translated, Path::new(P)).unwrap();
        assert_eq!(meta.title.as_deref(), Some("Traduit"));
        assert_eq!(meta.weight, Some(10));
    }

    #[test]
    fn tables_merge_recursively_arrays_replace() {
        let default = RawFrontMatter::Toml(
            "[extra]\ncover = \"cover.png\"\nflavor = \"default\"\ntypes = [\"a\", \"b\"]\n\
             [extra.abilities]\n\"Fire walk\" = \"yes\"\n\"Swim\" = \"no\"\n",
        );
        let translated = RawFrontMatter::Toml(
            "[extra]\nflavor = \"traduit\"\ntypes = []\n[extra.abilities]\n\"Fire walk\" = \"oui\"\n",
        );
        let meta = merge_inherited_raw(&default, &translated, Path::new(P)).unwrap();
        let extra = meta.extra.as_map().unwrap();
        assert_eq!(extra.get(&"cover".into()).unwrap(), &tera::Value::from("cover.png"));
        assert_eq!(extra.get(&"flavor".into()).unwrap(), &tera::Value::from("traduit"));
        assert!(extra.get(&"types".into()).unwrap().as_array().unwrap().is_empty());
        let abilities = extra.get(&"abilities".into()).unwrap().as_map().unwrap();
        assert_eq!(abilities.get(&"Fire walk".into()).unwrap(), &tera::Value::from("oui"));
        assert_eq!(abilities.get(&"Swim".into()).unwrap(), &tera::Value::from("no"));
    }

    #[test]
    fn slug_path_aliases_are_never_inherited() {
        let default = RawFrontMatter::Toml(
            "slug = \"default-slug\"\npath = \"/somewhere\"\naliases = [\"/old\"]\n",
        );
        let translated = RawFrontMatter::Toml("title = \"x\"\n");
        let meta = merge_inherited_raw(&default, &translated, Path::new(P)).unwrap();
        assert_eq!(meta.slug, None);
        assert_eq!(meta.path, None);
        assert_eq!(meta.aliases, Vec::<String>::new());
    }

    #[test]
    fn bools_and_dates_inherit_and_derive() {
        let default = RawFrontMatter::Toml("draft = true\ndate = 2002-10-12\n");
        let translated = RawFrontMatter::Toml("title = \"x\"\n");
        let meta = merge_inherited_raw(&default, &translated, Path::new(P)).unwrap();
        assert!(meta.draft);
        assert_eq!(meta.datetime.unwrap(), datetime!(2002 - 10 - 12 0:00 UTC));
    }

    #[test]
    fn explicit_bool_in_translation_wins() {
        let default = RawFrontMatter::Toml("draft = true\n");
        let translated = RawFrontMatter::Toml("draft = false\n");
        let meta = merge_inherited_raw(&default, &translated, Path::new(P)).unwrap();
        assert!(!meta.draft);
    }

    #[test]
    fn cross_format_merge() {
        let default = RawFrontMatter::Yaml("weight: 42\ndate: 2002-10-12\n");
        let translated = RawFrontMatter::Toml("title = \"x\"\n");
        let meta = merge_inherited_raw(&default, &translated, Path::new(P)).unwrap();
        assert_eq!(meta.weight, Some(42));
        assert_eq!(meta.datetime.unwrap(), datetime!(2002 - 10 - 12 0:00 UTC));
    }

    #[test]
    fn taxonomies_merge_per_key() {
        let default = RawFrontMatter::Toml("[taxonomies]\ntags = [\"hello\"]\nrelated = [\"a\"]\n");
        let translated = RawFrontMatter::Toml("[taxonomies]\ntags = [\"bonjour\"]\n");
        let meta = merge_inherited_raw(&default, &translated, Path::new(P)).unwrap();
        assert_eq!(meta.taxonomies["tags"], vec!["bonjour".to_string()]);
        assert_eq!(meta.taxonomies["related"], vec!["a".to_string()]);
    }

    #[test]
    fn invalid_inherited_value_errors_with_translated_path() {
        let default = RawFrontMatter::Toml("date = \"not-a-date\"\n");
        let translated = RawFrontMatter::Toml("title = \"x\"\n");
        let err = merge_inherited_raw(&default, &translated, Path::new(P)).unwrap_err();
        assert!(format!("{err:?}").contains(P));
    }

    #[test]
    fn null_values_error_with_translated_path() {
        // toml::Value has no null arm, so explicit YAML nulls fail during normalization
        let default = RawFrontMatter::Yaml("title: Hello\ndescription:\n");
        let translated = RawFrontMatter::Toml("title = \"x\"\n");
        let err = merge_inherited_raw(&default, &translated, Path::new(P)).unwrap_err();
        assert!(format!("{err:?}").contains(P));
    }
}
