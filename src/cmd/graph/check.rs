//! `zola graph check` — offline SEO merge gate over graph JSON + `public/`.
//!
//! No network. Landing CI greps the stable error codes in this module.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use errors::{bail, Result};
use regex::Regex;
use url::Url;

use super::ids::is_hostful_url;
use super::schema::GraphStore;

const PILLAR_HOME: &str = "content/_index.md";
const PILLAR_RESUME: &str = "content/ai-resume-builder/index.md";

static LINK_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?is)<link\b[^>]*>").unwrap());
static SITEMAP_LOC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<loc>\s*([^<\s]+)\s*</loc>").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckError {
    pub code: &'static str,
    pub page_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CheckReport {
    pub errors: Vec<CheckError>,
}

/// Load `<root>/data/graph` and run every offline rule. `public_dir` defaults to
/// `<root>/public` when omitted and `json_only` is false.
pub fn check(root_dir: &Path, public_dir: Option<&Path>, json_only: bool) -> Result<CheckReport> {
    let store = GraphStore::load(&root_dir.join("data/graph"))?;
    let resolved = resolve_public(root_dir, public_dir, json_only);
    Ok(check_at(&store, resolved.as_deref(), json_only, Some(root_dir)))
}

/// In-memory entry used by tests. `_redirects` is not consulted (no site root).
#[cfg(test)]
pub fn check_store(store: &GraphStore, public_dir: Option<&Path>, json_only: bool) -> CheckReport {
    check_at(store, public_dir, json_only, None)
}

/// CLI entry: print errors and fail the process when the report is non-empty.
pub fn run(root_dir: &Path, public_dir: Option<&Path>, json_only: bool) -> Result<()> {
    let report = check(root_dir, public_dir, json_only)?;
    for e in &report.errors {
        match &e.page_id {
            Some(id) => eprintln!("{}: {id}: {}", e.code, e.message),
            None => eprintln!("{}: {}", e.code, e.message),
        }
    }
    if !report.errors.is_empty() {
        bail!("graph check failed with {} error(s)", report.errors.len());
    }
    Ok(())
}

fn resolve_public(root_dir: &Path, public_dir: Option<&Path>, json_only: bool) -> Option<PathBuf> {
    match public_dir {
        Some(p) if p.is_absolute() => Some(p.to_path_buf()),
        Some(p) => Some(root_dir.join(p)),
        None if json_only => None,
        None => {
            let d = root_dir.join("public");
            if d.is_dir() {
                Some(d)
            } else {
                None
            }
        }
    }
}

fn check_at(
    store: &GraphStore,
    public_dir: Option<&Path>,
    json_only: bool,
    root_dir: Option<&Path>,
) -> CheckReport {
    let mut errors = Vec::new();
    check_path_ids(store, &mut errors);
    check_sitemap_flags(store, &mut errors);
    check_thin_published(store, &mut errors);
    check_duplicate_hash(store, &mut errors);
    check_translation_json(store, &mut errors);
    check_missing_person(store, &mut errors);
    check_claim_collision(store, &mut errors);
    check_pillar_split(store, &mut errors);
    check_redirect_about(store, root_dir, &mut errors);
    if !json_only {
        if let Some(public) = public_dir {
            check_hreflang(public, &mut errors);
            check_canonical_host(public, &mut errors);
            check_rating_without_claim(store, public, &mut errors);
        }
    }
    CheckReport { errors }
}

fn push(
    errors: &mut Vec<CheckError>,
    code: &'static str,
    page_id: Option<String>,
    message: impl Into<String>,
) {
    errors.push(CheckError { code, page_id, message: message.into() });
}

fn check_path_ids(store: &GraphStore, errors: &mut Vec<CheckError>) {
    for page in &store.pages {
        let hostful_id = is_hostful_url(&page.id);
        let hostful_path = is_hostful_url(&page.canonical_path);
        let missing_slash = !page.canonical_path.starts_with('/');
        if hostful_id || hostful_path || missing_slash {
            push(
                errors,
                "path_id",
                Some(page.id.clone()),
                "page.id and canonical_path must be hostless; canonical_path must start with /",
            );
        }
    }
}

