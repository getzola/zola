use std::collections::HashMap;
use std::path::{PathBuf};

use tera::{GlobalFn, Value, from_value, to_value, Result};

use content::{Page, Section};
use config::Config;
use utils::site::resolve_internal_link;


pub fn make_get_page(all_pages: &HashMap<PathBuf, Page>) -> GlobalFn {
    let mut pages = HashMap::new();
    for page in all_pages.values() {
        pages.insert(page.file.relative.clone(), page.clone());
    }

    Box::new(move |args| -> Result<Value> {
        match args.get("path") {
            Some(val) => match from_value::<String>(val.clone()) {
                Ok(v) => {
                    match pages.get(&v) {
                        Some(p) => Ok(to_value(p).unwrap()),
                        None => Err(format!("Page `{}` not found.", v).into())
                    }
                },
                Err(_) => Err(format!("`get_page` received path={:?} but it requires a string", val).into()),
            },
            None => Err("`get_page` requires a `path` argument.".into()),
        }
    })
}

pub fn make_get_section(all_sections: &HashMap<PathBuf, Section>) -> GlobalFn {
    let mut sections = HashMap::new();
    for section in all_sections.values() {
        sections.insert(section.file.relative.clone(), section.clone());
    }

    Box::new(move |args| -> Result<Value> {
        match args.get("path") {
            Some(val) => match from_value::<String>(val.clone()) {
                Ok(v) => {
                    match sections.get(&v) {
                        Some(p) => Ok(to_value(p).unwrap()),
                        None => Err(format!("Section `{}` not found.", v).into())
                    }
                },
                Err(_) => Err(format!("`get_section` received path={:?} but it requires a string", val).into()),
            },
            None => Err("`get_section` requires a `path` argument.".into()),
        }
    })
}

pub fn make_get_url(permalinks: HashMap<String, String>, config: Config) -> GlobalFn {
    Box::new(move |args| -> Result<Value> {
        let cachebust = args
            .get("cachebust")
            .map_or(false, |c| {
                from_value::<bool>(c.clone()).unwrap_or(false)
            });
        
        let trailing_slash = args
            .get("trailing_slash")
            .map_or(true, |c| {
                from_value::<bool>(c.clone()).unwrap_or(true)
            });
        
        match args.get("path") {
            Some(val) => match from_value::<String>(val.clone()) {
                Ok(v) => {
                    // Internal link
                    if v.starts_with("./") {
                        match resolve_internal_link(&v, &permalinks) {
                            Ok(url) => Ok(to_value(url).unwrap()),
                            Err(_) => Err(format!("Could not resolve URL for link `{}` not found.", v).into())
                        }
                    } else {
                        // anything else
                        let mut permalink = config.make_permalink(&v);
                        if !trailing_slash && permalink.ends_with("/") {
                            permalink.pop(); // Removes the slash
                        }

                        if cachebust {
                            permalink = format!("{}?t={}", permalink, config.build_timestamp.unwrap());
                        }
                        Ok(to_value(permalink).unwrap())
                    }
                },
                Err(_) => Err(format!("`get_url` received path={:?} but it requires a string", val).into()),
            },
            None => Err("`get_url` requires a `path` argument.".into()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::make_get_url;

    use std::collections::HashMap;

    use tera::to_value;

    use config::Config;


    #[test]
    fn can_add_cachebust_to_url() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css/?t=1");
    }
    
    #[test]
    fn can_remove_trailing_slashes() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(false).unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css");
    }
    
    #[test]
    fn can_remove_slashes_and_cachebust() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        args.insert("trailing_slash".to_string(), to_value(false).unwrap());
        args.insert("cachebust".to_string(), to_value(true).unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css?t=1");
    }

    #[test]
    fn can_link_to_some_static_file() {
        let config = Config::default();
        let static_fn = make_get_url(HashMap::new(), config);
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("app.css").unwrap());
        assert_eq!(static_fn(args).unwrap(), "http://a-website.com/app.css/");
    }
}
