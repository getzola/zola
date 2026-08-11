//! Firecrawl fetcher — **migrate-only**. Only `migrate.rs` imports this module;
//! `refresh.rs` must not (hard rule: Firecrawl never on refresh).
//!
//! Wraps Firecrawl's `/v1/scrape` endpoint with `formats:["markdown"]` so the
//! page body arrives as markdown. The [`PageFetcher`] trait lets unit tests
//! inject a [`MockFetcher`] without touching the network or a real API key.

use std::time::Duration;

use errors::{Result, anyhow, bail};
use reqwest::blocking::Client;
use serde_json::{Value, json};

use super::html_to_md;

const FIRECRAWL_URL: &str = "https://api.firecrawl.dev/v1/scrape";

/// One fetched remote page.
///
/// - `markdown` is the raw body returned by Firecrawl (still carries theme
///   chrome; `clean::strip_boilerplate` is applied by the migrate driver).
/// - `description` is the page's source-meta description (Yoast / og:description),
///   used verbatim for front-matter so the body's first junk lines never become
///   the meta description. Empty when the site exposes none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchedPage {
    pub url: String,
    pub title: String,
    pub markdown: String,
    pub description: String,
}

/// Page fetch abstraction so tests mock without Firecrawl/a key.
pub trait PageFetcher {
    fn fetch(&self, url: &str) -> Result<FetchedPage>;
}

/// Live Firecrawl client. `FIRECRAWL_API_KEY` is required.
pub struct FirecrawlFetcher {
    api_key: String,
    client: Client,
}

impl FirecrawlFetcher {
    pub fn new(api_key: String) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(90))
            .user_agent("zola-graph/0.1 (firecrawl)")
            .build()?;
        Ok(Self { api_key, client })
    }
}

