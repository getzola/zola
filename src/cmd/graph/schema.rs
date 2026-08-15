//! `zola graph` JSON artifact schema + load/save.
//!
//! Committed under `<site-root>/data/graph/`:
//!
//! ```text
//! pages.json          # id, canonical_path, path, title, … (schema v2)
//! topics.json         # id, label, aliases, page_ids
//! relations.json      # {from, to, kind}
//! persons.json        # Person nodes
//! organizations.json  # Organization nodes
//! claims.json         # Claim nodes
//! meta.json           # source_origin, migrated_at, schema_version, last_refresh
//! ```
//!
//! `meta.source_origin` + non-empty `pages` is the migrate-once lock: a second
//! `graph migrate --from <same origin>` bails unless `--force`. `schema_version`
//! is pinned to 2; [`GraphStore::load`] upgrades on-disk v1 (`url` identity) in
//! memory. Hostful ids are not canonical — ids are content paths.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use errors::{Result, anyhow};
use serde::{Deserialize, Serialize};
use url::Url;

use super::ids::{canonical_path_from_rel, is_hostful_url, page_id_from_rel};

/// Pinned on-disk shape. Bump only when the JSON layout changes.
pub const SCHEMA_VERSION: u32 = 2;

fn default_true() -> bool {
    true
}

/// In-memory graph; serialised to `data/graph/*.json` files.
#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphStore {
    #[serde(default)]
    pub pages: Vec<Page>,
    #[serde(default)]
    pub topics: Vec<Topic>,
    #[serde(default)]
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub persons: Vec<Person>,
    #[serde(default)]
    pub organizations: Vec<Organization>,
    #[serde(default)]
    pub claims: Vec<Claim>,
    #[serde(default)]
    pub meta: Meta,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    pub id: String,             // content/features/index.md
    pub canonical_path: String, // /features/
    pub path: String,           // same as id (keep for refresh walk)
    pub title: String,
    #[serde(default)]
    pub h1: String,
    #[serde(default)]
    pub word_count: u32,
    #[serde(default)]
    pub stub: bool,
    #[serde(default)]
    pub thin: bool,
    #[serde(default)]
    pub noindex: bool,
    #[serde(default = "default_true")]
    pub sitemap: bool,
    #[serde(default)]
    pub lang: String,
    #[serde(default)]
    pub translation_of: Option<String>,
    #[serde(default)]
    pub author: Option<String>, // person id
    #[serde(default)]
    pub date_published: Option<String>,
    #[serde(default)]
    pub date_modified: Option<String>,
    #[serde(default)]
    pub og_image: Option<String>,
    #[serde(default)]
    pub schema_types: Vec<String>,
    #[serde(default)]
    pub overview: Option<String>, // pillars only, 134–167 words
    pub summary: String,
    pub content_hash: String,
    #[serde(default)]
    pub topic_ids: Vec<String>,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            id: String::new(),
            canonical_path: String::new(),
            path: String::new(),
            title: String::new(),
            h1: String::new(),
            word_count: 0,
            stub: false,
            thin: false,
            noindex: false,
            sitemap: true,
            lang: String::new(),
            translation_of: None,
            author: None,
            date_published: None,
            date_modified: None,
            og_image: None,
            schema_types: Vec::new(),
            overview: None,
            summary: String::new(),
            content_hash: String::new(),
            topic_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Person {
    pub id: String, // person:siddharth
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub job_title: String,
    #[serde(default)]
    pub same_as: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organization {
    pub id: String, // org:curriculo
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub logo: String,
    #[serde(default)]
    pub same_as: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,     // claim:trusted-teams
    pub metric: String, // "companies" | "founders" | "rating" | "reviews"
    pub value: String,  // "20" | "200" | "4.9"
    #[serde(default)]
    pub evidence_url: String, // empty => not evidenced
    #[serde(default)]
    pub allows_aggregate_rating: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub page_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation {
    /// page id (page_*) or topic id (topic_topic "from").
    pub from: String,
    /// topic id (page_topic) / page id (related|redirect|…) / topic id (topic_topic "to").
    pub to: String,
    /// `page_topic` | `topic_topic` | `related` | `redirect` | `translation` | `canonical` | `authored_by`.
    /// v1 `page_page` is kept on upgrade until refresh assigns a v2 kind.
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    pub schema_version: u32,
    #[serde(default)]
    pub source_origin: String,
    #[serde(default)]
    pub migrated_at: String,
    #[serde(default)]
    pub last_refresh: String,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            source_origin: String::new(),
            migrated_at: String::new(),
            last_refresh: String::new(),
        }
    }
}

/// On-disk v1 shapes (`Page.url` was the identity). Kept private for upgrade.
mod v1 {
    use super::{Meta, Relation, Topic};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Page {
        pub url: String,
        pub path: String,
        pub title: String,
        pub summary: String,
        pub content_hash: String,
        #[serde(default)]
        pub topic_ids: Vec<String>,
    }

