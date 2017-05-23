use std::collections::HashMap;
use std::path::{PathBuf};

use tera::{GlobalFn, Value, from_value, to_value, Result};

use content::{Page, Section};
use site::resolve_internal_link;


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

pub fn make_get_url(permalinks: HashMap<String, String>,) -> GlobalFn {
    Box::new(move |args| -> Result<Value> {
        match args.get("link") {
            Some(val) => match from_value::<String>(val.clone()) {
                Ok(v) => match resolve_internal_link(&v, &permalinks) {
                    Ok(url) => Ok(to_value(url).unwrap()),
                    Err(_) => Err(format!("Could not resolve URL for link `{}` not found.", v).into())
                },
                Err(_) => Err(format!("`get_url` received link={:?} but it requires a string", val).into()),
            },
            None => Err("`get_url` requires a `link` argument.".into()),
        }
    })
}
