//! `zola indexnow` — submit page/section URLs to an IndexNow endpoint.
//!
//! Default mode resolves changed content via `git diff` against `--diff-base`
//! (default `HEAD~1`, CI: `origin/master`) and submits the permalinks of the
//! matched pages/sections in EVERY language (a changed default-language page
//! also refreshes its translation permalinks). `--urls-file` takes an explicit
//! list, `--full` submits every loaded page/section permalink.
//!
//! Ceiling (deliberate): taxonomy and pagination URLs are not part of the
//! page/section library and are never submitted — pass those via `--urls-file`.
//! Deleted content cannot be resolved from the loaded site either (warned).
//!
//! Network lives ONLY here — `zola build` stays offline; the site library is
//! loaded read-only (same as `zola check`) and nothing is written to the
//! output directory. Conventions mirror `src/cmd/translate.rs` (`http_client`,
//! env-var secret, `--dry-run`, test stub server behind `INDEXNOW_URL`).

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use errors::{Result, anyhow, bail};
use serde_json::{Value, json};
use site::Site;
use url::Url;

/// Default IndexNow endpoint; tests override with `INDEXNOW_URL`.
const DEFAULT_INDEXNOW_URL: &str = "https://api.indexnow.org/indexnow";
/// IndexNow accepts at most 10,000 URLs per request.
const BATCH_MAX: usize = 10_000;
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Everything [`run`] needs, split from [`indexnow`] so tests inject a fake
/// permalink map, key and endpoint URL without loading a real site.
struct IndexNowRun<'a> {
    root_dir: &'a Path,
    config_file: &'a Path,
    permalinks: &'a HashMap<String, String>,
    /// Configured non-default language codes, e.g. ["es", "fr"].
    lang_codes: &'a [String],
    /// Effective base URL: `--base-url` override else config `base_url`.
    base_url: &'a str,
    endpoint: &'a str,
    key: &'a str,
    full: bool,
    urls_file: Option<&'a Path>,
    diff_base: Option<&'a str>,
    dry_run: bool,
}

