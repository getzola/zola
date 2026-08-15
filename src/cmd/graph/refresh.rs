//! `zola graph refresh` — **local only**. Never imports the firecrawl module,
//! never re-fetches remote HTML. Walks markdown under `content/` (all langs,
//! including `_index.md` section pages), fills node fields, re-topics default-
//! language pages whose stored `content_hash` no longer matches the on-disk
//! body, writes pillar overviews, and stamps `meta.last_refresh`.
//!
//! Public [`refresh`] reads `OPENROUTER_API_KEY`; [`refresh_with`] is the
//! offline-testable core taking an injected [`TopicClient`].

use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::sync::LazyLock;

use errors::{Result, anyhow, bail};
use regex::Regex;
use toml::Value;

use super::ids::{canonical_path_from_rel, lang_from_filename, page_id_from_rel};
use super::openrouter::{OpenRouterTopicClient, TopicClient, TopicInput};
use super::schema::{Organization, Page};
use super::{content_hash, is_default_page, now_iso, parse_page, read_langs, summarize, walk_md};

const PILLARS: &[&str] = &["content/_index.md", "content/ai-resume-builder/index.md"];
const OVERVIEW_MIN: usize = 134;
const OVERVIEW_MAX: usize = 167;

static MD_H1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?m)^#\s+(.+)$").unwrap());
static HTML_H1: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<h1\b[^>]*>(.*?)</h1>").unwrap());

