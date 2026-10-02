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
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
/// Umbrella ADR-003: gpt-4o-mini only, no local models.
const MODEL: &str = "openai/gpt-4o-mini";
/// INV-5: brand tokens that must survive translation verbatim when present in source.
const GLOSSARY: &[&str] = &["Curriculo", "CurriculoATS"];
const MAX_TOKENS: u32 = 16384;
/// Bodies larger than this are translated in H2-sized chunks so each OpenRouter
/// call stays under the JSON output cap (the old 113 KB pillar hit truncation).
const BODY_CHUNK_CHARS: usize = 6_000;
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

/// Live OpenRouter client (blocking reqwest). Pool built once per run.
pub struct OpenRouterClient {
    client: reqwest::blocking::Client,
}

impl OpenRouterClient {
    pub fn new() -> Result<Self> {
        Ok(Self { client: http_client(Duration::from_secs(120))? })
    }
}

impl Default for OpenRouterClient {
    fn default() -> Self {
        Self::new().expect("reqwest client with default TLS")
    }
}

impl LlmClient for OpenRouterClient {
    fn translate(&self, fields: &Translatable, lang: &str, key: &str) -> Result<Translatable> {
        translate_fields_openrouter(&self.client, fields, lang, key)
    }
}

/// Which of the three fields were actually sent, in request order. Empty fields
/// are not sent at all: a body-only chunk would otherwise spend an engine call
/// translating two empty strings.
type SentFields = Vec<&'static str>;

/// Client for a self-hosted translation endpoint.
///
/// Selected by setting `TRANSLATE_URL`; when it is unset the OpenRouter client
/// above is used and behaviour is unchanged. Deliberately knows nothing about
/// any particular engine or deployment — it speaks one small JSON contract:
///
/// ```text
/// POST $TRANSLATE_URL
///   {"texts": ["..."], "target_language": "ko", "source_language": "en",
///    "preserve_terms": ["Acme Widgets"]}
/// → {"translations": ["..."]}
/// ```
///
/// Translations come back in request order, one per input. `preserve_terms`
/// (#23 review) carries the caller's own untranslatables — brand and product
/// names that must survive verbatim — so the endpoint masks them out of the
/// engine and restores them after (curriculo-ai #1023/#1133). Sourced from
/// `TRANSLATE_PRESERVE_TERMS` (comma-separated) via [`preserve_terms_from_env`],
/// and only included in the body when non-empty.
///
/// Note the endpoint ignores terms shorter than 3 characters, and the INV-5
/// [`glossary_ok`] gate still checks only the built-in [`GLOSSARY`] — caller
/// terms are trusted to the endpoint's mask/restore machinery.
///
/// Failure envelopes are hard failures, never passthroughs: the endpoint may
/// answer HTTP 200 with `"ok": false` (or a per-string `"ok"` array with any
/// false) or a mirrored error code (e.g. `"code": 1003`), with `translations`
/// carrying the ORIGINAL source text. Writing that would stamp English content
/// as a fresh translation, so any of those shapes is an error and the sibling
/// file is not written.
pub struct TranslateApiClient {
    url: String,
    /// Built once per run and reused across chunks/requests: a blocking client
    /// owns a connection pool, so rebuilding it per chunk throws the pool (and
    /// keep-alive) away on every page.
    client: reqwest::blocking::Client,
    /// Caller-supplied untranslatables sent as `preserve_terms` on every
    /// request. Empty means the field is omitted entirely.
    preserve_terms: Vec<String>,
}

impl TranslateApiClient {
    pub fn new(
        url: impl Into<String>,
        timeout: Duration,
        preserve_terms: Vec<String>,
    ) -> Result<Self> {
        Ok(Self { url: url.into(), client: http_client(timeout)?, preserve_terms })
    }
}

/// Shared client constructor. A generous timeout by default: a self-hosted CPU
/// model is far slower than a hosted LLM, and a page body is the whole request.
/// Override with `TRANSLATE_TIMEOUT` (seconds, floor 30 — see [`timeout_from_env`]).
fn http_client(timeout: Duration) -> Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder().timeout(timeout).build()?)
}

