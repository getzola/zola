use ahash::AHashMap;

#[derive(Debug, PartialEq, Eq)]
pub enum WikilinkError {
    Missing,
    Ambiguous { candidates: Vec<String> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WikilinkTarget {
    Content(String),
    Asset(String),
    TaxonomyTerm(String),
}

impl WikilinkTarget {
    fn identity(&self) -> &str {
        match self {
            Self::Content(path) => identity(path),
            Self::Asset(path) | Self::TaxonomyTerm(path) => path,
        }
    }
}

/// Resolves content wikilinks by source path, output alias, or bare stem; assets by path or bare
/// filename; and taxonomy terms by qualified identity or bare term slug.
///
/// Content targets are indexed by three forms that point back to the full source path:
///
/// 1. The source path without its `.md` extension, such as `docs/overview`.
/// 2. Each existing Zola alias, with surrounding slashes removed, such as `overview-old`.
/// 3. The bare stem, such as `overview`, when it differs from the full source path.
///
/// Assets are indexed by their exact content-root-relative path and bare filename. Taxonomy terms
/// use their language-neutral `taxonomy-slug/term-slug` identity and bare term slug. A short name
/// that is identical to the full path is not indexed twice. Collisions are retained so resolution
/// can suggest qualified keys instead of choosing one arbitrarily. Exact paths take precedence over
/// aliases, and aliases take precedence over short names.
#[derive(Clone, Debug, Default)]
pub struct WikilinkResolver {
    targets: Vec<WikilinkTarget>,
    paths: AHashMap<String, Vec<usize>>,
    aliases: AHashMap<String, Vec<usize>>,
    names: AHashMap<String, Vec<usize>>,
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
        self.insert(WikilinkTarget::Content(source_path.to_string()), aliases);
    }

    pub fn add_asset(&mut self, path: &str) {
        let Some(path) = normalize_lookup(path) else {
            return;
        };
        self.insert(WikilinkTarget::Asset(path.to_string()), &[]);
    }

    pub fn add_taxonomy_term(&mut self, identity: &str) {
        self.insert(WikilinkTarget::TaxonomyTerm(identity.to_string()), &[]);
    }

    fn insert(&mut self, target: WikilinkTarget, aliases: &[String]) {
        let index = self.targets.len();
        let id = target.identity();
        self.paths.entry(id.to_string()).or_default().push(index);

        if let Some(name) = id.rsplit('/').next()
            && name != id
        {
            self.names.entry(name.to_string()).or_default().push(index);
        }

        if matches!(target, WikilinkTarget::Content(_)) {
            for alias in aliases {
                if let Some(s) = normalize_lookup(alias) {
                    let candidates = self.aliases.entry(s.to_string()).or_default();
                    if !candidates.contains(&index) {
                        candidates.push(index);
                    }
                }
            }
        }
        self.targets.push(target);
    }

    fn select(&self, candidates: &[usize]) -> Result<&WikilinkTarget, WikilinkError> {
        if let [index] = candidates {
            return Ok(&self.targets[*index]);
        }

        let mut paths =
            candidates.iter().map(|index| self.targets[*index].identity()).collect::<Vec<_>>();
        paths.sort_unstable();
        paths.dedup();

        match paths.as_slice() {
            [_] => Ok(&self.targets[candidates[0]]),
            // We can't have missing here since we are only called if we have candidates
            _ => Err(WikilinkError::Ambiguous {
                candidates: paths.into_iter().map(str::to_string).collect(),
            }),
        }
    }

    pub fn resolve(&self, target: &str) -> Result<&WikilinkTarget, WikilinkError> {
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
            && let Some(candidates) = self.names.get(normalized)
        {
            return self.select(candidates);
        }

        Err(WikilinkError::Missing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_content(resolver: &WikilinkResolver, target: &str, expected: &str) {
        assert_eq!(resolver.resolve(target), Ok(&WikilinkTarget::Content(expected.to_string())));
    }

    fn assert_asset(resolver: &WikilinkResolver, target: &str, expected: &str) {
        assert_eq!(resolver.resolve(target), Ok(&WikilinkTarget::Asset(expected.to_string())));
    }

    #[test]
    fn resolves_paths_aliases_and_unique_names() {
        let inputs = vec![
            ("blog/overview.md", Vec::new()),
            ("docs/overview.md", Vec::new()),
            ("about.md", Vec::new()),
            ("blog/_index.md", Vec::new()),
            ("_index.md", Vec::new()),
            ("guides/quickstart.md", vec!["/start/".to_string()]),
            ("archive/manual.pdf.md", Vec::new()),
        ];
        let mut resolver = WikilinkResolver::default();
        for (a, b) in inputs {
            resolver.add(a, &b);
        }
        resolver.add_asset("guides/source.pdf");
        resolver.add_asset("guides/manual.pdf");
        resolver.add_taxonomy_term("tags/evidence");
        resolver.add_taxonomy_term("tags/overview");
        resolver.add_taxonomy_term("tags/manual.pdf");
        resolver.add_taxonomy_term("categories/evidence");
        resolver.add_taxonomy_term("tags/unique");
        resolver.add_taxonomy_term("tags/about");
        resolver.add_taxonomy_term("tags/start");

        // Full paths always resolve.
        assert_content(&resolver, "blog/overview", "blog/overview.md");
        assert_content(&resolver, "docs/overview", "docs/overview.md");
        assert_content(&resolver, "about", "about.md");
        assert_content(&resolver, "blog/_index", "blog/_index.md");
        assert_content(&resolver, "guides/quickstart", "guides/quickstart.md");

        // Unique stems and aliases resolve to the same source path.
        assert_content(&resolver, "quickstart", "guides/quickstart.md");
        assert_content(&resolver, "start", "guides/quickstart.md");

        // The exact root path takes precedence over the colliding blog/_index.md stem.
        assert_content(&resolver, "_index", "_index.md");

        // Colliding bare stems suggest valid qualified keys for an actionable error.
        assert_eq!(
            resolver.resolve("overview"),
            Err(WikilinkError::Ambiguous {
                candidates: vec![
                    "blog/overview".to_string(),
                    "docs/overview".to_string(),
                    "tags/overview".to_string()
                ],
            })
        );

        // Accepting Markdown extensions would be an unrelated syntax expansion.
        assert_eq!(resolver.resolve("about.md"), Err(WikilinkError::Missing));

        // Asset paths and unique filenames use the same resolver.
        assert_asset(&resolver, "guides/source.pdf", "guides/source.pdf");
        assert_asset(&resolver, "source.pdf", "guides/source.pdf");

        // Colliding short names require a qualified path regardless of target kind.
        assert_eq!(
            resolver.resolve("manual.pdf"),
            Err(WikilinkError::Ambiguous {
                candidates: vec![
                    "archive/manual.pdf".to_string(),
                    "guides/manual.pdf".to_string(),
                    "tags/manual.pdf".to_string()
                ],
            })
        );
        assert_asset(&resolver, "guides/manual.pdf", "guides/manual.pdf");
        assert_content(&resolver, "archive/manual.pdf", "archive/manual.pdf.md");
        assert_eq!(resolver.resolve("manual"), Err(WikilinkError::Missing));

        for (key, identity) in [
            ("tags/evidence", "tags/evidence"),
            ("/tags/evidence/", "tags/evidence"),
            ("unique", "tags/unique"),
            ("tags/unique", "tags/unique"),
            ("tags/about", "tags/about"),
            ("tags/start", "tags/start"),
        ] {
            assert_eq!(resolver.resolve(key), Ok(&WikilinkTarget::TaxonomyTerm(identity.into())));
        }
        assert_eq!(
            resolver.resolve("evidence"),
            Err(WikilinkError::Ambiguous {
                candidates: vec!["categories/evidence".into(), "tags/evidence".into()],
            })
        );
    }
}