impl PageFetcher for FirecrawlFetcher {
    fn fetch(&self, url: &str) -> Result<FetchedPage> {
        let payload = scrape_payload(url);
        let body = serde_json::to_vec(&payload)?;
        let resp = self
            .client
            .post(FIRECRAWL_URL)
            .bearer_auth(&self.api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()?;
        let status = resp.status();
        let text = resp.text()?;
        if !status.is_success() {
            bail!("Firecrawl HTTP {status}: {}", take200(&text));
        }
        let data: Value =
            serde_json::from_str(&text).map_err(|e| anyhow!("Firecrawl non-JSON response: {e}"))?;
        let inner = &data["data"];
        let md = inner["markdown"].as_str().unwrap_or("").to_string();
        let mut title = inner["metadata"]["title"].as_str().unwrap_or("").to_string();
        let description = extract_description(&inner["metadata"]);
        // markdown fallback: some sites return html only — convert then.
        let (markdown, html) = if md.trim().is_empty() {
            let html = inner["html"].as_str().unwrap_or("").to_string();
            (html_to_md::html_to_markdown(&html), html)
        } else {
            (md, String::new())
        };
        if title.is_empty() && !html.is_empty() {
            title = html_to_md::extract_title(&html);
        }
        if markdown.trim().is_empty() {
            bail!("Firecrawl: empty body for {url}");
        }
        Ok(FetchedPage { url: url.to_string(), title, markdown, description })
    }
}

/// Build the `/v1/scrape` request body for `url`. Extracted so the chrome-asking
/// payload is unit-testable without the network.
///
/// `onlyMainContent` alone does not clean this WordPress theme (it lacks clean
/// `<main>`/`<article>` semantics), so `excludeTags` also drops structural chrome
/// tags and common theme selectors. `clean::strip_boilerplate` is the
/// deterministic backstop regardless of what Firecrawl returns.
fn scrape_payload(url: &str) -> Value {
    json!({
        "url": url,
        "formats": ["markdown"],
        "onlyMainContent": true,
        "excludeTags": [
            "nav", "header", "footer", "aside", "form",
            "script", "style", "noscript",
            ".site-header", ".site-footer", ".entry-footer",
            ".comments-area", ".related-posts", ".yarpp-related",
            ".jp-relatedposts", ".sharedaddy",
            ".cookie-notice", ".cookie-consent", "#cookie-notice",
        ],
    })
}

/// First non-empty source-meta description: Yoast `description`, then
/// `og:description`, then `ogDescription`. Whitespace-collapsed to a single
/// line so the emitted TOML stays valid. Empty when the site exposes none —
/// the migrate driver then falls back to a cleaned-body summary.
fn extract_description(metadata: &Value) -> String {
    for key in ["description", "og:description", "ogDescription"] {
        if let Some(s) = metadata.get(key).and_then(|v| v.as_str()) {
            let collapsed = s.split_whitespace().collect::<Vec<_>>().join(" ");
            if !collapsed.is_empty() {
                return collapsed;
            }
        }
    }
    String::new()
}

/// In-memory fetcher for tests and offline runs.
#[cfg(test)]
pub struct MockFetcher {
    pages: std::collections::HashMap<String, FetchedPage>,
}

#[cfg(test)]
impl MockFetcher {
    pub fn new() -> Self {
        Self { pages: std::collections::HashMap::new() }
    }
    pub fn with(mut self, url: &str, title: &str, markdown: &str, description: &str) -> Self {
        self.pages.insert(
            url.to_string(),
            FetchedPage {
                url: url.to_string(),
                title: title.to_string(),
                markdown: markdown.to_string(),
                description: description.to_string(),
            },
        );
        self
    }
}

#[cfg(test)]
impl Default for MockFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl PageFetcher for MockFetcher {
    fn fetch(&self, url: &str) -> Result<FetchedPage> {
        self.pages.get(url).cloned().ok_or_else(|| anyhow!("MockFetcher: no page for {url}"))
    }
}

fn take200(s: &str) -> String {
    s.chars().take(200).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_fetcher_returns_registered_page() {
        let f =
            MockFetcher::new().with("https://x/a", "A", "# A\n\nBody of a.\n", "Meta description");
        let p = f.fetch("https://x/a").unwrap();
        assert_eq!(p.title, "A");
        assert_eq!(p.description, "Meta description");
        assert!(p.markdown.contains("Body of a."));
    }

    #[test]
    fn mock_fetcher_missing_errors() {
        let f = MockFetcher::new();
        assert!(f.fetch("https://nope").is_err());
    }

    #[test]
    fn scrape_payload_asks_main_content_and_exclude_tags() {
        let p = scrape_payload("https://x/a");
        assert_eq!(p["url"], "https://x/a");
        assert_eq!(p["formats"][0], "markdown");
        assert_eq!(p["onlyMainContent"], true);
        let tags = p["excludeTags"].as_array().unwrap();
        for needed in ["nav", "header", "footer", "aside", "form", "script", "style", "noscript"] {
            assert!(
                tags.iter().any(|t| t == needed),
                "excludeTags missing structural tag {needed:?}"
            );
        }
    }

    #[test]
    fn extract_description_prefers_yoast_then_og() {
        assert_eq!(
            extract_description(&json!({"description": "Yoast summary", "og:description": "og"})),
            "Yoast summary"
        );
        assert_eq!(extract_description(&json!({"og:description": "og only"})), "og only");
        assert_eq!(extract_description(&json!({"ogDescription": "camel"})), "camel");
        assert_eq!(extract_description(&json!({})), "");
    }

    #[test]
    fn extract_description_collapses_whitespace_to_one_line() {
        assert_eq!(
            extract_description(&json!({"description": "  line one\n\n  line two  "})),
            "line one line two"
        );
        let d = extract_description(&json!({"description": "a\nb"}));
        assert!(!d.contains('\n'), "description must be single-line: {d:?}");
        // whitespace-only value falls through (treated as absent)
        assert_eq!(extract_description(&json!({"description": "   \n  "})), "");
    }
}