/// `TRANSLATE_TIMEOUT` in seconds: default 300, clamped to a ≥30s floor (a
/// typo like `3` would otherwise time out every real request). Non-numeric
/// values are a config error and fail loudly rather than silently defaulting.
fn timeout_from_env() -> Result<Duration> {
    const DEFAULT_SECS: u64 = 300;
    const MIN_SECS: u64 = 30;
    let Ok(raw) = env::var("TRANSLATE_TIMEOUT") else {
        return Ok(Duration::from_secs(DEFAULT_SECS));
    };
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(Duration::from_secs(DEFAULT_SECS));
    }
    let secs: u64 = raw
        .parse()
        .map_err(|_| anyhow!("TRANSLATE_TIMEOUT must be a number of seconds, got {raw:?}"))?;
    if secs < MIN_SECS {
        log::warn!(
            "translate: TRANSLATE_TIMEOUT={secs}s below the {MIN_SECS}s floor; using {MIN_SECS}s"
        );
        return Ok(Duration::from_secs(MIN_SECS));
    }
    Ok(Duration::from_secs(secs))
}

/// `TRANSLATE_PRESERVE_TERMS`: comma-separated terms the caller needs back
/// verbatim (its own brand/product names), forwarded to the endpoint as
/// `preserve_terms`. Blank segments are dropped, everything else is kept
/// verbatim after a trim — the endpoint does its own cap/filtering (100 terms
/// of ≤100 chars, terms under 3 chars ignored), so this side stays dumb.
fn preserve_terms_from_env() -> Vec<String> {
    env::var("TRANSLATE_PRESERVE_TERMS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

impl LlmClient for TranslateApiClient {
    fn translate(&self, fields: &Translatable, lang: &str, _key: &str) -> Result<Translatable> {
        // Chunking lives inside the client, as it does for OpenRouter: the
        // caller hands the whole body over in one piece, so without this a
        // pillar page is a single enormous request. `body_only` is already
        // implied here — build_translate_request skips empty fields.
        translate_fields_impl(|f, _body_only| self.translate_once(f, lang), fields)
    }
}

impl TranslateApiClient {
    fn translate_once(&self, fields: &Translatable, lang: &str) -> Result<Translatable> {
        let (payload, sent) = build_translate_request(fields, lang, &self.preserve_terms);
        if sent.is_empty() {
            return Ok(fields.clone());
        }
        let resp = self
            .client
            .post(&self.url)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_vec(&payload)?)
            .send()?;
        let status = resp.status();
        let text = resp.text()?;
        if !status.is_success() {
            bail!("translate endpoint HTTP {status}: {}", take200(&text));
        }
        parse_translate_response(&text, &sent, fields)
    }
}

/// Build the request body, and record which fields it carries so the response
/// can be mapped back positionally. `preserve_terms` is included only when
/// non-empty: an empty array is the server default (curriculo-ai #1023), and
/// omitting it keeps the body byte-identical for callers with no glossary.
fn build_translate_request(
    fields: &Translatable,
    lang: &str,
    preserve_terms: &[String],
) -> (Value, SentFields) {
    let mut texts: Vec<&str> = Vec::new();
    let mut sent: SentFields = Vec::new();
    for (name, value) in
        [("title", &fields.title), ("description", &fields.description), ("body", &fields.body)]
    {
        if !value.is_empty() {
            texts.push(value.as_str());
            sent.push(name);
        }
    }
    let mut payload = json!({
        "texts": texts,
        "target_language": lang,
        "source_language": "en",
    });
    if !preserve_terms.is_empty() {
        payload["preserve_terms"] = json!(preserve_terms);
    }
    (payload, sent)
}

/// Map `translations` back onto the fields that were sent. A field that was not
/// sent keeps its original (empty) value.
fn parse_translate_response(
    text: &str,
    sent: &SentFields,
    fields: &Translatable,
) -> Result<Translatable> {
    let data: Value = serde_json::from_str(text)
        .map_err(|e| anyhow!("translate endpoint non-JSON response: {e}"))?;
    // #1003-style failure envelopes arrive as HTTP 200 with the ORIGINAL source
    // text in `translations`. Any of them means "not translated" — writing the
    // file anyway would mark English content as a fresh translation, so they are
    // hard errors. `ok` may be a bool, a per-string bool array (false = that
    // string came back as source), or null/absent on older deployments.
    if let Some(ok) = data.get("ok") {
        match ok {
            Value::Bool(false) => {
                bail!(
                    "translate endpoint reported ok:false — source text returned untranslated, not writing"
                )
            }
            Value::Array(flags) => {
                if let Some(i) = flags.iter().position(|f| !f.is_boolean() || !f.as_bool().unwrap())
                {
                    let field = sent.get(i).copied().unwrap_or("?");
                    bail!(
                        "translate endpoint: ok[{i}]=false — `{field}` came back untranslated, not writing"
                    );
                }
            }
            _ => {}
        }
    }
    // Some gateways mirror the error status in the body of a 200 response
    // (e.g. {"code": 1003}). 0 and 200 mean success; anything else does not.
    for key in ["code", "status", "error_code"] {
        let code = data
            .get(key)
            .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())));
        if let Some(code) = code {
            if code != 0 && code != 200 {
                bail!("translate endpoint error code {code} in `{key}` — not writing");
            }
        }
    }
    let arr = data["translations"].as_array().ok_or_else(|| {
        anyhow!("translate endpoint: no `translations` array: {}", take160(&data.to_string()))
    })?;
    // A short array would silently shift every field onto the wrong key, so the
    // length is a hard error rather than something to paper over.
    if arr.len() != sent.len() {
        bail!(
            "translate endpoint returned {} translation(s) for {} text(s)",
            arr.len(),
            sent.len()
        );
    }
    let mut out =
        Translatable { title: String::new(), description: String::new(), body: String::new() };
    for (name, value) in sent.iter().zip(arr) {
        // A non-string entry (null/list/object) used to coerce to "" and write
        // a blank title/description/body. Skip-with-warning is not an option
        // here: there is nothing sane to write for the field, so the request
        // fails and the existing sibling (if any) is left untouched.
        let s = value.as_str().ok_or_else(|| {
            anyhow!("translate endpoint: value for `{name}` is not a string ({}) — not writing blanks", json_kind(value))
        })?.to_string();
        match *name {
            "title" => out.title = s,
            "description" => out.description = s,
            _ => out.body = s,
        }
    }
    // Fields we never sent were empty on the way in; keep them empty on the way
    // out rather than inventing content.
    let _ = fields;
    Ok(out)
}

