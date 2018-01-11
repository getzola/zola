use std::path::{Path, Component};

use errors::Result;
use site::Site;
use content::{Page, Section};
use front_matter::{PageFrontMatter, SectionFrontMatter};


/// Finds the section that contains the page given if there is one
pub fn find_parent_section<'a>(site: &'a Site, page: &Page) -> Option<&'a Section> {
    for section in site.sections.values() {
        if section.is_child_page(&page.file.path) {
            return Some(section)
        }
    }

    None
}


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
    /// Editing `paginate_by`, `paginate_path` or `insert_anchor_links`
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

    if current.paginate_by != other.paginate_by
        || current.paginate_path != other.paginate_path
        || current.insert_anchor_links != other.insert_anchor_links {
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
        // A folder got deleted, ignore this event
        if !site.sections.contains_key(path) && !site.pages.contains_key(path) {
            return Ok(());
        }

        if is_section {
            // A section was deleted, many things can be impacted:
            // - the pages of the section are becoming orphans
            // - any page that was referencing the section (index, etc)
            let relative_path = site.sections[path].file.relative.clone();
            // Remove the link to it and the section itself from the Site
            site.permalinks.remove(&relative_path);
            site.sections.remove(path);
            site.populate_sections();
        } else {
            // A page was deleted, many things can be impacted:
            // - the section the page is in
            // - any page that was referencing the section (index, etc)
            let relative_path = site.pages[path].file.relative.clone();
            site.permalinks.remove(&relative_path);
            if let Some(p) = site.pages.remove(path) {
                if p.meta.has_tags() || p.meta.category.is_some() {
                    site.populate_tags_and_categories();
                }

                if find_parent_section(site, &p).is_some() {
                    site.populate_sections();
                }
            };
        }
        // Ensure we have our fn updated so it doesn't contain the permalinks deleted
        site.register_tera_global_fns();
        // Deletion is something that doesn't happen all the time so we
        // don't need to optimise it too much
        return site.build();
    }

    // A section was edited
    if is_section {
        let section = Section::from_file(path, &site.config)?;
        match site.add_section(section, true)? {
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
                site.register_tera_global_fns();
                return site.render_section(&site.sections[path], true);
            }
        };
    }

    // A page was edited
    let page = Page::from_file(path, &site.config)?;
    match site.add_page(page, true)? {
        Some(prev) => {
            // Updating a page
            let current = site.pages[path].clone();
            // Front matter didn't change, only content did
            // so we render only the section page, not its content
            if current.meta == prev.meta {
                return site.render_page(&current);
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
                        let section_path = match find_parent_section(site, &site.pages[path]) {
                            Some(s) => s.file.path.clone(),
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
            site.register_tera_global_fns();
            return Ok(());

        },
        None => {
            // It's a new page!
            site.populate_sections();
            site.populate_tags_and_categories();
            site.register_tera_global_fns();
            // No need to optimise that yet, we can revisit if it becomes an issue
            site.build()?;
        }
    }

    Ok(())
}

/// What happens when a template is changed
pub fn after_template_change(site: &mut Site, path: &Path) -> Result<()> {
    site.tera.full_reload()?;
    let filename = path.file_name().unwrap().to_str().unwrap();

    match filename {
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
            // If we are updating a shortcode, re-render the markdown of all pages/site
            // because we have no clue which one needs rebuilding
            // TODO: look if there the shortcode is used in the markdown instead of re-rendering
            // everything
            if path.components().collect::<Vec<_>>().contains(&Component::Normal("shortcodes".as_ref())) {
                site.render_markdown()?;
            }
            site.populate_sections();
            site.render_sections()?;
            site.render_orphan_pages()?;
            site.render_categories()?;
            site.render_tags()
        },
    }
}
