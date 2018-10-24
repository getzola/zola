extern crate toml;
extern crate serde_json;

use utils::fs::{read_file, is_path_in_directory, get_file_time};

use crypto_hash::{Algorithm, hex_digest};
use chrono::{DateTime, Utc};
use reqwest::{Client, header};
use url::Url;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};


use csv::Reader;
use std::collections::HashMap;
use tera::{GlobalFn, Value, from_value, to_value, Result, Map};

static GET_DATA_ARGUMENT_ERROR_MESSAGE: &str = "`load_data`: requires EITHER a `path` or `url` argument";

enum DataSource {
    Url(Url),
    Path(PathBuf)
}


fn get_cache_key(data_source: &DataSource, format: &String) -> String {
    let content_based_data = match data_source {
        DataSource::Url(url) => url.clone().into_string(),
        DataSource::Path(path) => {
            let file_time = get_file_time(&path).expect("get file time");
            let file_datetime: DateTime<Utc> = DateTime::from(file_time);
            format!("{}{}", file_datetime.timestamp_millis().to_string(), path.display())
        }
    };
    return hex_digest(Algorithm::MD5, format!("{}{}", format, content_based_data).as_bytes());
}


fn get_data_from_args(content_path: &PathBuf, args: &HashMap<String, Value>) -> Result<DataSource> {
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

    if path_arg.is_some() && url_arg.is_some() {
        return Err(GET_DATA_ARGUMENT_ERROR_MESSAGE.into());
    }

    if let Some(path) = path_arg {
        let full_path = content_path.join(path);
        if !full_path.exists() {
            return Err(format!("{} doesn't exist", full_path.display()).into());
        }
        return Ok(DataSource::Path(full_path));
    }

    if let Some(url) = url_arg {
        return Url::parse(&url).map(|parsed_url| DataSource::Url(parsed_url)).map_err(|e| format!("Failed to parse {} as url: {}", url, e).into());
    }

    return Err(GET_DATA_ARGUMENT_ERROR_MESSAGE.into());
}

fn read_data_file(base_path: &PathBuf, full_path: PathBuf) -> Result<String> {
    if !is_path_in_directory(&base_path, &full_path).map_err(|e| format!("Failed to read data file {}: {}", full_path.display(), e))? {
        return Err(format!("{} is not inside the base site directory {}", full_path.display(), base_path.display()).into());
    }
    return read_file(&full_path)
        .map_err(|e| format!("`load_data`: error {} loading file {}", full_path.to_str().unwrap(), e).into());
}

fn get_output_format_from_args(args: &HashMap<String, Value>, data_source: &DataSource) -> Result<String> {
    let format_arg = optional_arg!(
        String,
        args.get("format"),
        "`load_data`: `format` needs to be an argument with a string value, being one of the supported `load_data` file types (csv, json, toml)"
    );

    if let Some(format) = format_arg {
        return Ok(format);
    }
    return match data_source {
        DataSource::Path(path) => path.extension().map(|extension| extension.to_str().unwrap().to_string()).ok_or(format!("Could not determine format for {} from extension", path.display()).into()),
        _ => Ok(String::from("plain"))
    }
}


/// A global function to load data from a data file.
/// Currently the supported formats are json, toml and csv
pub fn make_load_data(content_path: PathBuf, base_path: PathBuf) -> GlobalFn {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, "zola".parse().unwrap());
    let client = Arc::new(Mutex::new(Client::builder().build().expect("reqwest client build")));
    let result_cache: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));
    Box::new(move |args| -> Result<Value> {
        let data_source = get_data_from_args(&content_path, &args)?;

        let file_format = get_output_format_from_args(&args, &data_source)?;

        let cache_key = get_cache_key(&data_source, &file_format);

        let mut cache = result_cache.lock().expect("result cache lock");
        let response_client = client.lock().expect("response client lock");
        if let Some(cached_result) = cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }

        let data = match data_source {
            DataSource::Path(path) => read_data_file(&base_path, path),
            DataSource::Url(url) => {
                let mut response = response_client.get(url.as_str()).send().and_then(|res| res.error_for_status()).map_err(|e| format!("Failed to request {}: {}", url, e.status().expect("response status")))?;
                response.text().map_err(|e| format!("Failed to parse response from {}: {:?}", url, e).into())
            },
        }?;

        let result_value: Result<Value> = match file_format.as_str() {
            "toml" => load_toml(data),
            "csv" => load_csv(data),
            "json" => load_json(data),
            "plain" => to_value(data).map_err(|e| e.into()),
            format => return Err(format!("'load_data': {} is an unsupported file format", format).into())
        };

        if let Ok(data_result) = &result_value {
            cache.insert(cache_key, data_result.clone());
        }

        result_value
    })
}

