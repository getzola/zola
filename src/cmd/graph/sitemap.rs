//! Sitemap URL collection for `graph migrate`.
//!
//! Pure [`parse_sitemap`] over well-formed XML, then a live [`collect_urls`]
//! that fetches + recurses `sitemapindex` entries. Namespaced variants
//! (`<ns:urlset>`) are handled by substring on the local tag name.
//!
//! ponytail: regex `<loc>` extraction. Ceiling = malformed/CDATA sitemaps,
//! `.xml.gz` compression, or `<html>` error pages served 200 with empty locs.
//! Upgrade to `quick-xml` if a real site's sitemap breaks the regex.

use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Duration;

use errors::{Result, anyhow, bail};
use regex::Regex;
use reqwest::blocking::Client;

static LOC: OnceLock<Regex> = OnceLock::new();
fn loc_re() -> &'static Regex {
    LOC.get_or_init(|| {
        // local-name agnostic: matches <...loc> (any namespace prefix) until </...loc>.
        Regex::new(r"(?s)<\w*:?\w*?loc>(.*?)</\w*:?\w*?loc>").unwrap()
    })
}

/// What a single sitemap document contains.
#[derive(Debug, PartialEq, Eq)]
pub enum Sitemap {
    /// A `<sitemapindex>`: children are more sitemap URLs to recurse into.
    Index(Vec<String>),
    /// A `<urlset>`: leaf page URLs.
    UrlSet(Vec<String>),
    /// No `<loc>` found (404 HTML body, empty doc, etc.).
    Empty,
}

/// Parse one sitemap document. Pure: no I/O, no network — fixture-testable.
pub fn parse_sitemap(xml: &str) -> Sitemap {
    let urls: Vec<String> = loc_re()
        .captures_iter(xml)
        .map(|c| html_decode(c[1].trim()))
        .filter(|s| !s.is_empty())
        .collect();
    if xml.contains("sitemapindex") {
        Sitemap::Index(urls)
    } else if urls.is_empty() {
        Sitemap::Empty
    } else {
        Sitemap::UrlSet(urls)
    }
}

/// Fetch `sitemap_url`, recursing sitemap indexes. Returns deduped, sorted leaf
/// page URLs. Live (network); not unit-tested.
pub fn collect_urls(sitemap_url: &str, client: &Client) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut seen_req = HashSet::new();
    recurse(sitemap_url, client, &mut out, &mut seen_req)?;
    out.sort();
    out.dedup();
    Ok(out)
}

fn recurse(
    url: &str,
    client: &Client,
    out: &mut Vec<String>,
    seen: &mut HashSet<String>,
) -> Result<()> {
    if !seen.insert(url.to_string()) {
        return Ok(()); // loop guard
    }
    let body = fetch_text(url, client)?;
    match parse_sitemap(&body) {
        Sitemap::Index(children) => {
            for child in children {
                recurse(&child, client, out, seen)?;
            }
        }
        Sitemap::UrlSet(urls) => out.extend(urls),
        Sitemap::Empty => bail!("{url}: sitemap parsed empty (not a sitemapindex/urlset?)"),
    }
    Ok(())
}

/// Try `<origin>/sitemap_index.xml`, then `<origin>/sitemap.xml`. Returns the
/// first that yields leaf URLs.
pub fn discover(origin: &str, client: &Client) -> Result<Vec<String>> {
    let origin = origin.trim_end_matches('/');
    for path in ["sitemap_index.xml", "sitemap.xml"] {
        let url = format!("{origin}/{path}");
        match collect_urls(&url, client) {
            Ok(urls) if !urls.is_empty() => return Ok(urls),
            Ok(_) => log::info!("sitemap: {url} returned no URLs, trying next"),
            Err(e) => log::info!("sitemap: {url} failed ({e}), trying next"),
        }
    }
    Err(anyhow!("no usable sitemap at {origin} (tried sitemap_index.xml, sitemap.xml)"))
}

fn fetch_text(url: &str, client: &Client) -> Result<String> {
    let resp = client.get(url).send()?;
    let status = resp.status();
    let text = resp.text()?;
    if !status.is_success() {
        bail!("GET {url}: HTTP {status}");
    }
    Ok(text)
}

