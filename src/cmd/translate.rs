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
//! Network lives ONLY here — `zola build` stays offline. The OpenRouter call
//! is behind the [`LlmClient`] trait so unit tests mock it without touching
//! the network. With `TRANSLATE_URL` set, pages go through the batched
//! endpoint driver instead: one language is drained at a time, each request
//! carries at most [`BATCH_MAX_PAGES`] pages and [`BATCH_MAX_TEXT_BYTES`] bytes
//! of text, and a per-input `ok` flag fails only the page owning that text.
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
/// Endpoint batching caps: at most this many distinct pages per request.
const BATCH_MAX_PAGES: usize = 4;
/// …and at most this many bytes of `texts` per request. Bytes, not chars —
/// the endpoint's request limit is a wire limit.
const BATCH_MAX_TEXT_BYTES: usize = 16 * 1024;
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

/// Bulk translation client for the self-hosted endpoint: one request
/// translates many texts at once. A trait so unit tests inject a mock without
/// a network, exactly like [`LlmClient`].
pub trait BatchTranslateClient {
    /// Translate `texts` into `lang`; returns one (translation, ok) pair per
    /// input, in input order. `ok == false` marks that input as returned
    /// untranslated — the caller must not write it.
    fn translate_texts(&self, texts: &[&str], lang: &str) -> Result<Vec<(String, bool)>>;
}

/// Client for a self-hosted translation endpoint.
///
/// Selected by setting `TRANSLATE_URL`; when it is unset the OpenRouter client
/// above is used and behaviour is unchanged. Deliberately knows nothing about
/// any particular engine or deployment — it speaks one small JSON contract:
///
/// ```text
/// POST $TRANSLATE_URL
///   {"texts": ["...", "..."], "target_language": "ko", "source_language": "en",
///    "preserve_terms": ["Acme Widgets"]}
/// → {"translations": ["..."], "ok": [true, true]}
/// ```
///
/// Requests are bulk: the driver packs whole pages in — at most
/// [`BATCH_MAX_PAGES`] distinct pages and [`BATCH_MAX_TEXT_BYTES`] bytes of
/// text per request — and drains one language at a time. Translations come
/// back in request order, one per input; a per-input false `ok` flag means
/// that text came back untranslated, and only the page owning it is failed.
/// `preserve_terms` (#23 review) carries the caller's own untranslatables —
/// brand and product names that must survive verbatim — so the endpoint masks
/// them out of the engine and restores them after (curriculo-ai #1023/#1133).
/// Sourced from `TRANSLATE_PRESERVE_TERMS` (comma-separated) via
/// [`preserve_terms_from_env`], and only included in the body when non-empty.
///
/// Note the endpoint ignores terms shorter than 3 characters, and the INV-5
/// [`glossary_ok`] gate still checks only the built-in [`GLOSSARY`] — caller
/// terms are trusted to the endpoint's mask/restore machinery.
///
/// Failure envelopes are hard failures, never passthroughs: the endpoint may
/// answer HTTP 200 with a scalar `"ok": false` or a mirrored error code
/// (e.g. `"code": 1003`), with `translations` carrying the ORIGINAL source
/// text. Writing that would stamp English content as a fresh translation, so
/// any of those shapes is a whole-batch error and none of its pages are
/// written.
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

impl BatchTranslateClient for TranslateApiClient {
    fn translate_texts(&self, texts: &[&str], lang: &str) -> Result<Vec<(String, bool)>> {
        if texts.is_empty() {
            return Ok(Vec::new()); // the driver never packs an empty batch
        }
        let payload = build_batch_request(texts, lang, &self.preserve_terms);
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
        parse_batch_translate_response(&text, texts.len())
    }
}

/// Build one bulk request body. `preserve_terms` is included only when
/// non-empty: an empty array is the server default (curriculo-ai #1023), and
/// omitting it keeps the body byte-identical for callers with no glossary.
fn build_batch_request(texts: &[&str], lang: &str, preserve_terms: &[String]) -> Value {
    let mut payload = json!({
        "texts": texts,
        "target_language": lang,
        "source_language": "en",
    });
    if !preserve_terms.is_empty() {
        payload["preserve_terms"] = json!(preserve_terms);
    }
    payload
}

