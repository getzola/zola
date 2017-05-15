use std::collections::HashMap;
use std::path::{PathBuf};

use tera::{GlobalFn, Value, from_value, to_value, Result};

use content::Page;


pub fn make_get_page(all_pages: &HashMap<PathBuf, Page>) -> GlobalFn {
    let mut pages = HashMap::new();
    for page in all_pages.values() {
        pages.insert(page.relative_path.clone(), page.clone());
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