    #[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct GraphStore {
        #[serde(default)]
        pub pages: Vec<Page>,
        #[serde(default)]
        pub topics: Vec<Topic>,
        #[serde(default)]
        pub relations: Vec<Relation>,
        #[serde(default)]
        pub meta: Meta,
    }
}

fn hostful_url_path(raw: &str) -> Option<String> {
    let parsed = Url::parse(raw).ok()?;
    let path = parsed.path();
    if path.is_empty() || path == "/" {
        Some("/".into())
    } else if path.ends_with('/') {
        Some(path.to_string())
    } else {
        Some(format!("{path}/"))
    }
}

/// Rewrite a v1 store (hostful `url` identity) into v2 path ids.
pub fn upgrade_v1(old: v1::GraphStore) -> GraphStore {
    let mut url_to_id: HashMap<String, String> = HashMap::new();
    let pages: Vec<Page> = old
        .pages
        .into_iter()
        .map(|p| {
            let id = page_id_from_rel(&p.path);
            url_to_id.insert(p.url.clone(), id.clone());
            let canonical_path = if is_hostful_url(&p.url) {
                hostful_url_path(&p.url).unwrap_or_else(|| canonical_path_from_rel(&p.path, "en"))
            } else {
                canonical_path_from_rel(&p.path, "en")
            };
            Page {
                id: id.clone(),
                canonical_path,
                path: id,
                title: p.title,
                summary: p.summary,
                content_hash: p.content_hash,
                topic_ids: p.topic_ids,
                ..Default::default()
            }
        })
        .collect();

    let rewrite = |s: &str| url_to_id.get(s).cloned().unwrap_or_else(|| s.to_string());

    let topics: Vec<Topic> = old
        .topics
        .into_iter()
        .map(|mut t| {
            t.page_ids = t.page_ids.iter().map(|s| rewrite(s)).collect();
            t
        })
        .collect();

    let relations: Vec<Relation> = old
        .relations
        .into_iter()
        .map(|mut r| {
            r.from = rewrite(&r.from);
            r.to = rewrite(&r.to);
            r
        })
        .collect();

    let mut meta = old.meta;
    meta.schema_version = SCHEMA_VERSION;

    GraphStore {
        pages,
        topics,
        relations,
        persons: vec![],
        organizations: vec![],
        claims: vec![],
        meta,
    }
}

impl GraphStore {
    /// Directory of the JSON files (`data/graph`).
    pub fn load(dir: &Path) -> Result<Self> {
        let meta: Meta = load_json(&dir.join("meta.json"))?;
        if meta.schema_version <= 1 {
            let old = v1::GraphStore {
                pages: load_json(&dir.join("pages.json"))?,
                topics: load_json(&dir.join("topics.json"))?,
                relations: load_json(&dir.join("relations.json"))?,
                meta,
            };
            return Ok(upgrade_v1(old));
        }
        Ok(GraphStore {
            pages: load_json(&dir.join("pages.json"))?,
            topics: load_json(&dir.join("topics.json"))?,
            relations: load_json(&dir.join("relations.json"))?,
            persons: load_json(&dir.join("persons.json"))?,
            organizations: load_json(&dir.join("organizations.json"))?,
            claims: load_json(&dir.join("claims.json"))?,
            meta,
        })
    }

    /// Pretty-print all graph files, creating `dir` if needed.
    pub fn save(&self, dir: &Path) -> Result<()> {
        fs::create_dir_all(dir)?;
        save_json(&dir.join("pages.json"), &self.pages)?;
        save_json(&dir.join("topics.json"), &self.topics)?;
        save_json(&dir.join("relations.json"), &self.relations)?;
        save_json(&dir.join("persons.json"), &self.persons)?;
        save_json(&dir.join("organizations.json"), &self.organizations)?;
        save_json(&dir.join("claims.json"), &self.claims)?;
        save_json(&dir.join("meta.json"), &self.meta)?;
        Ok(())
    }

    /// True iff this origin was already migrated AND has pages on disk — the
    /// state in which `migrate` must refuse a second crawl without `--force`.
    pub fn is_migrated_for(&self, origin: &str) -> bool {
        !self.pages.is_empty() && self.meta.source_origin == origin
    }
}

