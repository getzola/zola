use content::Library;
use markdown::{WikilinkResolver, WikilinkTarget};

/// Build wikilink targets from every page and section in the content library.
///
/// Render-disabled content remains addressable to preserve the lookup behavior of the original
/// permalink-based implementation. Existing Zola aliases resolve to the canonical source path.
pub fn build_wikilinks(library: &Library) -> WikilinkResolver {
    let pages = library
        .pages
        .values()
        .map(|page| WikilinkTarget::new(page.file.relative.clone(), page.meta.aliases.clone()));
    let sections = library.sections.values().map(|section| {
        WikilinkTarget::new(section.file.relative.clone(), section.meta.aliases.clone())
    });
    WikilinkResolver::from_targets(pages.chain(sections))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use config::Config;
    use content::{Library, Page, PageFrontMatter, Section, SectionFrontMatter};

    use super::*;

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

        let resolver = build_wikilinks(&library);
        assert_eq!(resolver.resolve("start"), Ok("guides/quickstart.md"));
        assert_eq!(resolver.resolve("notes/private"), Ok("notes/private.md"));
        assert_eq!(resolver.resolve("private-note"), Ok("notes/private.md"));
        assert_eq!(resolver.resolve("docs/_index"), Ok("docs/_index.md"));
        assert_eq!(resolver.resolve("documentation"), Ok("docs/_index.md"));
    }
}