/// Entry point from `main.rs`. Loads the site read-only (no build, no output
/// writes — same shape as `zola check`), reads `INDEXNOW_KEY` (fail-fast when
/// absent and not a dry-run) and `INDEXNOW_URL`, then delegates to [`run`].
pub fn indexnow(
    root_dir: &Path,
    config_file: &Path,
    base_url: Option<&str>,
    diff_base: Option<&str>,
    full: bool,
    urls_file: Option<&Path>,
    dry_run: bool,
) -> Result<usize> {
    let mut site = Site::new(root_dir, config_file)?;
    if let Some(b) = base_url {
        site.set_base_url(b.to_string());
    }
    site.load()?;

    let endpoint = env::var("INDEXNOW_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_INDEXNOW_URL.to_string());
    let key = if dry_run {
        String::new()
    } else {
        env::var("INDEXNOW_KEY").ok().filter(|s| !s.is_empty()).ok_or_else(|| {
            anyhow!("INDEXNOW_KEY not set — indexnow needs it (or pass --dry-run)")
        })?
    };
    let lang_codes: Vec<String> =
        site.config.other_languages_codes().iter().map(|s| s.to_string()).collect();
    let r = IndexNowRun {
        root_dir,
        config_file,
        permalinks: &site.permalinks,
        lang_codes: &lang_codes,
        base_url: &site.config.base_url,
        endpoint: &endpoint,
        key: &key,
        full,
        urls_file,
        diff_base,
        dry_run,
    };
    run(&r)
}

/// Testable core. Returns the number of URLs submitted (0 when nothing to do).
fn run(r: &IndexNowRun) -> Result<usize> {
    let changed_mode = !r.full && r.urls_file.is_none();

    // 1. Candidate URLs by mode.
    let mut urls = if r.full {
        r.permalinks.values().cloned().collect::<Vec<_>>()
    } else if let Some(file) = r.urls_file {
        read_urls_file(file)?
    } else {
        changed_urls(r.root_dir, r.diff_base, r.permalinks, r.lang_codes, r.config_file)?
    };
    urls.sort();
    urls.dedup();

    // 2. Effective base URL must be an absolute http(s) URL.
    let base = Url::parse(r.base_url.trim())
        .map_err(|_| anyhow!("base_url {:?} is not an absolute http(s) URL", r.base_url))?;
    if !matches!(base.scheme(), "http" | "https") {
        bail!("base_url {:?} is not an http(s) URL", r.base_url);
    }
    let Some(host) = base.host_str() else {
        bail!("base_url {:?} has no host", r.base_url);
    };

    // 3. Keep only URLs on the base URL's host.
    let (kept, skipped) = split_by_host(urls, host);
    for u in &skipped {
        log::warn!("indexnow: skipping {u}: host does not match base URL host {host}");
    }
    if kept.is_empty() {
        if changed_mode {
            log::info!("indexnow: nothing changed; no ping sent");
        } else {
            log::info!("indexnow: no URLs to submit; no ping sent");
        }
        return Ok(0);
    }

    // 4. Dry-run: print what would be sent, no key, no network.
    if r.dry_run {
        for u in &kept {
            log::info!("indexnow [dry-run]: {u}");
        }
        log::info!(
            "indexnow [dry-run]: {} URL(s) would be submitted to {}",
            kept.len(),
            r.endpoint
        );
        return Ok(0);
    }

    // 5. Key-file precheck: engines 403 until <base>/<key>.txt is reachable.
    let key_file = r.root_dir.join("static").join(format!("{}.txt", r.key));
    if !key_file.exists() {
        log::warn!(
            "indexnow: {} not found — engines will reject submissions with 403 until the key file is committed",
            key_file.display()
        );
    }

    // 6. Submit in batches of at most BATCH_MAX.
    let key_location = format!("{}{}.txt", normalize_base_url(r.base_url), r.key);
    let client = http_client()?;
    let batches = chunk_urls(&kept, BATCH_MAX);
    let total_batches = batches.len();
    let mut submitted = 0usize;
    for (i, batch) in batches.iter().enumerate() {
        let note = submit_batch(&client, r.endpoint, host, r.key, &key_location, batch)?;
        log::info!("indexnow: batch {}/{}: {} URL(s) — {note}", i + 1, total_batches, batch.len());
        submitted += batch.len();
    }
    log::info!("indexnow: submitted {submitted} URL(s) in {total_batches} batch(es)");
    Ok(submitted)
}

// ---- changed-URLs mode (git) ----

/// Resolve changed-content URLs: diff `content/` against `base` (default
/// `HEAD~1`), map each changed file to its page/section permalinks in every
/// language, and warn about unmatched files, deleted content and a changed
/// config file. `--relative` keeps the diff paths relative to the site root —
/// without it a site root nested in the parent repository (e.g. `zola --root
/// docs`) yields `docs/content/...` and nothing ever matches.
fn changed_urls(
    root_dir: &Path,
    diff_base: Option<&str>,
    permalinks: &HashMap<String, String>,
    lang_codes: &[String],
    config_file: &Path,
) -> Result<Vec<String>> {
    let base = diff_base.unwrap_or("HEAD~1");
    let names = git_output(
        root_dir,
        &["diff", "--name-only", "--relative", "--diff-filter=ACMRT", base, "--", "content/"],
    )?;
    let changed: Vec<String> =
        names.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect();
    let (urls, unmatched) = urls_for_changed_files(&changed, permalinks, lang_codes);
    if !unmatched.is_empty() {
        log::warn!(
            "indexnow: {} changed file(s) matched no loaded page/section: {}",
            unmatched.len(),
            unmatched.join(", ")
        );
    }

    let deleted_raw = git_output(
        root_dir,
        &["diff", "--name-only", "--relative", "--diff-filter=D", base, "--", "content/"],
    )?;
    let deleted: Vec<&str> = deleted_raw.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if !deleted.is_empty() {
        log::warn!(
            "indexnow: deleted content: {} — deleted URLs cannot be resolved from the loaded site; submit them via `--urls-file` or `--full`",
            deleted.join(", ")
        );
    }

    if let Some(name) = config_file.file_name().and_then(|n| n.to_str()) {
        let cfg = git_output(root_dir, &["diff", "--name-only", base, "--", name])?;
        if !cfg.trim().is_empty() {
            log::warn!(
                "indexnow: {name} changed in this diff — permalinks may have moved; consider `--full`"
            );
        }
    }
    Ok(urls)
}

/// Run `git -C <root> <args>` and capture stdout. Any git failure (not a repo,
/// bad ref) is a hard error that suggests the modes that do not need git.
fn git_output(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| anyhow!("could not run git: {e}"))?;
    if !out.status.success() {
        bail!(
            "git {} failed ({}) — cannot resolve changed URLs; pass `--urls-file <file>` or `--full` instead: {}",
            args.join(" "),
            out.status,
            take200(&String::from_utf8_lossy(&out.stderr))
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Map changed `content/` files to permalinks: each file resolves to its
/// language-neutral page family (default page/section + every translation).
/// Returns (permalinks, files that matched nothing).
fn urls_for_changed_files(
    changed: &[String],
    permalinks: &HashMap<String, String>,
    lang_codes: &[String],
) -> (Vec<String>, Vec<String>) {
    let mut families: HashMap<String, Vec<String>> = HashMap::new();
    for (rel, permalink) in permalinks {
        let key = language_neutral_path(rel, lang_codes).unwrap_or_else(|| rel.clone());
        families.entry(key).or_default().push(permalink.clone());
    }
    let mut urls = Vec::new();
    let mut unmatched = Vec::new();
    for path in changed {
        let rel = path.strip_prefix("content/").unwrap_or(path);
        let neutral = language_neutral_path(rel, lang_codes).unwrap_or_else(|| rel.to_string());
        match families.get(&neutral) {
            Some(permalinks) => urls.extend(permalinks.iter().cloned()),
            None => unmatched.push(path.clone()),
        }
    }
    (urls, unmatched)
}

/// `post/index.es.md` + codes ["es"] → `post/index.md`; `_index.fr.md` →
/// `_index.md`. Non-translation dotted stems (`v1.2.md`, `.es.md`) and unknown
/// codes are left alone (None).
fn language_neutral_path(rel: &str, lang_codes: &[String]) -> Option<String> {
    let path = Path::new(rel);
    let name = path.file_name()?.to_str()?;
    let stem = name.strip_suffix(".md")?;
    for code in lang_codes {
        let suffix = format!(".{code}");
        if let Some(base_stem) = stem.strip_suffix(&suffix) {
            if base_stem.is_empty() {
                continue;
            }
            let mut neutral = path.parent().map(Path::to_path_buf).unwrap_or_default();
            neutral.push(format!("{base_stem}.md"));
            return Some(neutral.to_string_lossy().into_owned());
        }
    }
    None
}

// ---- pure helpers ----

fn http_client() -> Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .build()?)
}

fn read_urls_file(path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(path).map_err(|e| anyhow!("read {}: {e}", path.display()))?;
    Ok(text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect())
}

/// Base URL normalized to exactly one trailing slash (`https://x.com` and
/// `https://x.com///` both become `https://x.com/`).
fn normalize_base_url(base: &str) -> String {
    format!("{}/", base.trim_end_matches('/'))
}

/// Split URLs into (on-host, skipped): a URL is kept when it parses to an
/// absolute http(s) URL whose host equals `host` (case-insensitive).
fn split_by_host(urls: Vec<String>, host: &str) -> (Vec<String>, Vec<String>) {
    let mut kept = Vec::new();
    let mut skipped = Vec::new();
    for u in urls {
        let on_host = Url::parse(&u).is_ok_and(|p| {
            matches!(p.scheme(), "http" | "https")
                && p.host_str().is_some_and(|h| h.eq_ignore_ascii_case(host))
        });
        if on_host {
            kept.push(u);
        } else {
            skipped.push(u);
        }
    }
    (kept, skipped)
}

fn chunk_urls(urls: &[String], max: usize) -> Vec<&[String]> {
    if urls.is_empty() {
        return vec![];
    }
    urls.chunks(max.max(1)).collect()
}

/// IndexNow request body, canonical field order irrelevant but shape fixed.
fn build_indexnow_request(host: &str, key: &str, key_location: &str, urls: &[String]) -> Value {
    json!({
        "host": host,
        "key": key,
        "keyLocation": key_location,
        "urlList": urls,
    })
}

/// One POST per batch. 200 → ok; 202 → accepted pending key validation;
/// 400/403/422/429 get a specific human explanation; anything else shows the
/// response body truncated.
fn submit_batch(
    client: &reqwest::blocking::Client,
    endpoint: &str,
    host: &str,
    key: &str,
    key_location: &str,
    urls: &[String],
) -> Result<&'static str> {
    let payload = build_indexnow_request(host, key, key_location, urls);
    let resp = client
        .post(endpoint)
        .header(reqwest::header::CONTENT_TYPE, "application/json; charset=utf-8")
        .body(serde_json::to_vec(&payload)?)
        .send()?;
    let status = resp.status();
    let text = resp.text()?;
    interpret_status(status.as_u16(), &text, key_location, host)
}

