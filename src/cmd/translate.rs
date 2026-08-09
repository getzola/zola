//! `zola translate` — generate co-located translation siblings via OpenRouter.
//!
//! For each default-language page that has body content, for each non-default
//! language in config.toml: the sibling `<slug>.<lang>.md` is *fresh* iff its
//! `extra.source_hash` equals sha256 of the default page's translatable fields
//! (title, description, body). Missing/stale siblings are (re)generated through
//! OpenRouter, written with `extra.source_hash` set, and brand-token-gated
//! (INV-5: a glossary token present in the source must survive verbatim, else
//! the file is NOT written and counts as a failure).
//!
//! Network lives ONLY here — `zola build` stays offline. The LLM call is behind
//! the [`LlmClient`] trait so unit tests mock it without touching the network.
//!
//! Field/transport contract ported from landing-website
//! `backend/cms/openrouter.py` (ADR-003: `openai/gpt-4o-mini`, JSON-object
//! response, system-prompt → keys). Fields are the Zola page set
//! {title, description, body}, not the CMS's five-field set.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use errors::{Result, anyhow, bail};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
/// Umbrella ADR-003: gpt-4o-mini only, no local models.
const MODEL: &str = "openai/gpt-4o-mini";
/// INV-5: brand tokens that must survive translation verbatim when present in source.
const GLOSSARY: &[&str] = &["Curriculo", "CurriculoATS"];
const MAX_TOKENS: u32 = 16384;
const FM_DELIM: &str = "+++";

/// Target language code → human name for the system prompt.
fn lang_name(code: &str) -> Option<&'static str> {
    Some(match code {
        "es" => "Spanish",
        "pt" => "Portuguese",
        "zh" => "Chinese (Simplified)",
        "ko" => "Korean",
        "ja" => "Japanese",
        "fr" => "French",
        "ar" => "Arabic",
        _ => return None,
    })
}

/// Translatable fields of a default-language page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Translatable {
    pub title: String,
    pub description: String,
    pub body: String,
}

/// LLM translation client. A trait so unit tests inject a mock without a network.
pub trait LlmClient {
    fn translate(&self, fields: &Translatable, lang: &str, key: &str) -> Result<Translatable>;
}

/// Live OpenRouter client (blocking reqwest).
pub struct OpenRouterClient;

impl LlmClient for OpenRouterClient {
    fn translate(&self, fields: &Translatable, lang: &str, key: &str) -> Result<Translatable> {
        let name =
            lang_name(lang).ok_or_else(|| anyhow!("unsupported target language {lang:?}"))?;
        let payload = json!({
            "model": MODEL,
            "response_format": {"type": "json_object"},
            "max_tokens": MAX_TOKENS,
            "messages": [
                {"role": "system", "content": format!(
                    "You translate marketing web content into {name}. Translate ONLY human-readable text. \
                    Preserve these brand tokens verbatim, untranslated: {}. \
                    Return a single JSON object with exactly these keys: title, description, body. No prose.",
                    GLOSSARY.join(", ")
                )},
                {"role": "user", "content": json!({
                    "title": fields.title, "description": fields.description, "body": fields.body
                }).to_string()},
            ],
        });
        let body = serde_json::to_vec(&payload)?;
        let resp = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()?
            .post(OPENROUTER_URL)
            .bearer_auth(key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()?;
        let status = resp.status();
        let text = resp.text()?;
        if !status.is_success() {
            bail!("OpenRouter HTTP {status}: {}", take200(&text));
        }
        let data: Value =
            serde_json::from_str(&text).map_err(|e| anyhow!("OpenRouter non-JSON response: {e}"))?;
        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("OpenRouter: unexpected shape: {}", take160(&data.to_string())))?;
        let out: Value = serde_json::from_str(content).map_err(|_| {
            anyhow!("model returned non-JSON (likely truncated at output cap): {}", take160(content))
        })?;
        Ok(Translatable {
            title: out["title"].as_str().unwrap_or("").to_string(),
            description: out["description"].as_str().unwrap_or("").to_string(),
            body: out["body"].as_str().unwrap_or("").to_string(),
        })
    }
}

