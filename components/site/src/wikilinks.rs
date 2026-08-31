use content::Library;
use markdown::WikilinkResolver;

/// We take all pages/sections that will be rendered and build a resolver from those
pub fn build_wikilinks(library: &Library) -> WikilinkResolver {
    let mut resolver = WikilinkResolver::default();
    for p in library.pages.values().filter(|x| x.meta.render) {
        resolver.add(&p.file.relative, &p.meta.aliases);
    }
    for s in library.sections.values().filter(|x| x.meta.render) {
        resolver.add(&s.file.relative, &s.meta.aliases);
    }
    resolver
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use config::Config;
    use content::{Library, Page, PageFrontMatter, Section, SectionFrontMatter};
    use markdown::WikilinkError;

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
        assert_eq!(resolver.resolve("notes/private"), Err(WikilinkError::Missing));
        assert_eq!(resolver.resolve("private-note"), Err(WikilinkError::Missing));
        assert_eq!(resolver.resolve("docs/_index"), Ok("docs/_index.md"));
        assert_eq!(resolver.resolve("documentation"), Ok("docs/_index.md"));
    }
}
