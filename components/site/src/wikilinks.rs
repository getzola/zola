use ahash::AHashSet;
use content::Library;
use markdown::WikilinkResolver;

/// We take all pages/sections that will be rendered and build a resolver from those
pub fn build_wikilinks(library: &Library) -> WikilinkResolver {
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
    resolver
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
    fn includes_aliases_and_render_disabled_content() {
        let config = Config::default_for_test();
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

        let resolver = build_wikilinks(&library);
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
    }
}
