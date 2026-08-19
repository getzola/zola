use ahash::AHashMap;

#[derive(Clone, Debug)]
pub struct WikilinkTarget {
    source_path: String,
    aliases: Vec<String>,
}

impl WikilinkTarget {
    pub fn new(source_path: impl Into<String>, aliases: Vec<String>) -> Self {
        Self { source_path: source_path.into(), aliases }
    }
}

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
    targets: Vec<WikilinkTarget>,
    paths: AHashMap<String, Vec<usize>>,
    aliases: AHashMap<String, Vec<usize>>,
    stems: AHashMap<String, Vec<usize>>,
}

fn normalize_source_path(value: &str) -> Option<String> {
    let normalized = value.trim_matches('/');
    let normalized = normalized.strip_suffix(".md").unwrap_or(normalized);
    (!normalized.is_empty()).then(|| normalized.to_string())
}

fn normalize_lookup(value: &str) -> Option<&str> {
    let normalized = value.trim_matches('/');
    (!normalized.is_empty()).then_some(normalized)
}

impl WikilinkResolver {
    pub fn from_targets(targets: impl IntoIterator<Item = WikilinkTarget>) -> Self {
        let mut resolver = Self::default();
        for target in targets {
            resolver.insert(target);
        }
        resolver
    }

    fn insert(&mut self, target: WikilinkTarget) {
        let Some(identity) = normalize_source_path(&target.source_path) else {
            return;
        };
        let index = self.targets.len();

        // Store the full source path without its Markdown extension.
        self.paths.entry(identity.clone()).or_default().push(index);

        // A bare stem is useful only when it differs from the full path. Keeping it in a separate
        // index lets resolution report collisions without overwriting an exact path.
        if let Some(stem) = identity.rsplit('/').next()
            && stem != identity
        {
            self.stems.entry(stem.to_string()).or_default().push(index);
        }

        // Aliases are existing Zola output paths, normalized to wikilink syntax.
        for alias in &target.aliases {
            if let Some(alias) = normalize_lookup(alias) {
                let candidates = self.aliases.entry(alias.to_string()).or_default();
                if !candidates.contains(&index) {
                    candidates.push(index);
                }
            }
        }
        self.targets.push(target);
    }

    fn select(&self, candidates: &[usize]) -> std::result::Result<&str, WikilinkError> {
        if let [index] = candidates {
            return Ok(&self.targets[*index].source_path);
        }

        let mut paths = candidates
            .iter()
            .map(|index| self.targets[*index].source_path.as_str())
            .collect::<Vec<_>>();
        paths.sort_unstable();
        paths.dedup();

        match paths.as_slice() {
            [] => Err(WikilinkError::Missing),
            [path] => Ok(path),
            _ => Err(WikilinkError::Ambiguous {
                candidates: paths
                    .into_iter()
                    .map(|path| {
                        normalize_source_path(path)
                            .expect("indexed targets have normalized source paths")
                    })
                    .collect(),
            }),
        }
    }

    pub fn resolve(&self, target: &str) -> std::result::Result<&str, WikilinkError> {
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

    fn target(path: &str) -> WikilinkTarget {
        WikilinkTarget::new(path, Vec::new())
    }

    #[test]
    fn resolves_paths_aliases_and_unique_stems() {
        let resolver = WikilinkResolver::from_targets([
            target("blog/overview.md"),
            target("docs/overview.md"),
            target("about.md"),
            target("blog/_index.md"),
            target("_index.md"),
            WikilinkTarget::new("guides/quickstart.md", vec!["/start/".to_string()]),
        ]);

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
