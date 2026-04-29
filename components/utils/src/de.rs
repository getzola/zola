use tera::{Map, Value};

/// Returns key/value for a converted date from TOML.
/// If the table itself is the TOML struct, only return its value without the key
fn convert_toml_date(table: Map) -> Value {
    let mut new = Map::new();

    for (k, v) in table {
        if k.as_str() == Some("$__toml_private_datetime") {
            return v;
        }

        if v.is_map() {
            new.insert(k, convert_toml_date(v.into_map().unwrap()));
        } else {
            new.insert(k, v);
        }
    }

    Value::from(new)
}

/// TOML datetimes will be serialized as a struct but we want the
/// stringified version for our Value, otherwise that's just not going to work
pub fn fix_toml_dates(table: Map) -> Value {
    let mut new = Map::new();

    for (key, value) in table {
        if value.is_map() {
            new.insert(key, convert_toml_date(value.into_map().unwrap()));
        } else if let Some(arr) = value.as_vec() {
            let new_arr: Vec<Value> =
                arr.iter()
                    .map(|v| {
                        if let Some(o) = v.as_map() { fix_toml_dates(o.clone()) } else { v.clone() }
                    })
                    .collect();
            new.insert(key, Value::from(new_arr));
        } else {
            new.insert(key, value);
        }
    }

    Value::from(new)
}