/// Build the shared blocking client (UA + sane timeout). Live callers use this.
pub fn http_client() -> Result<Client> {
    Ok(Client::builder().timeout(Duration::from_secs(60)).user_agent("zola-graph/0.1").build()?)
}

/// Minimal entity decode for `<loc>` URL contents (`&amp;` `&lt;` `&gt;`).
fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&apos;", "'")
        .replace("&quot;", "\"")
}

/// Static-asset extensions filtered out of the crawl scope before fetching.
/// ponytail: flat extension list, no MIME lookup. Ceiling = an asset served
/// with no extension (e.g. a CDN image URL like `/img/abc`) is not caught —
/// acceptable, Firecrawl would still 4xx/5xx it and it'd count as a failure
/// rather than silently ship. Add MIME sniffing only if a real site trips it.
const ASSET_EXT: &[&str] = &[
    ".ico", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".avif", ".css", ".js", ".json",
    ".xml", ".pdf", ".zip", ".mp4", ".webm", ".woff", ".woff2", ".ttf",
];

/// True if `url` points at a static asset (image / font / doc / media / style /
/// script / feed), not an HTML page. Case-insensitive; ignores the query string
/// and fragment. `migrate` applies this **before** the `--max` cap so the cap
/// yields N real pages, and skipped assets are not counted as failures.
pub fn is_asset_url(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let lower = path.to_ascii_lowercase();
    ASSET_EXT.iter().any(|ext| lower.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_yoast_index() {
        let xml = r#"<?xml version="1.0"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <sitemap><loc>https://x/post-sitemap.xml</loc></sitemap>
  <sitemap><loc>https://x/page-sitemap.xml</loc></sitemap>
</sitemapindex>"#;
        assert_eq!(
            parse_sitemap(xml),
            Sitemap::Index(vec![
                "https://x/post-sitemap.xml".into(),
                "https://x/page-sitemap.xml".into(),
            ])
        );
    }

    #[test]
    fn parses_urlset_and_entities() {
        let xml = r#"<?xml version="1.0"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://x/blog/a?x=1&amp;y=2</loc></url>
  <url><loc>https://x/blog/b</loc></url>
</urlset>"#;
        assert_eq!(
            parse_sitemap(xml),
            Sitemap::UrlSet(vec!["https://x/blog/a?x=1&y=2".into(), "https://x/blog/b".into(),])
        );
    }

    #[test]
    fn empty_or_html_is_empty() {
        assert_eq!(parse_sitemap("<html>404</html>"), Sitemap::Empty);
        assert_eq!(parse_sitemap(""), Sitemap::Empty);
    }

    #[test]
    fn namespaced_sitemap_works() {
        // some generators emit a namespace prefix on loc
        let xml = "<urlset xmlns:sm=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\
                   <sm:url><sm:loc>https://x/c</sm:loc></sm:url></urlset>";
        match parse_sitemap(xml) {
            Sitemap::UrlSet(v) => assert_eq!(v, vec!["https://x/c".to_string()]),
            other => panic!("expected UrlSet, got {other:?}"),
        }
    }

    #[test]
    fn is_asset_url_flags_common_assets_case_insensitive() {
        for asset in [
            "https://ats.curriculo.me/favicon.ico",
            "https://x/a.png?ver=1",
            "https://x/a.JPG",
            "https://x/logo.webp#logo",
            "https://x/feed.xml",
            "https://x/site.css",
            "https://x/app.js?v=2",
            "https://x/data.json",
            "https://x/font.woff2",
            "https://x/clip.mp4",
            "https://x/doc.pdf",
        ] {
            assert!(is_asset_url(asset), "expected asset: {asset}");
        }
    }

    #[test]
    fn is_asset_url_keeps_html_pages() {
        for page in [
            "https://x/blog/how-ats-works-2026/",
            "https://x/page#section",
            "https://x/index.html",
            "https://x/ico", // no dot, not an extension
            "https://x/",    // homepage
            "https://x/about?utm=1",
        ] {
            assert!(!is_asset_url(page), "expected page (not asset): {page}");
        }
    }
}
