//! `zola graph migrate` — the **only Firecrawl entrypoint**. Once-per-origin
//! bootstrap: sitemap → Firecrawl fetch → write `content/<slug>/index.md` →
//! OpenRouter topics → commit `data/graph/*.json`. A second migrate for the
//! same origin bails unless `--force`.
//!
//! Network + keys are read in [`migrate`] (the public entry); the testable
//! core is [`migrate_with`], which takes injectable sitemap/fetcher/topic
//! clients so the integration test runs fully offline.

use std::env;
use std::path::Path;

use errors::{Result, anyhow, bail};

use super::firecrawl::{FirecrawlFetcher, PageFetcher};
use super::openrouter::{OpenRouterTopicClient, TopicClient, TopicInput};
use super::schema::{GraphStore, Meta, Page};
use super::sitemap;
use super::{content_hash, now_iso, read_langs, summarize, url_to_content_path, write_page};

/// Injectable sitemap source so tests run without network.
pub trait SitemapSource {
    fn urls(&self, origin: &str) -> Result<Vec<String>>;
}

/// Live sitemap source (HTTP discovery).
pub struct LiveSitemap {
    pub client: reqwest::blocking::Client,
}

impl SitemapSource for LiveSitemap {
    fn urls(&self, origin: &str) -> Result<Vec<String>> {
        sitemap::discover(origin, &self.client)
    }
}

