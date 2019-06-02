extern crate serde_json;
extern crate toml;

use utils::de::fix_toml_dates;
use utils::fs::{get_file_time, is_path_in_directory, read_file};

use reqwest::{header, Client};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use url::Url;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use csv::Reader;
use std::collections::HashMap;
use tera::{from_value, to_value, Error, Function as TeraFn, Map, Result, Value};

static GET_DATA_ARGUMENT_ERROR_MESSAGE: &str =
    "`load_data`: requires EITHER a `path` or `url` argument";

enum DataSource {
    Url(Url),
    Path(PathBuf),
}

#[derive(Debug)]
enum OutputFormat {
    Toml,
    Json,
    Csv,
    Plain,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Hash for OutputFormat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl FromStr for OutputFormat {
    type Err = Error;

    fn from_str(output_format: &str) -> Result<Self> {
        match output_format {
            "toml" => Ok(OutputFormat::Toml),
            "csv" => Ok(OutputFormat::Csv),
            "json" => Ok(OutputFormat::Json),
            "plain" => Ok(OutputFormat::Plain),
            format => Err(format!("Unknown output format {}", format).into()),
        }
    }
}

impl OutputFormat {
    fn as_accept_header(&self) -> header::HeaderValue {
        header::HeaderValue::from_static(match self {
            OutputFormat::Json => "application/json",
            OutputFormat::Csv => "text/csv",
            OutputFormat::Toml => "application/toml",
            OutputFormat::Plain => "text/plain",
        })
    }
}

impl DataSource {
    fn from_args(
        path_arg: Option<String>,
        url_arg: Option<String>,
        content_path: &PathBuf,
    ) -> Result<Self> {
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
            return Url::parse(&url)
                .map(DataSource::Url)
                .map_err(|e| format!("Failed to parse {} as url: {}", url, e).into());
        }

        Err(GET_DATA_ARGUMENT_ERROR_MESSAGE.into())
    }

    fn get_cache_key(&self, format: &OutputFormat) -> u64 {
        let mut hasher = DefaultHasher::new();
        format.hash(&mut hasher);
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Hash for DataSource {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DataSource::Url(url) => url.hash(state),
            DataSource::Path(path) => {
                path.hash(state);
                get_file_time(&path).expect("get file time").hash(state);
            }
        };
    }
}

fn get_data_source_from_args(
    content_path: &PathBuf,
    args: &HashMap<String, Value>,
) -> Result<DataSource> {
    let path_arg = optional_arg!(String, args.get("path"), GET_DATA_ARGUMENT_ERROR_MESSAGE);
    let url_arg = optional_arg!(String, args.get("url"), GET_DATA_ARGUMENT_ERROR_MESSAGE);

    DataSource::from_args(path_arg, url_arg, content_path)
}

fn read_data_file(base_path: &PathBuf, full_path: PathBuf) -> Result<String> {
    if !is_path_in_directory(&base_path, &full_path)
        .map_err(|e| format!("Failed to read data file {}: {}", full_path.display(), e))?
    {
        return Err(format!(
            "{} is not inside the base site directory {}",
            full_path.display(),
            base_path.display()
        )
        .into());
    }
    read_file(&full_path).map_err(|e| {
        format!("`load_data`: error {} loading file {}", full_path.to_str().unwrap(), e).into()
    })
}

fn get_output_format_from_args(
    args: &HashMap<String, Value>,
    data_source: &DataSource,
) -> Result<OutputFormat> {
    let format_arg = optional_arg!(
        String,
        args.get("format"),
        "`load_data`: `format` needs to be an argument with a string value, being one of the supported `load_data` file types (csv, json, toml, plain)"
    );

    if let Some(format) = format_arg {
        if format == "plain" {
            return Ok(OutputFormat::Plain);
        }
        return OutputFormat::from_str(&format);
    }

    let from_extension = if let DataSource::Path(path) = data_source {
        path.extension().map(|extension| extension.to_str().unwrap()).unwrap_or_else(|| "plain")
    } else {
        "plain"
    };

    // Always default to Plain if we don't know what it is
    OutputFormat::from_str(from_extension).or_else(|_| Ok(OutputFormat::Plain))
}

