extern crate reqwest;
extern crate scraper;
#[macro_use]
extern crate lazy_static;

extern crate errors;

use reqwest::header::{HeaderMap, ACCEPT};
use reqwest::{Response, StatusCode};
use scraper::{Html, Selector};

use errors::Result;

use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug, PartialEq)]
pub struct LinkResult {
    pub code: Option<StatusCode>,
    /// Whether the HTTP request didn't make it to getting a HTTP code
    pub error: Option<String>,
}

impl LinkResult {
    pub fn is_valid(&self) -> bool {
        if self.error.is_some() {
            return false;
        }

        if let Some(c) = self.code {
            return c.is_success();
        }

        true
    }

    pub fn message(&self) -> String {
        if let Some(ref e) = self.error {
            return e.clone();
        }

        if let Some(c) = self.code {
            return format!("{}", c);
        }

        "Unknown error".to_string()
    }
}

lazy_static! {
    // Keep history of link checks so a rebuild doesn't have to check again
    static ref LINKS: Arc<RwLock<HashMap<String, LinkResult>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub fn check_url(url: &str) -> LinkResult {
    {
        let guard = LINKS.read().unwrap();
        if let Some(res) = guard.get(url) {
            return res.clone();
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "text/html".parse().unwrap());
    headers.append(ACCEPT, "*/*".parse().unwrap());

    let client = reqwest::Client::new();

    // Need to actually do the link checking
    let res = match client.get(url).headers(headers).send() {
        Ok(ref mut response) if has_anchor(url) => match check_page_for_anchor(url, response) {
            Ok(_) => LinkResult { code: Some(response.status()), error: None },
            Err(e) => LinkResult { code: None, error: Some(e.to_string()) }
        },
        Ok(response) => LinkResult { code: Some(response.status()), error: None },
        Err(e) => LinkResult { code: None, error: Some(e.description().to_string()) },
    };

    LINKS.write().unwrap().insert(url.to_string(), res.clone());
    res
}

fn has_anchor(url: &str) -> bool {
    match url.find('#') {
        Some(index) => match url.get(index..=index + 1) {
            Some("#/") | Some("#!") | None => false,
            Some(_) => true,
        },
        None => false
    }
}

fn check_page_for_anchor(url: &str, response: &mut Response) -> Result<()> {
    let body = response.text().unwrap();
    let document = Html::parse_document(&body[..]);
    let index = url.find('#').unwrap();
    let href = url.get(index..).unwrap().replace('.', "\\.");
    let selector = Selector::parse(&href[..]).unwrap();

    if document.select(&selector).count() == 0 {
        Err(errors::Error::from(format!("Anchor `{}` not found on page", href)))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{check_url, LINKS};

    #[test]
    fn can_validate_ok_links() {
        let url = "https://google.com";
        let res = check_url(url);
        assert!(res.is_valid());
        assert!(LINKS.read().unwrap().get(url).is_some());
        let res = check_url(url);
        assert!(res.is_valid());
    }

    #[test]
    fn can_fail_404_links() {
        let res = check_url("https://google.comys");
        assert_eq!(res.is_valid(), false);
        assert!(res.code.is_none());
        assert!(res.error.is_some());
    }

    #[test]
    fn can_validate_anchors() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let res = check_url(url);
        assert!(res.is_valid());
        assert!(LINKS.read().unwrap().get(url).is_some());
        let res = check_url(url);
        assert!(res.is_valid());
    }

    #[test]
    fn can_fail_when_anchor_not_found() {
        let res = check_url("https://doc.rust-lang.org/std/iter/trait.Iterator.html#me");
        assert_eq!(res.is_valid(), false);
        assert!(res.code.is_none());
        assert_eq!(res.error, Some("Anchor `#me` not found on page".to_string()));
    }
}