/// sha256 over the translatable fields, NUL-separated for an unambiguous boundary.
pub fn source_hash(t: &Translatable) -> String {
    let mut h = Sha256::new();
    h.update(t.title.as_bytes());
    h.update(b"\x00");
    h.update(t.description.as_bytes());
    h.update(b"\x00");
    h.update(t.body.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// INV-5 post-check: every glossary token in the source must appear verbatim in
/// the translation. Failure ⇒ the caller must NOT write the file.
pub fn glossary_ok(en: &Translatable, t: &Translatable) -> Result<()> {
    let en_blob = format!("{} {} {}", en.title, en.description, en.body);
    let t_blob = format!("{} {} {}", t.title, t.description, t.body);
    for token in GLOSSARY {
        if en_blob.contains(token) && !t_blob.contains(token) {
            bail!("glossary: brand token {token:?} lost in translation");
        }
    }
    Ok(())
}

/// Entry point from `main.rs`. Reads `OPENROUTER_API_KEY` (fail-fast when absent
/// and not a dry-run), then delegates to [`translate_with`].
pub fn translate(root_dir: &Path, config_file: &Path, max: Option<usize>, dry_run: bool) -> Result<()> {
    let key = if dry_run {
        String::new()
    } else {
        env::var("OPENROUTER_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("OPENROUTER_API_KEY not set — translate needs it"))?
    };
    translate_with(root_dir, config_file, max, dry_run, &key, &OpenRouterClient)
}

/// Testable core; takes the key and LLM client as parameters so tests inject a
/// mock without a network or a real key.
pub fn translate_with<C: LlmClient>(
    root_dir: &Path,
    config_file: &Path,
    max: Option<usize>,
    dry_run: bool,
    key: &str,
    client: &C,
) -> Result<()> {
    let (_default_lang, langs) = read_langs(config_file)?;
    if langs.is_empty() {
        log::info!("translate: no non-default languages configured; nothing to do");
        return Ok(());
    }
    let lang_set: HashSet<&str> = langs.iter().map(|s| s.as_str()).collect();

    let content_dir = root_dir.join("content");
    let mut pages: Vec<PathBuf> = Vec::new();
    walk_md(&content_dir, &mut pages)?;
    pages.sort();

    let cap = max.unwrap_or(usize::MAX);
    let mut calls = 0usize; // API calls made this run (success + fail)
    let mut failures = 0usize;
    let mut written = 0usize;
    let mut skipped_fresh = 0usize;
    let mut skipped_cap = 0usize;

    for page in &pages {
        // Only default-language, non-section pages.
        let name = page.file_name().unwrap().to_string_lossy().into_owned();
        if !is_default_page(&name, &lang_set) {
            continue;
        }
        let (en_fm, en_body) = match parse_page(page) {
            Ok(v) => v,
            Err(e) => {
                failures += 1;
                log::error!("translate: {}: parse failed: {e}", page.display());
                continue;
            }
        };
        let en = Translatable {
            title: get_str(&en_fm, "title"),
            description: get_str(&en_fm, "description"),
            body: en_body.trim().to_string(),
        };
        // ponytail: nothing meaningful to translate without a body (title-only
        // stubs). They get picked up once real content is authored.
        if en.body.is_empty() {
            continue;
        }
        let hash = source_hash(&en);

        for lang in &langs {
            let sibling = sibling_path(page, lang);
            if let Ok((sib_fm, _)) = parse_page(&sibling) {
                if extra_hash(&sib_fm).as_deref() == Some(hash.as_str()) {
                    skipped_fresh += 1;
                    continue; // fresh — skip
                }
            }
            // missing or stale
            if dry_run {
                log::info!("translate [dry-run]: {} → {lang} (stale/missing)", page.display());
                continue;
            }
            if calls >= cap {
                skipped_cap += 1;
                continue; // --max reached; resumes next run
            }
            calls += 1;
            match client
                .translate(&en, lang, &key)
                .and_then(|t| { glossary_ok(&en, &t)?; Ok(t) })
            {
                Ok(t) => {
                    write_sibling(&sibling, &en_fm, &t, &hash)?;
                    written += 1;
                    log::info!("translate: {} → {lang}", page.display());
                }
                Err(e) => {
                    failures += 1;
                    log::error!("translate: {} → {lang} FAILED: {e}", page.display());
                    // file intentionally NOT written on failure
                }
            }
        }
    }

    log::info!(
        "translate: written={written} fresh={skipped_fresh} capped={skipped_cap} calls={calls} failures={failures}"
    );
    if failures > 0 {
        bail!("translate completed with {failures} failure(s)");
    }
    Ok(())
}

// ---- helpers ----

fn read_langs(config_file: &Path) -> Result<(String, Vec<String>)> {
    let text =
        fs::read_to_string(config_file).map_err(|e| anyhow!("read config: {e}"))?;
    let cfg: toml::Value = toml::from_str(&text).map_err(|e| anyhow!("parse config: {e}"))?;
    let default = cfg
        .get("default_language")
        .and_then(|v| v.as_str())
        .unwrap_or("en")
        .to_string();
    let mut langs: Vec<String> = cfg
        .get("languages")
        .and_then(|v| v.as_table())
        .map(|t| t.keys().filter(|k| *k != &default).cloned().collect())
        .unwrap_or_default();
    langs.sort();
    Ok((default, langs))
}

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

/// True if `file_name` (`index.md`, `index.es.md`, `_index.md`, …) is a
/// default-language page: not a section, not a per-language translation.
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
fn parse_page(path: &Path) -> Result<(toml::Value, String)> {
    let text = fs::read_to_string(path)?;
    let mut lines = text.lines();
    let first = lines.next().unwrap_or("");
    if first.trim() != FM_DELIM {
        bail!(
            "{}: expected `{}` frontmatter (TOML). `zola translate` handles `+++` pages only.",
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
    let fm: toml::Value =
        toml::from_str(&fm_buf).map_err(|e| anyhow!("{}: frontmatter parse: {e}", path.display()))?;
    Ok((fm, body_buf))
}

fn get_str(fm: &toml::Value, key: &str) -> String {
    fm.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

fn extra_hash(fm: &toml::Value) -> Option<String> {
    Some(fm.get("extra")?.get("source_hash")?.as_str()?.to_string())
}

/// `<dir>/<stem>.<lang>.md` sibling of a default page `<dir>/<stem>.md`.
fn sibling_path(page: &Path, lang: &str) -> PathBuf {
    let stem = page.file_stem().unwrap().to_string_lossy().into_owned();
    page.with_file_name(format!("{stem}.{lang}.md"))
}

fn write_sibling(path: &Path, en_fm: &toml::Value, t: &Translatable, hash: &str) -> Result<()> {
    let mut fm = en_fm.clone();
    if let Some(table) = fm.as_table_mut() {
        table.insert("title".into(), toml::Value::String(t.title.clone()));
        table.insert("description".into(), toml::Value::String(t.description.clone()));
        let extra = table
            .entry("extra")
            .or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
        if let Some(et) = extra.as_table_mut() {
            et.insert("source_hash".into(), toml::Value::String(hash.to_string()));
        }
    }
    let fm_str = toml::to_string(&fm).map_err(|e| anyhow!("serialize frontmatter: {e}"))?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, format!("+++\n{fm_str}+++\n\n{}\n", t.body.trim_end()))?;
    Ok(())
}

fn take200(s: &str) -> String {
    s.chars().take(200).collect()
}
fn take160(s: &str) -> String {
    s.chars().take(160).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    /// Unique temp dir per test (no tempfile dep): lives under std temp.
    struct Fixture {
        root: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
            let root = env::temp_dir().join(format!("zola-translate-test-{id}-{}", process_id()));
            fs::create_dir_all(&root).unwrap();
            // config.toml: default en + es, fr
            fs::write(
                root.join("config.toml"),
                "base_url = \"https://x/\"\ndefault_language = \"en\"\n[languages.es]\n[languages.fr]\n",
            )
            .unwrap();
            Fixture { root }
        }
        fn write_page(&self, rel: &str, fm: &str, body: &str) {
            let p = self.root.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, format!("+++\n{fm}+++\n\n{body}")).unwrap();
        }
        fn page_body(&self, rel: &str) -> String {
            fs::read_to_string(self.root.join(rel)).unwrap()
        }
        fn config(&self) -> PathBuf {
            self.root.join("config.toml")
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

    #[cfg(not(target_pointer_width = "32"))]
    fn process_id() -> usize {
        std::process::id() as usize
    }
    #[cfg(target_pointer_width = "32")]
    fn process_id() -> usize {
        std::process::id()
    }

    /// Mock that prepends "[lang]" and counts calls via interior mutability.
    struct EchoClient {
        calls: Cell<usize>,
    }
    impl LlmClient for EchoClient {
        fn translate(&self, f: &Translatable, lang: &str, _key: &str) -> Result<Translatable> {
            self.calls.set(self.calls.get() + 1);
            Ok(Translatable {
                title: format!("[{lang}] {}", f.title),
                description: format!("[{lang}] {}", f.description),
                body: format!("[{lang}] {}", f.body),
            })
        }
    }

    /// Mock that drops the "Curriculo" token → glossary must reject.
    struct DroppingClient;
    impl LlmClient for DroppingClient {
        fn translate(&self, f: &Translatable, _lang: &str, _key: &str) -> Result<Translatable> {
            Ok(Translatable {
                title: f.title.replace("Curriculo", "Brand"),
                description: f.description.replace("Curriculo", "Brand"),
                body: f.body.replace("Curriculo", "Brand"),
            })
        }
    }

    /// Mock that always fails.
    struct FailClient;
    impl LlmClient for FailClient {
        fn translate(&self, _: &Translatable, _: &str, _: &str) -> Result<Translatable> {
            bail!("boom")
        }
    }

    fn en_page_fm() -> &'static str {
        "title = \"Hello Curriculo\"\ndescription = \"A desc\"\n[date]\ntaxonomies = {tags = [\"x\"]}\n"
    }

    #[test]
    fn hash_gate_fresh_skip_and_stale_regen() {
        let fx = Fixture::new();
        fx.write_page("content/post/index.md", en_page_fm(), "Body one.\n");

        let c = EchoClient { calls: Cell::new(0) };
        // run 1: both langs stale → 2 calls, both written
        translate_with(fx.root(), &fx.config(), None, false, "test-key", &c).unwrap();
        assert_eq!(c.calls.get(), 2);
        let es = fx.page_body("content/post/index.es.md");
        assert!(es.contains("[es] Hello Curriculo"));
        assert!(es.contains("source_hash"));
        assert!(es.contains("Curriculo")); // brand preserved by EchoClient

        // run 2: source unchanged → both fresh → no calls
        translate_with(fx.root(), &fx.config(), None, false, "test-key", &c).unwrap();
        assert_eq!(c.calls.get(), 2, "fresh siblings must not re-translate");

        // mutate en body → both stale → 2 more calls
        fx.write_page("content/post/index.md", en_page_fm(), "Body two.\n");
        translate_with(fx.root(), &fx.config(), None, false, "test-key", &c).unwrap();
        assert_eq!(c.calls.get(), 4, "stale siblings must regenerate");
    }

    #[test]
    fn glossary_rejection_does_not_write() {
        let fx = Fixture::new();
        fx.write_page("content/post/index.md", en_page_fm(), "Curriculo body.\n");

        let res = translate_with(fx.root(), &fx.config(), None, false, "test-key", &DroppingClient);
        assert!(res.is_err(), "glossary failure must surface as non-zero exit");
        assert!(!fx.root().join("content/post/index.es.md").exists(), "file must NOT be written");
        assert!(!fx.root().join("content/post/index.fr.md").exists());
    }

    #[test]
    fn max_cap_resumes_next_run() {
        let fx = Fixture::new();
        fx.write_page("content/post/index.md", en_page_fm(), "Body.\n");

        // cap at 1 call: only one of {es, fr} translated
        let c1 = EchoClient { calls: Cell::new(0) };
        translate_with(fx.root(), &fx.config(), Some(1), false, "test-key", &c1).unwrap();
        assert_eq!(c1.calls.get(), 1);
        let one_written = fx.root().join("content/post/index.es.md").exists()
            ^ fx.root().join("content/post/index.fr.md").exists();
        assert!(one_written, "exactly one sibling written under --max 1");

        // resume: the remaining lang is still missing → translated now
        let c2 = EchoClient { calls: Cell::new(0) };
        translate_with(fx.root(), &fx.config(), None, false, "test-key", &c2).unwrap();
        assert_eq!(c2.calls.get(), 1, "only the previously-capped lang remains");
        assert!(fx.root().join("content/post/index.es.md").exists());
        assert!(fx.root().join("content/post/index.fr.md").exists());
    }

    #[test]
    fn non_zero_exit_on_failures() {
        let fx = Fixture::new();
        fx.write_page("content/post/index.md", en_page_fm(), "Body.\n");
        let res = translate_with(fx.root(), &fx.config(), None, false, "test-key", &FailClient);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("failure"));
    }

    #[test]
    fn empty_body_skipped_no_key_needed_for_dry_run() {
        let fx = Fixture::new();
        // title-only page: empty body → skipped by translate
        fx.write_page("content/stub/index.md", "title = \"Stub\"\n", "\n");
        let c = EchoClient { calls: Cell::new(0) };
        // dry_run, no key in env for this process slice: must NOT error
        translate_with(fx.root(), &fx.config(), None, true, "test-key", &c).unwrap();
        assert_eq!(c.calls.get(), 0);
        assert!(!fx.root().join("content/stub/index.es.md").exists());
    }

    #[test]
    fn source_hash_is_deterministic_and_field_scoped() {
        let a = Translatable { title: "t".into(), description: "d".into(), body: "b".into() };
        let b = Translatable { title: "t".into(), description: "d".into(), body: "b".into() };
        let c = Translatable { title: "t".into(), description: "d".into(), body: "B".into() };
        assert_eq!(source_hash(&a), source_hash(&b));
        assert_ne!(source_hash(&a), source_hash(&c));
    }

    #[test]
    fn glossary_passes_when_token_preserved() {
        let en = Translatable {
            title: "Welcome to Curriculo".into(),
            description: "".into(),
            body: "".into(),
        };
        let ok = Translatable { title: "Bienvenue sur Curriculo".into(), ..en.clone() };
        let lost = Translatable { title: "Bienvenue sur Brand".into(), ..en.clone() };
        assert!(glossary_ok(&en, &ok).is_ok());
        assert!(glossary_ok(&en, &lost).is_err());
    }
}
