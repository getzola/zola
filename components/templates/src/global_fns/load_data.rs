use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use libs::csv::Reader;
use libs::reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE};
use libs::reqwest::{blocking::Client, header};
use libs::tera::{
    from_value, to_value, Error, Error as TeraError, Function as TeraFn, Map, Result, Value,
};
use libs::url::Url;
use libs::{nom_bibtex, serde_json, serde_yaml, toml};
use utils::de::fix_toml_dates;
use utils::fs::{get_file_time, read_file};

use crate::global_fns::helpers::search_for_file;

const GET_DATA_ARGUMENT_ERROR_MESSAGE: &str =
    "`load_data`: requires EITHER a `path`, `url`, or `literal` argument";

#[derive(Debug, PartialEq, Clone, Copy, Hash)]
enum Method {
    Post,
    Get,
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_ref() {
            "post" => Ok(Method::Post),
            "get" => Ok(Method::Get),
            _ => Err("`load_data` method must either be POST or GET.".into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum OutputFormat {
    Toml,
    Json,
    Csv,
    Bibtex,
    Plain,
    Xml,
    Yaml,
}

impl FromStr for OutputFormat {
    type Err = Error;

    fn from_str(output_format: &str) -> Result<Self> {
        match output_format.to_lowercase().as_ref() {
            "toml" => Ok(OutputFormat::Toml),
            "csv" => Ok(OutputFormat::Csv),
            "json" => Ok(OutputFormat::Json),
            "bibtex" => Ok(OutputFormat::Bibtex),
            "xml" => Ok(OutputFormat::Xml),
            "plain" => Ok(OutputFormat::Plain),
            "yaml" | "yml" => Ok(OutputFormat::Yaml),
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
            OutputFormat::Bibtex => "application/x-bibtex",
            OutputFormat::Xml => "text/xml",
            OutputFormat::Plain => "text/plain",
            OutputFormat::Yaml => "application/x-yaml",
        })
    }
}

#[derive(Debug)]
enum DataSource {
    Url(Url),
    Path(PathBuf),
    Literal(String),
}

impl DataSource {
    /// Returns Some(DataSource) on success, from optional load_data() path/url arguments
    /// Returns an Error when a URL could not be parsed and Ok(None) when the path
    /// is missing, so that the load_data() function can decide whether this is an error
    /// Note: if the signature of this function changes, please update LoadData::call()
    /// so we don't mistakenly unwrap things over there
    fn from_args(
        path_arg: Option<String>,
        url_arg: Option<String>,
        literal_arg: Option<String>,
        base_path: &Path,
        theme: &Option<String>,
        output_path: &Path,
    ) -> Result<Option<Self>> {
        // only one of `path`, `url`, or `literal` can be specified
        if (path_arg.is_some() && url_arg.is_some())
            || (path_arg.is_some() && literal_arg.is_some())
            || (url_arg.is_some() && literal_arg.is_some())
        {
            return Err(GET_DATA_ARGUMENT_ERROR_MESSAGE.into());
        }

        if let Some(path) = path_arg {
            return match search_for_file(base_path, &path, theme, output_path)
                .map_err(|e| format!("`load_data`: {}", e))?
            {
                Some((f, _)) => Ok(Some(DataSource::Path(f))),
                None => Ok(None),
            };
        }

        if let Some(url) = url_arg {
            return Url::parse(&url)
                .map(DataSource::Url)
                .map(Some)
                .map_err(|e| format!("`load_data`: Failed to parse {} as url: {}", url, e).into());
        }

        if let Some(string_literal) = literal_arg {
            return Ok(Some(DataSource::Literal(string_literal)));
        }

        Err(GET_DATA_ARGUMENT_ERROR_MESSAGE.into())
    }

    fn get_cache_key(
        &self,
        format: &OutputFormat,
        method: Method,
        post_body: &Option<String>,
        post_content_type: &Option<String>,
        headers: &Option<Vec<String>>,
    ) -> u64 {
        let mut hasher = DefaultHasher::new();
        format.hash(&mut hasher);
        method.hash(&mut hasher);
        post_body.hash(&mut hasher);
        post_content_type.hash(&mut hasher);
        headers.hash(&mut hasher);
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
                get_file_time(path).expect("get file time").hash(state);
            }
            // TODO: double check expectations here
            DataSource::Literal(string_literal) => string_literal.hash(state),
        };
    }
}