fn check_sitemap_flags(store: &GraphStore, errors: &mut Vec<CheckError>) {
    for page in &store.pages {
        if page.stub && page.sitemap {
            push(
                errors,
                "stub_in_sitemap",
                Some(page.id.clone()),
                "stub page must not be in the sitemap",
            );
        }
        if page.noindex && page.sitemap {
            push(
                errors,
                "noindex_in_sitemap",
                Some(page.id.clone()),
                "noindex page must not be in the sitemap",
            );
        }
    }
}

fn is_thin_exempt(canonical_path: &str) -> bool {
    canonical_path.starts_with("/privacy")
        || canonical_path.starts_with("/terms")
        || canonical_path.starts_with("/404")
}

fn check_thin_published(store: &GraphStore, errors: &mut Vec<CheckError>) {
    for page in &store.pages {
        if page.thin && page.sitemap && !page.noindex && !is_thin_exempt(&page.canonical_path) {
            push(
                errors,
                "thin_published",
                Some(page.id.clone()),
                "thin page is published in the sitemap without noindex",
            );
        }
    }
}

fn has_rel(store: &GraphStore, kind: &str, from: &str, to: &str) -> bool {
    store.relations.iter().any(|r| r.kind == kind && r.from == from && r.to == to)
}

fn check_duplicate_hash(store: &GraphStore, errors: &mut Vec<CheckError>) {
    let mut by_hash: HashMap<&str, Vec<&super::schema::Page>> = HashMap::new();
    for page in &store.pages {
        if page.sitemap && !page.content_hash.is_empty() {
            by_hash.entry(page.content_hash.as_str()).or_default().push(page);
        }
    }
    for pages in by_hash.values() {
        if pages.len() < 2 {
            continue;
        }
        let preferred = pages
            .iter()
            .find(|p| {
                pages.iter().any(|o| o.id != p.id && has_rel(store, "canonical", &o.id, &p.id))
            })
            .map(|p| p.id.as_str())
            .unwrap_or_else(|| pages.iter().map(|p| p.id.as_str()).min().unwrap_or(""));
        for page in pages {
            if page.id == preferred {
                continue;
            }
            let pointed = pages
                .iter()
                .any(|o| o.id != page.id && has_rel(store, "canonical", &page.id, &o.id));
            if !pointed {
                push(
                    errors,
                    "duplicate_hash",
                    Some(page.id.clone()),
                    format!("sitemap page shares content_hash with {preferred} but has no canonical edge"),
                );
            }
        }
    }
}

fn check_translation_json(store: &GraphStore, errors: &mut Vec<CheckError>) {
    for rel in &store.relations {
        if rel.kind != "translation" {
            continue;
        }
        if !has_rel(store, "translation", &rel.to, &rel.from) {
            push(
                errors,
                "translation_reciprocal",
                Some(rel.from.clone()),
                format!("translation {} → {} has no reverse edge", rel.from, rel.to),
            );
        }
    }
}

fn check_missing_person(store: &GraphStore, errors: &mut Vec<CheckError>) {
    for page in &store.pages {
        let article = page.schema_types.iter().any(|t| t == "Article" || t == "BlogPosting");
        if !article {
            continue;
        }
        let ok = page
            .author
            .as_ref()
            .is_some_and(|a| !a.is_empty() && store.persons.iter().any(|p| p.id == *a));
        if !ok {
            push(
                errors,
                "missing_person",
                Some(page.id.clone()),
                "Article/BlogPosting page has empty author or author not in persons",
            );
        }
    }
}

fn check_claim_collision(store: &GraphStore, errors: &mut Vec<CheckError>) {
    let mut first: HashMap<&str, &str> = HashMap::new();
    for claim in &store.claims {
        match first.get(claim.metric.as_str()) {
            Some(prev) if *prev != claim.value => {
                push(
                    errors,
                    "claim_collision",
                    Some(claim.id.clone()),
                    format!(
                        "claim metric '{}' has conflicting values '{}' and '{}'",
                        claim.metric, prev, claim.value
                    ),
                );
            }
            Some(_) => {}
            None => {
                first.insert(&claim.metric, &claim.value);
            }
        }
    }
}

fn check_pillar_split(store: &GraphStore, errors: &mut Vec<CheckError>) {
    let split = store.relations.iter().any(|r| {
        r.kind == "related"
            && ((r.from == PILLAR_HOME && r.to == PILLAR_RESUME)
                || (r.from == PILLAR_RESUME && r.to == PILLAR_HOME))
    });
    if split {
        push(
            errors,
            "pillar_split",
            Some(PILLAR_HOME.into()),
            "related edge between homepage and /ai-resume-builder/ is forbidden",
        );
    }
}

