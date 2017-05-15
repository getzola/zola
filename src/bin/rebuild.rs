use std::path::Path;

use gutenberg::{Site, SectionFrontMatter, PageFrontMatter};
use gutenberg::errors::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
enum PageChangesNeeded {
    /// Editing `tags`
    Tags,
    /// Editing `categories`
    Categories,
    /// Editing `date` or `order`
    Sort,
    /// Editing anything else
    Render,
}

// TODO: seems like editing sort_by/render do weird stuff
#[derive(Debug, Clone, Copy, PartialEq)]
enum SectionChangesNeeded {
    /// Editing `sort_by`
    Sort,
    /// Editing `title`, `description`, `extra`, `template` or setting `render` to true
    Render,
    /// Editing `paginate_by` or `paginate_path`
    RenderWithPages,
    /// Setting `render` to false
    Delete,
}

/// Evaluates all the params in the front matter that changed so we can do the smallest
/// delta in the serve command
fn find_section_front_matter_changes(current: &SectionFrontMatter, other: &SectionFrontMatter) -> Vec<SectionChangesNeeded> {
    let mut changes_needed = vec![];

    if current.sort_by != other.sort_by {
        changes_needed.push(SectionChangesNeeded::Sort);
    }

    if !current.should_render() && other.should_render() {
        changes_needed.push(SectionChangesNeeded::Delete);
        // Nothing else we can do
        return changes_needed;
    }

    if current.paginate_by != other.paginate_by || current.paginate_path != other.paginate_path {
        changes_needed.push(SectionChangesNeeded::RenderWithPages);
        // Nothing else we can do
        return changes_needed;
    }

    // Any other change will trigger a re-rendering of the section page only
    changes_needed.push(SectionChangesNeeded::Render);
    changes_needed
}

/// Evaluates all the params in the front matter that changed so we can do the smallest
/// delta in the serve command
fn find_page_front_matter_changes(current: &PageFrontMatter, other: &PageFrontMatter) -> Vec<PageChangesNeeded> {
    let mut changes_needed = vec![];

    if current.tags != other.tags {
        changes_needed.push(PageChangesNeeded::Tags);
    }

    if current.category != other.category {
        changes_needed.push(PageChangesNeeded::Categories);
    }

    if current.date != other.date || current.order != other.order {
        changes_needed.push(PageChangesNeeded::Sort);
    }

    changes_needed.push(PageChangesNeeded::Render);
    changes_needed
}

// What happens when a section or a page is changed
pub fn after_content_change(site: &mut Site, path: &Path) -> Result<()> {
    let is_section = path.file_name().unwrap() == "_index.md";

    // A page or section got deleted
    if !path.exists() {
        if is_section {
            // A section was deleted, many things can be impacted:
            // - the pages of the section are becoming orphans
            // - any page that was referencing the section (index, etc)
            let relative_path = site.sections[path].relative_path.clone();
            // Remove the link to it and the section itself from the Site
            site.permalinks.remove(&relative_path);
            site.sections.remove(path);
            site.populate_sections();
        } else {
            // A page was deleted, many things can be impacted:
            // - the section the page is in
            // - any page that was referencing the section (index, etc)
            let relative_path = site.pages[path].relative_path.clone();
            site.permalinks.remove(&relative_path);
            if let Some(p) = site.pages.remove(path) {
                if p.meta.has_tags() || p.meta.category.is_some() {
                    site.populate_tags_and_categories();
                }

                if site.find_parent_section(&p).is_some() {
                    site.populate_sections();
                }
            };
        }
        // Deletion is something that doesn't happen all the time so we
        // don't need to optimise it too much
        return site.build();
    }

    // A section was edited
    if is_section {
        match site.add_section(path, true)? {
            Some(prev) => {
                // Updating a section
                let current_meta = site.sections[path].meta.clone();
                // Front matter didn't change, only content did
                // so we render only the section page, not its pages
                if current_meta == prev.meta {
                    return site.render_section(&site.sections[path], false);
                }

                // Front matter changed
                for changes in find_section_front_matter_changes(&current_meta, &prev.meta) {
                    // Sort always comes first if present so the rendering will be fine
                    match changes {
                        SectionChangesNeeded::Sort => site.sort_sections_pages(Some(path)),
                        SectionChangesNeeded::Render => site.render_section(&site.sections[path], false)?,
                        SectionChangesNeeded::RenderWithPages => site.render_section(&site.sections[path], true)?,
                        // can't be arsed to make the Delete efficient, it's not a common enough operation
                        SectionChangesNeeded::Delete => {
                            site.populate_sections();
                            site.build()?;
                        },
                    };
                }
                return Ok(());
            },
            None => {
                // New section, only render that one
                site.populate_sections();
                return site.render_section(&site.sections[path], true);
            }
        };
    }

    // A page was edited
    match site.add_page(path, true)? {
        Some(prev) => {
            // Updating a page
            let current = site.pages[path].clone();
            // Front matter didn't change, only content did
            // so we render only the section page, not its pages
            if current.meta == prev.meta {
                return site.render_page(&site.pages[path]);
            }

            // Front matter changed
            for changes in find_page_front_matter_changes(&current.meta, &prev.meta) {
                // Sort always comes first if present so the rendering will be fine
                match changes {
                    PageChangesNeeded::Tags => {
                        site.populate_tags_and_categories();
                        site.render_tags()?;
                    },
                    PageChangesNeeded::Categories => {
                        site.populate_tags_and_categories();
                        site.render_categories()?;
                    },
                    PageChangesNeeded::Sort => {
                        let section_path = match site.find_parent_section(&site.pages[path]) {
                            Some(s) => s.file_path.clone(),
                            None => continue  // Do nothing if it's an orphan page
                        };
                        site.populate_sections();
                        site.sort_sections_pages(Some(&section_path));
                        site.render_index()?;
                    },
                    PageChangesNeeded::Render => {
                        site.render_page(&site.pages[path])?;
                    },
                };
            }
            return Ok(());

        },
        None => {
            // It's a new page!
            site.populate_sections();
            site.populate_tags_and_categories();
            // No need to optimise that yet, we can revisit if it becomes an issue
            site.build()?;
        }
    }

    Ok(())
}

/// What happens when a template is changed
pub fn after_template_change(site: &mut Site, path: &Path) -> Result<()> {
    site.tera.full_reload()?;

    match path.file_name().unwrap().to_str().unwrap() {
        "sitemap.xml" => site.render_sitemap(),
        "rss.xml" => site.render_rss_feed(),
        "robots.txt" => site.render_robots(),
        "categories.html" | "category.html" => site.render_categories(),
        "tags.html" | "tag.html" => site.render_tags(),
        "page.html" => {
            site.render_sections()?;
            site.render_orphan_pages()
        },
        "section.html" => site.render_sections(),
        // Either the index or some unknown template changed
        // We can't really know what this change affects so rebuild all
        // the things
        _ => {
            site.render_sections()?;
            site.render_orphan_pages()?;
            site.render_categories()?;
            site.render_tags()
        },
    }
}
