//! `zola graph refresh` — **local only**. Never imports the firecrawl module,
//! never re-fetches remote HTML. Walks default-language markdown under
//! `content/`, re-topics pages whose stored `content_hash` no longer matches
//! the on-disk body, and stamps `meta.last_refresh`.
//!
//! Public [`refresh`] reads `OPENROUTER_API_KEY`; [`refresh_with`] is the
//! offline-testable core taking an injected [`TopicClient`].

use std::collections::HashSet;
use std::env;
use std::path::Path;

use errors::{Result, anyhow, bail};

use super::openrouter::{OpenRouterTopicClient, TopicClient, TopicInput};
use super::schema::Page;
use super::{
    content_hash, is_default_page, now_iso, parse_page, read_langs, summarize, walk_md,
};

/// Public entry from `main.rs`.
pub fn refresh(
    root_dir: &Path,
    config_file: &Path,
    max: Option<usize>,
    dry_run: bool,
) -> Result<()> {
    let (_default_lang, langs) = read_langs(config_file)?;
    let lang_set: HashSet<&str> = langs.iter().map(|s| s.as_str()).collect();
    let key = if dry_run {
        String::new()
    } else {
        env::var("OPENROUTER_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("OPENROUTER_API_KEY not set — refresh needs it"))?
    };
    refresh_with_inner(root_dir, max, dry_run, &lang_set, &OpenRouterTopicClient, &key)
}

/// Testable core (offline; no env, no live client). Test seam — not called by
/// the public [`refresh`] path, hence the allow.
#[allow(dead_code)]
pub fn refresh_with<C: TopicClient>(
    root_dir: &Path,
    max: Option<usize>,
    dry_run: bool,
    topic_client: &C,
    openrouter_key: &str,
) -> Result<()> {
    // tests use default-language "en" only
    refresh_with_inner(root_dir, max, dry_run, &HashSet::new(), topic_client, openrouter_key)
}

fn refresh_with_inner<C: TopicClient>(
    root_dir: &Path,
    max: Option<usize>,
    dry_run: bool,
    lang_set: &HashSet<&str>,
    topic_client: &C,
    openrouter_key: &str,
) -> Result<()> {
    let graph_dir = root_dir.join("data/graph");
    let content_dir = root_dir.join("content");
    let mut store = super::schema::GraphStore::load(&graph_dir)?;

    if store.meta.source_origin.is_empty() {
        bail!("no prior migrate found (meta.source_origin empty); run `zola graph migrate` first");
    }

    let mut files: Vec<std::path::PathBuf> = Vec::new();
    walk_md(&content_dir, &mut files)?;
    files.sort();

    // collect (page_url, input) for stale/new pages
    let mut todo: Vec<(String, TopicInput)> = Vec::new();
    let mut failures = 0usize;

    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        if !is_default_page(&name, lang_set) {
            continue;
        }
        let (fm, body) = match parse_page(file) {
            Ok(v) => v,
            Err(e) => {
                failures += 1;
                log::error!("refresh: {}: parse failed: {e}", file.display());
                continue;
            }
        };
        let title = fm.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = fm.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let body_trim = body.trim();
        if body_trim.is_empty() {
            continue; // ponytail: nothing to topic on empty/stub bodies
        }
        let hash = content_hash(body_trim);
        let rel = file
            .strip_prefix(root_dir)
            .map_err(|e| anyhow!("strip prefix {}: {e}", file.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let url = fm
            .get("extra")
            .and_then(|e| e.get("source_url"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("local:{rel}"));

        let pos = store.pages.iter().position(|p| p.path == rel);
        match pos {
            Some(i) if store.pages[i].content_hash == hash => {
                continue; // fresh
            }
            Some(i) => {
                // stale: detach old topic edges, will re-merge
                let url = store.pages[i].url.clone();
                detach_page_topics(&mut store, &url);
                store.pages[i].title = title.clone();
                store.pages[i].summary = summarize(body_trim);
                store.pages[i].content_hash = hash;
                store.pages[i].topic_ids.clear();
                let input = TopicInput { title, description, body: body_trim.to_string() };
                todo.push((url, input));
            }
            None => {
                // new page since migrate
                let page = Page {
                    url: url.clone(),
                    path: rel,
                    title: title.clone(),
                    summary: summarize(body_trim),
                    content_hash: hash,
                    topic_ids: vec![],
                };
                store.pages.push(page);
                let input = TopicInput { title, description, body: body_trim.to_string() };
                todo.push((url, input));
            }
        }
    }

    log::info!("refresh: {} page(s) stale/new out of {} default-language files", todo.len(), files.len());

    let cap = max.unwrap_or(usize::MAX);
    let mut enriched = 0usize;
    for (i, (url, input)) in todo.iter().enumerate() {
        if i >= cap {
            log::info!("refresh: --max {cap} reached; remaining resume next run");
            break;
        }
        if dry_run {
            log::info!("refresh [dry-run]: would enrich {url}");
            continue;
        }
        match super::topics::enrich_one(&mut store, url, input, topic_client, openrouter_key, false)
        {
            Ok(true) => enriched += 1,
            Ok(false) => {}
            Err(e) => {
                failures += 1;
                log::error!("refresh: topics {url} FAILED: {e}");
            }
        }
    }

    if !dry_run {
        store.meta.last_refresh = now_iso();
        store.save(&graph_dir)?;
    }
    log::info!("refresh: enriched {enriched}, {failures} failure(s)");
    if failures > 0 {
        bail!("refresh completed with {failures} failure(s)");
    }
    Ok(())
}

/// Remove all `page_topic` edges for `url` and drop `url` from every topic's
/// `page_ids` — prepares a stale page for re-merge.
fn detach_page_topics(store: &mut super::schema::GraphStore, url: &str) {
    store.relations.retain(|r| !(r.from == url && r.kind == "page_topic"));
    for t in store.topics.iter_mut() {
        t.page_ids.retain(|u| u != url);
    }
    // ponytail: topics left with zero page_ids are kept (may reattach); prune
    // later if the orphan set grows. Ceiling: harmless empty topics.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::graph::openrouter::{TopicExtract, TopicInput, TopicSpec};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    fn tmp_root() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let r = std::env::temp_dir().join(format!("zola-graph-refresh-{id}-{}", std::process::id()));
        fs::create_dir_all(&r).unwrap();
        r
    }

    struct FixedTopics;
    impl TopicClient for FixedTopics {
        fn extract(&self, input: &TopicInput, _key: &str) -> Result<TopicExtract> {
            Ok(TopicExtract {
                topics: vec![TopicSpec { label: format!("Topic-{}", input.title), aliases: vec![] }],
                relations: vec![],
            })
        }
    }

    fn seed_migrated(root: &Path) -> super::super::schema::GraphStore {
        // minimal prior graph: one page, meta.source_origin set
        let store = super::super::schema::GraphStore {
            pages: vec![Page {
                url: "https://x/a".into(),
                path: "content/a/index.md".into(),
                title: "A".into(),
                summary: "s".into(),
                content_hash: "oldhash".into(),
                topic_ids: vec![],
            }],
            meta: super::super::schema::Meta {
                source_origin: "https://x".into(),
                ..Default::default()
            },
            ..Default::default()
        };
        store.save(&root.join("data/graph")).unwrap();
        store
    }

    #[test]
    fn refresh_bails_without_prior_migrate() {
        let root = tmp_root();
        let err = refresh_with(&root, None, false, &FixedTopics, "k").unwrap_err();
        assert!(err.to_string().contains("no prior migrate"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_retopics_stale_and_skips_fresh() {
        let root = tmp_root();
        seed_migrated(&root);
        // write the page whose stored hash is "oldhash" → stale
        let body = "Edited body.\n";
        let hash = content_hash(body.trim());
        fs::create_dir_all(root.join("content/a")).unwrap();
        fs::write(
            root.join("content/a/index.md"),
            format!(
                "+++\ntitle = \"A\"\n[extra]\nsource_url = \"https://x/a\"\n+++\n\n{body}"
            ),
        )
        .unwrap();
        assert_ne!(hash, "oldhash");

        refresh_with(&root, None, false, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        assert_eq!(after.pages[0].content_hash, hash, "hash updated to current body");
        assert!(!after.meta.last_refresh.is_empty());
        assert!(!after.topics.is_empty(), "stale page re-enriched");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_dry_run_no_mutate() {
        let root = tmp_root();
        seed_migrated(&root);
        let body = "Edited body.\n";
        fs::create_dir_all(root.join("content/a")).unwrap();
        fs::write(
            root.join("content/a/index.md"),
            format!("+++\ntitle = \"A\"\n+++\n\n{body}"),
        )
        .unwrap();
        refresh_with(&root, None, true, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        assert_eq!(after.pages[0].content_hash, "oldhash", "dry-run must not change hash");
        assert!(after.meta.last_refresh.is_empty(), "dry-run must not stamp");
        fs::remove_dir_all(&root).unwrap();
    }
}
