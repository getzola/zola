//! Path-id helpers for graph v2. Page identity is a content path; the host
//! comes from `--base-url` at build time.

/// `rel` is site-root relative, always forward slashes.
/// `"content/features/index.md"` → `"content/features/index.md"`
/// `"features/index.md"` → `"content/features/index.md"`
pub fn page_id_from_rel(rel: &str) -> String {
    let r = rel.replace('\\', "/");
    if r.starts_with("content/") { r } else { format!("content/{r}") }
}

/// `index.fr.md` → `fr`; `_index.md` → `default_lang`.
pub fn lang_from_filename(file_name: &str, default_lang: &str) -> String {
    let stem = file_name.strip_suffix(".md").unwrap_or(file_name);
    match stem.rsplit_once('.') {
        Some(("index" | "_index", lang)) if !lang.is_empty() => lang.to_string(),
        _ => default_lang.to_string(),
    }
}

/// Content-relative path → hostless canonical URL path.
/// `content/_index.md` → `/`
/// `content/_index.fr.md` → `/fr/`
/// `content/features/index.md` → `/features/`
/// `content/features/index.fr.md` → `/fr/features/`
/// `content/blogs/foo/index.md` → `/blogs/foo/`
pub fn canonical_path_from_rel(rel: &str, default_lang: &str) -> String {
    let id = page_id_from_rel(rel);
    let rest = id.strip_prefix("content/").unwrap_or(&id);
    let (dir, file_name) = match rest.rsplit_once('/') {
        Some((d, f)) => (d, f),
        None => ("", rest),
    };
    let lang = lang_from_filename(file_name, default_lang);
    let path_part =
        if dir.is_empty() { "/".to_string() } else { format!("/{}/", dir.trim_matches('/')) };
    if lang == default_lang {
        path_part
    } else if path_part == "/" {
        format!("/{lang}/")
    } else {
        format!("/{lang}{path_part}")
    }
}

/// `base_url` has a trailing slash; `canonical_path` starts with `/`.
pub fn absolute_url(base_url: &str, canonical_path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if canonical_path == "/" { format!("{base}/") } else { format!("{base}{canonical_path}") }
}

pub fn is_hostful_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_path_from_rel_home_and_locale() {
        assert_eq!(canonical_path_from_rel("content/_index.md", "en"), "/");
        assert_eq!(canonical_path_from_rel("content/_index.fr.md", "en"), "/fr/");
        assert_eq!(canonical_path_from_rel("content/features/index.md", "en"), "/features/");
        assert_eq!(canonical_path_from_rel("content/features/index.fr.md", "en"), "/fr/features/");
    }

    #[test]
    fn page_id_normalizes_content_prefix() {
        assert_eq!(page_id_from_rel("features/index.md"), "content/features/index.md");
        assert_eq!(page_id_from_rel("content/features/index.md"), "content/features/index.md");
    }

    #[test]
    fn absolute_url_prefixes_base() {
        assert_eq!(
            absolute_url("https://curriculo-me.pages.dev/", "/features/"),
            "https://curriculo-me.pages.dev/features/"
        );
    }
}