fn json_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// One OpenRouter JSON-object call for a (possibly partial) page payload.
fn openrouter_translate_once(
    client: &reqwest::blocking::Client,
    fields: &Translatable,
    lang: &str,
    key: &str,
    body_only: bool,
) -> Result<Translatable> {
    let name = lang_name(lang).ok_or_else(|| anyhow!("unsupported target language {lang:?}"))?;
    let system = if body_only {
        format!(
            "You translate marketing web page BODY markdown into {name}. Translate ONLY human-readable text. \
            Preserve these brand tokens verbatim, untranslated: {}. \
            Return a single JSON object with exactly these keys: title, description, body. \
            Leave title and description as empty strings. No prose.",
            GLOSSARY.join(", ")
        )
    } else {
        format!(
            "You translate marketing web content into {name}. Translate ONLY human-readable text. \
            Preserve these brand tokens verbatim, untranslated: {}. \
            Return a single JSON object with exactly these keys: title, description, body. No prose.",
            GLOSSARY.join(", ")
        )
    };
    let user_title = if body_only { "" } else { fields.title.as_str() };
    let user_desc = if body_only { "" } else { fields.description.as_str() };
    let payload = json!({
        "model": MODEL,
        "response_format": {"type": "json_object"},
        "max_tokens": MAX_TOKENS,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": json!({
                "title": user_title, "description": user_desc, "body": fields.body
            }).to_string()},
        ],
    });
    let body = serde_json::to_vec(&payload)?;
    let resp = client
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

