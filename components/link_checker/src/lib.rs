use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, ACCEPT};
use reqwest::{blocking::Client, StatusCode};

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
            return c.is_success() || c == StatusCode::NOT_MODIFIED;
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

    let client = Client::new();

    let check_anchor = !config.skip_anchor_prefixes.iter().any(|prefix| url.starts_with(prefix));

    // Need to actually do the link checking
    let res = match client.get(url).headers(headers).send() {
        Ok(ref mut response) if check_anchor && has_anchor(url) => {
            let body = {
                let mut buf: Vec<u8> = vec![];
                response.copy_to(&mut buf).unwrap();
                String::from_utf8(buf).unwrap()
            };

            match check_page_for_anchor(url, body) {
                Ok(_) => LinkResult { code: Some(response.status()), error: None },
                Err(e) => LinkResult { code: None, error: Some(e.to_string()) },
            }
        }
        Ok(response) => {
            if response.status().is_success() || response.status() == StatusCode::NOT_MODIFIED {
                LinkResult { code: Some(response.status()), error: None }
            } else {
                let error_string = if response.status().is_informational() {
                    format!("Informational status code ({}) received", response.status())
                } else if response.status().is_redirection() {
                    format!("Redirection status code ({}) received", response.status())
                } else if response.status().is_client_error() {
                    format!("Client error status code ({}) received", response.status())
                } else if response.status().is_server_error() {
                    format!("Server error status code ({}) received", response.status())
                } else {
                    format!("Non-success status code ({}) received", response.status())
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

fn check_page_for_anchor(url: &str, body: String) -> Result<()> {
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

    // NOTE: HTTP mock paths below are randomly generated to avoid name
    // collisions. Mocks with the same path can sometimes bleed between tests
    // and cause them to randomly pass/fail. Please make sure to use unique
    // paths when adding or modifying tests that use Mockito.

    #[test]
    fn can_validate_ok_links() {
        let url = format!("{}{}", mockito::server_url(), "/ekbtwxfhjw");
        let _m = mock("GET", "/ekbtwxfhjw")
            .with_header("Content-Type", "text/html")
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
    fn can_follow_301_links() {
        let _m1 = mock("GET", "/c7qrtrv3zz")
            .with_status(301)
            .with_header("Content-Type", "text/plain")
            .with_header("Location", format!("{}/rbs5avjs8e", mockito::server_url()).as_str())
            .with_body("Redirecting...")
            .create();

        let _m2 = mock("GET", "/rbs5avjs8e")
            .with_header("Content-Type", "text/plain")
            .with_body("Test")
            .create();

        let url = format!("{}{}", mockito::server_url(), "/c7qrtrv3zz");
        let res = check_url(&url, &LinkChecker::default());
        assert!(res.is_valid());
        assert!(res.code.is_some());
        assert!(res.error.is_none());
    }

    #[test]
    fn can_fail_301_to_404_links() {
        let _m1 = mock("GET", "/cav9vibhsc")
            .with_status(301)
            .with_header("Content-Type", "text/plain")
            .with_header("Location", format!("{}/72zmfg4smd", mockito::server_url()).as_str())
            .with_body("Redirecting...")
            .create();

        let _m2 = mock("GET", "/72zmfg4smd")
            .with_status(404)
            .with_header("Content-Type", "text/plain")
            .with_body("Not Found")
            .create();

        let url = format!("{}{}", mockito::server_url(), "/cav9vibhsc");
        let res = check_url(&url, &LinkChecker::default());
        assert_eq!(res.is_valid(), false);
        assert!(res.code.is_none());
        assert!(res.error.is_some());
    }

    #[test]
    fn can_fail_404_links() {
        let _m = mock("GET", "/nlhab9c1vc")
            .with_status(404)
            .with_header("Content-Type", "text/plain")
            .with_body("Not Found")
            .create();

        let url = format!("{}{}", mockito::server_url(), "/nlhab9c1vc");
        let res = check_url(&url, &LinkChecker::default());
        assert_eq!(res.is_valid(), false);
        assert!(res.code.is_none());
        assert!(res.error.is_some());
    }

    #[test]
    fn can_fail_500_links() {
        let _m = mock("GET", "/qdbrssazes")
            .with_status(500)
            .with_header("Content-Type", "text/plain")
            .with_body("Internal Server Error")
            .create();

        let url = format!("{}{}", mockito::server_url(), "/qdbrssazes");
        let res = check_url(&url, &LinkChecker::default());
        assert_eq!(res.is_valid(), false);
        assert!(res.code.is_none());
        assert!(res.error.is_some());
    }

    #[test]
    fn can_fail_unresolved_links() {
        let res = check_url("https://t6l5cn9lpm.lxizfnzckd", &LinkChecker::default());
        assert_eq!(res.is_valid(), false);
        assert!(res.code.is_none());
        assert!(res.error.is_some());
    }

    #[test]
    fn can_validate_anchors() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = r#"<body><h3 id="method.collect">collect</h3></body>"#.to_string();
        let res = check_page_for_anchor(url, body);
        assert!(res.is_ok());
    }

    #[test]
    fn can_validate_anchors_with_other_quotes() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = r#"<body><h3 id="method.collect">collect</h3></body>"#.to_string();
        let res = check_page_for_anchor(url, body);
        assert!(res.is_ok());
    }

    #[test]
    fn can_validate_anchors_with_name_attr() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect";
        let body = r#"<body><h3 name="method.collect">collect</h3></body>"#.to_string();
        let res = check_page_for_anchor(url, body);
        assert!(res.is_ok());
    }

    #[test]
    fn can_fail_when_anchor_not_found() {
        let url = "https://doc.rust-lang.org/std/iter/trait.Iterator.html#me";
        let body = r#"<body><h3 id="method.collect">collect</h3></body>"#.to_string();
        let res = check_page_for_anchor(url, body);
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

        let _m1 = mock("GET", "/ignore/i30hobj1cy")
            .with_header("Content-Type", "text/html")
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
        let ignore = format!("{}{}", mockito::server_url(), "/ignore/i30hobj1cy#nonexistent");
        assert!(check_url(&ignore, &config).is_valid());

        let _m2 = mock("GET", "/guvqcqwmth")
            .with_header("Content-Type", "text/html")
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
        let existent = format!("{}{}", mockito::server_url(), "/guvqcqwmth#existent");
        assert!(check_url(&existent, &config).is_valid());

        let nonexistent = format!("{}{}", mockito::server_url(), "/guvqcqwmth#nonexistent");
        assert_eq!(check_url(&nonexistent, &config).is_valid(), false);
    }
}
