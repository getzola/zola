extern crate site;
#[macro_use]
extern crate errors;
extern crate content;
extern crate front_matter;

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
pub enum PageChangesNeeded {
    /// Editing `tags`
    Tags,
    /// Editing `categories`
    Categories,
    /// Editing `date`, `order` or `weight`
    Sort,
    /// Editing anything causes a re-render of the page
    Render,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SectionChangesNeeded {
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
/// Order matters as the actions will be done in insertion order
fn find_section_front_matter_changes(current: &SectionFrontMatter, new: &SectionFrontMatter) -> Vec<SectionChangesNeeded> {
    let mut changes_needed = vec![];

    if current.sort_by != new.sort_by {
        changes_needed.push(SectionChangesNeeded::Sort);
    }

    // We want to hide the section
    // TODO: what to do on redirect_path change?
    if current.render && !new.render {
        changes_needed.push(SectionChangesNeeded::Delete);
        // Nothing else we can do
        return changes_needed;
    }

    if current.paginate_by != new.paginate_by
        || current.paginate_path != new.paginate_path
        || current.insert_anchor_links != new.insert_anchor_links {
        changes_needed.push(SectionChangesNeeded::RenderWithPages);
        // Nothing else we can do
        return changes_needed;
    }

    // Any new change will trigger a re-rendering of the section page only
    changes_needed.push(SectionChangesNeeded::Render);
    changes_needed
}

/// Evaluates all the params in the front matter that changed so we can do the smallest
/// delta in the serve command
/// Order matters as the actions will be done in insertion order
fn find_page_front_matter_changes(current: &PageFrontMatter, other: &PageFrontMatter) -> Vec<PageChangesNeeded> {
    let mut changes_needed = vec![];

    if current.tags != other.tags {
        changes_needed.push(PageChangesNeeded::Tags);
    }

    if current.category != other.category {
        changes_needed.push(PageChangesNeeded::Categories);
    }

    if current.date != other.date || current.order != other.order || current.weight != other.weight {
        changes_needed.push(PageChangesNeeded::Sort);
    }

    changes_needed.push(PageChangesNeeded::Render);
    changes_needed
}

/// Handles a path deletion: could be a page, a section, a folder
fn delete_element(site: &mut Site, path: &Path, is_section: bool) -> Result<()> {
    // Ignore the event if this path was not known
    if !site.sections.contains_key(path) && !site.pages.contains_key(path) {
        return Ok(());
    }

    if is_section {
        if let Some(s) = site.pages.remove(path) {
            site.permalinks.remove(&s.file.relative);
            site.populate_sections();
        }
    } else {
        if let Some(p) = site.pages.remove(path) {
            site.permalinks.remove(&p.file.relative);

            if p.meta.has_tags() || p.meta.category.is_some() {
                site.populate_tags_and_categories();
            }

            // if there is a parent section, we will need to re-render it
            // most likely
            if find_parent_section(site, &p).is_some() {
                site.populate_sections();
            }
        };
    }

    // Ensure we have our fn updated so it doesn't contain the permalink(s)/section/page deleted
    site.register_tera_global_fns();
    // Deletion is something that doesn't happen all the time so we
    // don't need to optimise it too much
    return site.build();
}

/// Handles a `_index.md` (a section) being edited in some ways
fn handle_section_editing(site: &mut Site, path: &Path) -> Result<()> {
    let section = Section::from_file(path, &site.config)?;
    match site.add_section(section, true)? {
        // Updating a section
        Some(prev) => {
            if site.sections[path].meta == prev.meta {
                // Front matter didn't change, only content did
                // so we render only the section page, not its pages
                return site.render_section(&site.sections[path], false);
            }

            // Front matter changed
            for changes in find_section_front_matter_changes(&site.sections[path].meta, &prev.meta) {
                // Sort always comes first if present so the rendering will be fine
                match changes {
                    SectionChangesNeeded::Sort => {
                        site.sort_sections_pages(Some(path));
                        site.register_tera_global_fns();
                    },
                    SectionChangesNeeded::Render => site.render_section(&site.sections[path], false)?,
                    SectionChangesNeeded::RenderWithPages => site.render_section(&site.sections[path], true)?,
                    // not a common enough operation to make it worth optimizing
                    SectionChangesNeeded::Delete => {
                        site.populate_sections();
                        site.build()?;
                    },
                };
            }
            return Ok(());
        },
        // New section, only render that one
        None => {
            site.populate_sections();
            site.register_tera_global_fns();
            return site.render_section(&site.sections[path], true);
        }
    };
}

macro_rules! render_parent_section {
    ($site: expr, $path: expr) => {
        match find_parent_section($site, &$site.pages[$path]) {
            Some(s) => {
                $site.render_section(s, false)?;
            },
            None => (),
        };
    }
}

/// Handles a page being edited in some ways
fn handle_page_editing(site: &mut Site, path: &Path) -> Result<()> {
    let page = Page::from_file(path, &site.config)?;
    match site.add_page(page, true)? {
        // Updating a page
        Some(prev) => {
            // Front matter didn't change, only content did
            if site.pages[path].meta == prev.meta {
                // Other than the page itself, the summary might be seen
                // on a paginated list for a blog for example
                if site.pages[path].summary.is_some() {
                    render_parent_section!(site, path);
                }
                // TODO: register_tera_global_fns is expensive as it involves lots of cloning
                // I can't think of a valid usecase where you would need the content
                // of a page through a global fn so it's commented out for now
                // site.register_tera_global_fns();
                return site.render_page(& site.pages[path]);
            }

            // Front matter changed
            let mut taxonomies_populated = false;
            let mut sections_populated = false;
            for changes in find_page_front_matter_changes(&site.pages[path].meta, &prev.meta) {
                // Sort always comes first if present so the rendering will be fine
                match changes {
                    PageChangesNeeded::Tags => {
                        if !taxonomies_populated {
                            site.populate_tags_and_categories();
                            taxonomies_populated = true;
                        }
                        site.register_tera_global_fns();
                        site.render_tags()?;
                    },
                    PageChangesNeeded::Categories => {
                        if !taxonomies_populated {
                            site.populate_tags_and_categories();
                            taxonomies_populated = true;
                        }
                        site.register_tera_global_fns();
                        site.render_categories()?;
                    },
                    PageChangesNeeded::Sort => {
                        let section_path = match find_parent_section(site, &site.pages[path]) {
                            Some(s) => s.file.path.clone(),
                            None => continue  // Do nothing if it's an orphan page
                        };
                        if !sections_populated {
                            site.populate_sections();
                            sections_populated = true;
                        }
                        site.sort_sections_pages(Some(&section_path));
                        site.register_tera_global_fns();
                        site.render_index()?;
                    },
                    PageChangesNeeded::Render => {
                        if !sections_populated {
                            site.populate_sections();
                            sections_populated = true;
                        }
                        site.register_tera_global_fns();
                        render_parent_section!(site, path);
                        site.render_page(&site.pages[path])?;
                    },
                };
            }
            Ok(())
        },
        // It's a new page!
        None => {
            site.populate_sections();
            site.populate_tags_and_categories();
            site.register_tera_global_fns();
            // No need to optimise that yet, we can revisit if it becomes an issue
            site.build()
        }
    }
}


/// What happens when a section or a page is changed
pub fn after_content_change(site: &mut Site, path: &Path) -> Result<()> {
    let is_section = path.file_name().unwrap() == "_index.md";
    let is_md = path.extension().unwrap() == "md";
    let index = path.parent().unwrap().join("index.md");

    // A few situations can happen:
    // 1. Change on .md files
    //    a. Is there an `index.md`? Return an error if it's something other than delete
    //    b. Deleted? remove the element
    //    c. Edited?
    //       1. filename is `_index.md`, this is a section
    //       1. it's a page otherwise
    // 2. Change on non .md files
    //    a. Try to find a corresponding `_index.md`
    //       1. Nothing? Return Ok
    //       2. Something? Update the page
    if is_md {
        // only delete if it was able to be added in the first place
        if !index.exists() && !path.exists() {
            delete_element(site, path, is_section)?;
        }

        // Added another .md in a assets directory
        if index.exists() && path.exists() && path != index {
            bail!(
                "Change on {:?} detected but there is already an `index.md` in the same folder",
                path.display()
            );
        } else if index.exists() && !path.exists() {
            // deleted the wrong .md, do nothing
            return Ok(());
        }

        if is_section {
            handle_section_editing(site, path)
        } else {
            handle_page_editing(site, path)
        }
    } else {
        if index.exists()  {
            handle_page_editing(site, &index)
        } else {
            Ok(())
        }
    }
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


#[cfg(test)]
mod tests {
    use front_matter::{PageFrontMatter, SectionFrontMatter, SortBy};
    use super::{
        find_page_front_matter_changes, find_section_front_matter_changes,
        PageChangesNeeded, SectionChangesNeeded
    };

    #[test]
    fn can_find_tag_changes_in_page_frontmatter() {
        let new = PageFrontMatter { tags: Some(vec!["a tag".to_string()]), ..PageFrontMatter::default() };
        let changes = find_page_front_matter_changes(&PageFrontMatter::default(), &new);
        assert_eq!(changes, vec![PageChangesNeeded::Tags, PageChangesNeeded::Render]);
    }

    #[test]
    fn can_find_category_changes_in_page_frontmatter() {
        let current = PageFrontMatter { category: Some("a category".to_string()), ..PageFrontMatter::default() };
        let changes = find_page_front_matter_changes(&current, &PageFrontMatter::default());
        assert_eq!(changes, vec![PageChangesNeeded::Categories, PageChangesNeeded::Render]);
    }

    #[test]
    fn can_find_multiple_changes_in_page_frontmatter() {
        let current = PageFrontMatter { category: Some("a category".to_string()), order: Some(1), ..PageFrontMatter::default() };
        let changes = find_page_front_matter_changes(&current, &PageFrontMatter::default());
        assert_eq!(changes, vec![PageChangesNeeded::Categories, PageChangesNeeded::Sort, PageChangesNeeded::Render]);
    }

    #[test]
    fn can_find_sort_changes_in_section_frontmatter() {
        let new = SectionFrontMatter { sort_by: SortBy::Date, ..SectionFrontMatter::default() };
        let changes = find_section_front_matter_changes(&SectionFrontMatter::default(), &new);
        assert_eq!(changes, vec![SectionChangesNeeded::Sort, SectionChangesNeeded::Render]);
    }

    #[test]
    fn can_find_render_changes_in_section_frontmatter() {
        let new = SectionFrontMatter { render: false, ..SectionFrontMatter::default() };
        let changes = find_section_front_matter_changes(&SectionFrontMatter::default(), &new);
        assert_eq!(changes, vec![SectionChangesNeeded::Delete]);
    }

    #[test]
    fn can_find_paginate_by_changes_in_section_frontmatter() {
        let new = SectionFrontMatter { paginate_by: Some(10), ..SectionFrontMatter::default() };
        let changes = find_section_front_matter_changes(&SectionFrontMatter::default(), &new);
        assert_eq!(changes, vec![SectionChangesNeeded::RenderWithPages]);
    }
}