fn interpret_status(code: u16, body: &str, key_location: &str, host: &str) -> Result<&'static str> {
    match code {
        200 => Ok("ok"),
        202 => Ok("accepted; key validation pending"),
        400 => {
            bail!("IndexNow HTTP 400: invalid request (malformed body or URLs): {}", take200(body))
        }
        403 => bail!(
            "IndexNow HTTP 403: the key file is unreachable at {key_location} or the submitted key does not match it"
        ),
        422 => {
            bail!("IndexNow HTTP 422: a submitted URL's host does not match request host {host}")
        }
        429 => bail!("IndexNow HTTP 429: rate limited — retry later"),
        other => bail!("IndexNow HTTP {other}: {}", take200(body)),
    }
}

fn take200(s: &str) -> String {
    s.chars().take(200).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

    /// Unique temp dir per test (no tempfile dep), translate.rs style.
    struct Fixture {
        root: std::path::PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
            let root = env::temp_dir().join(format!("zola-indexnow-test-{id}-{}", process_id()));
            fs::create_dir_all(&root).unwrap();
            Fixture { root }
        }
        fn write_raw(&self, rel: &str, body: &str) {
            let p = self.root.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, body).unwrap();
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

    fn permalinks(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    // ── pure helpers ─────────────────────────────────────────────────

    #[test]
    fn request_body_carries_indexnow_shape() {
        let urls = vec!["https://example.com/a/".to_string()];
        let body =
            build_indexnow_request("example.com", "k123", "https://example.com/k123.txt", &urls);
        assert_eq!(body["host"], "example.com");
        assert_eq!(body["key"], "k123");
        assert_eq!(body["keyLocation"], "https://example.com/k123.txt");
        assert_eq!(body["urlList"], json!(urls));
    }

    #[test]
    fn base_url_gets_exactly_one_trailing_slash() {
        assert_eq!(normalize_base_url("https://x.com"), "https://x.com/");
        assert_eq!(normalize_base_url("https://x.com/"), "https://x.com/");
        assert_eq!(normalize_base_url("https://x.com///"), "https://x.com/");
        assert_eq!(normalize_base_url("https://x.com/base"), "https://x.com/base/");
    }

    #[test]
    fn language_suffix_strips_to_default_page() {
        let codes = vec!["es".to_string(), "fr".to_string()];
        assert_eq!(
            language_neutral_path("post/index.es.md", &codes),
            Some("post/index.md".to_string())
        );
        assert_eq!(language_neutral_path("_index.fr.md", &codes), Some("_index.md".to_string()));
        // Not a configured language: left alone.
        assert_eq!(language_neutral_path("v1.2.md", &codes), None);
        assert_eq!(language_neutral_path("index.ko.md", &codes), None);
        // A bare `.es.md` is not a translation of an empty stem.
        assert_eq!(language_neutral_path(".es.md", &codes), None);
    }

    #[test]
    fn changed_default_page_submits_all_language_permalinks() {
        let codes = vec!["es".to_string(), "fr".to_string()];
        let map = permalinks(&[
            ("post/index.md", "https://example.com/post/"),
            ("post/index.es.md", "https://example.com/es/post/"),
            ("post/index.fr.md", "https://example.com/fr/post/"),
            ("_index.md", "https://example.com/"),
        ]);
        let changed = vec!["content/post/index.md".to_string()];
        let (mut urls, unmatched) = urls_for_changed_files(&changed, &map, &codes);
        urls.sort();
        assert_eq!(
            urls,
            vec![
                "https://example.com/es/post/".to_string(),
                "https://example.com/fr/post/".to_string(),
                "https://example.com/post/".to_string()
            ]
        );
        assert!(unmatched.is_empty());

        // A changed translation file resolves to the same family.
        let changed = vec!["content/post/index.es.md".to_string()];
        let (urls, _) = urls_for_changed_files(&changed, &map, &codes);
        assert_eq!(urls.len(), 3);

        // Non-content matches are reported, not silently dropped.
        let changed = vec!["content/missing/index.md".to_string()];
        let (urls, unmatched) = urls_for_changed_files(&changed, &map, &codes);
        assert!(urls.is_empty());
        assert_eq!(unmatched, vec!["content/missing/index.md".to_string()]);
    }

    #[test]
    fn host_filter_keeps_only_base_host_urls() {
        let (kept, skipped) = split_by_host(
            vec![
                "https://example.com/a/".into(),
                "HTTPS://EXAMPLE.COM/b/".into(),
                "https://other.com/x/".into(),
                "mailto:root@example.com".into(),
                "not a url".into(),
            ],
            "example.com",
        );
        assert_eq!(
            kept,
            vec!["https://example.com/a/".to_string(), "HTTPS://EXAMPLE.COM/b/".to_string()]
        );
        assert_eq!(skipped.len(), 3);
    }

    #[test]
    fn batches_cap_at_10000() {
        let urls: Vec<String> =
            (0..BATCH_MAX + 1).map(|i| format!("https://example.com/{i}/")).collect();
        let batches = chunk_urls(&urls, BATCH_MAX);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), BATCH_MAX);
        assert_eq!(batches[1].len(), 1);
        assert!(chunk_urls(&[], BATCH_MAX).is_empty());
    }

    #[test]
    fn status_codes_get_specific_explanations() {
        assert_eq!(interpret_status(200, "", "k", "h").unwrap(), "ok");
        assert_eq!(
            interpret_status(202, "", "k", "h").unwrap(),
            "accepted; key validation pending"
        );

        let e = interpret_status(403, "", "https://example.com/k.txt", "example.com").unwrap_err();
        assert!(format!("{e}").contains("403"), "{e}");
        assert!(format!("{e}").contains("https://example.com/k.txt"), "{e}");

        let e = interpret_status(422, "", "k", "example.com").unwrap_err();
        assert!(format!("{e}").contains("host"), "{e}");

        let e = interpret_status(429, "", "k", "h").unwrap_err();
        assert!(format!("{e}").contains("retry"), "{e}");

        let e = interpret_status(500, "internal misery", "k", "h").unwrap_err();
        assert!(format!("{e}").contains("internal misery"), "{e}");
    }

    #[test]
    fn urls_file_skips_blanks_and_comments() {
        let fx = Fixture::new();
        fx.write_raw(
            "urls.txt",
            "https://example.com/a/\n\n# comment\n  https://example.com/b/  \n",
        );
        assert_eq!(
            read_urls_file(&fx.root().join("urls.txt")).unwrap(),
            vec!["https://example.com/a/".to_string(), "https://example.com/b/".to_string()]
        );
    }

    // ── over-HTTP, stub server ───────────────────────────────────────

    /// Minimal keep-alive HTTP/1.1 server answering every request with the
    /// same (status, body); records each request body for assertions.
    struct IndexNowStub {
        url: String,
        requests: Arc<Mutex<Vec<Value>>>,
    }

    impl IndexNowStub {
        fn start(status: u16, body: &'static str) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let rec = requests.clone();
            std::thread::spawn(move || {
                for stream in listener.incoming().flatten() {
                    let mut stream = stream;
                    while let Some(body_in) = read_request_body(&mut stream) {
                        if let Ok(v) = serde_json::from_str::<Value>(&body_in) {
                            rec.lock().unwrap().push(v);
                        }
                        let resp = format!(
                            "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{body}",
                            reason(status),
                            body.len()
                        );
                        use std::io::Write;
                        if stream.write_all(resp.as_bytes()).is_err() {
                            break;
                        }
                    }
                }
            });
            IndexNowStub { url: format!("http://{addr}/indexnow"), requests }
        }

        fn bodies(&self) -> Vec<Value> {
            self.requests.lock().unwrap().clone()
        }
    }

    fn reason(code: u16) -> &'static str {
        match code {
            200 => "OK",
            202 => "Accepted",
            400 => "Bad Request",
            403 => "Forbidden",
            422 => "Unprocessable Entity",
            429 => "Too Many Requests",
            _ => "X",
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

    fn run_args<'a>(
        fx: &'a Fixture,
        permalinks: &'a HashMap<String, String>,
        endpoint: &'a str,
        key: &'a str,
        full: bool,
        dry_run: bool,
    ) -> IndexNowRun<'a> {
        IndexNowRun {
            root_dir: fx.root(),
            config_file: Path::new("config.toml"),
            permalinks,
            lang_codes: &[],
            base_url: "https://example.com",
            endpoint,
            key,
            full,
            urls_file: None,
            diff_base: None,
            dry_run,
        }
    }

    #[test]
    fn full_mode_posts_on_host_urls_in_one_batch() {
        let fx = Fixture::new();
        let map = permalinks(&[
            ("post/index.md", "https://example.com/post/"),
            ("_index.md", "https://example.com/"),
            ("docs/index.md", "https://other.com/docs/"), // foreign host, skipped
        ]);
        let stub = IndexNowStub::start(200, "{}");
        let n = run(&run_args(&fx, &map, &stub.url, "k1234567", true, false)).unwrap();
        assert_eq!(n, 2);
        let bodies = stub.bodies();
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0]["host"], "example.com");
        assert_eq!(bodies[0]["key"], "k1234567");
        assert_eq!(bodies[0]["keyLocation"], "https://example.com/k1234567.txt");
        assert_eq!(
            bodies[0]["urlList"],
            json!(["https://example.com/", "https://example.com/post/"])
        );
    }

    #[test]
    fn ten_thousand_plus_urls_split_into_two_batches() {
        let fx = Fixture::new();
        let map: HashMap<String, String> = (0..BATCH_MAX + 1)
            .map(|i| (format!("p{i}/index.md"), format!("https://example.com/p{i}/")))
            .collect();
        let stub = IndexNowStub::start(202, "{}");
        let n = run(&run_args(&fx, &map, &stub.url, "k", true, false)).unwrap();
        assert_eq!(n, BATCH_MAX + 1);
        let bodies = stub.bodies();
        assert_eq!(bodies.len(), 2);
        assert_eq!(bodies[0]["urlList"].as_array().unwrap().len(), BATCH_MAX);
        assert_eq!(bodies[1]["urlList"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn http_403_fails_with_key_file_explanation() {
        let fx = Fixture::new();
        let map = permalinks(&[("index.md", "https://example.com/")]);
        let stub = IndexNowStub::start(403, "{}");
        let err = run(&run_args(&fx, &map, &stub.url, "k", true, false)).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("403"), "{msg}");
        assert!(msg.contains("key file"), "{msg}");
    }

    #[test]
    fn http_500_shows_truncated_body() {
        let fx = Fixture::new();
        let map = permalinks(&[("index.md", "https://example.com/")]);
        let stub = IndexNowStub::start(500, "{\"error\":\"upstream exploded\"}");
        let err = run(&run_args(&fx, &map, &stub.url, "k", true, false)).unwrap_err();
        assert!(format!("{err}").contains("upstream exploded"), "{err}");
    }

    #[test]
    fn dry_run_sends_nothing_without_a_key() {
        let fx = Fixture::new();
        let map = permalinks(&[("index.md", "https://example.com/")]);
        let stub = IndexNowStub::start(200, "{}");
        let n = run(&run_args(&fx, &map, &stub.url, "", true, true)).unwrap();
        assert_eq!(n, 0);
        assert!(stub.bodies().is_empty(), "dry-run must not touch the network");
    }

    #[test]
    fn empty_url_list_is_not_an_error() {
        let fx = Fixture::new();
        let stub = IndexNowStub::start(200, "{}");
        let empty = HashMap::new();
        assert_eq!(run(&run_args(&fx, &empty, &stub.url, "k", true, false)).unwrap(), 0);
        assert!(stub.bodies().is_empty());
    }

    #[test]
    fn urls_file_mode_posts_file_contents() {
        let fx = Fixture::new();
        fx.write_raw("urls.txt", "https://example.com/a/\n");
        let map = HashMap::new();
        let stub = IndexNowStub::start(200, "{}");
        let mut args = run_args(&fx, &map, &stub.url, "k", false, false);
        let urls_file = fx.root().join("urls.txt");
        args.urls_file = Some(urls_file.as_path());
        assert_eq!(run(&args).unwrap(), 1);
        let bodies = stub.bodies();
        assert_eq!(bodies[0]["urlList"], json!(["https://example.com/a/"]));
    }

    #[test]
    fn relative_base_url_is_rejected() {
        let fx = Fixture::new();
        let map = permalinks(&[("index.md", "https://example.com/")]);
        let stub = IndexNowStub::start(200, "{}");
        let mut args = run_args(&fx, &map, &stub.url, "k", true, false);
        args.base_url = "example.com";
        let err = run(&args).unwrap_err();
        assert!(format!("{err}").contains("absolute"), "{err}");
    }

    // ── changed-URLs mode against a real temp git repo ───────────────

    fn git_available() -> bool {
        Command::new("git").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
    }

    fn git(root: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["-c", "user.email=t@example.com", "-c", "user.name=T"])
            .args(args)
            .output()
            .expect("git runs");
        assert!(out.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&out.stderr));
    }

    #[test]
    fn changed_mode_maps_git_diff_to_permalinks() {
        if !git_available() {
            return;
        }
        let fx = Fixture::new();
        fx.write_raw("content/keep/index.md", "one\n");
        fx.write_raw("content/gone/index.md", "one\n");
        fx.write_raw("config.toml", "base_url = \"https://example.com/\"\n");
        git(fx.root(), &["init", "-q"]);
        git(fx.root(), &["add", "."]);
        git(fx.root(), &["commit", "-q", "-m", "init"]);

        // modify keep/, delete gone/, touch config.toml
        fx.write_raw("content/keep/index.md", "two\n");
        fs::remove_file(fx.root().join("content/gone/index.md")).unwrap();
        fx.write_raw("config.toml", "base_url = \"https://example.com/\"\n# touched\n");

        let map = permalinks(&[("keep/index.md", "https://example.com/keep/")]);
        let urls = changed_urls(
            fx.root(),
            Some("HEAD"),
            &map,
            &[],
            fx.root().join("config.toml").as_path(),
        )
        .unwrap();
        assert_eq!(urls, vec!["https://example.com/keep/".to_string()]);

        // clean tree → nothing changed
        git(fx.root(), &["add", "-A"]);
        git(fx.root(), &["commit", "-q", "-m", "second"]);
        let urls = changed_urls(
            fx.root(),
            Some("HEAD"),
            &map,
            &[],
            fx.root().join("config.toml").as_path(),
        )
        .unwrap();
        assert!(urls.is_empty());

        // bad ref → hard error suggesting the git-free modes
        let err = changed_urls(
            fx.root(),
            Some("no-such-ref"),
            &map,
            &[],
            fx.root().join("config.toml").as_path(),
        )
        .unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("--urls-file"), "{msg}");
        assert!(msg.contains("--full"), "{msg}");
    }

    #[test]
    fn changed_mode_works_when_site_root_is_nested_in_parent_repo() {
        if !git_available() {
            return;
        }
        let parent = Fixture::new();
        parent.write_raw("site/content/post/index.md", "one\n");
        parent.write_raw("site/config.toml", "base_url = \"https://example.com/\"\n");
        git(parent.root(), &["init", "-q"]);
        git(parent.root(), &["add", "."]);
        git(parent.root(), &["commit", "-q", "-m", "init"]);
        parent.write_raw("site/content/post/index.md", "two\n");

        // Regression: git prints repo-root-relative paths (`site/content/...`)
        // unless told `--relative`, so nothing matched and nothing was pinged.
        let site = parent.root().join("site");
        let map = permalinks(&[("post/index.md", "https://example.com/post/")]);
        let urls = changed_urls(&site, Some("HEAD"), &map, &[], site.join("config.toml").as_path())
            .unwrap();
        assert_eq!(urls, vec!["https://example.com/post/".to_string()]);
    }

    #[test]
    fn changed_mode_end_to_end_over_http() {
        if !git_available() {
            return;
        }
        let fx = Fixture::new();
        fx.write_raw("content/post/index.md", "one\n");
        fx.write_raw("config.toml", "base_url = \"https://example.com/\"\n");
        git(fx.root(), &["init", "-q"]);
        git(fx.root(), &["add", "."]);
        git(fx.root(), &["commit", "-q", "-m", "init"]);
        fx.write_raw("content/post/index.md", "two\n");

        let map = permalinks(&[("post/index.md", "https://example.com/post/")]);
        let stub = IndexNowStub::start(200, "{}");
        let mut args = run_args(&fx, &map, &stub.url, "k", false, false);
        args.diff_base = Some("HEAD");
        assert_eq!(run(&args).unwrap(), 1);
        let bodies = stub.bodies();
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0]["urlList"], json!(["https://example.com/post/"]));
    }
}