/// Public entry from `main.rs`. Reads `FIRECRAWL_API_KEY` + `OPENROUTER_API_KEY`
/// (fail-fast when absent and not a dry-run), wires live clients.
pub fn migrate(
    root_dir: &Path,
    config_file: &Path,
    from: &str,
    max: Option<usize>,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let _ = read_langs(config_file)?; // validates config readability early
    let (firecrawl_key, openrouter_key) = if dry_run {
        (String::new(), String::new())
    } else {
        let fc = env::var("FIRECRAWL_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("FIRECRAWL_API_KEY not set — migrate needs it"))?;
        let or = env::var("OPENROUTER_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("OPENROUTER_API_KEY not set — migrate needs it"))?;
        (fc, or)
    };
    let http = sitemap::http_client()?;
    let fetcher = if dry_run { None } else { Some(FirecrawlFetcher::new(firecrawl_key)?) };
    migrate_with(
        root_dir,
        from,
        max,
        dry_run,
        force,
        &LiveSitemap { client: http.clone() },
        fetcher.as_ref(),
        &OpenRouterTopicClient,
        &openrouter_key,
    )
}

/// Testable core.
///
/// - `dry_run` reports the planned crawl without fetching/writing/calling the LLM.
/// - `force` bypasses the once-per-origin guard.
#[allow(clippy::too_many_arguments)]
pub fn migrate_with<S, F, C>(
    root_dir: &Path,
    from: &str,
    max: Option<usize>,
    dry_run: bool,
    force: bool,
    sitemap_src: &S,
    fetcher: Option<&F>,
    topic_client: &C,
    openrouter_key: &str,
) -> Result<()>
where
    S: SitemapSource,
    F: PageFetcher,
    C: TopicClient,
{
    let graph_dir = root_dir.join("data/graph");
    let content_dir = root_dir.join("content");
    let existing = GraphStore::load(&graph_dir)?;

    if !force && existing.is_migrated_for(from) {
        bail!(
            "already migrated for {from} ({} pages). Re-run with --force to re-crawl.",
            existing.pages.len()
        );
    }

    let urls = sitemap_src.urls(from)?;
    let total = urls.len();
    // Filter non-HTML assets (favicon.ico, .png/.xml/.css/…) BEFORE the cap so
    // the cap yields N real pages. Assets 4xx/5xx in Firecrawl and would trip
    // the run's failure threshold; skipped assets are not counted as failures.
    let pages: Vec<String> = urls.into_iter().filter(|u| !sitemap::is_asset_url(u)).collect();
    let skipped_assets = total - pages.len();
    let cap = max.unwrap_or(usize::MAX);
    let planned: Vec<&String> = pages.iter().take(cap).collect();
    log::info!(
        "migrate: {from} → {total} sitemap URLs, {skipped_assets} asset(s) filtered, {} in scope",
        planned.len()
    );

    if dry_run {
        log::info!(
            "migrate [dry-run]: would fetch + enrich {} pages; no network, no writes",
            planned.len()
        );
        return Ok(());
    }

    let fetcher = fetcher.ok_or_else(|| anyhow!("migrate: fetcher required (not a dry-run)"))?;

    // Fresh bootstrap (force re-crawl discards the old graph).
    let mut store = GraphStore::default();
    store.meta = Meta {
        schema_version: super::schema::SCHEMA_VERSION,
        source_origin: from.to_string(),
        migrated_at: now_iso(),
        last_refresh: String::new(),
    };

    let mut failures = 0usize;
    let mut enriched = 0usize;
    for url in &planned {
        let fetched = match fetcher.fetch(url) {
            Ok(p) => p,
            Err(e) => {
                failures += 1;
                log::error!("migrate: fetch {url} FAILED: {e}");
                continue;
            }
        };
        let rel = match url_to_content_path(&fetched.url) {
            Ok(p) => p,
            Err(e) => {
                failures += 1;
                log::error!("migrate: bad url {}: {e}", fetched.url);
                continue;
            }
        };
        let cleaned = super::clean::strip_boilerplate(&fetched.markdown);
        let body_trim = cleaned.trim();
        let summary = summarize(body_trim);
        // Front-matter (and topic) description comes from source metadata, not
        // the body — otherwise the fetched markdown's first junk lines leak into
        // the meta description. Fall back to a cleaned-body summary only when the
        // site exposes no meta description at all.
        let description = if fetched.description.is_empty() {
            summary.clone()
        } else {
            fetched.description.clone()
        };
        let disk_path = root_dir.join(&rel);
        let hash = content_hash(body_trim);
        let fm = format!(
            "title = {t:?}\ndescription = {d:?}\n[extra]\nsource_url = {u:?}\ncontent_hash = {h:?}\n",
            t = fetched.title,
            d = description,
            u = fetched.url,
            h = hash,
        );
        if let Err(e) = write_page(&disk_path, &fm, &cleaned) {
            failures += 1;
            log::error!("migrate: write {}: {e}", disk_path.display());
            continue;
        }
        let page = Page {
            url: fetched.url.clone(),
            path: rel,
            title: fetched.title.clone(),
            summary,
            content_hash: hash,
            topic_ids: vec![],
        };
        store.pages.push(page);
        let input = TopicInput { title: fetched.title, description, body: cleaned };
        let page_url = fetched.url.clone();
        match super::topics::enrich_one(
            &mut store,
            &page_url,
            &input,
            topic_client,
            openrouter_key,
            false,
        ) {
            Ok(true) => enriched += 1,
            Ok(false) => {}
            Err(e) => {
                failures += 1;
                log::error!("migrate: topics {page_url} FAILED: {e}");
            }
        }
    }

    store.save(&graph_dir)?;
    log::info!(
        "migrate: wrote {} pages, enriched {enriched}, {failures} failure(s)",
        store.pages.len()
    );
    let _ = content_dir; // content dir created implicitly by write_page
    if failures > 0 {
        bail!("migrate completed with {failures} failure(s)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::graph::openrouter::{TopicExtract, TopicInput, TopicSpec};
    use crate::cmd::graph::refresh::refresh_with;
    use crate::cmd::graph::schema::SCHEMA_VERSION;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    struct Fixture {
        root: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
            let root = std::env::temp_dir()
                .join(format!("zola-graph-migrate-{id}-{}", std::process::id()));
            fs::create_dir_all(&root).unwrap();
            fs::write(
                root.join("config.toml"),
                "base_url = \"https://x/\"\ndefault_language = \"en\"\n",
            )
            .unwrap();
            Fixture { root }
        }
        fn root(&self) -> &Path {
            &self.root
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    /// Sitemap source returning a fixed list.
    struct FixedSitemap(Vec<String>);
    impl SitemapSource for FixedSitemap {
        fn urls(&self, _origin: &str) -> Result<Vec<String>> {
            Ok(self.0.clone())
        }
    }

    struct FixedTopics;
    impl TopicClient for FixedTopics {
        fn extract(&self, input: &TopicInput, _key: &str) -> Result<TopicExtract> {
            Ok(TopicExtract {
                topics: vec![TopicSpec {
                    label: format!("Topic-{}", input.title),
                    aliases: vec![],
                }],
                relations: vec![],
            })
        }
    }

    /// Faithful slice of a real polluted curriculo.me page
    /// (content/ai-resume-builder/blogs/how-ats-works-2026/index.md as migrated
    /// by the buggy first run): leading search-widget + nav/category chrome, an
    /// author/avatar line, real article prose, then _Next Post_ / You May Also
    /// Like / mashed-title toolkit / Close Menu / cookie banner / RejectAccept.
    const POLLUTED_HOW_ATS_WORKS: &str = "\
Hit enter to search or ESC to closeSearch

[Close Search](https://curriculo.me/ai-resume-builder/blogs/how-ats-works-2026/#)

[ATS Optimization](https://curriculo.me/ai-resume-builder/blogs/category/ats-optimization/) [Resume Tips](https://curriculo.me/ai-resume-builder/blogs/category/resume-tips/)

# How ATS Really Works in 2026 — Parsing, Scoring & AI Ranking Explained

Learn how applicant tracking systems really work in 2026 — from document parsing to AI-powered ranking. Understand why 75% of resumes fail ATS screening and how to optimize yours.

![Dev](https://curriculo.me/wp-content/litespeed/avatar/cdf39dbf4bf50ef8434972ae57d5da8a.jpg?ver=1786017117)[Dev](https://curriculo.me/ai-resume-builder/blogs/author/bill/)July 4, 2026

![How ATS applicant tracking systems work in 2026 - parsing scoring and AI ranking](https://curriculo.me/wp-content/uploads/2026/03/featured_existing_01.png.webp)

_Reviewed by the Curriculo Engineering Team_

## What Is an Applicant Tracking System (ATS)?

An applicant tracking system is software employers use to collect, organize, screen, and rank job applications. According to research by TopResume, approximately 80% of resumes are rejected by ATS before reaching a hiring manager.

## Why 75% of Resumes Fail ATS Screening

Research from Jobscan indicates that 75% of resumes fail ATS screening due to three overlapping issues: formatting incompatibility, missing keywords, and weak content.

_**Disclosure:** This article was produced by Curriculo Inc., which develops AI resume building and ATS products._

Ready to build your resume?

Curriculo helps you create an ATS-optimized resume that gets interviews.

Get Started Free →

_Next Post_

### You May Also Like

[![LinkedIn profile vs resume](data:image/svg+xml;charset=utf-8,%3Csvg%3E)](https://curriculo.me/ai-resume-builder/blogs/linkedin-vs-resume-optimization-2026/) [Resume Tips](https://curriculo.me/ai-resume-builder/blogs/category/resume-tips/) [LinkedIn Profile vs Resume: Why You Need Both Optimized in 2026](https://curriculo.me/ai-resume-builder/blogs/linkedin-vs-resume-optimization-2026/)

## The complete resume toolkit

Everything you need to build a resume that clears the screen and lands the interview.

- [AI Resume BuilderBuild an ATS-ready resume that gets past the filters.](https://curriculo.me/ai-resume-builder/)
- [FeaturesKeyword matching, formatting checks, and AI rewrite.](https://curriculo.me/ai-resume-builder/features/)

[Close Menu](https://curriculo.me/ai-resume-builder/blogs/how-ats-works-2026/#)

We use cookies to improve your experience and analyze site traffic. [Privacy Policy](https://curriculo.me/privacy/)

RejectAccept
";

    #[test]
    fn migrate_writes_pages_and_graph_then_guards() {
        let fx = Fixture::new();
        let urls = vec!["https://x/a".into(), "https://x/b".into()];
        let fetcher = super::super::firecrawl::MockFetcher::new()
            .with("https://x/a", "Page A", "Body of A.", "")
            .with("https://x/b", "Page B", "Body of B.", "");

        // first migrate succeeds
        migrate_with(
            fx.root(),
            "https://x",
            None,
            false,
            false,
            &FixedSitemap(urls.clone()),
            Some(&fetcher),
            &FixedTopics,
            "k",
        )
        .unwrap();

        // content written
        assert!(fx.root().join("content/a/index.md").exists());
        assert!(fx.root().join("content/b/index.md").exists());
        let a = fs::read_to_string(fx.root().join("content/a/index.md")).unwrap();
        assert!(a.contains("title = \"Page A\""));
        assert!(a.contains("source_url = \"https://x/a\""));
        assert!(a.contains("content_hash"));

        // graph written
        let store = GraphStore::load(&fx.root().join("data/graph")).unwrap();
        assert_eq!(store.pages.len(), 2);
        assert_eq!(store.meta.source_origin, "https://x");
        assert_eq!(store.meta.schema_version, SCHEMA_VERSION);
        assert!(!store.topics.is_empty(), "topics enriched");

        // second migrate without --force bails (once guard)
        let err = migrate_with(
            fx.root(),
            "https://x",
            None,
            false,
            false,
            &FixedSitemap(urls.clone()),
            Some(&fetcher),
            &FixedTopics,
            "k",
        )
        .unwrap_err();
        assert!(err.to_string().contains("already migrated"));

        // --force re-migrates
        migrate_with(
            fx.root(),
            "https://x",
            None,
            false,
            true,
            &FixedSitemap(urls),
            Some(&fetcher),
            &FixedTopics,
            "k",
        )
        .unwrap();
    }

    #[test]
    fn dry_run_writes_nothing() {
        let fx = Fixture::new();
        let fetcher =
            super::super::firecrawl::MockFetcher::new().with("https://x/a", "A", "body", "");
        migrate_with(
            fx.root(),
            "https://x",
            None,
            true,
            false,
            &FixedSitemap(vec!["https://x/a".into()]),
            Some(&fetcher),
            &FixedTopics,
            "k",
        )
        .unwrap();
        assert!(!fx.root().join("content/a/index.md").exists());
        assert!(!fx.root().join("data/graph/meta.json").exists());
    }

    /// The front-matter `description` must come from source metadata, not the
    /// body's first (polluted) lines, and the written body must be cleaned.
    #[test]
    fn migrate_description_uses_source_metadata_and_cleans_body() {
        let fx = Fixture::new();
        let body = "Hit enter to search or ESC to closeSearch\n\
                    [Close Search](https://x/#)\n\
                    [ATS Optimization](https://x/cat/ats) [Resume Tips](https://x/cat/tips)\n\
                    # Real Title\n\nReal intro sentence about ATS.\n";
        let fetcher = super::super::firecrawl::MockFetcher::new().with(
            "https://x/a",
            "Real Title",
            body,
            "How ATS really works in 2026.",
        );
        migrate_with(
            fx.root(),
            "https://x",
            None,
            false,
            false,
            &FixedSitemap(vec!["https://x/a".into()]),
            Some(&fetcher),
            &FixedTopics,
            "k",
        )
        .unwrap();
        let page = fs::read_to_string(fx.root().join("content/a/index.md")).unwrap();
        assert!(
            page.contains("description = \"How ATS really works in 2026.\""),
            "meta description must be source metadata, not the body:\n{page}"
        );
        assert!(!page.contains("Hit enter to search"), "chrome leaked into page");
        assert!(!page.contains("Close Search"), "chrome leaked into page");
        assert!(page.contains("Real intro sentence about ATS."), "real body dropped");
        assert!(page.contains("# Real Title"));
    }

    /// Regression guard: a faithful polluted body (the real curriculo.me theme
    /// chrome — search widget, nav/category link runs, avatar/author line,
    /// _Next Post_ / You May Also Like / mashed-title toolkit / Close Menu /
    /// cookie banner / RejectAccept) driven through the migrate write path must
    /// produce an index.md with none of the chrome markers, all real article
    /// prose intact, and the front-matter `description` set from source metadata
    /// (not the polluted body). Offline: MockFetcher + FixedTopics.
    #[test]
    fn migrate_regression_strips_chrome_and_uses_meta_description() {
        let body = POLLUTED_HOW_ATS_WORKS;
        let url = "https://curriculo.me/ai-resume-builder/blogs/how-ats-works-2026/";
        let meta_desc = "Learn how applicant tracking systems really work in 2026 — parsing, scoring, and AI ranking.";
        let fetcher = super::super::firecrawl::MockFetcher::new().with(
            url,
            "How ATS Works in 2026: Complete Guide | Curriculo",
            body,
            meta_desc,
        );
        let fx = Fixture::new();
        migrate_with(
            fx.root(),
            "https://curriculo.me",
            None,
            false,
            false,
            &FixedSitemap(vec![url.into()]),
            Some(&fetcher),
            &FixedTopics,
            "k",
        )
        .unwrap();

        let page_path =
            fx.root().join("content/ai-resume-builder/blogs/how-ats-works-2026/index.md");
        let written = fs::read_to_string(&page_path).unwrap();

        // description = source metadata, not the polluted body's first lines
        assert!(
            written.starts_with("+++\n")
                && written.contains(&format!("description = {meta_desc:?}")),
            "front-matter description must be the metadata description:\n{written}"
        );
        // the polluted body's first line must not have leaked into the description
        assert!(!written.contains("description = \"Hit enter to search"));

        // no theme-chrome marker anywhere in the written page
        for marker in [
            "Hit enter to search",
            "Close Search",
            "Close Menu",
            "RejectAccept",
            "wp-content/litespeed",
            "_Next Post_",
            "You May Also Like",
            "We use cookies",
        ] {
            assert!(!written.contains(marker), "chrome marker survived: {marker:?}");
        }

        // real article prose + featured image + disclosure survived the clean
        for needle in [
            "# How ATS Really Works in 2026",
            "An applicant tracking system is software employers use",
            "![How ATS applicant tracking systems work in 2026",
            "Disclosure:** This article was produced by Curriculo",
        ] {
            assert!(written.contains(needle), "lost real content: {needle:?}");
        }
    }

    /// Asset URLs are filtered out of crawl scope before the --max cap, so the
    /// cap yields N real pages and an asset is never fetched (MockFetcher has no
    /// entry for it, so a fetch would count as a failure and bail the run).
    #[test]
    fn migrate_skips_asset_urls_before_cap() {
        let fx = Fixture::new();
        // sitemap: 1 asset + 2 real pages; --max 2 must hit the 2 real pages,
        // not [asset, page1].
        let urls = vec!["https://x/favicon.ico".into(), "https://x/a".into(), "https://x/b".into()];
        let fetcher = super::super::firecrawl::MockFetcher::new()
            .with("https://x/a", "A", "# A\n\nBody A.\n", "")
            .with("https://x/b", "B", "# B\n\nBody B.\n", "");
        migrate_with(
            fx.root(),
            "https://x",
            Some(2),
            false,
            false,
            &FixedSitemap(urls),
            Some(&fetcher),
            &FixedTopics,
            "k",
        )
        .unwrap(); // unwraps ⇒ no failure counted for the filtered asset
        assert!(fx.root().join("content/a/index.md").exists());
        assert!(fx.root().join("content/b/index.md").exists());
        assert!(!fx.root().join("content/favicon.ico/index.md").exists());
    }

    /// Full loop: migrate once → refresh after a body edit → second migrate w/o
    /// force fails. Refresh must never touch Firecrawl (no fetcher passed).
    #[test]
    fn integration_migrate_then_refresh_then_guard() {
        let fx = Fixture::new();
        let urls = vec!["https://x/a".into()];
        let fetcher = super::super::firecrawl::MockFetcher::new().with(
            "https://x/a",
            "A",
            "Original body.",
            "",
        );

        migrate_with(
            fx.root(),
            "https://x",
            None,
            false,
            false,
            &FixedSitemap(urls.clone()),
            Some(&fetcher),
            &FixedTopics,
            "k",
        )
        .unwrap();
        let before = GraphStore::load(&fx.root().join("data/graph")).unwrap();
        let hash_before = before.pages[0].content_hash.clone();
        assert_eq!(before.pages[0].title, "A");

        // edit the page body locally (preserve frontmatter, change body)
        let page_path = fx.root().join("content/a/index.md");
        let mut txt = fs::read_to_string(&page_path).unwrap();
        txt = txt.replace("Original body.", "Edited body — refreshed.");
        fs::write(&page_path, txt).unwrap();

        // refresh: local only (no fetcher arg at all) — re-topics stale page
        refresh_with(fx.root(), None, false, &FixedTopics, "k").unwrap();
        let after = GraphStore::load(&fx.root().join("data/graph")).unwrap();
        assert_ne!(after.pages[0].content_hash, hash_before, "hash updated post-edit");
        assert!(!after.meta.last_refresh.is_empty(), "last_refresh stamped");

        // second migrate without --force still bails
        let err = migrate_with(
            fx.root(),
            "https://x",
            None,
            false,
            false,
            &FixedSitemap(urls),
            Some(&fetcher),
            &FixedTopics,
            "k",
        )
        .unwrap_err();
        assert!(err.to_string().contains("already migrated"));
    }
}
