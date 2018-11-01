use serde::{Deserialize, Deserializer};
use tera::{Map, Value};
use toml;

/// Used as an attribute when we want to convert from TOML to a string date
pub fn from_toml_datetime<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    toml::value::Datetime::deserialize(deserializer).map(|s| Some(s.to_string()))
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
            Value::Object(mut o) => {
                new.insert(key, convert_toml_date(o));
            }
            _ => {
                new.insert(key, value);
            }
        }
    }

    Value::Object(new)
}