fn page_canonical(store: &GraphStore, id: &str) -> String {
    store
        .pages
        .iter()
        .find(|p| p.id == id)
        .map(|p| norm_path(&p.canonical_path))
        .unwrap_or_else(|| norm_path(id))
}

fn json_has_about_redirect(store: &GraphStore) -> bool {
    store.relations.iter().any(|r| {
        r.kind == "redirect"
            && page_canonical(store, &r.from) == "/about/"
            && page_canonical(store, &r.to) == "/about-us/"
    })
}

fn static_has_about_redirect(root: &Path) -> bool {
    let text = match fs::read_to_string(root.join("static/_redirects")) {
        Ok(t) => t,
        Err(_) => return false,
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(src) = parts.next() else { continue };
        let Some(dst) = parts.next() else { continue };
        if norm_path(src) == "/about/" && norm_path(dst) == "/about-us/" {
            return true;
        }
    }
    false
}

fn check_redirect_about(store: &GraphStore, root_dir: Option<&Path>, errors: &mut Vec<CheckError>) {
    if json_has_about_redirect(store) {
        return;
    }
    if root_dir.is_some_and(static_has_about_redirect) {
        return;
    }
    push(
        errors,
        "redirect_about",
        None,
        "missing redirect /about/ → /about-us/ (JSON edge or static/_redirects)",
    );
}

fn check_hreflang(public: &Path, errors: &mut Vec<CheckError>) {
    let mut files = Vec::new();
    walk_html(public, &mut files);
    let mut alts: HashMap<String, Vec<String>> = HashMap::new();
    for path in &files {
        let Ok(html) = fs::read_to_string(path) else { continue };
        let identity = page_identity(path, public, &html);
        let mut hrefs = Vec::new();
        for tag in LINK_TAG.find_iter(&html) {
            let t = tag.as_str();
            if !rel_is(t, "alternate") {
                continue;
            }
            if attr_value(t, "hreflang").is_none() {
                continue;
            }
            if let Some(href) = attr_value(t, "href") {
                hrefs.push(norm_href(&href));
            }
        }
        alts.insert(identity, hrefs);
    }
    for (from, hrefs) in &alts {
        for to in hrefs {
            if to == from {
                continue;
            }
            let reverse = alts.get(to).is_some_and(|hs| hs.iter().any(|h| h == from));
            if !reverse {
                push(
                    errors,
                    "translation_reciprocal",
                    Some(from.clone()),
                    format!("hreflang {from} → {to} has no reverse in public/"),
                );
            }
        }
    }
}

fn check_canonical_host(public: &Path, errors: &mut Vec<CheckError>) {
    let Some(expected) = sitemap_host(public) else { return };
    let mut files = Vec::new();
    walk_html(public, &mut files);
    for path in &files {
        let Ok(html) = fs::read_to_string(path) else { continue };
        for tag in LINK_TAG.find_iter(&html) {
            let t = tag.as_str();
            if !rel_is(t, "canonical") {
                continue;
            }
            let Some(href) = attr_value(t, "href") else { continue };
            let Ok(url) = Url::parse(&href) else { continue };
            let Some(host) = url.host_str() else { continue };
            if host.eq_ignore_ascii_case(&expected) {
                continue;
            }
            let rel = path.strip_prefix(public).unwrap_or(path);
            push(
                errors,
                "canonical_host",
                Some(rel.display().to_string()),
                format!("canonical host '{host}' != sitemap host '{expected}'"),
            );
        }
    }
}

fn check_rating_without_claim(store: &GraphStore, public: &Path, errors: &mut Vec<CheckError>) {
    let evidenced =
        store.claims.iter().any(|c| c.allows_aggregate_rating && !c.evidence_url.is_empty());
    if evidenced {
        return;
    }
    let mut files = Vec::new();
    walk_html(public, &mut files);
    let found = files
        .iter()
        .any(|f| fs::read_to_string(f).map(|t| t.contains("AggregateRating")).unwrap_or(false));
    if found {
        push(
            errors,
            "rating_without_claim",
            None,
            "public/ JSON-LD contains AggregateRating but no evidenced Claim allows it",
        );
    }
}