/// Public entry from `main.rs`.
pub fn refresh(
    root_dir: &Path,
    config_file: &Path,
    max: Option<usize>,
    dry_run: bool,
) -> Result<()> {
    let (default_lang, langs) = read_langs(config_file)?;
    let lang_set: HashSet<&str> = langs.iter().map(|s| s.as_str()).collect();
    let key = if dry_run {
        String::new()
    } else {
        env::var("OPENROUTER_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("OPENROUTER_API_KEY not set — refresh needs it"))?
    };
    refresh_with_inner(
        root_dir,
        max,
        dry_run,
        &default_lang,
        &lang_set,
        &OpenRouterTopicClient,
        &key,
    )
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
    refresh_with_inner(root_dir, max, dry_run, "en", &HashSet::new(), topic_client, openrouter_key)
}

fn refresh_with_inner<C: TopicClient>(
    root_dir: &Path,
    max: Option<usize>,
    dry_run: bool,
    default_lang: &str,
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

    // collect (page_id, input) for stale/new default-lang pages that need topics
    let mut todo: Vec<(String, TopicInput)> = Vec::new();
    let mut failures = 0usize;
    let mut seen: HashSet<String> = HashSet::new();

    for file in &files {
        let name = file.file_name().unwrap().to_string_lossy().into_owned();
        let (fm, body) = match parse_page(file) {
            Ok(v) => v,
            Err(e) => {
                failures += 1;
                log::error!("refresh: {}: parse failed: {e}", file.display());
                continue;
            }
        };
        let rel = file
            .strip_prefix(root_dir)
            .map_err(|e| anyhow!("strip prefix {}: {e}", file.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let id = page_id_from_rel(&rel);
        seen.insert(id.clone());
        let body_trim = body.trim();
        let hash = content_hash(body_trim);
        let lang = lang_from_filename(&name, default_lang);
        let default_page = lang == default_lang && is_default_page(&name, lang_set);

        let idx = store.pages.iter().position(|p| p.id == id || p.path == rel || p.path == id);
        let old_hash = idx.map(|i| store.pages[i].content_hash.clone());
        let idx = match idx {
            Some(i) => i,
            None => {
                store.pages.push(Page { id: id.clone(), path: id.clone(), ..Default::default() });
                store.pages.len() - 1
            }
        };
        let hash_stale = old_hash.as_deref() != Some(hash.as_str());
        fill_page_fields(&mut store.pages[idx], &rel, &name, &fm, &body, default_lang);

        let stub = store.pages[idx].stub;
        if hash_stale && default_page && !stub && !body_trim.is_empty() {
            detach_page_topics(&mut store, &id);
            store.pages[idx].topic_ids.clear();
            let title = store.pages[idx].title.clone();
            let description =
                fm.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
            todo.push((id.clone(), TopicInput { title, description, body: body_trim.to_string() }));
        }

        if is_pillar(&id) && !dry_run && !stub {
            let missing = store.pages[idx].overview.as_ref().is_none_or(|s| s.is_empty());
            if missing || hash_stale {
                let title = store.pages[idx].title.clone();
                match fetch_overview(topic_client, &title, &body, openrouter_key) {
                    Ok(text) => store.pages[idx].overview = Some(text),
                    Err(e) => {
                        log::warn!("refresh: overview {id}: {e}");
                    }
                }
            }
        }
    }

    prune_store(&mut store, &seen);

    if store.organizations.is_empty() {
        store.organizations.push(Organization {
            id: "org:curriculo".into(),
            name: "Curriculo".into(),
            url: String::new(), // hostless; templates prefix base_url
            logo: "/v3-assets/curriculo-logo-144.webp".into(),
            same_as: vec![],
        });
    }

    log::info!("refresh: {} page(s) stale/new out of {} markdown files", todo.len(), files.len());

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

fn is_pillar(id: &str) -> bool {
    PILLARS.contains(&id)
}

fn fill_page_fields(
    page: &mut Page,
    rel: &str,
    file_name: &str,
    fm: &Value,
    body: &str,
    default_lang: &str,
) {
    let title = fm.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let body_trim = body.trim();
    let wc = word_count_of(body);
    let stub = title.is_empty() || title == "—" || body_trim.is_empty();
    let noindex = extra_bool(fm, "noindex").unwrap_or(false);
    let extra_sitemap_false = extra_bool(fm, "sitemap") == Some(false);
    let canonical_path = canonical_path_from_rel(rel, default_lang);
    let thin = !stub && wc < 300 && !is_thin_exempt(&canonical_path);
    let sitemap = !(stub || noindex || extra_sitemap_false);
    let lang = lang_from_filename(file_name, default_lang);
    let translation_of =
        if lang == default_lang { None } else { Some(default_lang_page_id(rel, file_name, &lang)) };

    page.id = page_id_from_rel(rel);
    page.canonical_path = canonical_path;
    page.path = page.id.clone();
    page.title = title;
    page.h1 = extract_h1(body);
    page.word_count = wc;
    page.stub = stub;
    page.thin = thin;
    page.noindex = noindex;
    page.sitemap = sitemap;
    page.lang = lang;
    page.translation_of = translation_of;
    page.author = extra_str(fm, "author");
    page.date_published = fm_date(fm, "date");
    page.date_modified = fm_date(fm, "updated");
    page.og_image = extra_str(fm, "og_image");
    page.schema_types = extra_string_list(fm, "schema_types");
    page.summary = if body_trim.is_empty() { String::new() } else { summarize(body_trim) };
    page.content_hash = content_hash(body_trim);
}

fn default_lang_page_id(rel: &str, file_name: &str, lang: &str) -> String {
    let suffix = format!(".{lang}.md");
    let default_name = match file_name.strip_suffix(&suffix) {
        Some(stem) => format!("{stem}.md"),
        None => file_name.to_string(),
    };
    let default_rel = match rel.rsplit_once('/') {
        Some((dir, _)) => format!("{dir}/{default_name}"),
        None => default_name,
    };
    page_id_from_rel(&default_rel)
}

fn is_thin_exempt(canonical_path: &str) -> bool {
    canonical_path.starts_with("/privacy")
        || canonical_path.starts_with("/terms")
        || canonical_path.starts_with("/404")
}

fn strip_html_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

fn word_count_of(body: &str) -> u32 {
    strip_html_tags(body).split_whitespace().count() as u32
}

fn extract_h1(body: &str) -> String {
    if let Some(c) = MD_H1.captures(body) {
        return c.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
    }
    if let Some(c) = HTML_H1.captures(body) {
        return strip_html_tags(c.get(1).map(|m| m.as_str()).unwrap_or("")).trim().to_string();
    }
    String::new()
}

fn extra<'a>(fm: &'a Value) -> Option<&'a Value> {
    fm.get("extra")
}

fn extra_bool(fm: &Value, key: &str) -> Option<bool> {
    extra(fm)?.get(key)?.as_bool()
}

fn extra_str(fm: &Value, key: &str) -> Option<String> {
    extra(fm)?.get(key)?.as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn extra_string_list(fm: &Value, key: &str) -> Vec<String> {
    let Some(v) = extra(fm).and_then(|e| e.get(key)) else {
        return Vec::new();
    };
    if let Some(s) = v.as_str() {
        return if s.trim().is_empty() { vec![] } else { vec![s.trim().to_string()] };
    }
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn fm_date(fm: &Value, key: &str) -> Option<String> {
    let v = fm.get(key)?;
    if let Some(s) = v.as_str() {
        let t = s.trim();
        return if t.is_empty() { None } else { Some(t.to_string()) };
    }
    v.as_datetime().map(|dt| dt.to_string())
}

fn overview_in_range(text: &str) -> bool {
    let n = text.split_whitespace().count();
    (OVERVIEW_MIN..=OVERVIEW_MAX).contains(&n)
}

fn fetch_overview<C: TopicClient>(
    client: &C,
    title: &str,
    body: &str,
    key: &str,
) -> Result<String> {
    let text = client.overview(title, body, key)?;
    if overview_in_range(&text) {
        return Ok(text);
    }
    log::warn!(
        "refresh: overview word count {} outside {OVERVIEW_MIN}–{OVERVIEW_MAX}; retrying once",
        text.split_whitespace().count()
    );
    let text = client.overview(title, body, key)?;
    if overview_in_range(&text) {
        return Ok(text);
    }
    bail!(
        "overview word count {} still outside {OVERVIEW_MIN}–{OVERVIEW_MAX}; leaving unset",
        text.split_whitespace().count()
    )
}

/// Drop pages absent from this refresh walk, scrub their edges, and emit
/// reciprocal `translation` relations from `translation_of`.
fn prune_store(store: &mut super::schema::GraphStore, seen: &HashSet<String>) {
    store.pages.retain(|p| seen.contains(&p.id));

    store.relations.retain(|r| {
        if r.from.starts_with("content/") && !seen.contains(&r.from) {
            return false;
        }
        if r.to.starts_with("content/") && !seen.contains(&r.to) {
            return false;
        }
        true
    });

    for t in store.topics.iter_mut() {
        t.page_ids.retain(|id| seen.contains(id));
    }

    let translations: Vec<(String, String)> = store
        .pages
        .iter()
        .filter_map(|p| p.translation_of.as_ref().map(|base| (p.id.clone(), base.clone())))
        .collect();
    for (a, b) in translations {
        upsert_relation(
            &mut store.relations,
            &super::schema::Relation { from: a.clone(), to: b.clone(), kind: "translation".into() },
        );
        upsert_relation(
            &mut store.relations,
            &super::schema::Relation { from: b, to: a, kind: "translation".into() },
        );
    }
}

fn upsert_relation(rels: &mut Vec<super::schema::Relation>, rel: &super::schema::Relation) {
    let exists = rels.iter().any(|r| r.from == rel.from && r.to == rel.to && r.kind == rel.kind);
    if !exists {
        rels.push(rel.clone());
    }
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
        let r =
            std::env::temp_dir().join(format!("zola-graph-refresh-{id}-{}", std::process::id()));
        fs::create_dir_all(&r).unwrap();
        r
    }

    fn words(n: usize) -> String {
        (0..n).map(|i| format!("w{i}")).collect::<Vec<_>>().join(" ")
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
        fn overview(&self, _title: &str, _body: &str, _key: &str) -> Result<String> {
            Ok(words(140))
        }
    }

    fn seed_migrated(root: &Path) -> super::super::schema::GraphStore {
        // minimal prior graph: one page, meta.source_origin set
        let store = super::super::schema::GraphStore {
            pages: vec![Page {
                id: "content/a/index.md".into(),
                canonical_path: "/a/".into(),
                path: "content/a/index.md".into(),
                title: "A".into(),
                summary: "s".into(),
                content_hash: "oldhash".into(),
                ..Default::default()
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
            format!("+++\ntitle = \"A\"\n[extra]\nsource_url = \"https://x/a\"\n+++\n\n{body}"),
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
        fs::write(root.join("content/a/index.md"), format!("+++\ntitle = \"A\"\n+++\n\n{body}"))
            .unwrap();
        refresh_with(&root, None, true, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        assert_eq!(after.pages[0].content_hash, "oldhash", "dry-run must not change hash");
        assert!(after.meta.last_refresh.is_empty(), "dry-run must not stamp");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_marks_em_dash_title_stub_and_ids_are_paths() {
        let root = tmp_root();
        seed_migrated(&root);
        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(
            root.join("content/_index.md"),
            "+++\ntitle = \"AI ATS for high-volume hiring\"\n[extra]\ncanonical = \"https://curriculo.me/\"\n+++\n\n# AI ATS\n\nHomepage body.\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("content/x")).unwrap();
        fs::write(root.join("content/x/index.md"), "+++\ntitle = \"—\"\n+++\n\nLeftover body.\n")
            .unwrap();

        refresh_with(&root, None, false, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();

        let home = after
            .pages
            .iter()
            .find(|p| p.id == "content/_index.md")
            .expect("ATS homepage _index.md must be in the graph");
        assert_eq!(home.canonical_path, "/");
        assert!(!home.id.starts_with("http"), "id must be a path, not a hostful url");
        assert_ne!(home.id, "https://curriculo.me/", "extra.canonical must be ignored");
        assert_eq!(home.h1, "AI ATS");
        assert_eq!(home.lang, "en");

        let dash = after
            .pages
            .iter()
            .find(|p| p.id == "content/x/index.md")
            .expect("em-dash title page must be upserted");
        assert!(dash.stub, "title = \"—\" is a stub");
        assert!(!dash.sitemap, "stubs must be omitted from the sitemap");
        assert!(!dash.thin, "stubs are not thin");

        for p in &after.pages {
            assert!(
                !p.id.starts_with("http://") && !p.id.starts_with("https://"),
                "no http ids: {}",
                p.id
            );
            assert!(!p.id.starts_with("local:"), "never local: ids: {}", p.id);
        }
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_seeds_org_curriculo_not_claims() {
        let root = tmp_root();
        seed_migrated(&root);
        fs::create_dir_all(root.join("content/a")).unwrap();
        fs::write(root.join("content/a/index.md"), "+++\ntitle = \"A\"\n+++\n\nBody.\n").unwrap();
        refresh_with(&root, None, false, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        assert_eq!(after.organizations.len(), 1);
        assert_eq!(after.organizations[0].id, "org:curriculo");
        assert_eq!(after.organizations[0].name, "Curriculo");
        assert!(after.organizations[0].url.is_empty(), "org url is hostless");
        assert!(after.claims.is_empty(), "must not seed Claims");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_sets_translation_of_and_walks_locale_index() {
        let root = tmp_root();
        seed_migrated(&root);
        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(
            root.join("content/_index.md"),
            "+++\ntitle = \"AI ATS\"\n+++\n\n# Home\n\nEnglish home.\n",
        )
        .unwrap();
        fs::write(root.join("content/_index.fr.md"), "+++\ntitle = \"ATS IA\"\n+++\n\nAccueil.\n")
            .unwrap();
        refresh_with(&root, None, false, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        let fr = after
            .pages
            .iter()
            .find(|p| p.id == "content/_index.fr.md")
            .expect("locale section page must be walked");
        assert_eq!(fr.lang, "fr");
        assert_eq!(fr.canonical_path, "/fr/");
        assert_eq!(fr.translation_of.as_deref(), Some("content/_index.md"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_marks_thin_noindex_and_sitemap_false() {
        let root = tmp_root();
        seed_migrated(&root);
        fs::create_dir_all(root.join("content/short")).unwrap();
        fs::write(
            root.join("content/short/index.md"),
            "+++\ntitle = \"Short\"\n[extra]\nnoindex = true\n+++\n\nOnly a few words here.\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("content/hidden")).unwrap();
        fs::write(
            root.join("content/hidden/index.md"),
            "+++\ntitle = \"Hidden\"\n[extra]\nsitemap = false\n+++\n\nAlso a few words only.\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("content/privacy")).unwrap();
        fs::write(
            root.join("content/privacy/index.md"),
            "+++\ntitle = \"Privacy\"\n+++\n\nLegal short page.\n",
        )
        .unwrap();
        refresh_with(&root, None, false, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();

        let short = after.pages.iter().find(|p| p.id == "content/short/index.md").unwrap();
        assert!(short.thin);
        assert!(short.noindex);
        assert!(!short.sitemap, "noindex implies sitemap false");

        let hidden = after.pages.iter().find(|p| p.id == "content/hidden/index.md").unwrap();
        assert!(!hidden.sitemap);
        assert!(!hidden.noindex);
        assert!(hidden.thin);

        let privacy = after.pages.iter().find(|p| p.id == "content/privacy/index.md").unwrap();
        assert!(!privacy.thin, "legal paths are thin-exempt");
        assert!(privacy.sitemap);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_empty_body_is_stub() {
        let root = tmp_root();
        seed_migrated(&root);
        fs::create_dir_all(root.join("content/empty")).unwrap();
        fs::write(root.join("content/empty/index.md"), "+++\ntitle = \"Empty\"\n+++\n\n").unwrap();
        refresh_with(&root, None, false, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        let empty = after.pages.iter().find(|p| p.id == "content/empty/index.md").unwrap();
        assert!(empty.stub);
        assert!(!empty.sitemap);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn extract_h1_prefers_markdown_then_html() {
        assert_eq!(extract_h1("# Hello\n\nbody"), "Hello");
        assert_eq!(extract_h1("## Not h1\n\n<h1>HTML</h1>"), "HTML");
        assert_eq!(extract_h1("no heading"), "");
    }

    #[test]
    fn word_count_strips_html_tags() {
        assert_eq!(word_count_of("<p>one two</p> three"), 3);
    }

    struct ShortThenOk {
        calls: AtomicUsize,
    }
    impl TopicClient for ShortThenOk {
        fn extract(&self, _input: &TopicInput, _key: &str) -> Result<TopicExtract> {
            Ok(TopicExtract::default())
        }
        fn overview(&self, _title: &str, _body: &str, _key: &str) -> Result<String> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst);
            if n == 0 { Ok(words(10)) } else { Ok(words(140)) }
        }
    }

    struct AlwaysShort;
    impl TopicClient for AlwaysShort {
        fn extract(&self, _input: &TopicInput, _key: &str) -> Result<TopicExtract> {
            Ok(TopicExtract::default())
        }
        fn overview(&self, _title: &str, _body: &str, _key: &str) -> Result<String> {
            Ok(words(10))
        }
    }

    #[test]
    fn refresh_overview_retries_then_accepts() {
        let root = tmp_root();
        seed_migrated(&root);
        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(
            root.join("content/_index.md"),
            "+++\ntitle = \"AI ATS\"\n+++\n\n# Home\n\nPillar body.\n",
        )
        .unwrap();
        let client = ShortThenOk { calls: AtomicUsize::new(0) };
        refresh_with(&root, None, false, &client, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        let home = after.pages.iter().find(|p| p.id == "content/_index.md").unwrap();
        let ov = home.overview.as_ref().expect("overview set after retry");
        assert_eq!(ov.split_whitespace().count(), 140);
        assert_eq!(client.calls.load(Ordering::SeqCst), 2);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_prunes_ghost_pages_and_topic_links() {
        let root = tmp_root();
        let ghost = "content/gone/index.md";
        let home = "content/_index.md";
        let topic_id = "gone-topic";
        let store = super::super::schema::GraphStore {
            pages: vec![
                Page {
                    id: home.into(),
                    canonical_path: "/".into(),
                    path: home.into(),
                    title: "Home".into(),
                    summary: "s".into(),
                    content_hash: "h".into(),
                    ..Default::default()
                },
                Page {
                    id: ghost.into(),
                    canonical_path: "/gone/".into(),
                    path: ghost.into(),
                    title: "Gone".into(),
                    summary: "ghost".into(),
                    content_hash: "ghost".into(),
                    topic_ids: vec![topic_id.into()],
                    ..Default::default()
                },
            ],
            topics: vec![super::super::schema::Topic {
                id: topic_id.into(),
                label: "Gone topic".into(),
                aliases: vec![],
                page_ids: vec![ghost.into(), home.into()],
            }],
            relations: vec![super::super::schema::Relation {
                from: ghost.into(),
                to: topic_id.into(),
                kind: "page_topic".into(),
            }],
            meta: super::super::schema::Meta {
                source_origin: "https://x".into(),
                ..Default::default()
            },
            ..Default::default()
        };
        store.save(&root.join("data/graph")).unwrap();
        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(
            root.join("content/_index.md"),
            "+++\ntitle = \"Home\"\n+++\n\n# Home\n\nEnglish home.\n",
        )
        .unwrap();

        refresh_with(&root, None, false, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        assert!(
            after.pages.iter().all(|p| p.id != ghost),
            "ghost page must be dropped"
        );
        assert!(
            !after.relations.iter().any(|r| r.from == ghost || r.to == ghost),
            "ghost page_topic edges must be dropped"
        );
        let topic = after.topics.iter().find(|t| t.id == topic_id).unwrap();
        assert!(
            !topic.page_ids.iter().any(|id| id == ghost),
            "ghost must be removed from topic.page_ids"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_emits_reciprocal_translation_relations() {
        let root = tmp_root();
        seed_migrated(&root);
        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(
            root.join("content/_index.md"),
            "+++\ntitle = \"AI ATS\"\n+++\n\n# Home\n\nEnglish home.\n",
        )
        .unwrap();
        fs::write(root.join("content/_index.fr.md"), "+++\ntitle = \"ATS IA\"\n+++\n\nAccueil.\n")
            .unwrap();
        refresh_with(&root, None, false, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        let en = "content/_index.md";
        let fr = "content/_index.fr.md";
        assert!(after.relations.iter().any(|r| r.kind == "translation" && r.from == fr && r.to == en));
        assert!(after.relations.iter().any(|r| r.kind == "translation" && r.from == en && r.to == fr));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_prunes_ghost_even_when_max_zero() {
        let root = tmp_root();
        let ghost = "content/gone/index.md";
        let store = super::super::schema::GraphStore {
            pages: vec![
                Page {
                    id: "content/_index.md".into(),
                    canonical_path: "/".into(),
                    path: "content/_index.md".into(),
                    title: "Home".into(),
                    summary: "s".into(),
                    content_hash: "old".into(),
                    ..Default::default()
                },
                Page {
                    id: ghost.into(),
                    canonical_path: "/gone/".into(),
                    path: ghost.into(),
                    title: "Gone".into(),
                    summary: "ghost".into(),
                    content_hash: "ghost".into(),
                    ..Default::default()
                },
            ],
            meta: super::super::schema::Meta {
                source_origin: "https://x".into(),
                ..Default::default()
            },
            ..Default::default()
        };
        store.save(&root.join("data/graph")).unwrap();
        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(
            root.join("content/_index.md"),
            "+++\ntitle = \"Home\"\n+++\n\n# Home\n\nFresh body for retopic.\n",
        )
        .unwrap();

        refresh_with(&root, Some(0), false, &FixedTopics, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        assert!(after.pages.iter().all(|p| p.id != ghost));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn refresh_overview_out_of_range_left_unset() {
        let root = tmp_root();
        seed_migrated(&root);
        fs::create_dir_all(root.join("content")).unwrap();
        fs::write(
            root.join("content/_index.md"),
            "+++\ntitle = \"AI ATS\"\n+++\n\n# Home\n\nPillar body.\n",
        )
        .unwrap();
        refresh_with(&root, None, false, &AlwaysShort, "k").unwrap();
        let after = super::super::schema::GraphStore::load(&root.join("data/graph")).unwrap();
        let home = after.pages.iter().find(|p| p.id == "content/_index.md").unwrap();
        assert!(home.overview.is_none(), "must not write a short stub overview");
        fs::remove_dir_all(&root).unwrap();
    }
}
