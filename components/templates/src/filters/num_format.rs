use tera::{Error, Filter, Kwargs, State, TeraResult};

#[derive(Debug)]
pub struct NumFormatFilter {
    default_language: String,
}

impl NumFormatFilter {
    pub fn new<S: Into<String>>(default_language: S) -> Self {
        Self { default_language: default_language.into() }
    }
}

impl Filter<i128, TeraResult<String>> for NumFormatFilter {
    fn call(&self, value: i128, kwargs: Kwargs, _: &State) -> TeraResult<String> {
        use num_format::{Locale, ToFormattedString};

        let locale: Option<String> = kwargs.get("locale")?;
        let locale_str = locale.as_deref().unwrap_or(&self.default_language);

        let locale = Locale::from_name(locale_str).map_err(|_| {
            Error::message(format!(
                "Filter `num_format` was called with an invalid `locale` argument: `{}`.",
                locale_str
            ))
        })?;

        Ok(value.to_formatted_string(&locale))
    }
}

#[cfg(test)]
mod tests {
    use super::NumFormatFilter;
    use tera::{Context, Filter, Kwargs, State, Value};

    #[test]
    fn num_format_filter() {
        let tests = vec![
            (100, "100"),
            (1_000, "1,000"),
            (10_000, "10,000"),
            (100_000, "100,000"),
            (1_000_000, "1,000,000"),
        ];

        let ctx = Context::new();
        let state = State::new(&ctx);

        for (input, expected) in tests {
            let kwargs = Kwargs::from([]);
            let result = NumFormatFilter::new("en").call(input, kwargs, &state);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected);
        }
    }

    #[test]
    fn num_format_filter_with_locale() {
        let tests = vec![
            ("en", 1_000_000, "1,000,000"),
            ("en-IN", 1_000_000, "10,00,000"),
            // Note:
            // U+202F is the "NARROW NO-BREAK SPACE" code point.
            // When displayed to the screen, it looks like a space.
            ("fr", 1_000_000, "1\u{202f}000\u{202f}000"),
        ];

        let ctx = Context::new();
        let state = State::new(&ctx);

        for (locale, input, expected) in tests {
            let kwargs = Kwargs::from([("locale", Value::from(locale))]);
            let result = NumFormatFilter::new("en").call(input, kwargs, &state);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), expected);
        }
    }
}
