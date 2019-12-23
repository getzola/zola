use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, ACCEPT};
use reqwest::StatusCode;

use config::LinkChecker;
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
        Ok(response) => {
            if response.status().is_success() {
                LinkResult { code: Some(response.status()), error: None }
            } else {
                let error_string = if response.status().is_informational() {
                    String::from(format!(
                        "Informational status code ({}) received",
                        response.status()
                    ))
                } else if response.status().is_redirection() {
                    String::from(format!(
                        "Redirection status code ({}) received",
                        response.status()
                    ))
                } else if response.status().is_client_error() {
                    String::from(format!(
                        "Client error status code ({}) received",
                        response.status()
                    ))
                } else if response.status().is_server_error() {
                    String::from(format!(
                        "Server error status code ({}) received",
                        response.status()
                    ))
                } else {
                    String::from("Non-success status code received")
                };

                LinkResult { code: None, error: Some(error_string) }
            }
        }
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
    use mockito::mock;

    #[test]
    fn can_validate_ok_links() {
        let url = format!("{}{}", mockito::server_url(), "/test");
        let _m = mock("GET", "/test")
            .with_header("content-type", "text/html")
            .with_body(format!(
                r#"<!DOCTYPE html>
<html>
<head>
  <title>Test</title>
</head>
<body>
  <a href="{}">Mock URL</a>
</body>
</html>
"#,
                url
            ))
            .create();

        let res = check_url(&url, &LinkChecker::default());
        assert!(res.is_valid());
        assert!(LINKS.read().unwrap().get(&url).is_some());
    }

    #[test]
    fn can_fail_unresolved_links() {
        let res = check_url("https://google.comys", &LinkChecker::default());
        assert_eq!(res.is_valid(), false);
        assert!(res.code.is_none());
        assert!(res.error.is_some());
    }

    #[test]
    fn can_fail_404_links() {
        let _m = mock("GET", "/404")
            .with_status(404)
            .with_header("content-type", "text/plain")
            .with_body("Not Found")
            .create();

        let url = format!("{}{}", mockito::server_url(), "/404");
        let res = check_url(&url, &LinkChecker::default());
        assert_eq!(res.is_valid(), false);
        assert!(res.code.is_none());
        assert!(res.error.is_some());
    }

    #[test]
    fn can_validate_anchors() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = r#"<body><h3 id="method.collect">collect</h3></body>"#.to_string();
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
        let body = r#"<body><h3 id="method.collect">collect</h3></body>"#.to_string();
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
        let ignore_url = format!("{}{}", mockito::server_url(), "/ignore/");
        let config = LinkChecker { skip_prefixes: vec![], skip_anchor_prefixes: vec![ignore_url] };

        let _m1 = mock("GET", "/ignore/test")
            .with_header("content-type", "text/html")
            .with_body(
                r#"<!DOCTYPE html>
<html>
<head>
  <title>Ignore</title>
</head>
<body>
  <p id="existent"></p>
</body>
</html>
"#,
            )
            .create();

        // anchor check is ignored because the url matches the prefix
        let ignore = format!("{}{}", mockito::server_url(), "/ignore/test#nonexistent");
        assert!(check_url(&ignore, &config).is_valid());

        let _m2 = mock("GET", "/test")
            .with_header("content-type", "text/html")
            .with_body(
                r#"<!DOCTYPE html>
<html>
<head>
  <title>Test</title>
</head>
<body>
  <p id="existent"></p>
</body>
</html>
"#,
            )
            .create();

        // other anchors are checked
        let existent = format!("{}{}", mockito::server_url(), "/test#existent");
        assert!(check_url(&existent, &config).is_valid());

        let nonexistent = format!("{}{}", mockito::server_url(), "/test#nonexistent");
        assert_eq!(check_url(&nonexistent, &config).is_valid(), false);
    }
}
