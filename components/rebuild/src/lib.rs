extern crate site;
#[macro_use]
extern crate errors;
extern crate front_matter;
extern crate library;

use std::path::{Component, Path};

use errors::Result;
use front_matter::{PageFrontMatter, SectionFrontMatter};
use library::{Page, Section};
use site::Site;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PageChangesNeeded {
    /// Editing `taxonomies`
    Taxonomies,
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
    /// Changing `transparent`
    Transparent,
}

/// Evaluates all the params in the front matter that changed so we can do the smallest
/// delta in the serve command
/// Order matters as the actions will be done in insertion order
fn find_section_front_matter_changes(
    current: &SectionFrontMatter,
    new: &SectionFrontMatter,
) -> Vec<SectionChangesNeeded> {
    let mut changes_needed = vec![];

    if current.sort_by != new.sort_by {
        changes_needed.push(SectionChangesNeeded::Sort);
    }

    if current.transparent != new.transparent {
        changes_needed.push(SectionChangesNeeded::Transparent);
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
        || current.insert_anchor_links != new.insert_anchor_links
    {
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
fn find_page_front_matter_changes(
    current: &PageFrontMatter,
    other: &PageFrontMatter,
) -> Vec<PageChangesNeeded> {
    let mut changes_needed = vec![];

    if current.taxonomies != other.taxonomies {
        changes_needed.push(PageChangesNeeded::Taxonomies);
    }

    if current.date != other.date || current.order != other.order || current.weight != other.weight
    {
        changes_needed.push(PageChangesNeeded::Sort);
    }

    changes_needed.push(PageChangesNeeded::Render);
    changes_needed
}

/// Handles a path deletion: could be a page, a section, a folder
fn delete_element(site: &mut Site, path: &Path, is_section: bool) -> Result<()> {
    {
        let mut library = site.library.write().unwrap();
        // Ignore the event if this path was not known
        if !library.contains_section(&path.to_path_buf())
            && !library.contains_page(&path.to_path_buf())
        {
            return Ok(());
        }

        if is_section {
            if let Some(s) = library.remove_section(&path.to_path_buf()) {
                site.permalinks.remove(&s.file.relative);
            }
        } else if let Some(p) = library.remove_page(&path.to_path_buf()) {
            site.permalinks.remove(&p.file.relative);
        }
    }

    // We might have delete the root _index.md so ensure we have at least the default one
    // before populating
    site.create_default_index_sections()?;
    site.populate_sections();
    site.populate_taxonomies()?;
    // Ensure we have our fn updated so it doesn't contain the permalink(s)/section/page deleted
    site.register_early_global_fns();
    site.register_tera_global_fns();
    // Deletion is something that doesn't happen all the time so we
    // don't need to optimise it too much
    site.build()
}

/// Handles a `_index.md` (a section) being edited in some ways
fn handle_section_editing(site: &mut Site, path: &Path) -> Result<()> {
    let section = Section::from_file(path, &site.config, &site.base_path)?;
    let pathbuf = path.to_path_buf();
    match site.add_section(section, true)? {
        // Updating a section
        Some(prev) => {
            site.populate_sections();
            {
                let library = site.library.read().unwrap();

                if library.get_section(&pathbuf).unwrap().meta == prev.meta {
                    // Front matter didn't change, only content did
                    // so we render only the section page, not its pages
                    return site.render_section(&library.get_section(&pathbuf).unwrap(), false);
                }
            }

            // Front matter changed
            let changes = find_section_front_matter_changes(
                &site.library.read().unwrap().get_section(&pathbuf).unwrap().meta,
                &prev.meta,
            );
            for change in changes {
                // Sort always comes first if present so the rendering will be fine
                match change {
                    SectionChangesNeeded::Sort => {
                        site.register_tera_global_fns();
                    }
                    SectionChangesNeeded::Render => site.render_section(
                        &site.library.read().unwrap().get_section(&pathbuf).unwrap(),
                        false,
                    )?,
                    SectionChangesNeeded::RenderWithPages => site.render_section(
                        &site.library.read().unwrap().get_section(&pathbuf).unwrap(),
                        true,
                    )?,
                    // not a common enough operation to make it worth optimizing
                    SectionChangesNeeded::Delete | SectionChangesNeeded::Transparent => {
                        site.build()?;
                    }
                };
            }
            Ok(())
        }
        // New section, only render that one
        None => {
            site.populate_sections();
            site.register_tera_global_fns();
            site.render_section(&site.library.read().unwrap().get_section(&pathbuf).unwrap(), true)
        }
    }
}

macro_rules! render_parent_sections {
    ($site: expr, $path: expr) => {
        for s in $site.library.read().unwrap().find_parent_sections($path) {
            $site.render_section(s, false)?;
        }
    };
}

/// Handles a page being edited in some ways
fn handle_page_editing(site: &mut Site, path: &Path) -> Result<()> {
    let page = Page::from_file(path, &site.config, &site.base_path)?;
    let pathbuf = path.to_path_buf();
    match site.add_page(page, true)? {
        // Updating a page
        Some(prev) => {
            site.populate_sections();
            site.populate_taxonomies()?;
            site.register_tera_global_fns();
            {
                let library = site.library.read().unwrap();

                // Front matter didn't change, only content did
                if library.get_page(&pathbuf).unwrap().meta == prev.meta {
                    // Other than the page itself, the summary might be seen
                    // on a paginated list for a blog for example
                    if library.get_page(&pathbuf).unwrap().summary.is_some() {
                        render_parent_sections!(site, path);
                    }
                    return site.render_page(&library.get_page(&pathbuf).unwrap());
                }
            }

            // Front matter changed
            let changes = find_page_front_matter_changes(
                &site.library.read().unwrap().get_page(&pathbuf).unwrap().meta,
                &prev.meta,
            );

            for change in changes {
                site.register_tera_global_fns();

                // Sort always comes first if present so the rendering will be fine
                match change {
                    PageChangesNeeded::Taxonomies => {
                        site.populate_taxonomies()?;
                        site.render_taxonomies()?;
                    }
                    PageChangesNeeded::Sort => {
                        site.render_index()?;
                    }
                    PageChangesNeeded::Render => {
                        render_parent_sections!(site, path);
                        site.render_page(
                            &site.library.read().unwrap().get_page(&path.to_path_buf()).unwrap(),
                        )?;
                    }
                };
            }
            Ok(())
        }
        // It's a new page!
        None => {
            site.populate_sections();
            site.populate_taxonomies()?;
            site.register_early_global_fns();
            site.register_tera_global_fns();
            // No need to optimise that yet, we can revisit if it becomes an issue
            site.build()
        }
    }
}

/// What happens when we rename a file/folder in the content directory.
/// Note that this is only called for folders when it isn't empty
pub fn after_content_rename(site: &mut Site, old: &Path, new: &Path) -> Result<()> {
    let new_path = if new.is_dir() {
        if new.join("_index.md").exists() {
            // This is a section keep the dir folder to differentiate from renaming _index.md
            // which doesn't do the same thing
            new.to_path_buf()
        } else if new.join("index.md").exists() {
            new.join("index.md")
        } else {
            bail!("Got unexpected folder {:?} while handling renaming that was not expected", new);
        }
    } else {
        new.to_path_buf()
    };

    // A section folder has been renamed: just reload the whole site and rebuild it as we
    // do not really know what needs to be rendered
    if new_path.is_dir() {
        site.load()?;
        return site.build();
    }

    // We ignore renames on non-markdown files for now
    if let Some(ext) = new_path.extension() {
        if ext != "md" {
            return Ok(());
        }
    }

    // Renaming a file to _index.md, let the section editing do something and hope for the best
    if new_path.file_name().unwrap() == "_index.md" {
        // We aren't entirely sure where the original thing was so just try to delete whatever was
        // at the old path
        {
            let mut library = site.library.write().unwrap();
            library.remove_page(&old.to_path_buf());
            library.remove_section(&old.to_path_buf());
        }
        return handle_section_editing(site, &new_path);
    }

    // If it is a page, just delete what was there before and
    // fake it's a new page
    let old_path = if new_path.file_name().unwrap() == "index.md" {
        old.join("index.md")
    } else {
        old.to_path_buf()
    };
    site.library.write().unwrap().remove_page(&old_path);
    handle_page_editing(site, &new_path)
}

/// What happens when a section or a page is created/edited
pub fn after_content_change(site: &mut Site, path: &Path) -> Result<()> {
    let is_section = path.file_name().unwrap() == "_index.md";
    let is_md = path.extension().unwrap() == "md";
    let index = path.parent().unwrap().join("index.md");

    let mut potential_indices = vec![path.parent().unwrap().join("index.md")];
    for language in &site.config.languages {
        potential_indices.push(path.parent().unwrap().join(format!("index.{}.md", language.code)));
    }
    let colocated_index = potential_indices.contains(&path.to_path_buf());

    // A few situations can happen:
    // 1. Change on .md files
    //    a. Is there already an `index.md`? Return an error if it's something other than delete
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
            return delete_element(site, path, is_section);
        }

        // Added another .md in a assets directory
        if index.exists() && path.exists() && !colocated_index {
            bail!(
                "Change on {:?} detected but only files named `index.md` with an optional language code are allowed",
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
    } else if index.exists() {
        handle_page_editing(site, &index)
    } else {
        Ok(())
    }
}

/// What happens when a template is changed
pub fn after_template_change(site: &mut Site, path: &Path) -> Result<()> {
    site.tera.full_reload()?;
    let filename = path.file_name().unwrap().to_str().unwrap();

    match filename {
        "sitemap.xml" => site.render_sitemap(),
        "rss.xml" => site.render_rss_feed(site.library.read().unwrap().pages_values(), None),
        "split_sitemap_index.xml" => site.render_sitemap(),
        "robots.txt" => site.render_robots(),
        "single.html" | "list.html" => site.render_taxonomies(),
        "page.html" => {
            site.render_sections()?;
            site.render_orphan_pages()
        }
        "section.html" => site.render_sections(),
        "404.html" => site.render_404(),
        // Either the index or some unknown template changed
        // We can't really know what this change affects so rebuild all
        // the things
        _ => {
            // If we are updating a shortcode, re-render the markdown of all pages/site
            // because we have no clue which one needs rebuilding
            // TODO: look if there the shortcode is used in the markdown instead of re-rendering
            // everything
            if path.components().any(|x| x == Component::Normal("shortcodes".as_ref())) {
                site.render_markdown()?;
            }
            site.populate_sections();
            site.populate_taxonomies()?;
            site.render_sections()?;
            site.render_orphan_pages()?;
            site.render_taxonomies()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        find_page_front_matter_changes, find_section_front_matter_changes, PageChangesNeeded,
        SectionChangesNeeded,
    };
    use front_matter::{PageFrontMatter, SectionFrontMatter, SortBy};

    #[test]
    fn can_find_taxonomy_changes_in_page_frontmatter() {
        let mut taxonomies = HashMap::new();
        taxonomies.insert("tags".to_string(), vec!["a tag".to_string()]);
        let new = PageFrontMatter { taxonomies, ..PageFrontMatter::default() };
        let changes = find_page_front_matter_changes(&PageFrontMatter::default(), &new);
        assert_eq!(changes, vec![PageChangesNeeded::Taxonomies, PageChangesNeeded::Render]);
    }

    #[test]
    fn can_find_multiple_changes_in_page_frontmatter() {
        let mut taxonomies = HashMap::new();
        taxonomies.insert("categories".to_string(), vec!["a category".to_string()]);
        let current = PageFrontMatter { taxonomies, order: Some(1), ..PageFrontMatter::default() };
        let changes = find_page_front_matter_changes(&current, &PageFrontMatter::default());
        assert_eq!(
            changes,
            vec![PageChangesNeeded::Taxonomies, PageChangesNeeded::Sort, PageChangesNeeded::Render]
        );
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
