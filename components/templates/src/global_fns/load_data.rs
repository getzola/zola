extern crate toml;
extern crate serde_json;

use utils::fs::read_file;

use std::path::PathBuf;

use csv::Reader;
use std::collections::HashMap;
use tera::{GlobalFn, Value, from_value, to_value, Result, Map};

static GET_DATA_ARGUMENT_ERROR_MESSAGE: &str = "`load_data`: requires a `path` argument with a string value, being a path to a file";

enum ProvidedArgument {
    URL(String),
    PATH(String)
}

fn get_data_from_args(args: &HashMap<String, Value>) -> Result<ProvidedArgument> {
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

    if let Some(path) = path_arg {
        return Ok(ProvidedArgument::PATH(path));
    }
    else if let Some(url) = url_arg {
        return Ok(ProvidedArgument::URL(url));
    }

    return Err(GET_DATA_ARGUMENT_ERROR_MESSAGE.into());
}

pub fn read_data_file(content_path: &PathBuf, path_arg: String) -> Result<String> {
    let full_path = content_path.join(&path_arg);
    return read_file(&full_path)
        .map_err(|e| format!("`load_data`: error {} loading file {}", full_path.to_str().unwrap(), e).into());
}

/// A global function to load data from a data file.
/// Currently the supported formats are json, toml and csv
pub fn make_load_data(content_path: PathBuf) -> GlobalFn {
    Box::new(move |args| -> Result<Value> {
        let kind_arg = optional_arg!(
            String,
            args.get("kind"),
            "`load_data`: `kind` needs to be an argument with a string value, being one of the supported `load_data` file types (csv, json, toml)"
        );

        let provided_argument = get_data_from_args(&args);

        let data = match provided_argument {
            Ok(ProvidedArgument::PATH(path)) => read_data_file(&content_path, path),
            Ok(ProvidedArgument::URL(_url)) => Ok(String::from("test")),
            Err(err) => return Err(err)
        }?;

        let file_kind = kind_arg.unwrap_or(String::from("plain"));

        let result_value: Result<Value> = match file_kind.as_str() {
            "toml" => load_toml(data),
            "csv" => load_csv(data),
            "json" => load_json(data),
            "plain" => Ok(to_value(data).unwrap()),
            kind => return Err(format!("'load_data': {} is an unsupported file kind", kind).into())
        };

        result_value
    })
}

/// load/parse a json file from the given path and place it into a
/// tera value
fn load_json(json_data: String) -> Result<Value> {
    let json_content = serde_json::from_str(json_data.as_str()).unwrap();
    let tera_value: Value = json_content;

    return Ok(tera_value);
}

/// load/parse a toml file from the given path, and place it into a
/// tera Value
fn load_toml(toml_data: String) -> Result<Value> {
    let toml_content: toml::Value = toml::from_str(&toml_data).map_err(|e| format!("{:?}", e))?;

    to_value(toml_content).map_err(|e| e.into())
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
fn load_csv(csv_data: String) -> Result<Value> {
    let mut reader = Reader::from_reader(csv_data.as_bytes());

    let mut csv_map = Map::new();

    {
        let hdrs = reader.headers()
            .map_err(|e| format!("'load_data': {} - unable to read CSV header line (line 1) for CSV file", e))?;

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
