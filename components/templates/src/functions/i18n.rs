use config::Config;
use tera::{Error, Function, Kwargs, State, TeraResult};

#[derive(Debug)]
pub struct Trans {
    config: Config,
}

impl Trans {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl Default for Trans {
    fn default() -> Self {
        Self { config: Config::default() }
    }
}

impl Function<TeraResult<String>> for Trans {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<String> {
        let key: String = kwargs.must_get("key")?;
        let lang: String = kwargs
            .get("lang")?
            .or(state.get("lang")?)
            .unwrap_or_else(|| self.config.default_language.clone());

        let term = self
            .config
            .get_translation(&lang, &key)
            .map_err(|e| Error::chain("Failed to retrieve term translation", e))?;

        Ok(term)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::{Context, Kwargs, Value};

    const TRANS_CONFIG: &str = r#"
base_url = "https://remplace-par-ton-url.fr"
default_language = "fr"

[translations]
title = "Un titre"

[languages]
[languages.en]
[languages.en.translations]
title = "A title" "#;

    fn make_context_with_lang(lang: &str) -> Context {
        let mut ctx = Context::new();
        ctx.insert("lang", &lang);
        ctx
    }

    #[test]
    fn can_translate_a_string() {
        let config = Config::parse(TRANS_CONFIG).unwrap();
        let trans = Trans::new(config);

        // Default lang (fr)
        let kwargs = Kwargs::from([("key", Value::from("title"))]);
        let ctx = Context::new();
        assert_eq!(trans.call(kwargs, &State::new(&ctx)).unwrap(), "Un titre");

        // Explicit lang in kwargs
        let kwargs = Kwargs::from([("key", Value::from("title")), ("lang", Value::from("en"))]);
        let ctx = Context::new();
        assert_eq!(trans.call(kwargs, &State::new(&ctx)).unwrap(), "A title");

        // Lang from context
        let kwargs = Kwargs::from([("key", Value::from("title"))]);
        let ctx = make_context_with_lang("en");
        assert_eq!(trans.call(kwargs, &State::new(&ctx)).unwrap(), "A title");

        // Explicit lang in kwargs overrides context
        let kwargs = Kwargs::from([("key", Value::from("title")), ("lang", Value::from("fr"))]);
        let ctx = make_context_with_lang("en");
        assert_eq!(trans.call(kwargs, &State::new(&ctx)).unwrap(), "Un titre");
    }

    #[test]
    fn error_on_absent_translation_lang() {
        let config = Config::parse(TRANS_CONFIG).unwrap();
        let trans = Trans::new(config);

        let kwargs = Kwargs::from([("key", Value::from("title")), ("lang", Value::from("absent"))]);
        let ctx = Context::new();
        let error = trans.call(kwargs, &State::new(&ctx)).unwrap_err();
        assert!(error.to_string().contains("Failed to retrieve term translation"));
    }

    #[test]
    fn error_on_absent_translation_key() {
        let config = Config::parse(TRANS_CONFIG).unwrap();
        let trans = Trans::new(config);

        let kwargs = Kwargs::from([("key", Value::from("absent")), ("lang", Value::from("en"))]);
        let ctx = Context::new();
        let error = trans.call(kwargs, &State::new(&ctx)).unwrap_err();
        assert!(error.to_string().contains("Failed to retrieve term translation"));
    }
}