fn get_output_format_from_args(
    format_arg: Option<String>,
    data_source: &DataSource,
) -> Result<OutputFormat> {
    if let Some(format) = format_arg {
        return OutputFormat::from_str(&format);
    }

    if let DataSource::Path(path) = data_source {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => OutputFormat::from_str(ext).or(Ok(OutputFormat::Plain)),
            None => Ok(OutputFormat::Plain),
        }
    } else {
        // Always default to Plain if we don't know what it is
        Ok(OutputFormat::Plain)
    }
}

fn add_headers_from_args(header_args: Option<Vec<String>>) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    if let Some(header_args) = header_args {
        for arg in header_args {
            let mut splitter = arg.splitn(2, '=');
            let key = splitter
                .next()
                .ok_or_else(|| {
                    format!("Invalid header argument. Expecting header key, got '{}'", arg)
                })?
                .to_string();
            let value = splitter.next().ok_or_else(|| {
                format!("Invalid header argument. Expecting header value, got '{}'", arg)
            })?;
            headers.append(
                HeaderName::from_str(&key)
                    .map_err(|e| format!("Invalid header name '{}': {}", key, e))?,
                value.parse().map_err(|e| format!("Invalid header value '{}': {}", value, e))?,
            );
        }
    }

    Ok(headers)
}

/// A Tera function to load data from a file or from a URL
/// Currently the supported formats are json, toml, csv, yaml, bibtex and plain text
#[derive(Debug)]
pub struct LoadData {
    base_path: PathBuf,
    theme: Option<String>,
    client: Arc<Mutex<Client>>,
    result_cache: Arc<Mutex<HashMap<u64, Value>>>,
    output_path: PathBuf,
}
impl LoadData {
    pub fn new(base_path: PathBuf, theme: Option<String>, output_path: PathBuf) -> Self {
        let client = Arc::new(Mutex::new(
            Client::builder()
                .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
                .build()
                .expect("reqwest client build"),
        ));
        let result_cache = Arc::new(Mutex::new(HashMap::new()));
        Self { base_path, client, result_cache, theme, output_path }
    }
}

impl TeraFn for LoadData {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        // Either a local path or a URL
        let path_arg = optional_arg!(String, args.get("path"), GET_DATA_ARGUMENT_ERROR_MESSAGE);
        let url_arg = optional_arg!(String, args.get("url"), GET_DATA_ARGUMENT_ERROR_MESSAGE);
        let literal_arg =
            optional_arg!(String, args.get("literal"), GET_DATA_ARGUMENT_ERROR_MESSAGE);
        // Optional general params
        let format_arg = optional_arg!(
            String,
            args.get("format"),
            "`load_data`: `format` needs to be an argument with a string value, being one of the supported `load_data` file types (csv, json, toml, bibtex, plain)"
        );
        let required = optional_arg!(
            bool,
            args.get("required"),
            "`load_data`: `required` must be a boolean (true or false)"
        )
        .unwrap_or(true);
        // Remote URL parameters only
        let post_body_arg =
            optional_arg!(String, args.get("body"), "`load_data` body must be a string, if set.");
        let post_content_type = optional_arg!(
            String,
            args.get("content_type"),
            "`load_data` content_type must be a string, if set."
        );
        let method_arg = optional_arg!(
            String,
            args.get("method"),
            "`load_data` method must either be POST or GET."
        );

        let method = match method_arg {
            Some(ref method_str) => match Method::from_str(method_str) {
                Ok(m) => m,
                Err(e) => return Err(e),
            },
            _ => Method::Get,
        };
        let headers = optional_arg!(
            Vec<String>,
            args.get("headers"),
            "`load_data`: `headers` needs to be an argument with a list of strings of format <name>=<value>."
        );

        // If the file doesn't exist, source is None
        let data_source = match (
            DataSource::from_args(
                path_arg.clone(),
                url_arg,
                literal_arg,
                &self.base_path,
                &self.theme,
                &self.output_path,
            ),
            required,
        ) {
            // If the file was not required, return a Null value to the template
            (Ok(None), false) | (Err(_), false) => {
                return Ok(Value::Null);
            }
            (Err(e), true) => {
                return Err(e);
            }
            // If the file was required, error
            (Ok(None), true) => {
                // source is None only with path_arg (not URL), so path_arg is safely unwrap
                return Err(format!(
                    "`load_data`: {} doesn't exist",
                    &self.base_path.join(path_arg.unwrap()).display()
                )
                .into());
            }
            (Ok(Some(data_source)), _) => data_source,
        };

