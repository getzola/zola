use ahash::AHashSet;
use content::{Library, Taxonomy};
use markdown::{TaxonomyPermalinks, WikilinkResolver};

/// Index published content, its colocated assets, and rendered taxonomy terms.
pub fn build_wikilinks(
    library: &Library,
    taxonomies: &[Taxonomy],
) -> (WikilinkResolver, TaxonomyPermalinks) {
    let mut resolver = WikilinkResolver::default();
    let mut asset_owners = AHashSet::new();
    for p in library.pages.values().filter(|x| x.meta.render) {
        resolver.add(&p.file.relative, &p.meta.aliases);
        asset_owners.insert(p.file.relative.as_str());
    }
    for s in library.sections.values().filter(|x| x.meta.render) {
        resolver.add(&s.file.relative, &s.meta.aliases);
        asset_owners.insert(s.file.relative.as_str());
    }
    for (path, (owner, _)) in &library.colocated_assets {
        if asset_owners.contains(owner.as_str()) {
            resolver.add_asset(path);
        }
    }
    let mut taxonomy_permalinks = TaxonomyPermalinks::default();
    for taxonomy in taxonomies.iter().filter(|taxonomy| taxonomy.kind.render) {
        for term in &taxonomy.items {
            taxonomy_permalinks
                .entry(format!("{}/{}", taxonomy.slug, term.slug))
                .or_default()
                .insert(taxonomy.lang.clone(), term.permalink.clone());
        }
    }
    for identity in taxonomy_permalinks.keys() {
        resolver.add_taxonomy_term(identity);
    }
    (resolver, taxonomy_permalinks)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use config::Config;
    use content::{Library, Page, PageFrontMatter, Section, SectionFrontMatter};
    use markdown::{WikilinkError, WikilinkTarget};

    fn page(path: &str, aliases: &[&str], render: bool) -> Page {
        let mut page = Page::new(
            Path::new(&format!("content/{path}")),
            PageFrontMatter::default(),
            Path::new(""),
        );
        page.file.relative = path.to_string();
        page.meta.aliases = aliases.iter().map(|alias| alias.to_string()).collect();
        page.meta.render = render;
        page
    }

    fn section(path: &str, aliases: &[&str]) -> Section {
        let mut section = Section::new(
            Path::new(&format!("content/{path}")),
            SectionFrontMatter::default(),
            Path::new(""),
        );
        section.file.relative = path.to_string();
        section.meta.aliases = aliases.iter().map(|alias| alias.to_string()).collect();
        section
    }

    #[test]
    fn includes_rendered_content_assets_and_taxonomies() {
        let mut config = Config::parse(
            r#"
base_url = "https://example.com/base"
taxonomy_root = "topics"
taxonomies = [{name = "Research Tags"}, {name = "disabled", render = false}]
"#,
        )
        .unwrap();
        config.markdown.wikilinks = true;
        let mut library = Library::new(&config);
        library.insert_page(page("guides/quickstart.md", &["/start/"], true));
        library.insert_page(page("notes/private.md", &["/private-note/"], false));
        library.insert_section(section("docs/_index.md", &["/documentation/"]));
        library.colocated_assets.insert(
            "docs/source.pdf".to_string(),
            ("docs/_index.md".to_string(), "source.pdf".to_string()),
        );
        library.colocated_assets.insert(
            "notes/private.pdf".to_string(),
            ("notes/private.md".to_string(), "private.pdf".to_string()),
        );

        let mut tagged = page("tagged.md", &[], true);
        tagged.lang = config.default_language.clone();
        tagged.meta.taxonomies.insert("Research Tags".into(), vec!["Some Evidence".into()]);
        tagged.meta.taxonomies.insert("disabled".into(), vec!["Secret".into()]);
        library.insert_page(tagged);
        let mut hidden = page("hidden.md", &[], true);
        hidden.lang = config.default_language.clone();
        hidden.hidden = true;
        hidden.meta.taxonomies.insert("Research Tags".into(), vec!["Hidden Term".into()]);
        library.insert_page(hidden);
        let taxonomies = library.find_taxonomies(&config).unwrap();
        let (resolver, taxonomy_permalinks) = build_wikilinks(&library, &taxonomies);
        assert_eq!(
            resolver.resolve("start"),
            Ok(&WikilinkTarget::Content("guides/quickstart.md".to_string()))
        );
        assert_eq!(resolver.resolve("notes/private"), Err(WikilinkError::Missing));
        assert_eq!(resolver.resolve("private-note"), Err(WikilinkError::Missing));
        assert_eq!(
            resolver.resolve("docs/_index"),
            Ok(&WikilinkTarget::Content("docs/_index.md".to_string()))
        );
        assert_eq!(
            resolver.resolve("documentation"),
            Ok(&WikilinkTarget::Content("docs/_index.md".to_string()))
        );
        assert_eq!(
            resolver.resolve("source.pdf"),
            Ok(&WikilinkTarget::Asset("docs/source.pdf".to_string()))
        );
        assert_eq!(resolver.resolve("private.pdf"), Err(WikilinkError::Missing));
        for key in ["research-tags/some-evidence", "some-evidence"] {
            assert_eq!(
                resolver.resolve(key),
                Ok(&WikilinkTarget::TaxonomyTerm("research-tags/some-evidence".into()))
            );
        }
        for key in ["disabled/secret", "secret", "research-tags/hidden-term", "hidden-term"] {
            assert_eq!(resolver.resolve(key), Err(WikilinkError::Missing));
        }
        let ctx = markdown::MarkdownContext {
            tera: &templates::ZOLA_TERA,
            config: &config,
            permalinks: &Default::default(),
            colocated_assets: &Default::default(),
            wikilinks: &resolver,
            taxonomy_permalinks: &taxonomy_permalinks,
            lang: &config.default_language,
            current_permalink: "",
            current_path: "",
            insert_anchor: utils::types::InsertAnchor::None,
        };
        let rendered = markdown::render_content("[[some-evidence]]", &ctx).unwrap();
        assert_eq!(
            rendered.body,
            "<p><a href=\"https://example.com/base/topics/research-tags/some-evidence/\">some-evidence</a></p>\n"
        );
    }
}
