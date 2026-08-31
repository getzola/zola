use ahash::AHashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum WikilinkError {
    Missing,
    Ambiguous { candidates: Vec<String> },
}

/// Resolves content wikilinks by source path, output alias, or bare stem.
///
/// For each target, the resolver indexes three forms that point back to the full source path:
///
/// 1. The source path without its `.md` extension, such as `docs/overview`.
/// 2. Each existing Zola alias, with surrounding slashes removed, such as `overview-old`.
/// 3. The bare stem, such as `overview`, when it differs from the full source path.
///
/// A stem that is identical to the full path is not indexed twice. Colliding aliases and stems are
/// retained so resolution can suggest a qualified key for every matching source path instead of
/// choosing one arbitrarily. Exact paths take precedence over aliases, and aliases take precedence
/// over stems.
#[derive(Clone, Debug, Default)]
pub struct WikilinkResolver {
    sources: Vec<String>,
    paths: AHashMap<String, Vec<usize>>,
    aliases: AHashMap<String, Vec<usize>>,
    stems: AHashMap<String, Vec<usize>>,
}

fn identity(value: &str) -> &str {
    value.strip_suffix(".md").unwrap_or(value)
}

fn normalize_lookup(value: &str) -> Option<&str> {
    let normalized = value.trim_matches('/');
    (!normalized.is_empty()).then_some(normalized)
}

impl WikilinkResolver {
    pub fn add(&mut self, source_path: &str, aliases: &[String]) {
        let index = self.sources.len();
        let id = identity(source_path);
        self.paths.entry(id.to_string()).or_default().push(index);

        // We want to keep the filename only if possible
        // eg docs/help.md --> help
        // but help.md --> N/A since it's already the stem
        if let Some(stem) = id.rsplit('/').next()
            && stem != id
        {
            self.stems.entry(stem.to_string()).or_default().push(index);
        }

        for alias in aliases {
            if let Some(s) = normalize_lookup(alias) {
                let candidates = self.aliases.entry(s.to_string()).or_default();
                if !candidates.contains(&index) {
                    candidates.push(index);
                }
            }
        }
        self.sources.push(source_path.to_string());
    }

    fn select(&self, candidates: &[usize]) -> Result<&str, WikilinkError> {
        if let [index] = candidates {
            return Ok(&self.sources[*index]);
        }

        let mut paths = candidates.iter().map(|index| &self.sources[*index]).collect::<Vec<_>>();
        paths.sort_unstable();
        paths.dedup();

        match paths.as_slice() {
            [path] => Ok(path),
            // We can't have missing here since we are only called if we have candidates
            _ => Err(WikilinkError::Ambiguous {
                candidates: paths.into_iter().map(|path| identity(path).to_string()).collect(),
            }),
        }
    }

    pub fn resolve(&self, target: &str) -> Result<&str, WikilinkError> {
        let Some(normalized) = normalize_lookup(target) else {
            return Err(WikilinkError::Missing);
        };

        if let Some(candidates) = self.paths.get(normalized) {
            return self.select(candidates);
        }
        if let Some(candidates) = self.aliases.get(normalized) {
            return self.select(candidates);
        }
        if !normalized.contains('/')
            && let Some(candidates) = self.stems.get(normalized)
        {
            return self.select(candidates);
        }

        Err(WikilinkError::Missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_paths_aliases_and_unique_stems() {
        let inputs = vec![
            ("blog/overview.md", Vec::new()),
            ("docs/overview.md", Vec::new()),
            ("about.md", Vec::new()),
            ("blog/_index.md", Vec::new()),
            ("_index.md", Vec::new()),
            ("guides/quickstart.md", vec!["/start/".to_string()]),
        ];
        let mut resolver = WikilinkResolver::default();
        for (a, b) in inputs {
            resolver.add(a, &b);
        }

        // Full paths always resolve.
        assert_eq!(resolver.resolve("blog/overview"), Ok("blog/overview.md"));
        assert_eq!(resolver.resolve("docs/overview"), Ok("docs/overview.md"));
        assert_eq!(resolver.resolve("about"), Ok("about.md"));
        assert_eq!(resolver.resolve("blog/_index"), Ok("blog/_index.md"));
        assert_eq!(resolver.resolve("guides/quickstart"), Ok("guides/quickstart.md"));

        // Unique stems and aliases resolve to the same source path.
        assert_eq!(resolver.resolve("quickstart"), Ok("guides/quickstart.md"));
        assert_eq!(resolver.resolve("start"), Ok("guides/quickstart.md"));

        // The exact root path takes precedence over the colliding blog/_index.md stem.
        assert_eq!(resolver.resolve("_index"), Ok("_index.md"));

        // A stem identical to its full path is not indexed separately.
        assert!(!resolver.stems.contains_key("about"));

        // Colliding bare stems suggest valid qualified keys for an actionable error.
        assert_eq!(
            resolver.resolve("overview"),
            Err(WikilinkError::Ambiguous {
                candidates: vec!["blog/overview".to_string(), "docs/overview".to_string()],
            })
        );

        // Accepting Markdown extensions would be an unrelated syntax expansion.
        assert_eq!(resolver.resolve("about.md"), Err(WikilinkError::Missing));
    }
}
