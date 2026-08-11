//! `zola graph` — build and maintain a topical knowledge graph.
//!
//! Two subcommands (see `cli.rs::GraphCommand`):
//!
//! - `graph migrate --from <origin>`: **once-per-origin**. Fetches the origin's
//!   sitemap, crawls each page via Firecrawl (**the only Firecrawl entrypoint**),
//!   writes `content/<slug>/index.md`, enriches topics via OpenRouter, and
//!   commits `data/graph/*.json`. Refuses a second crawl for the same origin
//!   unless `--force`.
//! - `graph refresh`: **local only**. Walks default-language markdown, re-topics
//!   pages whose `content_hash` changed, updates `meta.last_refresh`. Never
//!   imports the firecrawl module.
//!
//! Network lives only in migrate/refresh — `zola build` stays offline.

pub mod clean;
pub mod firecrawl;
pub mod html_to_md;
pub mod migrate;
pub mod openrouter;
pub mod refresh;
pub mod schema;
pub mod sitemap;
pub mod topics;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use errors::{Result, anyhow, bail};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use toml::Value;
use url::Url;

use crate::cli::GraphCommand;

const FM_DELIM: &str = "+++";

/// Dispatch entry from `main.rs`.
pub fn run(root_dir: &Path, config_file: &Path, command: GraphCommand) -> Result<()> {
    match command {
        GraphCommand::Migrate { from, max, dry_run, force } => {
            migrate::migrate(root_dir, config_file, &from, max, dry_run, force)
        }
        GraphCommand::Refresh { max, dry_run } => {
            refresh::refresh(root_dir, config_file, max, dry_run)
        }
    }
}

// ---- shared markdown helpers (used by migrate writer + refresh reader) ----

/// sha256 hex of a markdown body — the page staleness key.
fn content_hash(body: &str) -> String {
    let mut h = Sha256::new();
    h.update(body.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// RFC3339 UTC timestamp, empty on format failure.
fn now_iso() -> String {
    OffsetDateTime::now_utc().format(&Rfc3339).unwrap_or_default()
}

/// `(default_language, sorted non-default languages)` from a Zola config file.
fn read_langs(config_file: &Path) -> Result<(String, Vec<String>)> {
    let text = fs::read_to_string(config_file).map_err(|e| anyhow!("read config: {e}"))?;
    let cfg: Value = toml::from_str(&text).map_err(|e| anyhow!("parse config: {e}"))?;
    let default = cfg.get("default_language").and_then(|v| v.as_str()).unwrap_or("en").to_string();
    let mut langs: Vec<String> = cfg
        .get("languages")
        .and_then(|v| v.as_table())
        .map(|t| t.keys().filter(|k| *k != &default).cloned().collect())
        .unwrap_or_default();
    langs.sort();
    Ok((default, langs))
}

/// Recursively collect `*.md` files under `dir` (missing dir → empty).
fn walk_md(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let rd = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(anyhow!("read dir {}: {e}", dir.display())),
    };
    for entry in rd {
        let path = entry?.path();
        if path.is_dir() {
            walk_md(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

/// True if `file_name` is a default-language, non-section page (mirror of
/// `translate.rs`).
fn is_default_page(file_name: &str, lang_set: &HashSet<&str>) -> bool {
    let Some(stem) = file_name.strip_suffix(".md") else {
        return false;
    };
    if stem.starts_with("_index") {
        return false;
    }
    !lang_set.iter().any(|l| stem.ends_with(&format!(".{l}")))
}

/// Split a `+++`-delimited page into (frontmatter, body markdown).
fn parse_page(path: &Path) -> Result<(Value, String)> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let first = lines.next().unwrap_or("");
    if first.trim() != FM_DELIM {
        bail!(
            "{}: expected `{}` frontmatter (TOML). `zola graph` handles `+++` pages only.",
            path.display(),
            FM_DELIM
        );
    }
    let mut fm_buf = String::new();
    let mut body_buf = String::new();
    let mut closed = false;
    for line in lines {
        if !closed {
            if line.trim() == FM_DELIM {
                closed = true;
            } else {
                fm_buf.push_str(line);
                fm_buf.push('\n');
            }
        } else {
            body_buf.push_str(line);
            body_buf.push('\n');
        }
    }
    if !closed {
        bail!("{}: frontmatter not terminated by `{}`", path.display(), FM_DELIM);
    }
    let fm: Value = toml::from_str(&fm_buf)
        .map_err(|e| anyhow!("{}: frontmatter parse: {e}", path.display()))?;
    Ok((fm, body_buf))
}

/// Write a `+++`-delimited page with the given frontmatter + body.
fn write_page(path: &Path, fm_str: &str, body: &str) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, format!("+++\n{fm_str}+++\n\n{}\n", body.trim_end()))?;
    Ok(())
}

/// `content/<slug>/index.md` relative to root for a fetched URL. Homepage →
/// `content/home/index.md`. ponytail: ignores query string; trailing slashes
/// collapsed. Ceiling = same URL at ?x=1 and ?x=2 collide — fine for graph.
fn url_to_content_path(raw_url: &str) -> Result<String> {
    let parsed = Url::parse(raw_url)?;
    let mut segments: Vec<&str> =
        parsed.path().trim_end_matches('/').split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        segments.push("home");
    }
    Ok(format!("content/{}/index.md", segments.join("/")))
}

/// Plain-text summary from a markdown body: first ~160 chars of prose.
fn summarize(body: &str) -> String {
    let clean: String = body
        .lines()
        .map(|l| l.trim_start_matches('#').trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    clean.chars().take(160).collect()
}