fn sitemap_host(public: &Path) -> Option<String> {
    let text = fs::read_to_string(public.join("sitemap.xml")).ok()?;
    let loc = SITEMAP_LOC.captures(&text)?.get(1)?.as_str().trim();
    Url::parse(loc).ok()?.host_str().map(|h| h.to_ascii_lowercase())
}

fn page_identity(path: &Path, public: &Path, html: &str) -> String {
    for tag in LINK_TAG.find_iter(html) {
        let t = tag.as_str();
        if rel_is(t, "canonical") {
            if let Some(href) = attr_value(t, "href") {
                return norm_href(&href);
            }
        }
    }
    let rel = path.strip_prefix(public).unwrap_or(path);
    let mut s = rel.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = s.strip_suffix("index.html") {
        s = stripped.to_string();
    } else if let Some(stripped) = s.strip_suffix(".html") {
        s = stripped.to_string();
    }
    if !s.starts_with('/') {
        s.insert(0, '/');
    }
    if !s.ends_with('/') {
        s.push('/');
    }
    s
}

fn walk_html(dir: &Path, out: &mut Vec<PathBuf>) {
    let rd = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return,
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_html(&path, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("html"))
        {
            out.push(path);
        }
    }
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let needle = format!("{}=", name.to_ascii_lowercase());
    let idx = lower.find(&needle)?;
    let rest = tag[idx + needle.len()..].trim_start();
    if rest.is_empty() {
        return None;
    }
    let mut chars = rest.chars();
    match chars.next()? {
        q @ ('"' | '\'') => Some(chars.as_str().split(q).next().unwrap_or("").to_string()),
        _ => Some(
            rest.split(|c: char| c.is_whitespace() || c == '>').next().unwrap_or("").to_string(),
        ),
    }
}

fn rel_is(tag: &str, want: &str) -> bool {
    attr_value(tag, "rel").is_some_and(|v| v.eq_ignore_ascii_case(want))
}

fn norm_path(p: &str) -> String {
    let p = p.trim();
    if p.is_empty() {
        return String::new();
    }
    let mut s = if is_hostful_url(p) {
        Url::parse(p).ok().map(|u| u.path().to_string()).unwrap_or_else(|| p.to_string())
    } else {
        p.to_string()
    };
    if !s.starts_with('/') {
        s.insert(0, '/');
    }
    if s != "/" && !s.ends_with('/') {
        s.push('/');
    }
    s
}

