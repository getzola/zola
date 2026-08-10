//! Firecrawl fetcher — **migrate-only**. Only `migrate.rs` imports this module;
//! `refresh.rs` must not (hard rule: Firecrawl never on refresh).
//!
//! Wraps Firecrawl's `/v1/scrape` endpoint with `formats:["markdown"]` so the
//! page body arrives as markdown. The [`PageFetcher`] trait lets unit tests
//! inject a [`MockFetcher`] without touching the network or a real API key.

use std::time::Duration;

use errors::{Result, anyhow, bail};
use reqwest::blocking::Client;
use serde_json::{json, Value};

use super::html_to_md;

const FIRECRAWL_URL: &str = "https://api.firecrawl.dev/v1/scrape";

/// One fetched remote page. `markdown` is what gets written to disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchedPage {
    pub url: String,
    pub title: String,
    pub markdown: String,
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
        let payload = json!({
            "url": url,
            "formats": ["markdown"],
            "onlyMainContent": true,
        });
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
        let data: Value = serde_json::from_str(&text)
            .map_err(|e| anyhow!("Firecrawl non-JSON response: {e}"))?;
        let inner = &data["data"];
        let md = inner["markdown"].as_str().unwrap_or("").to_string();
        let mut title = inner["metadata"]["title"]
            .as_str()
            .unwrap_or("")
            .to_string();
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
        Ok(FetchedPage {
            url: url.to_string(),
            title,
            markdown,
        })
    }
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
    pub fn with(mut self, url: &str, title: &str, markdown: &str) -> Self {
        self.pages.insert(
            url.to_string(),
            FetchedPage {
                url: url.to_string(),
                title: title.to_string(),
                markdown: markdown.to_string(),
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
        self.pages
            .get(url)
            .cloned()
            .ok_or_else(|| anyhow!("MockFetcher: no page for {url}"))
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
        let f = MockFetcher::new().with(
            "https://x/a",
            "A",
            "# A\n\nBody of a.\n",
        );
        let p = f.fetch("https://x/a").unwrap();
        assert_eq!(p.title, "A");
        assert!(p.markdown.contains("Body of a."));
    }

    #[test]
    fn mock_fetcher_missing_errors() {
        let f = MockFetcher::new();
        assert!(f.fetch("https://nope").is_err());
    }
}