/// A Tera function to load data from a file or from a URL
/// Currently the supported formats are json, toml, csv and plain text
#[derive(Debug)]
pub struct LoadData {
    base_path: PathBuf,
    client: Arc<Mutex<Client>>,
    result_cache: Arc<Mutex<HashMap<u64, Value>>>,
}
impl LoadData {
    pub fn new(base_path: PathBuf) -> Self {
        let client = Arc::new(Mutex::new(Client::builder().build().expect("reqwest client build")));
        let result_cache = Arc::new(Mutex::new(HashMap::new()));
        Self { base_path, client, result_cache }
    }
}

impl TeraFn for LoadData {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let data_source = get_data_source_from_args(&self.base_path, &args)?;
        let file_format = get_output_format_from_args(&args, &data_source)?;
        let cache_key = data_source.get_cache_key(&file_format);

        let mut cache = self.result_cache.lock().expect("result cache lock");
        let response_client = self.client.lock().expect("response client lock");
        if let Some(cached_result) = cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }

        let data = match data_source {
            DataSource::Path(path) => read_data_file(&self.base_path, path),
            DataSource::Url(url) => {
                let mut response = response_client
                    .get(url.as_str())
                    .header(header::ACCEPT, file_format.as_accept_header())
                    .send()
                    .and_then(|res| res.error_for_status())
                    .map_err(|e| {
                        format!(
                            "Failed to request {}: {}",
                            url,
                            e.status().expect("response status")
                        )
                    })?;
                response
                    .text()
                    .map_err(|e| format!("Failed to parse response from {}: {:?}", url, e).into())
            }
        }?;

        let result_value: Result<Value> = match file_format {
            OutputFormat::Toml => load_toml(data),
            OutputFormat::Csv => load_csv(data),
            OutputFormat::Json => load_json(data),
            OutputFormat::Plain => to_value(data).map_err(|e| e.into()),
        };

        if let Ok(data_result) = &result_value {
            cache.insert(cache_key, data_result.clone());
        }

        result_value
    }
}

/// Parse a JSON string and convert it to a Tera Value
fn load_json(json_data: String) -> Result<Value> {
    let json_content: Value =
        serde_json::from_str(json_data.as_str()).map_err(|e| format!("{:?}", e))?;
    Ok(json_content)
}

/// Parse a TOML string and convert it to a Tera Value
fn load_toml(toml_data: String) -> Result<Value> {
    let toml_content: toml::Value = toml::from_str(&toml_data).map_err(|e| format!("{:?}", e))?;
    let toml_value = to_value(toml_content).expect("Got invalid JSON that was valid TOML somehow");

    match toml_value {
        Value::Object(m) => Ok(fix_toml_dates(m)),
        _ => unreachable!("Loaded something other than a TOML object"),
    }
}

