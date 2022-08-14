use libs::tera::{from_value, to_value, Error, Function as TeraFn, Result, Value};

use config::Config;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Trans {
    config: Config,
}
impl Trans {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}
impl TeraFn for Trans {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let key = required_arg!(String, args.get("key"), "`trans` requires a `key` argument.");
        let lang = optional_arg!(String, args.get("lang"), "`trans`: `lang` must be a string.")
            .unwrap_or_else(|| self.config.default_language.clone());

        let term = self
            .config
            .get_translation(&lang, &key)
            .map_err(|e| Error::chain("Failed to retrieve term translation", e))?;

        Ok(to_value(term).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRANS_CONFIG: &str = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr"

[translations]
title = "Un titre"

[languages]
[languages.en]
[languages.en.translations]
title = "A title" "#;

    #[test]
    fn can_translate_a_string() {
        let config = Config::parse(TRANS_CONFIG).unwrap();
        let static_fn = Trans::new(config);
        let mut args = HashMap::new();

        args.insert("key".to_string(), to_value("title").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "Un titre");

        args.insert("lang".to_string(), to_value("en").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "A title");

        args.insert("lang".to_string(), to_value("fr").unwrap());
        assert_eq!(static_fn.call(&args).unwrap(), "Un titre");
    }

    #[test]
    fn error_on_absent_translation_lang() {
        let mut args = HashMap::new();
        args.insert("lang".to_string(), to_value("absent").unwrap());
        args.insert("key".to_string(), to_value("title").unwrap());

        let config = Config::parse(TRANS_CONFIG).unwrap();
        let error = Trans::new(config).call(&args).unwrap_err();
        assert_eq!("Failed to retrieve term translation", format!("{}", error));
    }

    #[test]
    fn error_on_absent_translation_key() {
        let mut args = HashMap::new();
        args.insert("lang".to_string(), to_value("en").unwrap());
        args.insert("key".to_string(), to_value("absent").unwrap());

        let config = Config::parse(TRANS_CONFIG).unwrap();
        let error = Trans::new(config).call(&args).unwrap_err();
        assert_eq!("Failed to retrieve term translation", format!("{}", error));
    }
}
