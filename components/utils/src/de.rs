use serde::{Deserialize, Deserializer};
use serde_derive::Deserialize;
use tera::{Map, Value};

/// Used as an attribute when we want to convert from TOML to a string date
/// If a TOML datetime isn't present, it will accept a string and push it through
/// TOML's date time parser to ensure only valid dates are accepted.
/// Inspired by this proposal: https://github.com/alexcrichton/toml-rs/issues/269
pub fn from_toml_datetime<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use std::str::FromStr;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MaybeDatetime {
        Datetime(toml::value::Datetime),
        String(String),
    }

    match MaybeDatetime::deserialize(deserializer)? {
        MaybeDatetime::Datetime(d) => Ok(Some(d.to_string())),
        MaybeDatetime::String(s) => match toml::value::Datetime::from_str(&s) {
            Ok(d) => Ok(Some(d.to_string())),
            Err(e) => Err(D::Error::custom(e)),
        },
    }
}

/// Returns key/value for a converted date from TOML.
/// If the table itself is the TOML struct, only return its value without the key
fn convert_toml_date(table: Map<String, Value>) -> Value {
    let mut new = Map::new();

    for (k, v) in table {
        if k == "$__toml_private_datetime" {
            return v;
        }

        match v {
            Value::Object(o) => {
                new.insert(k, convert_toml_date(o));
            }
            _ => {
                new.insert(k, v);
            }
        }
    }

    Value::Object(new)
}

/// TOML datetimes will be serialized as a struct but we want the
/// stringified version for json, otherwise they are going to be weird
pub fn fix_toml_dates(table: Map<String, Value>) -> Value {
    let mut new = Map::new();

    for (key, value) in table {
        match value {
            Value::Object(o) => {
                new.insert(key, convert_toml_date(o));
            }
            Value::Array(arr) => {
                let mut new_arr = Vec::with_capacity(arr.len());
                for v in arr {
                    match v {
                        Value::Object(o) => new_arr.push(fix_toml_dates(o)),
                        _ => new_arr.push(v),
                    };
                }
                new.insert(key, Value::Array(new_arr));
            }
            _ => {
                new.insert(key, value);
            }
        }
    }

    Value::Object(new)
}