/// Parse a bulk response into one (translation, ok) pair per sent text, in
/// input order. Envelope semantics ported from the old single-page parser; the
/// one deliberate change: a per-string false `ok` flag is SOFT here — the flag
/// rides along on the pair and only the page owning that text is failed.
fn parse_batch_translate_response(text: &str, sent_count: usize) -> Result<Vec<(String, bool)>> {
    let data: Value = serde_json::from_str(text)
        .map_err(|e| anyhow!("translate endpoint non-JSON response: {e}"))?;
    // #1003-style failure envelopes arrive as HTTP 200 with the ORIGINAL source
    // text in `translations`. Any of them means "not translated" — writing the
    // file anyway would mark English content as a fresh translation, so they are
    // hard errors. `ok` may be a bool, a per-string bool array (false = that
    // string came back as source), or null/absent on older deployments.
    // A per-string false flag is SOFT: the flag rides on that index's pair and
    // the driver fails only the page owning the text.
    let mut soft = vec![true; sent_count];
    if let Some(ok) = data.get("ok") {
        match ok {
            Value::Bool(false) => {
                bail!(
                    "translate endpoint reported ok:false — source text returned untranslated, not writing"
                )
            }
            Value::Array(flags) => {
                if flags.len() != sent_count {
                    bail!(
                        "translate endpoint: malformed envelope: ok array length {} != {} sent text(s)",
                        flags.len(),
                        sent_count
                    );
                }
                for (i, f) in flags.iter().enumerate() {
                    // A non-bool entry is an envelope defect, not a "false".
                    match f.as_bool() {
                        Some(true) => {}
                        Some(false) => soft[i] = false,
                        None => bail!(
                            "translate endpoint: malformed envelope: ok[{i}] is not a boolean"
                        ),
                    }
                }
            }
            _ => {}
        }
    }
    // Some gateways mirror the error status in the body of a 200 response
    // (e.g. {"code": 1003}). 0 and 200 mean success; anything else does not.
    for key in ["code", "status", "error_code"] {
        // i64 first so numeric negatives ("code": -1) are not dropped: as_u64 is
        // None on signed JSON numbers, as_str is None on numbers. Do not fold
        // i64 → u64 (try_from drops negatives again). u64 → i64 is only for
        // values that did not fit as_i64.
        let code = data.get(key).and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_u64().and_then(|n| i64::try_from(n).ok()))
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        });
        if let Some(code) = code {
            if code != 0 && code != 200 {
                bail!("translate endpoint error code {code} in `{key}` — not writing");
            }
        }
    }
    let arr = data["translations"].as_array().ok_or_else(|| {
        anyhow!("translate endpoint: no `translations` array: {}", take160(&data.to_string()))
    })?;
    // A short array would silently shift every text onto the wrong page, so the
    // length is a hard error rather than something to paper over.
    if arr.len() != sent_count {
        bail!(
            "translate endpoint returned {} translation(s) for {} text(s)",
            arr.len(),
            sent_count
        );
    }
    let mut out = Vec::with_capacity(sent_count);
    for (i, value) in arr.iter().enumerate() {
        // A non-string entry (null/list/object) used to coerce to "" and write
        // a blank title/description/body. Skip-with-warning is not an option
        // here: there is nothing sane to write for that text, so the request
        // fails and the existing siblings (if any) are left untouched.
        let s = value.as_str().ok_or_else(|| {
            anyhow!("translate endpoint: translations[{i}] is not a string ({}) — not writing blanks", json_kind(value))
        })?.to_string();
        out.push((s, soft[i]));
    }
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
/// Opt-in alternative: when `TRANSLATE_URL` is set the run is served by
/// [`translate_with_endpoint`] (batched, no API key read) instead. Unset —
/// which is the default for every existing site — and nothing below this
/// block changes.
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
        return translate_with_endpoint(root_dir, config_file, max, dry_run, &client);
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

/// One (page, lang) unit of work for [`translate_with_endpoint`].
struct BatchJob {
    /// Default-language page; logs read its display path.
    page: PathBuf,
    lang: String,
    en: Translatable,
    /// Frontmatter template copied into the sibling.
    en_fm: toml::Value,
    /// sha256 of `en` — the freshness stamp written into the sibling.
    hash: String,
    sibling: PathBuf,
}

/// Where a flattened request text came from, for positional reassembly.
enum BatchSlot {
    Title,
    Description,
    Body(usize),
}

/// A flattened request text: its owning job (index into the current language's
/// job list), its slot, and the text itself.
struct FlatText {
    job: usize,
    slot: BatchSlot,
    text: String,
}