/// load/parse a json file from the given path and place it into a
/// tera value
fn load_json(json_data: String) -> Result<Value> {
    let json_content: Value = serde_json::from_str(json_data.as_str()).map_err(|e| format!("{:?}", e))?;
    return Ok(json_content);
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
    use super::{make_load_data, get_cache_key, DataSource};

    use std::collections::HashMap;
    use std::path::PathBuf;

    use tera::to_value;

    fn get_test_file(filename: &str) -> PathBuf {
        let test_files = PathBuf::from("../utils/test-files").canonicalize().unwrap();
        return test_files.join(filename);
    }

    #[test]
    fn fails_when_missing_file() {
        let static_fn = make_load_data(PathBuf::from("../utils/test-files"), PathBuf::from("../utils"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("../../../READMEE.md").unwrap());
        let result = static_fn(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().description(), "../utils/test-files/../../../READMEE.md doesn't exist");
    }

    #[test]
    fn cant_load_outside_content_dir() {
        let static_fn = make_load_data(PathBuf::from("../utils/test-files"), PathBuf::from("../utils"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("../../../README.md").unwrap());
        let result = static_fn(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().description(), "../utils/test-files/../../../README.md is not inside the base site directory ../utils");
    }

    #[test]
    fn calculates_cache_key() {
        let cache_key = get_cache_key(&DataSource::Path(get_test_file("test.toml")), &String::from("toml"));
        assert_eq!(cache_key, "830dc6839f945d93e86fec2cc6ca0ea1");
    }

    #[test]
    fn different_cache_key_per_filename() {
        let toml_cache_key = get_cache_key(&DataSource::Path(get_test_file("test.toml")), &String::from("toml"));
        let json_cache_key = get_cache_key(&DataSource::Path(get_test_file("test.json")), &String::from("toml"));
        assert_ne!(toml_cache_key, json_cache_key);
    }

    #[test]
    fn different_cache_key_per_format() {
        let toml_cache_key = get_cache_key(&DataSource::Path(get_test_file("test.toml")), &String::from("toml"));
        let json_cache_key = get_cache_key(&DataSource::Path(get_test_file("test.toml")), &String::from("json"));
        assert_ne!(toml_cache_key, json_cache_key);
    }

    #[test]
    fn can_load_remote_data() {
        let static_fn = make_load_data(PathBuf::new(), PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value("https://api.github.com/repos/getzola/zola").unwrap());
        args.insert("format".to_string(), to_value("json").unwrap());
        let result = static_fn(args).unwrap();
        assert_eq!(result.get("id").unwrap(), &to_value(75688610).unwrap());
    }

    #[test]
    fn fails_when_request_404s() {
        let static_fn = make_load_data(PathBuf::new(), PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value("https://api.github.com/repos/getzola/non-existent-zola-test-repo").unwrap());
        args.insert("format".to_string(), to_value("json").unwrap());
        let result = static_fn(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().description(), "Failed to request https://api.github.com/repos/getzola/non-existent-zola-test-repo: 404 Not Found");
    }

    #[test]
    fn can_load_toml()
    {
        let static_fn = make_load_data(PathBuf::from("../utils/test-files"), PathBuf::from("../utils/test-files"));
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
        let static_fn = make_load_data(PathBuf::from("../utils/test-files"), PathBuf::from("../utils/test-files"));
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
        let static_fn = make_load_data(PathBuf::from("../utils/test-files"), PathBuf::from("../utils/test-files"));
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
