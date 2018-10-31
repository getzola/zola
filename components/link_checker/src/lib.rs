extern crate reqwest;
#[macro_use]
extern crate lazy_static;

use reqwest::header::{HeaderMap, ACCEPT};
use reqwest::StatusCode;
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
        Ok(response) => LinkResult { code: Some(response.status()), error: None },
        Err(e) => LinkResult { code: None, error: Some(e.description().to_string()) },
    };

    LINKS.write().unwrap().insert(url.to_string(), res.clone());
    res
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
}
