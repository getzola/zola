//! `zola graph` JSON artifact schema + load/save.
//!
//! Committed under `<site-root>/data/graph/`:
//!
//! ```text
//! pages.json       # url, path, title, summary, content_hash, topic_ids
//! topics.json      # id, label, aliases, page_ids
//! relations.json   # {from, to, kind}  kind ∈ page_topic | topic_topic | page_page
//! meta.json        # source_origin, migrated_at, schema_version, last_refresh
//! ```
//!
//! `meta.source_origin` + non-empty `pages` is the migrate-once lock: a second
//! `graph migrate --from <same origin>` bails unless `--force`. `schema_version`
//! is pinned to 1 for this revision; bump only if the on-disk shape changes.

use std::fs;
use std::path::Path;

use errors::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Pinned on-disk shape. Bump only when the JSON layout changes.
pub const SCHEMA_VERSION: u32 = 1;

/// In-memory graph; serialised to the four `data/graph/*.json` files.
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    pub url: String,
    /// Content-relative path of the written markdown, e.g. `blog/foo/index.md`.
    pub path: String,
    pub title: String,
    pub summary: String,
    /// sha256 over the written markdown body; the refresh staleness key.
    pub content_hash: String,
    #[serde(default)]
    pub topic_ids: Vec<String>,
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
    /// page url (page_*) or topic id (topic_topic "from").
    pub from: String,
    /// topic id (page_topic) / page url (page_page) / topic id (topic_topic "to").
    pub to: String,
    /// `page_topic` | `topic_topic` | `page_page`.
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

impl GraphStore {
    /// Directory of the four JSON files (`data/graph`).
    pub fn load(dir: &Path) -> Result<Self> {
        Ok(GraphStore {
            pages: load_json(&dir.join("pages.json"))?,
            topics: load_json(&dir.join("topics.json"))?,
            relations: load_json(&dir.join("relations.json"))?,
            meta: load_json(&dir.join("meta.json"))?,
        })
    }

    /// Pretty-print all four files, creating `dir` if needed.
    pub fn save(&self, dir: &Path) -> Result<()> {
        fs::create_dir_all(dir)?;
        save_json(&dir.join("pages.json"), &self.pages)?;
        save_json(&dir.join("topics.json"), &self.topics)?;
        save_json(&dir.join("relations.json"), &self.relations)?;
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
                url: "https://x/blog/a".into(),
                path: "blog/a/index.md".into(),
                title: "A".into(),
                summary: "sum".into(),
                content_hash: "deadbeef".into(),
                topic_ids: vec!["t1".into()],
            }],
            topics: vec![Topic {
                id: "t1".into(),
                label: "Hiring".into(),
                aliases: vec!["Recruiting".into()],
                page_ids: vec!["https://x/blog/a".into()],
            }],
            relations: vec![Relation {
                from: "https://x/blog/a".into(),
                to: "t1".into(),
                kind: "page_topic".into(),
            }],
            meta: Meta {
                schema_version: SCHEMA_VERSION,
                source_origin: "https://x".into(),
                migrated_at: "2026-08-11T00:00:00Z".into(),
                last_refresh: String::new(),
            },
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
        // four distinct files written
        for f in ["pages.json", "topics.json", "relations.json", "meta.json"] {
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
}