/// Endpoint driver: same walk/parse/hash-gate rules as [`translate_with`], but
/// jobs are batched — one language drained at a time, at most
/// [`BATCH_MAX_PAGES`] pages and [`BATCH_MAX_TEXT_BYTES`] text bytes per
/// request — so a site-wide run is a handful of requests instead of one per
/// page. Testable core; takes the client as a parameter so tests inject a
/// mock without a network or sockets.
fn translate_with_endpoint<C: BatchTranslateClient>(
    root_dir: &Path,
    config_file: &Path,
    max: Option<usize>,
    dry_run: bool,
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

    let mut jobs: Vec<BatchJob> = Vec::new();
    let mut failures = 0usize;
    let mut skipped_fresh = 0usize;
    let mut skipped_cap = 0usize;

    // Collection: page-major, same walk/parse/skip rules as translate_with.
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
            jobs.push(BatchJob {
                page: page.clone(),
                lang: lang.clone(),
                en: en.clone(),
                en_fm: en_fm.clone(),
                hash: hash.clone(),
                sibling,
            });
        }
    }

    // Cap on jobs, page-major — the same iteration-order cut translate_with
    // applies per (page, lang); the rest resumes next run.
    let cap = max.unwrap_or(usize::MAX);
    if jobs.len() > cap {
        skipped_cap = jobs.len() - cap;
        jobs.truncate(cap);
    }
    let calls = jobs.len(); // jobs attempted this run (success + fail)
    let mut written = 0usize;

    // One language at a time: drain all of a language's jobs before the next,
    // so a partial run leaves whole languages consistent.
    for lang in &langs {
        let lang_jobs: Vec<&BatchJob> = jobs.iter().filter(|j| j.lang == *lang).collect();
        if lang_jobs.is_empty() {
            continue;
        }

        // Flatten: per job, title → description → body chunks, in field order
        // (empty fields are not sent). Every job has ≥1 text — bodies are
        // non-empty by the skip above.
        let mut flat: Vec<FlatText> = Vec::new();
        let mut ranges: Vec<(usize, usize)> = Vec::with_capacity(lang_jobs.len());
        for (ji, job) in lang_jobs.iter().enumerate() {
            let start = flat.len();
            if !job.en.title.is_empty() {
                flat.push(FlatText { job: ji, slot: BatchSlot::Title, text: job.en.title.clone() });
            }
            if !job.en.description.is_empty() {
                flat.push(FlatText {
                    job: ji,
                    slot: BatchSlot::Description,
                    text: job.en.description.clone(),
                });
            }
            for (ci, chunk) in chunk_body(&job.en.body).iter().enumerate() {
                flat.push(FlatText { job: ji, slot: BatchSlot::Body(ci), text: chunk.clone() });
            }
            ranges.push((start, flat.len()));
        }

        // Pack greedily in order: a batch closes when one more text would push
        // it past the page or byte cap. A lone over-cap text (a monster chunk)
        // rides alone in its own batch rather than being dropped.
        let mut batches: Vec<Vec<usize>> = Vec::new();
        let mut cur: Vec<usize> = Vec::new();
        let mut cur_jobs: HashSet<usize> = HashSet::new();
        let mut cur_bytes = 0usize;
        for (i, ft) in flat.iter().enumerate() {
            let adds_page = !cur_jobs.contains(&ft.job);
            if !cur.is_empty()
                && (cur_jobs.len() + usize::from(adds_page) > BATCH_MAX_PAGES
                    || cur_bytes + ft.text.len() > BATCH_MAX_TEXT_BYTES)
            {
                batches.push(std::mem::take(&mut cur));
                cur_jobs.clear();
                cur_bytes = 0;
            }
            cur.push(i);
            cur_jobs.insert(ft.job);
            cur_bytes += ft.text.len();
        }
        if !cur.is_empty() {
            batches.push(cur);
        }

        // One request in flight at a time: a self-hosted CPU endpoint serves a
        // batch no faster for being asked concurrently.
        let mut results: Vec<Option<(String, bool)>> = vec![None; flat.len()];
        let mut failed = vec![false; lang_jobs.len()];
        for batch in &batches {
            let texts: Vec<&str> = batch.iter().map(|&i| flat[i].text.as_str()).collect();
            match client.translate_texts(&texts, lang) {
                Err(e) => {
                    // Envelope/HTTP failure: every job owning a text in this
                    // batch is unwritten and counted, then the run moves on.
                    for &i in batch {
                        let ji = flat[i].job;
                        if !failed[ji] {
                            failed[ji] = true;
                            failures += 1;
                            log::error!(
                                "translate: {} → {lang} FAILED: {e}",
                                lang_jobs[ji].page.display()
                            );
                        }
                    }
                }
                Ok(pairs) => {
                    for (&i, pair) in batch.iter().zip(pairs) {
                        results[i] = Some(pair);
                    }
                }
            }
        }

        // Assemble survivors in job order: slots back onto fields, body chunks
        // rejoined with the same conditional translate_fields_impl uses.
        for (ji, job) in lang_jobs.iter().enumerate() {
            if failed[ji] {
                continue; // already counted + logged above
            }
            let (start, end) = ranges[ji];
            let mut t = Translatable {
                title: String::new(),
                description: String::new(),
                body: String::new(),
            };
            let mut ok_all = true;
            let mut body_seen = 0usize;
            for i in start..end {
                // A missing pair means a misbehaving client short-changed the
                // batch; treat it like a false flag, never as a blank field.
                let Some((s, ok)) = &results[i] else {
                    ok_all = false;
                    break;
                };
                if !*ok {
                    ok_all = false; // this text came back untranslated
                }
                match flat[i].slot {
                    BatchSlot::Title => t.title = s.clone(),
                    BatchSlot::Description => t.description = s.clone(),
                    BatchSlot::Body(ci) => {
                        // Chunks reassemble in request order — the slot index
                        // is that invariant, so it is checked, not assumed.
                        debug_assert_eq!(ci, body_seen, "body chunks out of request order");
                        body_seen += 1;
                        if !t.body.is_empty() && !s.is_empty() {
                            t.body.push_str("\n\n");
                        }
                        t.body.push_str(s);
                    }
                }
            }
            if !ok_all {
                failures += 1;
                log::error!(
                    "translate: {} → {lang} FAILED: endpoint reported ok=false — source text returned untranslated, not writing",
                    job.page.display()
                );
                continue;
            }
            if let Err(e) = glossary_ok(&job.en, &t) {
                failures += 1;
                log::error!("translate: {} → {lang} FAILED: {e}", job.page.display());
                continue; // file intentionally NOT written on failure
            }
            write_sibling(&job.sibling, &job.en_fm, &t, &job.hash)?;
            written += 1;
            log::info!("translate: {} → {lang}", job.page.display());
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
    fn request_carries_texts_languages_and_omits_empty_glossary() {
        let payload = build_batch_request(&["T", "D", "B"], "ko", &[]);
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
        let payload = build_batch_request(&["T", "D", "B"], "ko", &terms);
        assert_eq!(payload["preserve_terms"], json!(terms));
        assert_eq!(payload["target_language"], "ko");
    }

    #[test]
    fn body_only_chunk_sends_only_the_body() {
        // Continuation chunks carry no title/description (empty fields are
        // never sent); those empty strings would spend an engine call on
        // nothing.
        let payload = build_batch_request(&["B"], "ja", &[]);
        assert_eq!(payload["texts"], json!(["B"]));
    }

    #[test]
    fn response_pairs_map_back_in_input_order() {
        let out = parse_batch_translate_response(r#"{"translations":["本文"]}"#, 1).unwrap();
        assert_eq!(out, vec![("本文".to_string(), true)]);
    }

    #[test]
    fn short_response_is_an_error_not_a_silent_shift() {
        // Two translations for three texts would otherwise shift every later
        // page's fields onto the wrong slot and write a plausible-looking,
        // wrong page.
        let err = parse_batch_translate_response(r#"{"translations":["Titre","Description"]}"#, 3)
            .unwrap_err();
        assert!(format!("{err}").contains("2 translation(s) for 3"));
    }

    #[test]
    fn long_body_is_chunked_by_translate_fields_impl() {
        // Regression: chunking lives inside each client path, so a client that
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
        let err = parse_batch_translate_response(r#"{"oops":true}"#, 1).unwrap_err();
        assert!(format!("{err}").contains("translations"));
    }

    #[test]
    fn ok_false_is_a_hard_failure_not_source_passthrough() {
        // #1003: HTTP 200 + ok:false means `translations` carries the ORIGINAL
        // source text. Accepting it would write English into `<slug>.ko.md`
        // stamped fresh (source_hash set) — exactly the bug this guards.
        let err = parse_batch_translate_response(r#"{"ok":false,"translations":["T","D","B"]}"#, 3)
            .unwrap_err();
        assert!(format!("{err}").contains("ok:false"), "got: {err}");
    }

    #[test]
    fn ok_array_false_flag_is_soft_and_per_input() {
        // Bulk semantics: a false flag marks ONLY that input as untranslated.
        // The driver fails the page owning it; the rest of the batch writes.
        let out =
            parse_batch_translate_response(r#"{"ok":[true,false],"translations":["T","B"]}"#, 2)
                .unwrap();
        assert_eq!(out[0], ("T".to_string(), true));
        assert_eq!(out[1], ("B".to_string(), false));
    }

    #[test]
    fn ok_array_shorter_than_sent_is_malformed() {
        // Finding #3: an all-true but short ok array used to pass; the missing
        // flags must reject the envelope rather than silently covering unsent texts.
        let err =
            parse_batch_translate_response(r#"{"ok":[true],"translations":["T","D","B"]}"#, 3)
                .unwrap_err();
        assert!(
            format!("{err}").contains("malformed") || format!("{err}").contains("length"),
            "got: {err}"
        );
    }

    #[test]
    fn ok_array_with_a_non_bool_entry_is_malformed() {
        let err = parse_batch_translate_response(r#"{"ok":[true,1],"translations":["T","B"]}"#, 2)
            .unwrap_err();
        assert!(format!("{err}").contains("malformed"), "got: {err}");
    }

    #[test]
    fn ok_all_true_or_absent_still_passes() {
        // Older deployments send no `ok` at all; newer ones send all-true.
        // Both must keep working — guard against over-tightening the #1003 fix.
        for body in [r#"{"ok":[true],"translations":["本文"]}"#, r#"{"translations":["本文"]}"#]
        {
            let out = parse_batch_translate_response(body, 1).unwrap();
            assert_eq!(out, vec![("本文".to_string(), true)]);
        }
    }

    #[test]
    fn mirrored_error_code_1003_is_a_hard_failure() {
        for body in [
            r#"{"code":1003,"translations":["T"]}"#,
            r#"{"status":1003,"translations":["T"]}"#,
            r#"{"error_code":"1003","translations":["T"]}"#,
        ] {
            let err = parse_batch_translate_response(body, 1).unwrap_err();
            assert!(format!("{err}").contains("1003"), "body {body} -> {err}");
        }
    }

    #[test]
    fn numeric_negative_error_code_is_a_hard_failure() {
        // Finding #4: as_u64/as_str both miss JSON numbers like -1.
        let err =
            parse_batch_translate_response(r#"{"code":-1,"translations":["T"]}"#, 1).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("-1") || msg.contains("error code"), "got: {err}");
    }

    #[test]
    fn non_string_translation_is_an_error_not_a_blank() {
        // Batch-level: one bad entry sinks the whole request — there is nothing
        // sane to write for that slot's page, and a partial write would need a
        // partial-response contract the endpoint does not have.
        for body in [
            r#"{"translations":[null,"D"]}"#,
            r#"{"translations":[["T"],"D"]}"#,
            r#"{"translations":[{"v":"T"},"D"]}"#,
        ] {
            let err = parse_batch_translate_response(body, 2).unwrap_err();
            assert!(
                format!("{err}").contains("translations[0] is not a string"),
                "body {body} -> {err}"
            );
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
        let res = translate_with_endpoint(fx.root(), &fx.config(), None, false, &client);
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
        let res = translate_with_endpoint(fx.root(), &fx.config(), None, false, &client);
        assert!(res.is_err(), "mirrored error code 1003 must surface as a failure");
        assert!(!fx.root().join("content/post/index.es.md").exists(), "must not write es");
        assert!(!fx.root().join("content/post/index.fr.md").exists(), "must not write fr");
    }

    #[test]
    fn http_client_is_built_once_and_reused_across_batches() {
        // An over-16 KiB body spans several batches, each its own request; a
        // client built per request would open a TCP connection per batch. The
        // pooled client built once per run must keep them on ONE connection.
        let fx = Fixture::new();
        // 24 × ~1 KB sections ≈ 24 KB body → 4+ chunks → over the 16 KiB
        // batch cap → 2+ batches per language.
        let body = (0..24)
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
        translate_with_endpoint(fx.root(), &fx.config(), None, false, &client).unwrap();
        let reqs = server.reqs.load(Ordering::SeqCst);
        let conns = server.conns.load(Ordering::SeqCst);
        assert!(reqs > 2, "expected several batched requests, got {reqs}");
        assert_eq!(conns, 1, "all requests must share one pooled connection, got {conns}");
        assert!(fx.root().join("content/post/index.es.md").exists());
    }

    /// Rewrite the fixture config to a single non-default language (es).
    fn one_lang_config(fx: &Fixture) {
        fs::write(
            fx.root().join("config.toml"),
            "base_url = \"https://x/\"\ndefault_language = \"en\"\n[languages.es]\n",
        )
        .unwrap();
    }

    /// Recording mock for the endpoint driver: remembers every call's texts
    /// and lang, and answers from a scripted list of outcomes drained in
    /// order. When the script runs dry it echoes the input with ok=true
    /// (echo keeps glossary tokens, so happy-path runs pass INV-5).
    struct MockBatch {
        calls: Mutex<Vec<(Vec<String>, String)>>,
        script: Mutex<Vec<Result<Vec<(String, bool)>, String>>>,
    }

    impl MockBatch {
        fn new() -> Self {
            MockBatch { calls: Mutex::new(Vec::new()), script: Mutex::new(Vec::new()) }
        }

        /// Script one successful response: (translation, ok) per text.
        fn push_ok_flags(&self, pairs: Vec<(&str, bool)>) {
            self.script
                .lock()
                .unwrap()
                .push(Ok(pairs.into_iter().map(|(s, ok)| (s.to_string(), ok)).collect()));
        }

        /// Script one whole-batch failure (envelope/HTTP error).
        fn push_err(&self, msg: &str) {
            self.script.lock().unwrap().push(Err(msg.to_string()));
        }

        fn calls(&self) -> Vec<(Vec<String>, String)> {
            self.calls.lock().unwrap().clone()
        }

        fn texts_of(&self, i: usize) -> Vec<String> {
            self.calls()[i].0.clone()
        }

        fn langs(&self) -> Vec<String> {
            self.calls().into_iter().map(|(_, l)| l).collect()
        }
    }

    impl BatchTranslateClient for MockBatch {
        fn translate_texts(&self, texts: &[&str], lang: &str) -> Result<Vec<(String, bool)>> {
            self.calls
                .lock()
                .unwrap()
                .push((texts.iter().map(|s| s.to_string()).collect(), lang.to_string()));
            let mut script = self.script.lock().unwrap();
            if script.is_empty() {
                return Ok(texts.iter().map(|t| (t.to_string(), true)).collect());
            }
            script.remove(0).map_err(|e| anyhow!("{e}"))
        }
    }

    #[test]
    fn page_cap_splits_five_small_pages_into_two_requests() {
        let fx = Fixture::new();
        one_lang_config(&fx);
        for i in 1..=5 {
            fx.write_page(
                &format!("content/p{i}/index.md"),
                &format!("title = \"P{i}\"\ndescription = \"D{i}\"\n"),
                &format!("Body {i}.\n"),
            );
        }
        let mock = MockBatch::new();
        translate_with_endpoint(fx.root(), &fx.config(), None, false, &mock).unwrap();
        let calls = mock.calls();
        assert_eq!(calls.len(), 2, "5 pages under a 4-page cap ⇒ 2 requests");
        assert_eq!(calls[0].1, "es");
        // First request: pages 1-4 flattened in field order (title/desc/body).
        assert_eq!(
            calls[0].0,
            [
                "P1", "D1", "Body 1.", "P2", "D2", "Body 2.", "P3", "D3", "Body 3.", "P4", "D4",
                "Body 4."
            ]
        );
        // Second request: just page 5.
        assert_eq!(calls[1].0, ["P5", "D5", "Body 5."]);
        for i in 1..=5 {
            assert!(fx.root().join(format!("content/p{i}/index.es.md")).exists(), "page {i}");
        }
    }

    #[test]
    fn byte_cap_keeps_every_request_under_16kib() {
        let fx = Fixture::new();
        one_lang_config(&fx);
        // 8188-byte bodies (one chunk each): a page's texts weigh 8192 bytes,
        // so pages 1-2 land at exactly 16384 — the cap, allowed — and page 3's
        // title overflows it. ⇒ clean 2+1 page split, 6+3 texts.
        for i in 1..=3 {
            fx.write_page(
                &format!("content/p{i}/index.md"),
                &format!("title = \"P{i}\"\ndescription = \"D{i}\"\n"),
                &format!("Curriculo {}\n", "x".repeat(8178)),
            );
        }
        let mock = MockBatch::new();
        translate_with_endpoint(fx.root(), &fx.config(), None, false, &mock).unwrap();
        let calls = mock.calls();
        assert_eq!(calls.len(), 2, "expected the 2+1 page split");
        for (texts, lang) in &calls {
            let bytes: usize = texts.iter().map(|t| t.len()).sum();
            assert!(bytes <= BATCH_MAX_TEXT_BYTES, "{lang} request of {bytes} bytes over cap");
        }
        // Batch 1 sits exactly at the cap: two whole pages (title/desc/body
        // each), 16384 bytes.
        assert_eq!(calls[0].0.len(), 6, "first request carries pages 1-2 (3 texts each)");
        assert_eq!(calls[0].0[0], "P1");
        assert_eq!(calls[0].0[3], "P2");
        assert_eq!(calls[0].0[4], "D2");
        let bytes0: usize = calls[0].0.iter().map(|t| t.len()).sum();
        assert_eq!(bytes0, BATCH_MAX_TEXT_BYTES);
        assert_eq!(calls[1].0.first().map(String::as_str), Some("P3"));
        assert_eq!(calls[1].0.len(), 3, "second request carries page 3");
    }

    #[test]
    fn oversized_page_spans_batches_and_reassembles_in_order() {
        let fx = Fixture::new();
        one_lang_config(&fx);
        // One ~24 KB body: its chunks alone exceed the 16 KiB request cap, so
        // the page spans batches. The echo mock makes the written body equal to
        // the chunk join — order and separators both checked.
        let body = (0..24)
            .map(|i| format!("## H{i}\n\n{}", "x".repeat(1000)))
            .collect::<Vec<_>>()
            .join("\n\n");
        fx.write_page("content/post/index.md", "title = \"T\"\ndescription = \"D\"\n", &body);
        let mock = MockBatch::new();
        translate_with_endpoint(fx.root(), &fx.config(), None, false, &mock).unwrap();
        let calls = mock.calls();
        assert!(calls.len() >= 2, "over-cap page must span batches, got {}", calls.len());
        let sibling = fx.root().join("content/post/index.es.md");
        let (fm, written) = parse_page(&sibling).unwrap();
        let expected = chunk_body(&body).join("\n\n");
        assert_eq!(written.trim(), expected, "chunks must rejoin in request order");
        // …and the hash stamp matches the en fields, so the pair is now fresh.
        let en =
            Translatable { title: "T".into(), description: "D".into(), body: body.trim().into() };
        assert_eq!(extra_hash(&fm).as_deref(), Some(source_hash(&en).as_str()));
    }

    #[test]
    fn languages_drain_one_at_a_time_es_before_fr() {
        let fx = Fixture::new(); // default fixture config: es + fr
        for i in 1..=5 {
            fx.write_page(
                &format!("content/p{i}/index.md"),
                &format!("title = \"P{i}\"\n"),
                &format!("Body {i}.\n"),
            );
        }
        let mock = MockBatch::new();
        translate_with_endpoint(fx.root(), &fx.config(), None, false, &mock).unwrap();
        // 5 pages > the 4-page cap ⇒ 2 requests per language; all of es first.
        assert_eq!(mock.langs(), vec!["es", "es", "fr", "fr"]);
    }

    #[test]
    fn flattening_order_and_positional_reassembly() {
        let fx = Fixture::new();
        one_lang_config(&fx);
        fx.write_page("content/post/index.md", "title = \"T\"\ndescription = \"D\"\n", "Body.\n");
        let mock = MockBatch::new();
        mock.push_ok_flags(vec![("t-es", true), ("d-es", true), ("b-es", true)]);
        translate_with_endpoint(fx.root(), &fx.config(), None, false, &mock).unwrap();
        // One call: title → description → body, in field order.
        assert_eq!(mock.texts_of(0), ["T", "D", "Body."]);
        let (fm, body) = parse_page(&fx.root().join("content/post/index.es.md")).unwrap();
        assert_eq!(fm.get("title").and_then(|v| v.as_str()), Some("t-es"));
        assert_eq!(fm.get("description").and_then(|v| v.as_str()), Some("d-es"));
        assert_eq!(body.trim(), "b-es");
    }

    #[test]
    fn one_false_ok_flag_fails_only_its_page() {
        let fx = Fixture::new();
        one_lang_config(&fx);
        for i in 1..=4 {
            fx.write_page(
                &format!("content/p{i}/index.md"),
                &format!("title = \"P{i}\"\ndescription = \"D{i}\"\n"),
                &format!("Body {i}.\n"),
            );
        }
        let mock = MockBatch::new();
        // All four pages fit one request (12 texts); page 2's three slots come
        // back flagged untranslated.
        mock.push_ok_flags(vec![
            ("x0", true),
            ("x1", true),
            ("x2", true),
            ("x3", false),
            ("x4", false),
            ("x5", false),
            ("x6", true),
            ("x7", true),
            ("x8", true),
            ("x9", true),
            ("x10", true),
            ("x11", true),
        ]);
        let res = translate_with_endpoint(fx.root(), &fx.config(), None, false, &mock);
        let err = res.unwrap_err().to_string();
        assert!(err.contains("failure"), "got: {err}");
        // Page 2: no file (which also means no source_hash stamp). Pages
        // 1, 3, 4: written and stamped with the hash of their en fields.
        assert!(
            !fx.root().join("content/p2/index.es.md").exists(),
            "flagged page must not be written"
        );
        for i in [1, 3, 4] {
            let (fm, _) = parse_page(&fx.root().join(format!("content/p{i}/index.es.md"))).unwrap();
            let en = Translatable {
                title: format!("P{i}"),
                description: format!("D{i}"),
                body: format!("Body {i}."),
            };
            assert_eq!(extra_hash(&fm).as_deref(), Some(source_hash(&en).as_str()), "page {i}");
        }
        assert_eq!(mock.calls().len(), 1, "all four pages fit one request");
    }

    #[test]
    fn envelope_error_fails_the_whole_batch_but_not_the_run() {
        let fx = Fixture::new();
        one_lang_config(&fx);
        // 5459-byte bodies (one chunk each): a page weighs 5461 bytes, so
        // three pages land at 16383 and page 4's title overflows ⇒ batch 1 =
        // pages 1-3 exactly, batch 2 = pages 4-5.
        for i in 1..=5 {
            fx.write_page(
                &format!("content/p{i}/index.md"),
                &format!("title = \"P{i}\"\n"),
                &format!("Curriculo {}\n", "y".repeat(5449)),
            );
        }
        let mock = MockBatch::new();
        mock.push_err("translate endpoint reported ok:false — source text returned untranslated");
        // Batch 2 falls through to the echo default and writes pages 4-5.
        let res = translate_with_endpoint(fx.root(), &fx.config(), None, false, &mock);
        assert!(res.is_err(), "3 failed jobs must fail the run");
        for i in 1..=3 {
            assert!(
                !fx.root().join(format!("content/p{i}/index.es.md")).exists(),
                "page {i} must not be written"
            );
        }
        for i in 4..=5 {
            assert!(
                fx.root().join(format!("content/p{i}/index.es.md")).exists(),
                "page {i} should still be written"
            );
        }
        assert_eq!(mock.calls().len(), 2, "second batch must still be attempted");
    }

    #[test]
    fn max_cap_truncates_page_major_and_resumes() {
        let fx = Fixture::new();
        for i in 1..=2 {
            fx.write_page(
                &format!("content/p{i}/index.md"),
                &format!("title = \"P{i}\"\n"),
                &format!("Body {i}.\n"),
            );
        }
        // Jobs are page-major (p1-es, p1-fr, p2-es, p2-fr); --max 1 keeps the
        // first only — the same cut translate_with makes per (page, lang).
        let m1 = MockBatch::new();
        translate_with_endpoint(fx.root(), &fx.config(), Some(1), false, &m1).unwrap();
        assert_eq!(m1.calls().len(), 1);
        assert_eq!(m1.calls()[0].1, "es");
        assert_eq!(m1.texts_of(0), ["P1", "Body 1."]);
        assert!(fx.root().join("content/p1/index.es.md").exists());
        for p in ["content/p1/index.fr.md", "content/p2/index.es.md", "content/p2/index.fr.md"] {
            assert!(!fx.root().join(p).exists(), "{p} must stay untouched");
        }

        // Resume: exactly the 3 remaining jobs translate; es drained before fr.
        let m2 = MockBatch::new();
        translate_with_endpoint(fx.root(), &fx.config(), None, false, &m2).unwrap();
        assert_eq!(m2.langs(), vec!["es", "fr"], "es (only p2 left) before fr (p1, p2)");
        for p in ["content/p1/index.fr.md", "content/p2/index.es.md", "content/p2/index.fr.md"] {
            assert!(fx.root().join(p).exists(), "{p} must be written on resume");
        }
    }

    #[test]
    fn endpoint_dry_run_makes_no_calls_and_writes_nothing() {
        let fx = Fixture::new();
        fx.write_page("content/post/index.md", en_page_fm(), "Body one.\n");
        let mock = MockBatch::new();
        translate_with_endpoint(fx.root(), &fx.config(), None, true, &mock).unwrap();
        assert!(mock.calls().is_empty(), "dry-run must not hit the endpoint");
        assert!(!fx.root().join("content/post/index.es.md").exists());
        assert!(!fx.root().join("content/post/index.fr.md").exists());
    }

    #[test]
    fn fresh_endpoint_sibling_skips_the_call() {
        let fx = Fixture::new();
        fx.write_page("content/post/index.md", en_page_fm(), "Body one.\n");
        // Pre-stamp a fresh es sibling (correct source_hash) ⇒ not a job.
        let en = Translatable {
            title: "Hello Curriculo".into(),
            description: "A desc".into(),
            body: "Body one.".into(),
        };
        let (fm, _) = parse_page(&fx.root().join("content/post/index.md")).unwrap();
        write_sibling(&fx.root().join("content/post/index.es.md"), &fm, &en, &source_hash(&en))
            .unwrap();
        let mock = MockBatch::new();
        translate_with_endpoint(fx.root(), &fx.config(), None, false, &mock).unwrap();
        let calls = mock.calls();
        assert_eq!(calls.len(), 1, "only fr remains");
        assert_eq!(calls[0].1, "fr");
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