/// Parse a CSV string and convert it to a Tera Value
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
        let hdrs = reader.headers().map_err(|e| {
            format!("'load_data': {} - unable to read CSV header line (line 1) for CSV file", e)
        })?;

        let headers_array = hdrs.iter().map(|v| Value::String(v.to_string())).collect();

        csv_map.insert(String::from("headers"), Value::Array(headers_array));
    }

    {
        let records = reader.records();

        let mut records_array: Vec<Value> = Vec::new();

        for result in records {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    return Err(tera::Error::chain(
                        String::from("Error encountered when parsing csv records"),
                        e,
                    ));
                }
            };

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
    use super::{DataSource, LoadData, OutputFormat};

    use std::collections::HashMap;
    use std::path::PathBuf;

    use tera::{to_value, Function};

    fn get_test_file(filename: &str) -> PathBuf {
        let test_files = PathBuf::from("../utils/test-files").canonicalize().unwrap();
        return test_files.join(filename);
    }

    #[test]
    fn fails_when_missing_file() {
        let static_fn = LoadData::new(PathBuf::from("../utils"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("../../../READMEE.md").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("READMEE.md doesn't exist"));
    }

    #[test]
    fn cant_load_outside_content_dir() {
        let static_fn = LoadData::new(PathBuf::from(PathBuf::from("../utils")));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("../../README.md").unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("README.md is not inside the base site directory"));
    }

    #[test]
    fn calculates_cache_key_for_path() {
        // We can't test against a fixed value, due to the fact the cache key is built from the absolute path
        let cache_key =
            DataSource::Path(get_test_file("test.toml")).get_cache_key(&OutputFormat::Toml);
        let cache_key_2 =
            DataSource::Path(get_test_file("test.toml")).get_cache_key(&OutputFormat::Toml);
        assert_eq!(cache_key, cache_key_2);
    }

    #[test]
    fn calculates_cache_key_for_url() {
        let cache_key =
            DataSource::Url("https://api.github.com/repos/getzola/zola".parse().unwrap())
                .get_cache_key(&OutputFormat::Plain);
        assert_eq!(cache_key, 8916756616423791754);
    }

    #[test]
    fn different_cache_key_per_filename() {
        let toml_cache_key =
            DataSource::Path(get_test_file("test.toml")).get_cache_key(&OutputFormat::Toml);
        let json_cache_key =
            DataSource::Path(get_test_file("test.json")).get_cache_key(&OutputFormat::Toml);
        assert_ne!(toml_cache_key, json_cache_key);
    }

    #[test]
    fn different_cache_key_per_format() {
        let toml_cache_key =
            DataSource::Path(get_test_file("test.toml")).get_cache_key(&OutputFormat::Toml);
        let json_cache_key =
            DataSource::Path(get_test_file("test.toml")).get_cache_key(&OutputFormat::Json);
        assert_ne!(toml_cache_key, json_cache_key);
    }

    #[test]
    fn can_load_remote_data() {
        let static_fn = LoadData::new(PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value("https://httpbin.org/json").unwrap());
        args.insert("format".to_string(), to_value("json").unwrap());
        let result = static_fn.call(&args).unwrap();
        assert_eq!(
            result.get("slideshow").unwrap().get("title").unwrap(),
            &to_value("Sample Slide Show").unwrap()
        );
    }

    #[test]
    fn fails_when_request_404s() {
        let static_fn = LoadData::new(PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value("https://httpbin.org/status/404/").unwrap());
        args.insert("format".to_string(), to_value("json").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Failed to request https://httpbin.org/status/404/: 404 Not Found"
        );
    }

    #[test]
    fn can_load_toml() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.toml").unwrap());
        let result = static_fn.call(&args.clone()).unwrap();

        // TOML does not load in order
        assert_eq!(
            result,
            json!({
                "category": {
                    "date": "1979-05-27T07:32:00Z",
                    "lt1": "07:32:00",
                    "key": "value"
                },
            })
        );
    }

    #[test]
    fn unknown_extension_defaults_to_plain() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.css").unwrap());
        let result = static_fn.call(&args.clone()).unwrap();

        assert_eq!(result, ".hello {}\n",);
    }

    #[test]
    fn can_override_known_extension_with_format() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.csv").unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        let result = static_fn.call(&args.clone()).unwrap();

        assert_eq!(result, "Number,Title\n1,Gutenberg\n2,Printing",);
    }

    #[test]
    fn will_use_format_on_unknown_extension() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.css").unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        let result = static_fn.call(&args.clone()).unwrap();

        assert_eq!(result, ".hello {}\n",);
    }

    #[test]
    fn can_load_csv() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.csv").unwrap());
        let result = static_fn.call(&args.clone()).unwrap();

        assert_eq!(
            result,
            json!({
                "headers": ["Number", "Title"],
                "records": [
                                ["1", "Gutenberg"],
                                ["2", "Printing"]
                            ],
            })
        )
    }

    // Test points to bad csv file with uneven row lengths
    #[test]
    fn bad_csv_should_result_in_error() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("uneven_rows.csv").unwrap());
        let result = static_fn.call(&args.clone());

        assert!(result.is_err());

        let error_kind = result.err().unwrap().kind;
        match error_kind {
            tera::ErrorKind::Msg(msg) => {
                if msg != String::from("Error encountered when parsing csv records") {
                    panic!("Error message is wrong. Perhaps wrong error is being returned?");
                }
            }
            _ => panic!("Error encountered was not expected CSV error"),
        }
    }

    #[test]
    fn can_load_json() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"));
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.json").unwrap());
        let result = static_fn.call(&args.clone()).unwrap();

        assert_eq!(
            result,
            json!({
                "key": "value",
                "array": [1, 2, 3],
                "subpackage": {
                    "subkey": 5
                }
            })
        )
    }
}