/// Split a long markdown body on H2 boundaries for chunked translation.
fn chunk_body(body: &str) -> Vec<String> {
    let body = body.trim();
    if body.is_empty() || body.len() <= BODY_CHUNK_CHARS {
        return vec![body.to_string()];
    }
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    for (i, line) in body.lines().enumerate() {
        let is_h2 = line.starts_with("## ");
        if is_h2 && i > 0 && !current.is_empty() && current.len() >= BODY_CHUNK_CHARS / 2 {
            chunks.push(current.trim_end().to_string());
            current.clear();
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(line);
        if current.len() >= BODY_CHUNK_CHARS {
            chunks.push(current.trim_end().to_string());
            current.clear();
        }
    }
    if !current.is_empty() {
        chunks.push(current.trim_end().to_string());
    }
    if chunks.is_empty() { vec![body.to_string()] } else { chunks }
}

fn translate_fields_openrouter(
    client: &reqwest::blocking::Client,
    fields: &Translatable,
    lang: &str,
    key: &str,
) -> Result<Translatable> {
    translate_fields_impl(
        |f, body_only| openrouter_translate_once(client, f, lang, key, body_only),
        fields,
    )
}

/// Translate title/description once; chunk the body when it exceeds [`BODY_CHUNK_CHARS`].
fn translate_fields<C: LlmClient>(
    client: &C,
    fields: &Translatable,
    lang: &str,
    key: &str,
) -> Result<Translatable> {
    translate_fields_impl(
        |f, body_only| {
            let payload = if body_only {
                Translatable {
                    title: String::new(),
                    description: String::new(),
                    body: f.body.clone(),
                }
            } else {
                f.clone()
            };
            client.translate(&payload, lang, key)
        },
        fields,
    )
}

fn translate_fields_impl<F>(mut translate_one: F, fields: &Translatable) -> Result<Translatable>
where
    F: FnMut(&Translatable, bool) -> Result<Translatable>,
{
    let chunks = chunk_body(&fields.body);
    if chunks.len() == 1 {
        return translate_one(fields, false);
    }
    let head = Translatable {
        title: fields.title.clone(),
        description: fields.description.clone(),
        body: chunks[0].clone(),
    };
    let first = translate_one(&head, false)?;
    let mut body_out = first.body;
    for chunk in chunks.iter().skip(1) {
        let partial =
            Translatable { title: String::new(), description: String::new(), body: chunk.clone() };
        let t = translate_one(&partial, true)?;
        if !body_out.is_empty() && !t.body.is_empty() {
            body_out.push_str("\n\n");
        }
        body_out.push_str(&t.body);
    }
    Ok(Translatable { title: first.title, description: first.description, body: body_out })
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
///
/// Opt-in alternative: when `TRANSLATE_URL` is set the call is served by
/// [`TranslateApiClient`] instead and no API key is read. Unset — which is the
/// default for every existing site — and nothing below this block changes.
pub fn translate(
    root_dir: &Path,
    config_file: &Path,
    max: Option<usize>,
    dry_run: bool,
) -> Result<()> {
    if let Some(url) = env::var("TRANSLATE_URL").ok().filter(|s| !s.is_empty()) {
        let timeout = timeout_from_env()?;
        let preserve_terms = preserve_terms_from_env();
        log::info!(
            "translate: using translation endpoint at {url} (timeout {timeout:?}, {} preserve term(s)",
            preserve_terms.len()
        );
        let client = TranslateApiClient::new(url, timeout, preserve_terms)?;
        return translate_with(root_dir, config_file, max, dry_run, "", &client);
    }
    let key = if dry_run {
        String::new()
    } else {
        env::var("OPENROUTER_API_KEY")
            .ok()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| anyhow!("OPENROUTER_API_KEY not set — translate needs it"))?
    };
    translate_with(root_dir, config_file, max, dry_run, &key, &OpenRouterClient::new()?)
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
            match client.translate(&en, lang, &key).and_then(|t| {
                glossary_ok(&en, &t)?;
                Ok(t)
            }) {
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
    let text = fs::read_to_string(config_file).map_err(|e| anyhow!("read config: {e}"))?;
    let cfg: toml::Value = toml::from_str(&text).map_err(|e| anyhow!("parse config: {e}"))?;
    let default = cfg.get("default_language").and_then(|v| v.as_str()).unwrap_or("en").to_string();
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
    let fm: toml::Value = toml::from_str(&fm_buf)
        .map_err(|e| anyhow!("{}: frontmatter parse: {e}", path.display()))?;
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
        let extra =
            table.entry("extra").or_insert_with(|| toml::Value::Table(toml::value::Table::new()));
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
    use std::sync::{Arc, Mutex};

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
    // ── TranslateApiClient wire format ────────────────────────────────

    fn t(title: &str, description: &str, body: &str) -> Translatable {
        Translatable { title: title.into(), description: description.into(), body: body.into() }
    }

    #[test]
    fn request_carries_every_non_empty_field_in_order() {
        let (payload, sent) = build_translate_request(&t("T", "D", "B"), "ko", &[]);
        assert_eq!(sent, vec!["title", "description", "body"]);
        assert_eq!(payload["texts"], json!(["T", "D", "B"]));
        assert_eq!(payload["target_language"], "ko");
        assert_eq!(payload["source_language"], "en");
        // Empty glossary ⇒ field omitted, body byte-identical to pre-#23-review
        // shape (an empty array is the server default, curriculo-ai #1023).
        assert!(payload.get("preserve_terms").is_none());
    }

    #[test]
    fn request_carries_preserve_terms_when_supplied() {
        // #23 review / curriculo-ai #1023: the caller's untranslatables ride the
        // same request as `texts`, so the endpoint can mask them out of the
        // engine and restore them verbatim.
        let terms = vec!["Acme Widgets".to_string(), "Zephyr Analytics".to_string()];
        let (payload, _) = build_translate_request(&t("T", "D", "B"), "ko", &terms);
        assert_eq!(payload["preserve_terms"], json!(terms));
        assert_eq!(payload["target_language"], "ko");
    }

    #[test]
    fn body_only_chunk_sends_only_the_body() {
        // translate_fields blanks title/description for continuation chunks;
        // sending those empty strings would spend an engine call on nothing.
        let (payload, sent) = build_translate_request(&t("", "", "B"), "ja", &[]);
        assert_eq!(sent, vec!["body"]);
        assert_eq!(payload["texts"], json!(["B"]));
    }

    #[test]
    fn response_maps_back_onto_the_fields_that_were_sent() {
        let (_, sent) = build_translate_request(&t("", "", "B"), "ja", &[]);
        let out = parse_translate_response(r#"{"translations":["本文"]}"#, &sent, &t("", "", "B"))
            .unwrap();
        assert_eq!(out.body, "本文");
        assert!(out.title.is_empty() && out.description.is_empty());
    }

    #[test]
    fn short_response_is_an_error_not_a_silent_shift() {
        // Two translations for three texts would otherwise land the body on the
        // description key and write a plausible-looking, wrong page.
        let (_, sent) = build_translate_request(&t("T", "D", "B"), "fr", &[]);
        let err = parse_translate_response(
            r#"{"translations":["Titre","Description"]}"#,
            &sent,
            &t("T", "D", "B"),
        )
        .unwrap_err();
        assert!(format!("{err}").contains("2 translation(s) for 3"));
    }

    #[test]
    fn long_body_is_chunked_by_the_endpoint_client_too() {
        // Regression: chunking lives inside each client impl, so a client that
        // skips it sends a whole pillar page as one request. Counts the calls
        // translate_fields_impl makes for an over-cap body.
        let body = (0..40)
            .map(|i| format!("## Heading {i}\n\n{}", "word ".repeat(120)))
            .collect::<Vec<_>>()
            .join("\n\n");
        assert!(body.len() > BODY_CHUNK_CHARS, "fixture must exceed the cap");
        let mut calls = 0usize;
        let out = translate_fields_impl(
            |f, _body_only| {
                calls += 1;
                Ok(f.clone())
            },
            &t("T", "D", &body),
        )
        .unwrap();
        assert!(calls > 1, "expected the body to be chunked, got {calls} call(s)");
        assert_eq!(out.title, "T");
    }

    #[test]
    fn missing_translations_array_is_an_error() {
        let (_, sent) = build_translate_request(&t("T", "", ""), "fr", &[]);
        let err = parse_translate_response(r#"{"oops":true}"#, &sent, &t("T", "", "")).unwrap_err();
        assert!(format!("{err}").contains("translations"));
    }

    #[test]
    fn ok_false_is_a_hard_failure_not_source_passthrough() {
        // #1003: HTTP 200 + ok:false means `translations` carries the ORIGINAL
        // source text. Accepting it would write English into `<slug>.ko.md`
        // stamped fresh (source_hash set) — exactly the bug this guards.
        let (_, sent) = build_translate_request(&t("T", "D", "B"), "ko", &[]);
        let err = parse_translate_response(
            r#"{"ok":false,"translations":["T","D","B"]}"#,
            &sent,
            &t("T", "D", "B"),
        )
        .unwrap_err();
        assert!(format!("{err}").contains("ok:false"), "got: {err}");
    }

    #[test]
    fn ok_array_with_a_false_flag_is_a_hard_failure() {
        let (_, sent) = build_translate_request(&t("T", "", "B"), "ko", &[]);
        let err = parse_translate_response(
            r#"{"ok":[true,false],"translations":["T","B"]}"#,
            &sent,
            &t("T", "", "B"),
        )
        .unwrap_err();
        assert!(format!("{err}").contains("ok[1]=false"), "got: {err}");
        assert!(format!("{err}").contains("`body`"), "names the failed field: {err}");
    }

    #[test]
    fn ok_all_true_or_absent_still_passes() {
        // Older deployments send no `ok` at all; newer ones send all-true.
        // Both must keep working — guard against over-tightening the #1003 fix.
        let (_, sent) = build_translate_request(&t("", "", "B"), "ja", &[]);
        for body in [r#"{"ok":[true],"translations":["本文"]}"#, r#"{"translations":["本文"]}"#]
        {
            let out = parse_translate_response(body, &sent, &t("", "", "B")).unwrap();
            assert_eq!(out.body, "本文");
        }
    }

    #[test]
    fn mirrored_error_code_1003_is_a_hard_failure() {
        let (_, sent) = build_translate_request(&t("T", "", ""), "fr", &[]);
        for body in [
            r#"{"code":1003,"translations":["T"]}"#,
            r#"{"status":1003,"translations":["T"]}"#,
            r#"{"error_code":"1003","translations":["T"]}"#,
        ] {
            let err = parse_translate_response(body, &sent, &t("T", "", "")).unwrap_err();
            assert!(format!("{err}").contains("1003"), "body {body} -> {err}");
        }
    }

    #[test]
    fn non_string_translation_is_an_error_not_a_blank() {
        let (_, sent) = build_translate_request(&t("T", "D", "B"), "ko", &[]);
        for body in [
            r#"{"translations":[null,"D","B"]}"#,
            r#"{"translations":[["T"],"D","B"]}"#,
            r#"{"translations":[{"v":"T"},"D","B"]}"#,
        ] {
            let err = parse_translate_response(body, &sent, &t("T", "D", "B")).unwrap_err();
            assert!(format!("{err}").contains("`title` is not a string"), "body {body} -> {err}");
        }
    }

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn translate_timeout_env_default_floor_and_garbage() {
        let _g = ENV_LOCK.lock().unwrap();
        unsafe { env::remove_var("TRANSLATE_TIMEOUT") };
        assert_eq!(timeout_from_env().unwrap(), Duration::from_secs(300));
        unsafe { env::set_var("TRANSLATE_TIMEOUT", "900") };
        assert_eq!(timeout_from_env().unwrap(), Duration::from_secs(900));
        unsafe { env::set_var("TRANSLATE_TIMEOUT", "3") };
        assert_eq!(timeout_from_env().unwrap(), Duration::from_secs(30));
        unsafe { env::set_var("TRANSLATE_TIMEOUT", "  ") };
        assert_eq!(timeout_from_env().unwrap(), Duration::from_secs(300));
        unsafe { env::set_var("TRANSLATE_TIMEOUT", "soon") };
        assert!(timeout_from_env().is_err());
        unsafe { env::remove_var("TRANSLATE_TIMEOUT") };
    }

    #[test]
    fn preserve_terms_env_parses_trims_and_drops_blanks() {
        let _g = ENV_LOCK.lock().unwrap();
        unsafe { env::remove_var("TRANSLATE_PRESERVE_TERMS") };
        assert!(preserve_terms_from_env().is_empty());
        unsafe { env::set_var("TRANSLATE_PRESERVE_TERMS", " Acme Widgets ,, Zephyr ") };
        assert_eq!(
            preserve_terms_from_env(),
            vec!["Acme Widgets".to_string(), "Zephyr".to_string()]
        );
        unsafe { env::set_var("TRANSLATE_PRESERVE_TERMS", " , , ") };
        assert!(preserve_terms_from_env().is_empty());
        unsafe { env::remove_var("TRANSLATE_PRESERVE_TERMS") };
    }

    /// Minimal keep-alive HTTP/1.1 server for exercising [`TranslateApiClient`]
    /// over a real socket. Serves one JSON body per request, built from the
    /// request's `texts` length; counts sockets and requests so tests can
    /// assert on connection reuse.
    struct TinyServer {
        url: String,
        conns: Arc<AtomicUsize>,
        reqs: Arc<AtomicUsize>,
    }

    impl TinyServer {
        fn start(make_body: fn(usize) -> String) -> Self {
            use std::io::Write;
            use std::net::TcpListener;
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let conns = Arc::new(AtomicUsize::new(0));
            let reqs = Arc::new(AtomicUsize::new(0));
            let (c2, r2) = (conns.clone(), reqs.clone());
            std::thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    let mut stream = stream;
                    c2.fetch_add(1, Ordering::SeqCst);
                    while let Some(body) = read_request_body(&mut stream) {
                        let n = serde_json::from_str::<Value>(&body)
                            .ok()
                            .and_then(|v| v["texts"].as_array().map(|a| a.len()))
                            .unwrap_or(0);
                        r2.fetch_add(1, Ordering::SeqCst);
                        let out = make_body(n);
                        let resp = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{}",
                            out.len(),
                            out
                        );
                        if stream.write_all(resp.as_bytes()).is_err() {
                            break;
                        }
                    }
                }
            });
            TinyServer { url: format!("http://{addr}/translate"), conns, reqs }
        }
    }

    /// Read one request (headers + Content-Length body) off the stream; None on EOF.
    fn read_request_body(stream: &mut std::net::TcpStream) -> Option<String> {
        use std::io::Read;
        let mut buf: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 8192];
        let hdr_end = loop {
            if let Some(p) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break p + 4;
            }
            let n = stream.read(&mut chunk).ok()?;
            if n == 0 {
                return None;
            }
            buf.extend_from_slice(&chunk[..n]);
        };
        let headers = String::from_utf8_lossy(&buf[..hdr_end]).to_ascii_uppercase();
        let cl: usize = headers
            .lines()
            .find_map(|l| l.strip_prefix("CONTENT-LENGTH:"))
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0);
        while buf.len() < hdr_end + cl {
            let n = stream.read(&mut chunk).ok()?;
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&chunk[..n]);
        }
        Some(String::from_utf8_lossy(&buf[hdr_end..]).into_owned())
    }

    #[test]
    fn ok_false_over_http_writes_no_sibling() {
        // #1003 end-to-end: HTTP 200 + ok:false + source-text echo must NOT
        // produce `.es.md`/`.fr.md` stamped as fresh translations.
        let fx = Fixture::new();
        fx.write_page("content/post/index.md", en_page_fm(), "Body one.\n");
        let server = TinyServer::start(|n| {
            json!({"ok": false, "translations": vec!["passthrough"; n]}).to_string()
        });
        let client = TranslateApiClient::new(&server.url, Duration::from_secs(30), vec![]).unwrap();
        let res = translate_with(fx.root(), &fx.config(), None, false, "", &client);
        assert!(res.is_err(), "ok:false must surface as a failure");
        assert!(!fx.root().join("content/post/index.es.md").exists(), "must not write es");
        assert!(!fx.root().join("content/post/index.fr.md").exists(), "must not write fr");
        assert!(server.reqs.load(Ordering::SeqCst) >= 2, "both langs attempted");
    }

    #[test]
    fn mirrored_1003_over_http_writes_no_sibling() {
        let fx = Fixture::new();
        fx.write_page("content/post/index.md", en_page_fm(), "Body one.\n");
        let server = TinyServer::start(|n| {
            json!({"code": 1003, "translations": vec!["passthrough"; n]}).to_string()
        });
        let client = TranslateApiClient::new(&server.url, Duration::from_secs(30), vec![]).unwrap();
        let res = translate_with(fx.root(), &fx.config(), None, false, "", &client);
        assert!(res.is_err(), "mirrored error code 1003 must surface as a failure");
        assert!(!fx.root().join("content/post/index.es.md").exists(), "must not write es");
        assert!(!fx.root().join("content/post/index.fr.md").exists(), "must not write fr");
    }

    #[test]
    fn http_client_is_built_once_and_reused_across_chunks() {
        // A chunked body issues several sequential requests; a client built
        // per request would open a TCP connection per chunk. The pooled client
        // built once per run must keep them on ONE connection.
        let fx = Fixture::new();
        let body = (0..10)
            .map(|i| format!("## H{i}\n\n{}", "word ".repeat(200)))
            .collect::<Vec<_>>()
            .join("\n\n");
        fx.write_page("content/post/index.md", en_page_fm(), &body);
        let server = TinyServer::start(|n| {
            json!({
                "ok": vec![true; n],
                "translations": (0..n).map(|i| format!("Curriculo {i}")).collect::<Vec<_>>(),
            })
            .to_string()
        });
        let client = TranslateApiClient::new(&server.url, Duration::from_secs(30), vec![]).unwrap();
        translate_with(fx.root(), &fx.config(), None, false, "", &client).unwrap();
        let reqs = server.reqs.load(Ordering::SeqCst);
        let conns = server.conns.load(Ordering::SeqCst);
        assert!(reqs > 2, "expected several chunked requests, got {reqs}");
        assert_eq!(conns, 1, "all requests must share one pooled connection, got {conns}");
        assert!(fx.root().join("content/post/index.es.md").exists());
    }

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

    #[test]
    fn chunk_body_splits_on_h2_when_large() {
        let h2 = "## Section\n\n";
        let para = "word ".repeat(800); // ~4k chars each
        let body = format!("{h2}{para}\n{h2}{para}\n{h2}{para}");
        let chunks = chunk_body(&body);
        assert!(chunks.len() >= 2, "expected multiple chunks, got {}", chunks.len());
        let joined = chunks.join("\n\n");
        assert!(joined.contains("## Section"));
    }

    /// Mock that counts calls — chunked bodies should invoke translate >1 time.
    struct CountingClient {
        calls: Cell<usize>,
    }
    impl LlmClient for CountingClient {
        fn translate(&self, f: &Translatable, lang: &str, _key: &str) -> Result<Translatable> {
            self.calls.set(self.calls.get() + 1);
            Ok(Translatable {
                title: if f.title.is_empty() {
                    String::new()
                } else {
                    format!("[{lang}] {}", f.title)
                },
                description: if f.description.is_empty() {
                    String::new()
                } else {
                    format!("[{lang}] {}", f.description)
                },
                body: format!("[{lang}] {}", f.body),
            })
        }
    }

    #[test]
    fn large_body_uses_multiple_translate_calls() {
        let h2 = "## Part\n\n";
        let para = "Curriculo ".repeat(1200);
        let body = format!("{h2}{para}\n{h2}{para}\n{h2}{para}");
        let en =
            Translatable { title: "Big Curriculo page".into(), description: "desc".into(), body };
        let c = CountingClient { calls: Cell::new(0) };
        let out = translate_fields(&c, &en, "es", "k").unwrap();
        assert!(c.calls.get() > 1, "chunked body should call translate more than once");
        assert!(out.body.contains("[es]"));
        assert!(out.title.contains("Curriculo"));
    }
}