        let file_format = get_output_format_from_args(format_arg, &data_source)?;
        let cache_key = data_source.get_cache_key(
            &file_format,
            method,
            &post_body_arg,
            &post_content_type,
            &headers,
        );

        let mut cache = self.result_cache.lock().expect("result cache lock");
        if let Some(cached_result) = cache.get(&cache_key) {
            return Ok(cached_result.clone());
        }

        let data = match data_source {
            DataSource::Path(path) => read_file(&path)
                .map_err(|e| format!("`load_data`: error reading file {:?}: {}", path, e)),
            DataSource::Url(url) => {
                let response_client = self.client.lock().expect("response client lock");
                let req = match method {
                    Method::Get => response_client
                        .get(url.as_str())
                        .headers(add_headers_from_args(headers)?)
                        .header(header::ACCEPT, file_format.as_accept_header()),
                    Method::Post => {
                        let mut resp = response_client
                            .post(url.as_str())
                            .headers(add_headers_from_args(headers)?)
                            .header(header::ACCEPT, file_format.as_accept_header());
                        if let Some(content_type) = post_content_type {
                            match HeaderValue::from_str(&content_type) {
                                Ok(c) => {
                                    resp = resp.header(CONTENT_TYPE, c);
                                }
                                Err(_) => {
                                    return Err(format!(
                                        "`load_data`: {} is an illegal content type",
                                        &content_type
                                    )
                                    .into());
                                }
                            }
                        }
                        if let Some(body) = post_body_arg {
                            resp = resp.body(body);
                        }
                        resp
                    }
                };

                match req.send().and_then(|res| res.error_for_status()) {
                    Ok(r) => r.text().map_err(|e| {
                        format!("`load_data`: Failed to parse response from {}: {:?}", url, e)
                    }),
                    Err(e) => {
                        if !required {
                            // HTTP error is discarded (because required=false) and
                            // Null value is returned to the template
                            return Ok(Value::Null);
                        }
                        Err(match e.status() {
                            Some(status) => {
                                format!("`load_data`: Failed to request {}: {}", url, status)
                            }
                            None => format!(
                                "`load_data`: Could not get response status for url: {}",
                                url
                            ),
                        })
                    }
                }
            }
            DataSource::Literal(string_literal) => Ok(string_literal),
        }?;

