extern crate toml;
extern crate serde_json;

use utils::fs::read_file;

use std::path::PathBuf;

use csv::Reader;

use tera::{GlobalFn, Value, from_value, to_value, Result, Map};

static GET_DATA_ARGUMENT_ERROR_MESSAGE: &str = "`load_data`: requires a `path` argument with a string value, being a path to a file";

/// A global function to load data from a data file.
/// Currently the supported formats are json, toml and csv
pub fn make_load_data(content_path: PathBuf) -> GlobalFn {
    Box::new(move |args| -> Result<Value> {
        let path_arg = optional_arg!(
            String,
            args.get("path"),
            GET_DATA_ARGUMENT_ERROR_MESSAGE
        );

        let url_arg = optional_arg!(
            String,
            args.get("url"),
            GET_DATA_ARGUMENT_ERROR_MESSAGE
        );

        if path_arg.is_some() ^ url_arg.is_some() {
            return Err(GET_DATA_ARGUMENT_ERROR_MESSAGE.into());
        }

        let kind_arg = optional_arg!(
            String,
            args.get("kind"),
            "`load_data`: `kind` needs to be an argument with a string value, being one of the supported `load_data` file types (csv, json, toml)"
        );

        let full_path = content_path.join(&path_arg);

        let extension = match full_path.extension() {
            Some(value) => value.to_str().unwrap().to_lowercase(),
            None => return Err(format!("`load_data`: Cannot parse file extension of specified file: {}", path_arg).into())
        };

        let file_kind = kind_arg.unwrap_or(extension);

        let result_value: Result<Value> = match file_kind.as_str() {
            "toml" => load_toml(&full_path),
            "csv" => load_csv(&full_path),
            "json" => load_json(&full_path),
            _ => return Err(format!("'load_data': {} - is an unsupported file kind", file_kind).into())
        };

        result_value
    })
}

/// load/parse a json file from the given path and place it into a
/// tera value
fn load_json(json_path: &PathBuf) -> Result<Value> {

    let content_string: String = read_file(json_path)
        .map_err(|e| format!("`load_data`: error {} loading json file {}", json_path.to_str().unwrap(), e))?;

    let json_content = serde_json::from_str(content_string.as_str()).unwrap();
    let tera_value: Value = json_content;

    return Ok(tera_value);
}

/// load/parse a toml file from the given path, and place it into a
/// tera Value
fn load_toml(toml_path: &PathBuf) -> Result<Value> {
    let content_string: String = read_file(toml_path)
        .map_err(|e| format!("`load_data`: error {} loading toml file {}", toml_path.to_str().unwrap(), e))?;

    let toml_content: toml::Value = toml::from_str(&content_string)
        .map_err(|e| format!("'load_data': {} - {}", toml_path.to_str().unwrap(), e))?;

    to_value(toml_content).map_err(|err| err.into())
}

/// Load/parse a csv file from the given path, and place it into a
/// tera Value.
///
/// An example csv file `example.csv` could be:
/// ```csv
/// Number, Title
/// 1,Gutenberg
/// 2,Printing
/// ```
/// The json value output would be:
/// ```json
/// {
///     "headers": ["Number", "Title"],
///     "records": [
///                     ["1", "Gutenberg"],
///                     ["2", "Printing"]
///                ],
/// }
/// ```
fn load_csv(csv_path: &PathBuf) -> Result<Value> {
    let mut reader = Reader::from_path(csv_path.clone())
        .map_err(|e| format!("'load_data': {} - {}", csv_path.to_str().unwrap(), e))?;

    let mut csv_map = Map::new();

    {
        let hdrs = reader.headers()
            .map_err(|e| format!("'load_data': {} - {} - unable to read CSV header line (line 1) for CSV file", csv_path.to_str().unwrap(), e))?;

        let headers_array = hdrs.iter()
            .map(|v| Value::String(v.to_string()))
            .collect();

        csv_map.insert(String::from("headers"), Value::Array(headers_array));
    }

    {
        let records = reader.records();

        let mut records_array: Vec<Value> = Vec::new();

        for result in records {
            let record = result.unwrap();

            let mut elements_array: Vec<Value> = Vec::new();

            for e in record.into_iter() {
                elements_array.push(Value::String(String::from(e)));
            }

            records_array.push(Value::Array(elements_array));
        }

        csv_map.insert(String::from("records"), Value::Array(records_array));
    }

    let csv_value: Value = Value::Object(csv_map);
    to_value(csv_value).map_err(|err| err.into())
}


#[cfg(test)]
mod tests {
    use super::make_load_data;

    use std::collections::HashMap;
    use std::path::PathBuf;

    use tera::to_value;

    #[test]
    fn can_load_toml()
    {
        let static_fn = make_load_data(PathBuf::from("../utils/test-files"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.toml").unwrap());
        let result = static_fn(args.clone()).unwrap();

        //TOML does not load in order, and also dates are not returned as strings, but
        //rather as another object with a key and value
        assert_eq!(result, json!({
            "category": {
                "date": {
                    "$__toml_private_datetime": "1979-05-27T07:32:00Z"
                },
                "key": "value"
            },
        }));
    }

    #[test]
    fn can_load_csv()
    {
        let static_fn = make_load_data(PathBuf::from("../utils/test-files"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.csv").unwrap());
        let result = static_fn(args.clone()).unwrap();

        assert_eq!(result, json!({
            "headers": ["Number", "Title"],
            "records": [
                            ["1", "Gutenberg"],
                            ["2", "Printing"]
                        ],
        }))
    }

    #[test]
    fn can_load_json()
    {
        let static_fn = make_load_data(PathBuf::from("../utils/test-files"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.json").unwrap());
        let result = static_fn(args.clone()).unwrap();

        assert_eq!(result, json!({
            "key": "value",
            "array": [1, 2, 3],
            "subpackage": {
                "subkey": 5
            }
        }))
    }
}