fn norm_href(s: &str) -> String {
    let s = s.trim();
    if let Ok(u) = Url::parse(s) {
        let host = u.host_str().unwrap_or("").to_ascii_lowercase();
        let mut path = u.path().to_string();
        if path.is_empty() {
            path = "/".into();
        }
        if path != "/" && !path.ends_with('/') {
            path.push('/');
        }
        if host.is_empty() {
            path
        } else {
            format!("{}://{host}{path}", u.scheme())
        }
    } else {
        norm_path(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::graph::schema::{Claim, GraphStore, Page, Person, Relation};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    fn tmp_dir() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("zola-graph-check-{id}-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn has_code(report: &CheckReport, code: &str) -> bool {
        report.errors.iter().any(|e| e.code == code)
    }

    fn v2() -> GraphStore {
        let mut store = GraphStore::default();
        store.meta.schema_version = 2;
        store
    }

    #[test]
    fn path_id_rejects_hostful_page_id() {
        let mut store = v2();
        store.pages.push(Page {
            id: "https://curriculo.me/features/".into(),
            canonical_path: "/features/".into(),
            ..Default::default()
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "path_id"));
    }

    #[test]
    fn claim_collision_fails() {
        let mut store = v2();
        store.claims.push(Claim {
            id: "claim:founders-20".into(),
            metric: "founders".into(),
            value: "20".into(),
            evidence_url: String::new(),
            allows_aggregate_rating: false,
        });
        store.claims.push(Claim {
            id: "claim:founders-200".into(),
            metric: "founders".into(),
            value: "200".into(),
            evidence_url: String::new(),
            allows_aggregate_rating: false,
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "claim_collision"));
    }

    #[test]
    fn rating_without_claim_fails_on_html() {
        let store = v2();
        let root = tmp_dir();
        let public = root.join("public");
        fs::create_dir_all(&public).unwrap();
        fs::write(
            public.join("index.html"),
            r#"<script type="application/ld+json">{"@type":"AggregateRating","ratingValue":"5"}</script>"#,
        )
        .unwrap();
        let report = check_store(&store, Some(&public), false);
        assert!(has_code(&report, "rating_without_claim"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn path_id_rejects_canonical_without_slash() {
        let mut store = v2();
        store.pages.push(Page {
            id: "content/features/index.md".into(),
            canonical_path: "features/".into(),
            ..Default::default()
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "path_id"));
    }

    #[test]
    fn stub_in_sitemap_fails() {
        let mut store = v2();
        store.pages.push(Page {
            id: "content/x/index.md".into(),
            canonical_path: "/x/".into(),
            stub: true,
            sitemap: true,
            ..Default::default()
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "stub_in_sitemap"));
    }

    #[test]
    fn noindex_in_sitemap_fails() {
        let mut store = v2();
        store.pages.push(Page {
            id: "content/blogs/l005-pipeline-smoke-test/index.md".into(),
            canonical_path: "/blogs/l005-pipeline-smoke-test/".into(),
            noindex: true,
            sitemap: true,
            ..Default::default()
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "noindex_in_sitemap"));
    }

    #[test]
    fn thin_published_fails_on_non_exempt() {
        let mut store = v2();
        store.pages.push(Page {
            id: "content/thin/index.md".into(),
            canonical_path: "/thin/".into(),
            thin: true,
            sitemap: true,
            noindex: false,
            ..Default::default()
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "thin_published"));
    }

    #[test]
    fn thin_published_exempts_privacy() {
        let mut store = v2();
        store.pages.push(Page {
            id: "content/privacy/index.md".into(),
            canonical_path: "/privacy/".into(),
            thin: true,
            sitemap: true,
            ..Default::default()
        });
        let report = check_store(&store, None, true);
        assert!(!has_code(&report, "thin_published"));
    }

    #[test]
    fn duplicate_hash_fails_without_canonical() {
        let mut store = v2();
        store.pages.push(Page {
            id: "content/a/index.md".into(),
            canonical_path: "/a/".into(),
            sitemap: true,
            content_hash: "abc".into(),
            ..Default::default()
        });
        store.pages.push(Page {
            id: "content/b/index.md".into(),
            canonical_path: "/b/".into(),
            sitemap: true,
            content_hash: "abc".into(),
            ..Default::default()
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "duplicate_hash"));
    }

    #[test]
    fn duplicate_hash_ok_with_canonical_edge() {
        let mut store = v2();
        store.pages.push(Page {
            id: "content/a/index.md".into(),
            canonical_path: "/a/".into(),
            sitemap: true,
            content_hash: "abc".into(),
            ..Default::default()
        });
        store.pages.push(Page {
            id: "content/b/index.md".into(),
            canonical_path: "/b/".into(),
            sitemap: true,
            content_hash: "abc".into(),
            ..Default::default()
        });
        store.relations.push(Relation {
            from: "content/b/index.md".into(),
            to: "content/a/index.md".into(),
            kind: "canonical".into(),
        });
        let report = check_store(&store, None, true);
        assert!(!has_code(&report, "duplicate_hash"));
    }

    #[test]
    fn translation_reciprocal_fails_one_way() {
        let mut store = v2();
        store.relations.push(Relation {
            from: "content/_index.md".into(),
            to: "content/_index.fr.md".into(),
            kind: "translation".into(),
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "translation_reciprocal"));
    }

    #[test]
    fn missing_person_fails_without_author() {
        let mut store = v2();
        store.pages.push(Page {
            id: "content/blogs/x/index.md".into(),
            canonical_path: "/blogs/x/".into(),
            schema_types: vec!["Article".into()],
            author: None,
            ..Default::default()
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "missing_person"));
    }

    #[test]
    fn missing_person_fails_unknown_author() {
        let mut store = v2();
        store.pages.push(Page {
            id: "content/blogs/x/index.md".into(),
            canonical_path: "/blogs/x/".into(),
            schema_types: vec!["BlogPosting".into()],
            author: Some("person:ghost".into()),
            ..Default::default()
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "missing_person"));
    }

    #[test]
    fn missing_person_ok_when_person_exists() {
        let mut store = v2();
        store.pages.push(Page {
            id: "content/blogs/x/index.md".into(),
            canonical_path: "/blogs/x/".into(),
            schema_types: vec!["Article".into()],
            author: Some("person:curriculo-editorial".into()),
            ..Default::default()
        });
        store.persons.push(Person {
            id: "person:curriculo-editorial".into(),
            name: "Curriculo".into(),
            url: "/about-us/".into(),
            job_title: "Editorial".into(),
            same_as: vec![],
        });
        let report = check_store(&store, None, true);
        assert!(!has_code(&report, "missing_person"));
    }

    #[test]
    fn pillar_split_fails() {
        let mut store = v2();
        store.relations.push(Relation {
            from: "content/_index.md".into(),
            to: "content/ai-resume-builder/index.md".into(),
            kind: "related".into(),
        });
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "pillar_split"));
    }

    #[test]
    fn redirect_about_fails_when_missing() {
        let store = v2();
        let report = check_store(&store, None, true);
        assert!(has_code(&report, "redirect_about"));
    }

    #[test]
    fn redirect_about_ok_with_json_edge() {
        let mut store = v2();
        store.relations.push(Relation {
            from: "/about/".into(),
            to: "/about-us/".into(),
            kind: "redirect".into(),
        });
        let report = check_store(&store, None, true);
        assert!(!has_code(&report, "redirect_about"));
    }

    #[test]
    fn redirect_about_ok_with_static_file() {
        let store = v2();
        let root = tmp_dir();
        store.save(&root.join("data/graph")).unwrap();
        fs::create_dir_all(root.join("static")).unwrap();
        fs::write(root.join("static/_redirects"), "/about/    /about-us/     301\n").unwrap();
        let report = check(&root, None, true).unwrap();
        assert!(!has_code(&report, "redirect_about"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rating_without_claim_skipped_when_json_only() {
        let store = v2();
        let root = tmp_dir();
        let public = root.join("public");
        fs::create_dir_all(&public).unwrap();
        fs::write(public.join("index.html"), "AggregateRating").unwrap();
        let report = check_store(&store, Some(&public), true);
        assert!(!has_code(&report, "rating_without_claim"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rating_ok_with_evidenced_claim() {
        let mut store = v2();
        store.claims.push(Claim {
            id: "claim:rating".into(),
            metric: "rating".into(),
            value: "4.9".into(),
            evidence_url: "https://example.com/reviews".into(),
            allows_aggregate_rating: true,
        });
        let root = tmp_dir();
        let public = root.join("public");
        fs::create_dir_all(&public).unwrap();
        fs::write(public.join("index.html"), "AggregateRating").unwrap();
        let report = check_store(&store, Some(&public), false);
        assert!(!has_code(&report, "rating_without_claim"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn canonical_host_fails_on_unquoted_mismatch() {
        let store = v2();
        let root = tmp_dir();
        let public = root.join("public");
        fs::create_dir_all(&public).unwrap();
        fs::write(
            public.join("sitemap.xml"),
            r#"<urlset><url><loc>https://curriculo-me.pages.dev/</loc></url></urlset>"#,
        )
        .unwrap();
        fs::write(public.join("index.html"), r#"<link rel=canonical href=https://curriculo.me/>"#)
            .unwrap();
        let report = check_store(&store, Some(&public), false);
        assert!(has_code(&report, "canonical_host"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn canonical_host_ok_when_quoted_matches_sitemap() {
        let store = v2();
        let root = tmp_dir();
        let public = root.join("public");
        fs::create_dir_all(&public).unwrap();
        fs::write(
            public.join("sitemap.xml"),
            r#"<urlset><url><loc>https://curriculo-me.pages.dev/</loc></url></urlset>"#,
        )
        .unwrap();
        fs::write(
            public.join("index.html"),
            r#"<link rel="canonical" href="https://curriculo-me.pages.dev/">"#,
        )
        .unwrap();
        let report = check_store(&store, Some(&public), false);
        assert!(!has_code(&report, "canonical_host"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn translation_reciprocal_hreflang_missing_reverse() {
        let store = v2();
        let root = tmp_dir();
        let public = root.join("public");
        fs::create_dir_all(public.join("fr")).unwrap();
        fs::write(
            public.join("index.html"),
            r#"<link rel="alternate" hreflang="fr" href="https://x.test/fr/">
               <link rel="canonical" href="https://x.test/">"#,
        )
        .unwrap();
        fs::write(
            public.join("fr/index.html"),
            r#"<link rel="canonical" href="https://x.test/fr/">"#,
        )
        .unwrap();
        let report = check_store(&store, Some(&public), false);
        assert!(has_code(&report, "translation_reciprocal"));
        fs::remove_dir_all(&root).unwrap();
    }
}