        let result_value: Result<Value> = match file_format {
            OutputFormat::Toml => load_toml(data),
            OutputFormat::Csv => load_csv(data),
            OutputFormat::Json => load_json(data),
            OutputFormat::Bibtex => load_bibtex(data),
            OutputFormat::Xml => load_xml(data),
            OutputFormat::Yaml => load_yaml(data),
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

/// Parse a YAML string and convert it to a Tera Value
fn load_yaml(yaml_data: String) -> Result<Value> {
    let yaml_content: Value =
        serde_yaml::from_str(yaml_data.as_str()).map_err(|e| format!("{:?}", e))?;
    Ok(yaml_content)
}

/// Parse a TOML string and convert it to a Tera Value
fn load_toml(toml_data: String) -> Result<Value> {
    let toml_content: toml::Value = toml::from_str(&toml_data).map_err(|e| format!("{:?}", e))?;
    let toml_value = to_value(toml_content).expect("Got invalid JSON that was valid TOML somehow");

    match toml_value {
        Value::Object(m) => Ok(fix_toml_dates(m)),
        _ => Err("Loaded something other than a TOML object".into()),
    }
}

/// Parse a BIBTEX string and convert it to a Tera Value
fn load_bibtex(bibtex_data: String) -> Result<Value> {
    let bibtex_model = nom_bibtex::Bibtex::parse(&bibtex_data).map_err(|e| format!("{:?}", e))?;
    let mut bibtex_map = Map::new();

    let preambles_array =
        bibtex_model.preambles().iter().map(|v| Value::String(v.to_string())).collect();
    bibtex_map.insert(String::from("preambles"), Value::Array(preambles_array));

    let comments_array =
        bibtex_model.comments().iter().map(|v| Value::String(v.to_string())).collect();
    bibtex_map.insert(String::from("comments"), Value::Array(comments_array));

    let mut variables_map = Map::new();
    for (key, val) in bibtex_model.variables() {
        variables_map.insert(key.to_string(), Value::String(val.to_string()));
    }
    bibtex_map.insert(String::from("variables"), Value::Object(variables_map));

    let bibliographies_array = bibtex_model
        .bibliographies()
        .iter()
        .map(|b| {
            let mut m = Map::new();
            m.insert(String::from("entry_type"), Value::String(b.entry_type().to_string()));
            m.insert(String::from("citation_key"), Value::String(b.citation_key().to_string()));

            let mut tags = Map::new();
            for (key, val) in b.tags() {
                tags.insert(key.to_lowercase().to_string(), Value::String(val.to_string()));
            }
            m.insert(String::from("tags"), Value::Object(tags));
            Value::Object(m)
        })
        .collect();
    bibtex_map.insert(String::from("bibliographies"), Value::Array(bibliographies_array));

    let bibtex_value: Value = Value::Object(bibtex_map);
    to_value(bibtex_value).map_err(|err| err.into())
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
        let headers = reader.headers().map_err(|e| {
            format!("'load_data': {} - unable to read CSV header line (line 1) for CSV file", e)
        })?;

        let headers_array = headers.iter().map(|v| Value::String(v.to_string())).collect();

        csv_map.insert(String::from("headers"), Value::Array(headers_array));
    }

    {
        let records = reader.records();

        let mut records_array: Vec<Value> = Vec::new();

        for result in records {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    return Err(TeraError::chain(
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

/// Parse an XML string and convert it to a Tera Value
///
/// An example XML file `example.xml` could be:
/// ```xml
/// <root>
///   <headers>Number</headers>
///   <headers>Title</headers>
///   <records>
///     <item>1</item>
///     <item>Gutenberg</item>
///   </records>
///   <records>
///     <item>2</item>
///     <item>Printing</item>
///   </records>
/// </root>
/// ```
/// The json value output would be:
/// ```json
/// {   
///     "root": {
///         "headers": ["Number", "Title"],
///         "records": [
///                         ["1", "Gutenberg"],
///                         ["2", "Printing"]
///                    ]
///     }
/// }
/// ```
fn load_xml(xml_data: String) -> Result<Value> {
    let xml_content: Value =
        libs::quickxml_to_serde::xml_string_to_json(xml_data, &Default::default())
            .map_err(|e| format!("{:?}", e))?;
    Ok(xml_content)
}

#[cfg(test)]
mod tests {
    use super::{DataSource, LoadData, OutputFormat};

    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::global_fns::load_data::Method;
    use libs::serde_json::json;
    use libs::tera::{self, to_value, Function};
    use std::fs::{copy, create_dir_all};
    use tempfile::tempdir;

    // NOTE: HTTP mock paths below are randomly generated to avoid name
    // collisions. Mocks with the same path can sometimes bleed between tests
    // and cause them to randomly pass/fail. Please make sure to use unique
    // paths when adding or modifying tests that use Mockito.

    fn get_test_file(filename: &str) -> PathBuf {
        let test_files = PathBuf::from("../utils/test-files").canonicalize().unwrap();
        test_files.join(filename)
    }

    #[test]
    fn fails_illegal_method_parameter() {
        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value("https://example.com").unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        args.insert("method".to_string(), to_value("illegalmethod").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("`load_data` method must either be POST or GET."));
    }

    #[test]
    fn can_load_remote_data_using_post_method() {
        let mut server = mockito::Server::new();
        let _mg = server
            .mock("GET", "/kr1zdgbm4y")
            .with_header("content-type", "text/plain")
            .with_body("GET response")
            .expect(0)
            .create();
        let _mp = server
            .mock("POST", "/kr1zdgbm4y")
            .with_header("content-type", "text/plain")
            .with_body("POST response")
            .create();

        let url = format!("{}{}", server.url(), "/kr1zdgbm4y");

        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(url).unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        args.insert("method".to_string(), to_value("post").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "POST response");
        _mg.assert();
        _mp.assert();
    }

    #[test]
    fn can_load_remote_data_using_post_method_with_content_type_header() {
        let mut server = mockito::Server::new();
        let _mjson = server
            .mock("POST", "/kr1zdgbm4yw")
            .match_header("content-type", "application/json")
            .with_header("content-type", "application/json")
            .with_body("{i_am:'json'}")
            .expect(0)
            .create();
        let _mtext = server
            .mock("POST", "/kr1zdgbm4yw")
            .match_header("content-type", "text/plain")
            .with_header("content-type", "text/plain")
            .with_body("POST response text")
            .create();

        let url = format!("{}{}", server.url(), "/kr1zdgbm4yw");

        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(url).unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        args.insert("method".to_string(), to_value("post").unwrap());
        args.insert("content_type".to_string(), to_value("text/plain").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "POST response text");
        _mjson.assert();
        _mtext.assert();
    }

    #[test]
    fn can_load_remote_data_using_post_method_with_body() {
        let mut server = mockito::Server::new();
        let _mjson = server
            .mock("POST", "/kr1zdgbm4y")
            .match_body("qwerty")
            .with_header("content-type", "application/json")
            .with_body("{i_am:'json'}")
            .expect(0)
            .create();
        let _mtext = server
            .mock("POST", "/kr1zdgbm4y")
            .match_body("this is a match")
            .with_header("content-type", "text/plain")
            .with_body("POST response text")
            .create();

        let url = format!("{}{}", server.url(), "/kr1zdgbm4y");

        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(url).unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        args.insert("method".to_string(), to_value("post").unwrap());
        args.insert("content_type".to_string(), to_value("text/plain").unwrap());
        args.insert("body".to_string(), to_value("this is a match").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "POST response text");
        _mjson.assert();
        _mtext.assert();
    }

    #[test]
    fn fails_when_missing_file() {
        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("../../../READMEE.md").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("READMEE.md doesn't exist"));
    }

    #[test]
    fn doesnt_fail_when_missing_file_is_not_required() {
        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("../../../READMEE.md").unwrap());
        args.insert("required".to_string(), to_value(false).unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tera::Value::Null);
    }

    #[test]
    fn can_handle_various_local_file_locations() {
        let dir = tempdir().unwrap();
        create_dir_all(dir.path().join("content").join("gallery")).unwrap();
        create_dir_all(dir.path().join("static")).unwrap();
        copy(get_test_file("test.css"), dir.path().join("content").join("test.css")).unwrap();
        copy(get_test_file("test.css"), dir.path().join("content").join("gallery").join("new.css"))
            .unwrap();
        copy(get_test_file("test.css"), dir.path().join("static").join("test.css")).unwrap();

        let static_fn = LoadData::new(dir.path().to_path_buf(), None, PathBuf::new());
        let mut args = HashMap::new();
        let val = if cfg!(windows) { ".hello {}\r\n" } else { ".hello {}\n" };

        // 1. relative path in `static`
        args.insert("path".to_string(), to_value("static/test.css").unwrap());
        let data = static_fn.call(&args).unwrap().as_str().unwrap().to_string();
        assert_eq!(data, val);

        // 2. relative path in `content`
        args.insert("path".to_string(), to_value("content/test.css").unwrap());
        let data = static_fn.call(&args).unwrap().as_str().unwrap().to_string();
        assert_eq!(data, val);

        // 3. absolute path is the same
        args.insert("path".to_string(), to_value("/content/test.css").unwrap());
        let data = static_fn.call(&args).unwrap().as_str().unwrap().to_string();
        assert_eq!(data, val);

        // 4. path starting with @/
        args.insert("path".to_string(), to_value("@/test.css").unwrap());
        let data = static_fn.call(&args).unwrap().as_str().unwrap().to_string();
        assert_eq!(data, val);
    }

    #[test]
    fn cannot_load_outside_base_dir() {
        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("../../README.md").unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_err());
        println!("{:?} {:?}", std::env::current_dir(), result);
        assert!(result.unwrap_err().to_string().contains("is not inside the base site directory"));
    }

    #[test]
    fn calculates_cache_key_for_path() {
        // We can't test against a fixed value, due to the fact the cache key is built from the absolute path
        let cache_key = DataSource::Path(get_test_file("test.toml")).get_cache_key(
            &OutputFormat::Toml,
            Method::Get,
            &None,
            &None,
            &Some(vec![]),
        );
        let cache_key_2 = DataSource::Path(get_test_file("test.toml")).get_cache_key(
            &OutputFormat::Toml,
            Method::Get,
            &None,
            &None,
            &Some(vec![]),
        );
        assert_eq!(cache_key, cache_key_2);
    }

    #[test]
    fn different_cache_key_per_filename() {
        let toml_cache_key = DataSource::Path(get_test_file("test.toml")).get_cache_key(
            &OutputFormat::Toml,
            Method::Get,
            &None,
            &None,
            &Some(vec![]),
        );
        let json_cache_key = DataSource::Path(get_test_file("test.json")).get_cache_key(
            &OutputFormat::Toml,
            Method::Get,
            &None,
            &None,
            &Some(vec![]),
        );
        assert_ne!(toml_cache_key, json_cache_key);
    }

    #[test]
    fn different_cache_key_per_format() {
        let toml_cache_key = DataSource::Path(get_test_file("test.toml")).get_cache_key(
            &OutputFormat::Toml,
            Method::Get,
            &None,
            &None,
            &Some(vec![]),
        );
        let json_cache_key = DataSource::Path(get_test_file("test.toml")).get_cache_key(
            &OutputFormat::Json,
            Method::Get,
            &None,
            &None,
            &Some(vec![]),
        );
        assert_ne!(toml_cache_key, json_cache_key);
    }

    #[test]
    fn different_cache_key_per_headers() {
        let header1_cache_key = DataSource::Path(get_test_file("test.toml")).get_cache_key(
            &OutputFormat::Json,
            Method::Get,
            &None,
            &None,
            &Some(vec!["a=b".to_string()]),
        );
        let header2_cache_key = DataSource::Path(get_test_file("test.toml")).get_cache_key(
            &OutputFormat::Json,
            Method::Get,
            &None,
            &None,
            &Some(vec![]),
        );
        assert_ne!(header1_cache_key, header2_cache_key);
    }

    #[test]
    fn can_load_remote_data() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/zpydpkjj67")
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
  "test": {
    "foo": "bar"
  }
}
"#,
            )
            .create();

        let url = format!("{}{}", server.url(), "/zpydpkjj67");
        let static_fn = LoadData::new(PathBuf::new(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(&url).unwrap());
        args.insert("format".to_string(), to_value("json").unwrap());
        let result = static_fn.call(&args).unwrap();
        assert_eq!(result.get("test").unwrap().get("foo").unwrap(), &to_value("bar").unwrap());
    }

    #[test]
    fn fails_when_request_404s() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/aazeow0kog")
            .with_status(404)
            .with_header("content-type", "text/plain")
            .with_body("Not Found")
            .create();

        let url = format!("{}{}", server.url(), "/aazeow0kog");
        let static_fn = LoadData::new(PathBuf::new(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(&url).unwrap());
        args.insert("format".to_string(), to_value("json").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            format!("`load_data`: Failed to request {}: 404 Not Found", url)
        );
    }

    #[test]
    fn doesnt_fail_when_request_404s_is_not_required() {
        let mut server = mockito::Server::new();
        let _m = server
            .mock("GET", "/aazeow0kog")
            .with_status(404)
            .with_header("content-type", "text/plain")
            .with_body("Not Found")
            .create();

        let url = format!("{}{}", server.url(), "/aazeow0kog");
        let static_fn = LoadData::new(PathBuf::new(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(&url).unwrap());
        args.insert("format".to_string(), to_value("json").unwrap());
        args.insert("required".to_string(), to_value(false).unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tera::Value::Null);
    }

    #[test]
    fn set_default_user_agent() {
        let mut server = mockito::Server::new();
        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let _m = server
            .mock("GET", "/chu8aizahBiy")
            .match_header("User-Agent", user_agent)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
  "test": {
    "foo": "bar"
  }
}
"#,
            )
            .create();

        let url = format!("{}{}", server.url(), "/chu8aizahBiy");
        let static_fn = LoadData::new(PathBuf::new(), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(&url).unwrap());
        args.insert("format".to_string(), to_value("json").unwrap());
        let result = static_fn.call(&args).unwrap();
        assert_eq!(result.get("test").unwrap().get("foo").unwrap(), &to_value("bar").unwrap());
    }

