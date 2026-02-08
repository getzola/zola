use serde::Deserialize;
use tera::{Map, Value};

/// Convert a toml::Value to a tera Value
fn toml_value_to_tera(val: toml::Value) -> Value {
    match val {
        toml::Value::String(s) => Value::from(s),
        toml::Value::Integer(i) => Value::from(i),
        toml::Value::Float(f) => Value::from(f),
        toml::Value::Boolean(b) => Value::from(b),
        toml::Value::Datetime(d) => Value::from(d.to_string()),
        toml::Value::Array(arr) => {
            Value::from(arr.into_iter().map(toml_value_to_tera).collect::<Vec<_>>())
        }
        toml::Value::Table(t) => {
            let mut map = Map::new();
            for (k, v) in t {
                map.insert(k.into(), toml_value_to_tera(v));
            }
            Value::from(map)
        }
    }
}

/// Default value for the `extra` field - an empty map wrapped in Value
pub fn default_extra() -> Value {
    Value::from(Map::new())
}

/// Deserializer for the `extra` field in front matter.
/// Deserializes to a tera Value, validates it's a map, and applies date fixing.
pub fn deserialize_extra<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let toml_val = toml::Value::deserialize(deserializer)?;
    match toml_val {
        toml::Value::Table(t) => {
            let mut map = Map::new();
            for (k, v) in t {
                map.insert(k.into(), toml_value_to_tera(v));
            }
            Ok(Value::from(map))
        }
        _ => Err(serde::de::Error::custom("extra must be a map/table")),
    }
}
