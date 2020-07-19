use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, ACCEPT};
use reqwest::StatusCode;

use config::LinkChecker;
use errors::Result;

use std::collections::HashMap;
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

pub fn check_url(url: &str, config: &LinkChecker) -> LinkResult {
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

    let check_anchor = !config.skip_anchor_prefixes.iter().any(|prefix| url.starts_with(prefix));

    // Need to actually do the link checking
    let res = match client.get(url).headers(headers).send() {
        Ok(ref mut response) if check_anchor && has_anchor(url) => {
            match check_page_for_anchor(url, response.text()) {
                Ok(_) => LinkResult { code: Some(response.status()), error: None },
                Err(e) => LinkResult { code: None, error: Some(e.to_string()) },
            }
        }
        Ok(response) => LinkResult { code: Some(response.status()), error: None },
        Err(e) => LinkResult { code: None, error: Some(e.to_string()) },
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
        None => false,
    }
}

fn check_page_for_anchor(url: &str, body: reqwest::Result<String>) -> Result<()> {
    let body = body.unwrap();
    let index = url.find('#').unwrap();
    let anchor = url.get(index + 1..).unwrap();
    let checks: [String; 4] = [
        format!(" id='{}'", anchor),
        format!(r#" id="{}""#, anchor),
        format!(" name='{}'", anchor),
        format!(r#" name="{}""#, anchor),
    ];

    if checks.iter().any(|check| body[..].contains(&check[..])) {
        Ok(())
    } else {
        Err(errors::Error::from(format!("Anchor `#{}` not found on page", anchor)))
    }
}

#[cfg(test)]
mod tests {
    use super::{check_page_for_anchor, check_url, has_anchor, LinkChecker, LINKS};

    #[test]
    fn can_validate_ok_links() {
        let url = "https://google.com";
        let res = check_url(url, &LinkChecker::default());
        assert!(res.is_valid());
        assert!(LINKS.read().unwrap().get(url).is_some());
        let res = check_url(url, &LinkChecker::default());
        assert!(res.is_valid());
    }

    #[test]
    fn can_fail_404_links() {
        let res = check_url("https://google.comys", &LinkChecker::default());
        assert_eq!(res.is_valid(), false);
        assert!(res.code.is_none());
        assert!(res.error.is_some());
    }

    #[test]
    fn can_validate_anchors() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = "<body><h3 id='method.collect'>collect</h3></body>".to_string();
        let res = check_page_for_anchor(url, Ok(body));
        assert!(res.is_ok());
    }

    #[test]
    fn can_validate_anchors_with_other_quotes() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = r#"<body><h3 id="method.collect">collect</h3></body>"#.to_string();
        let res = check_page_for_anchor(url, Ok(body));
        assert!(res.is_ok());
    }

    #[test]
    fn can_validate_anchors_with_name_attr() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = r#"<body><h3 name="method.collect">collect</h3></body>"#.to_string();
        let res = check_page_for_anchor(url, Ok(body));
        assert!(res.is_ok());
    }

    #[test]
    fn can_fail_when_anchor_not_found() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#me";
        let body = "<body><h3 id='method.collect'>collect</h3></body>".to_string();
        let res = check_page_for_anchor(url, Ok(body));
        assert!(res.is_err());
    }

    #[test]
    fn can_check_url_for_anchor() {
        let url = "https://doc.rust-lang.org/std/index.html#the-rust-standard-library";
        let res = has_anchor(url);
        assert_eq!(res, true);
    }

    #[test]
    fn will_return_false_when_no_anchor() {
        let url = "https://doc.rust-lang.org/std/index.html";
        let res = has_anchor(url);
        assert_eq!(res, false);
    }

    #[test]
    fn will_return_false_when_has_router_url() {
        let url = "https://doc.rust-lang.org/#/std";
        let res = has_anchor(url);
        assert_eq!(res, false);
    }

    #[test]
    fn will_return_false_when_has_router_url_alt() {
        let url = "https://doc.rust-lang.org/#!/std";
        let res = has_anchor(url);
        assert_eq!(res, false);
    }

    #[test]
    fn skip_anchor_prefixes() {
        let config = LinkChecker {
            skip_prefixes: vec![],
            skip_anchor_prefixes: vec!["https://github.com/rust-lang/rust/blob/".to_owned()],
        };

        // anchor check is ignored because the url matches the prefix
        let permalink = "https://github.com/rust-lang/rust/blob/c772948b687488a087356cb91432425662e034b9/src/librustc_back/target/mod.rs#L194-L214";
        assert!(check_url(&permalink, &config).is_valid());

        // other anchors are checked
        let glossary = "https://help.github.com/en/articles/github-glossary#blame";
        assert!(check_url(&glossary, &config).is_valid());

        let glossary_invalid =
            "https://help.github.com/en/articles/github-glossary#anchor-does-not-exist";
        assert_eq!(check_url(&glossary_invalid, &config).is_valid(), false);
    }
}