fn load_json<T: for<'de> Deserialize<'de> + Default>(path: &Path) -> Result<T> {
    match fs::read_to_string(path) {
        Ok(text) if text.trim().is_empty() => Ok(T::default()),
        Ok(text) => {
            serde_json::from_str(&text).map_err(|e| anyhow!("{}: parse JSON: {e}", path.display()))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(anyhow!("{}: read: {e}", path.display())),
    }
}

fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::graph::ids::is_hostful_url;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    fn tmp_dir() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("zola-graph-schema-{id}-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample() -> GraphStore {
        GraphStore {
            pages: vec![Page {
                id: "content/features/index.md".into(),
                canonical_path: "/features/".into(),
                path: "content/features/index.md".into(),
                title: "A".into(),
                summary: "sum".into(),
                content_hash: "deadbeef".into(),
                topic_ids: vec!["t1".into()],
                ..Default::default()
            }],
            topics: vec![Topic {
                id: "t1".into(),
                label: "Hiring".into(),
                aliases: vec!["Recruiting".into()],
                page_ids: vec!["content/features/index.md".into()],
            }],
            relations: vec![Relation {
                from: "content/features/index.md".into(),
                to: "t1".into(),
                kind: "page_topic".into(),
            }],
            meta: Meta {
                schema_version: SCHEMA_VERSION,
                source_origin: "https://x".into(),
                migrated_at: "2026-08-11T00:00:00Z".into(),
                last_refresh: String::new(),
            },
            ..Default::default()
        }
    }

    #[test]
    fn empty_store_round_trips_and_loads_missing() {
        let dir = tmp_dir();
        let empty = GraphStore::default();
        empty.save(&dir).unwrap();
        let back = GraphStore::load(&dir).unwrap();
        assert_eq!(back, empty);
        assert_eq!(back.meta.schema_version, SCHEMA_VERSION);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn populated_store_round_trips() {
        let dir = tmp_dir();
        let store = sample();
        store.save(&dir).unwrap();
        let back = GraphStore::load(&dir).unwrap();
        assert_eq!(back, store);
        for f in [
            "pages.json",
            "topics.json",
            "relations.json",
            "persons.json",
            "organizations.json",
            "claims.json",
            "meta.json",
        ] {
            assert!(dir.join(f).exists(), "missing {f}");
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_missing_dir_gives_defaults() {
        let dir =
            std::env::temp_dir().join(format!("zola-graph-schema-missing-{}", std::process::id()));
        let back = GraphStore::load(&dir).unwrap();
        assert!(back.pages.is_empty());
        assert_eq!(back.meta.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn is_migrated_for_guard_logic() {
        let mut store = sample();
        assert!(store.is_migrated_for("https://x"), "origin + pages => migrated");
        assert!(!store.is_migrated_for("https://other"), "different origin");
        store.pages.clear();
        assert!(!store.is_migrated_for("https://x"), "empty pages => not migrated");
    }

    #[test]
    fn upgrade_v1_strips_host_and_rewrites_edges() {
        let v1 = v1::GraphStore {
            pages: vec![v1::Page {
                url: "https://curriculo.me/features/".into(),
                path: "content/features/index.md".into(),
                title: "Features".into(),
                summary: "sum".into(),
                content_hash: "deadbeef".into(),
                topic_ids: vec!["t1".into()],
            }],
            topics: vec![Topic {
                id: "t1".into(),
                label: "Hiring".into(),
                aliases: vec![],
                page_ids: vec!["https://curriculo.me/features/".into()],
            }],
            relations: vec![Relation {
                from: "https://curriculo.me/features/".into(),
                to: "t1".into(),
                kind: "page_topic".into(),
            }],
            meta: Meta {
                schema_version: 1,
                source_origin: "https://curriculo.me".into(),
                migrated_at: "2026-08-11T00:00:00Z".into(),
                last_refresh: String::new(),
            },
        };
        let v2 = upgrade_v1(v1);
        assert_eq!(v2.meta.schema_version, 2);
        assert_eq!(v2.pages[0].id, "content/features/index.md");
        assert_eq!(v2.pages[0].canonical_path, "/features/");
        assert!(!is_hostful_url(&v2.pages[0].id));
        assert_eq!(v2.relations[0].from, "content/features/index.md");
    }

    #[test]
    fn load_upgrades_v1_files_on_disk() {
        let dir = tmp_dir();
        fs::write(
            dir.join("pages.json"),
            r#"[{"url":"https://curriculo.me/features/","path":"content/features/index.md","title":"Features","summary":"sum","content_hash":"deadbeef","topic_ids":[]}]"#,
        )
        .unwrap();
        fs::write(dir.join("topics.json"), "[]").unwrap();
        fs::write(
            dir.join("relations.json"),
            r#"[{"from":"https://curriculo.me/features/","to":"t1","kind":"page_topic"}]"#,
        )
        .unwrap();
        fs::write(
            dir.join("meta.json"),
            r#"{"schema_version":1,"source_origin":"https://curriculo.me","migrated_at":"","last_refresh":""}"#,
        )
        .unwrap();
        let v2 = GraphStore::load(&dir).unwrap();
        assert_eq!(v2.meta.schema_version, 2);
        assert_eq!(v2.pages[0].id, "content/features/index.md");
        assert_eq!(v2.pages[0].canonical_path, "/features/");
        assert_eq!(v2.relations[0].from, "content/features/index.md");
        assert!(v2.persons.is_empty());
        fs::remove_dir_all(&dir).unwrap();
    }
}