    #[test]
    fn can_load_toml() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"), None, PathBuf::new());
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
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.css").unwrap());
        let result = static_fn.call(&args.clone()).unwrap();
        println!("{:?}", result);

        if cfg!(windows) {
            assert_eq!(result.as_str().unwrap().replace("\r\n", "\n"), ".hello {}\n",);
        } else {
            assert_eq!(result, ".hello {}\n",);
        };
    }

    #[test]
    fn can_override_known_extension_with_format() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.csv").unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        let result = static_fn.call(&args.clone()).unwrap();

        if cfg!(windows) {
            assert_eq!(
                result.as_str().unwrap().replace("\r\n", "\n"),
                "Number,Title\n1,Gutenberg\n2,Printing",
            );
        } else {
            assert_eq!(result, "Number,Title\n1,Gutenberg\n2,Printing",);
        };
    }

    #[test]
    fn will_use_format_on_unknown_extension() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.css").unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        let result = static_fn.call(&args.clone()).unwrap();

        if cfg!(windows) {
            assert_eq!(result.as_str().unwrap().replace("\r\n", "\n"), ".hello {}\n",);
        } else {
            assert_eq!(result, ".hello {}\n",);
        };
    }

    #[test]
    fn can_load_csv() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"), None, PathBuf::new());
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
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("uneven_rows.csv").unwrap());
        let result = static_fn.call(&args.clone());

        assert!(result.is_err());

        let error_kind = result.err().unwrap().kind;
        match error_kind {
            tera::ErrorKind::Msg(msg) => {
                if msg != *"Error encountered when parsing csv records" {
                    panic!("Error message is wrong. Perhaps wrong error is being returned?");
                }
            }
            _ => panic!("Error encountered was not expected CSV error"),
        }
    }

    #[test]
    fn bad_csv_should_result_in_error_even_when_not_required() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("uneven_rows.csv").unwrap());
        args.insert("required".to_string(), to_value(false).unwrap());
        let result = static_fn.call(&args.clone());

        assert!(result.is_err());

        let error_kind = result.err().unwrap().kind;
        match error_kind {
            tera::ErrorKind::Msg(msg) => {
                if msg != *"Error encountered when parsing csv records" {
                    panic!("Error message is wrong. Perhaps wrong error is being returned?");
                }
            }
            _ => panic!("Error encountered was not expected CSV error"),
        }
    }

    #[test]
    fn can_load_json() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"), None, PathBuf::new());
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

    #[test]
    fn can_load_xml() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.xml").unwrap());
        let result = static_fn.call(&args.clone()).unwrap();

        assert_eq!(
            result,
            json!({
                "root": {
                    "key": "value",
                    "array": [1, 2, 3],
                    "subpackage": {
                        "subkey": 5
                    }
                }
            })
        )
    }

    #[test]
    fn can_load_yaml() {
        let static_fn = LoadData::new(PathBuf::from("../utils/test-files"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("test.yaml").unwrap());
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

    #[test]
    fn is_load_remote_data_using_post_method_with_different_body_not_cached() {
        let mut server = mockito::Server::new();
        let _mjson = server
            .mock("POST", "/kr1zdgbm4y3")
            .with_header("content-type", "application/json")
            .with_body("{i_am:'json'}")
            .expect(2)
            .create();
        let url = format!("{}{}", server.url(), "/kr1zdgbm4y3");

        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(&url).unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        args.insert("method".to_string(), to_value("post").unwrap());
        args.insert("content_type".to_string(), to_value("text/plain").unwrap());
        args.insert("body".to_string(), to_value("this is a match").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_ok());

        let mut args2 = HashMap::new();
        args2.insert("url".to_string(), to_value(&url).unwrap());
        args2.insert("format".to_string(), to_value("plain").unwrap());
        args2.insert("method".to_string(), to_value("post").unwrap());
        args2.insert("content_type".to_string(), to_value("text/plain").unwrap());
        args2.insert("body".to_string(), to_value("this is a match2").unwrap());
        let result2 = static_fn.call(&args2);
        assert!(result2.is_ok());

        _mjson.assert();
    }

    #[test]
    fn is_load_remote_data_using_post_method_with_same_body_cached() {
        let mut server = mockito::Server::new();
        let _mjson = server
            .mock("POST", "/kr1zdgbm4y2")
            .match_body("this is a match")
            .with_header("content-type", "application/json")
            .with_body("{i_am:'json'}")
            .expect(1)
            .create();
        let url = format!("{}{}", server.url(), "/kr1zdgbm4y2");

        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(&url).unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        args.insert("method".to_string(), to_value("post").unwrap());
        args.insert("content_type".to_string(), to_value("text/plain").unwrap());
        args.insert("body".to_string(), to_value("this is a match").unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_ok());

        let mut args2 = HashMap::new();
        args2.insert("url".to_string(), to_value(&url).unwrap());
        args2.insert("format".to_string(), to_value("plain").unwrap());
        args2.insert("method".to_string(), to_value("post").unwrap());
        args2.insert("content_type".to_string(), to_value("text/plain").unwrap());
        args2.insert("body".to_string(), to_value("this is a match").unwrap());
        let result2 = static_fn.call(&args2);
        assert!(result2.is_ok());

        _mjson.assert();
    }

    #[test]
    fn is_custom_headers_working() {
        let mut server = mockito::Server::new();
        let _mjson = server
            .mock("POST", "/kr1zdgbm4y4")
            .with_header("content-type", "application/json")
            .match_header("accept", "text/plain")
            .match_header("x-custom-header", "some-values")
            .with_body("{i_am:'json'}")
            .expect(1)
            .create();
        let url = format!("{}{}", server.url(), "/kr1zdgbm4y4");

        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(&url).unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        args.insert("method".to_string(), to_value("post").unwrap());
        args.insert("content_type".to_string(), to_value("text/plain").unwrap());
        args.insert("body".to_string(), to_value("this is a match").unwrap());
        args.insert("headers".to_string(), to_value(["x-custom-header=some-values"]).unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_ok());

        _mjson.assert();
    }

    #[test]
    fn is_custom_headers_working_with_multiple_values() {
        let mut server = mockito::Server::new();
        let _mjson = server.mock("POST", "/kr1zdgbm4y5")
            .with_status(201)
            .with_header("content-type", "application/json")
            .match_header("authorization", "Bearer 123")
            // Mockito currently does not have a way to validate multiple headers with the same name
            // see https://github.com/lipanski/mockito/issues/117
            .match_header("accept", mockito::Matcher::Any)
            .match_header("x-custom-header", "some-values")
            .match_header("x-other-header", "some-other-values")
            .with_body("<html>I am a server that needs authentication and returns HTML with Accept set to JSON</html>")
            .expect(1)
            .create();
        let url = format!("{}{}", server.url(), "/kr1zdgbm4y5");

        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(&url).unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        args.insert("method".to_string(), to_value("post").unwrap());
        args.insert("content_type".to_string(), to_value("text/plain").unwrap());
        args.insert("body".to_string(), to_value("this is a match").unwrap());
        args.insert(
            "headers".to_string(),
            to_value([
                "x-custom-header=some-values",
                "x-other-header=some-other-values",
                "accept=application/json",
                "authorization=Bearer 123",
            ])
            .unwrap(),
        );
        let result = static_fn.call(&args);
        assert!(result.is_ok());

        _mjson.assert();
    }

    #[test]
    fn fails_when_specifying_invalid_headers() {
        let mut server = mockito::Server::new();
        let _mjson = server.mock("GET", "/kr1zdgbm4y6").with_status(204).expect(0).create();
        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let url = format!("{}{}", server.url(), "/kr1zdgbm4y6");
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(&url).unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        args.insert("headers".to_string(), to_value(["bad-entry::bad-header"]).unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_err());

        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        args.insert("url".to_string(), to_value(&url).unwrap());
        args.insert("format".to_string(), to_value("plain").unwrap());
        args.insert("headers".to_string(), to_value(["\n=\r"]).unwrap());
        let result = static_fn.call(&args);
        assert!(result.is_err());

        _mjson.assert();
    }

    #[test]
    fn can_load_plain_literal() {
        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        let plain_str = "abc 123";
        args.insert("literal".to_string(), to_value(plain_str).unwrap());

        let result = static_fn.call(&args.clone()).unwrap();

        assert_eq!(result, plain_str);
    }

    #[test]
    fn can_load_json_literal() {
        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        let json_str = r#"{
                "key": "value",
                "array": [1, 2, 3],
                "subpackage": {
                    "subkey": 5
                }
            }"#;
        args.insert("literal".to_string(), to_value(json_str).unwrap());
        args.insert("format".to_string(), to_value("json").unwrap());

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
        );
    }

    #[test]
    fn can_load_toml_literal() {
        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        let toml_str = r#"
        [category]
        key = "value"
        date = 1979-05-27T07:32:00Z
        lt1 = 07:32:00
        "#;
        args.insert("literal".to_string(), to_value(toml_str).unwrap());
        args.insert("format".to_string(), to_value("toml").unwrap());

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
    fn can_load_csv_literal() {
        let static_fn = LoadData::new(PathBuf::from("../utils"), None, PathBuf::new());
        let mut args = HashMap::new();
        let csv_str = r#"Number,Title
1,Gutenberg
2,Printing"#;
        args.insert("literal".to_string(), to_value(csv_str).unwrap());
        args.insert("format".to_string(), to_value("csv").unwrap());

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
}
