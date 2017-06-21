use std::collections::HashMap;
use std::fs::{remove_dir_all, copy, create_dir_all};
use std::path::{Path, PathBuf};

use glob::glob;
use tera::{Tera, Context};
use walkdir::WalkDir;

use errors::{Result, ResultExt};
use config::{Config, get_config};
use fs::{create_file, create_directory, ensure_directory_exists};
use content::{Page, Section, Paginator, SortBy, Taxonomy, populate_previous_and_next_pages, sort_pages};
use templates::{GUTENBERG_TERA, global_fns, render_redirect_template};
use front_matter::InsertAnchor;

use rayon::prelude::*;

#[derive(Debug)]
pub struct Site {
    /// The base path of the gutenberg site
    pub base_path: PathBuf,
    /// The parsed config for the site
    pub config: Config,
    pub pages: HashMap<PathBuf, Page>,
    pub sections: HashMap<PathBuf, Section>,
    pub tera: Tera,
    live_reload: bool,
    output_path: PathBuf,
    static_path: PathBuf,
    pub tags: Option<Taxonomy>,
    pub categories: Option<Taxonomy>,
    /// A map of all .md files (section and pages) and their permalink
    /// We need that if there are relative links in the content that need to be resolved
    pub permalinks: HashMap<String, String>,
}

impl Site {
    /// Parse a site at the given path. Defaults to the current dir
    /// Passing in a path is only used in tests
    pub fn new<P: AsRef<Path>>(path: P, config_file: &str) -> Result<Site> {
        let path = path.as_ref();

        let tpl_glob = format!("{}/{}", path.to_string_lossy().replace("\\", "/"), "templates/**/*.*ml");
        let mut tera = Tera::new(&tpl_glob).chain_err(|| "Error parsing templates")?;
        tera.extend(&GUTENBERG_TERA)?;

        let site = Site {
            base_path: path.to_path_buf(),
            config: get_config(path, config_file),
            pages: HashMap::new(),
            sections: HashMap::new(),
            tera: tera,
            live_reload: false,
            output_path: path.join("public"),
            static_path: path.join("static"),
            tags: None,
            categories: None,
            permalinks: HashMap::new(),
        };

        Ok(site)
    }

    /// What the function name says
    pub fn enable_live_reload(&mut self) {
        self.live_reload = true;
    }

    /// Get all the orphan (== without section) pages in the site
    pub fn get_all_orphan_pages(&self) -> Vec<&Page> {
        let mut pages_in_sections = vec![];
        let mut orphans = vec![];

        for s in self.sections.values() {
            pages_in_sections.extend(s.all_pages_path());
        }

        for page in self.pages.values() {
            if !pages_in_sections.contains(&page.file.path) {
                orphans.push(page);
            }
        }

        orphans
    }

    /// Used by tests to change the output path to a tmp dir
    #[doc(hidden)]
    pub fn set_output_path<P: AsRef<Path>>(&mut self, path: P) {
        self.output_path = path.as_ref().to_path_buf();
    }

    /// Reads all .md files in the `content` directory and create pages/sections
    /// out of them
    pub fn load(&mut self) -> Result<()> {
        let base_path = self.base_path.to_string_lossy().replace("\\", "/");
        let content_glob = format!("{}/{}", base_path, "content/**/*.md");

        for entry in glob(&content_glob).unwrap().filter_map(|e| e.ok()) {
            let path = entry.as_path();
            if path.file_name().unwrap() == "_index.md" {
                self.add_section(path, false)?;
            } else {
                self.add_page(path, false)?;
            }
        }
        // Insert a default index section if necessary so we don't need to create
        // a _index.md to render the index page
        let index_path = self.base_path.join("content").join("_index.md");
        if !self.sections.contains_key(&index_path) {
            let mut index_section = Section::default();
            index_section.permalink = self.config.make_permalink("");
            self.sections.insert(index_path, index_section);
        }

        // Silly thing needed to make the borrow checker happy
        let mut pages_insert_anchors = HashMap::new();
        for page in self.pages.values() {
            pages_insert_anchors.insert(page.file.path.clone(), self.find_parent_section_insert_anchor(&page.file.parent.clone()));
        }

        {
            // Another silly thing needed to not borrow &self in parallel and
            // make the borrow checker happy
            let permalinks = &self.permalinks;
            let tera = &self.tera;
            let config = &self.config;

            self.pages.par_iter_mut()
                .map(|(_, page)| page)
                .map(|page| {
                    let insert_anchor = pages_insert_anchors[&page.file.path];
                    page.render_markdown(&permalinks, &tera, &config, insert_anchor)
                })
                .fold(|| Ok(()), Result::and)
                .reduce(|| Ok(()), Result::and)?;

            self.sections.par_iter_mut()
                .map(|(_, section)| section)
                .map(|section| section.render_markdown(permalinks, tera, config))
                .fold(|| Ok(()), Result::and)
                .reduce(|| Ok(()), Result::and)?;
        }

        self.populate_sections();
        self.populate_tags_and_categories();

        self.tera.register_global_function("get_page", global_fns::make_get_page(&self.pages));
        self.tera.register_global_function("get_section", global_fns::make_get_section(&self.sections));
        self.register_get_url_fn();

        Ok(())
    }

    /// Separate fn as it can be called in the serve command
    pub fn register_get_url_fn(&mut self) {
        self.tera.register_global_function("get_url", global_fns::make_get_url(self.permalinks.clone()));
    }

    /// Add a page to the site
    /// The `render` parameter is used in the serve command, when rebuilding a page.
    /// If `true`, it will also render the markdown for that page
    /// Returns the previous page struct if there was one
    pub fn add_page(&mut self, path: &Path, render: bool) -> Result<Option<Page>> {
        let page = Page::from_file(&path, &self.config)?;
        self.permalinks.insert(page.file.relative.clone(), page.permalink.clone());
        let prev = self.pages.insert(page.file.path.clone(), page);

        if render {
            let insert_anchor = self.find_parent_section_insert_anchor(&self.pages[path].file.parent);
            let mut page = self.pages.get_mut(path).unwrap();
            page.render_markdown(&self.permalinks, &self.tera, &self.config, insert_anchor)?;
        }

        Ok(prev)
    }

    /// Add a section to the site
    /// The `render` parameter is used in the serve command, when rebuilding a page.
    /// If `true`, it will also render the markdown for that page
    /// Returns the previous section struct if there was one
    pub fn add_section(&mut self, path: &Path, render: bool) -> Result<Option<Section>> {
        let section = Section::from_file(path, &self.config)?;
        self.permalinks.insert(section.file.relative.clone(), section.permalink.clone());
        let prev = self.sections.insert(section.file.path.clone(), section);

        if render {
            let mut section = self.sections.get_mut(path).unwrap();
            section.render_markdown(&self.permalinks, &self.tera, &self.config)?;
        }

        Ok(prev)
    }

    /// Finds the insert_anchor for the parent section of the directory at `path`.
    /// Defaults to `AnchorInsert::None` if no parent section found
    pub fn find_parent_section_insert_anchor(&self, parent_path: &PathBuf) -> InsertAnchor {
        match self.sections.get(&parent_path.join("_index.md")) {
            Some(s) => s.meta.insert_anchor.unwrap(),
            None => InsertAnchor::None
        }
    }

    /// Find out the direct subsections of each subsection if there are some
    /// as well as the pages for each section
    pub fn populate_sections(&mut self) {
        let mut grandparent_paths = HashMap::new();
        for section in self.sections.values_mut() {
            if let Some(ref grand_parent) = section.file.grand_parent {
                grandparent_paths.entry(grand_parent.to_path_buf()).or_insert_with(|| vec![]).push(section.clone());
            }
            // Make sure the pages of a section are empty since we can call that many times on `serve`
            section.pages = vec![];
            section.ignored_pages = vec![];
        }

        for page in self.pages.values() {
            let parent_section_path = page.file.parent.join("_index.md");
            if self.sections.contains_key(&parent_section_path) {
                self.sections.get_mut(&parent_section_path).unwrap().pages.push(page.clone());
            }
        }

        for section in self.sections.values_mut() {
            match grandparent_paths.get(&section.file.parent) {
                Some(paths) => section.subsections.extend(paths.clone()),
                None => continue,
            };
        }

        self.sort_sections_pages(None);
    }

    /// Sorts the pages of the section at the given path
    /// By default will sort all sections but can be made to only sort a single one by providing a path
    pub fn sort_sections_pages(&mut self, only: Option<&Path>) {
        for (path, section) in &mut self.sections {
            if let Some(p) = only {
                if p != path {
                    continue;
                }
            }
            let (sorted_pages, cannot_be_sorted_pages) = sort_pages(section.pages.clone(), section.meta.sort_by());
            section.pages = populate_previous_and_next_pages(&sorted_pages);
            section.ignored_pages = cannot_be_sorted_pages;
        }
    }

    /// Find all the tags and categories if it's asked in the config
    pub fn populate_tags_and_categories(&mut self) {
        let generate_tags_pages = self.config.generate_tags_pages.unwrap();
        let generate_categories_pages = self.config.generate_categories_pages.unwrap();
        if !generate_tags_pages && !generate_categories_pages {
            return;
        }

        // TODO: can we pass a reference?
        let (tags, categories) = Taxonomy::find_tags_and_categories(
            self.pages.values().cloned().collect::<Vec<_>>()
        );
        if generate_tags_pages {
            self.tags = Some(tags);
        }
        if generate_categories_pages {
            self.categories = Some(categories);
        }
    }

    /// Inject live reload script tag if in live reload mode
    fn inject_livereload(&self, html: String) -> String {
        if self.live_reload {
            return html.replace(
                "</body>",
                r#"<script src="/livereload.js?port=1112&mindelay=10"></script></body>"#
            );
        }

        html
    }

    /// Copy static file to public directory.
    pub fn copy_static_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let relative_path = path.as_ref().strip_prefix(&self.static_path).unwrap();
        let target_path = self.output_path.join(relative_path);
        if let Some(parent_directory) = target_path.parent() {
            create_dir_all(parent_directory)?;
        }
        copy(path.as_ref(), &target_path)?;
        Ok(())
    }

    /// Copy the content of the `static` folder into the `public` folder
    pub fn copy_static_directory(&self) -> Result<()> {
        for entry in WalkDir::new(&self.static_path).into_iter().filter_map(|e| e.ok()) {
            let relative_path = entry.path().strip_prefix(&self.static_path).unwrap();
            let target_path = self.output_path.join(relative_path);

            if entry.path().is_dir() {
                if !target_path.exists() {
                    create_directory(&target_path)?;
                }
            } else {
                let entry_fullpath = self.base_path.join(entry.path());
                self.copy_static_file(entry_fullpath)?;
            }
        }
        Ok(())
    }

    /// Deletes the `public` directory if it exists
    pub fn clean(&self) -> Result<()> {
        if self.output_path.exists() {
            // Delete current `public` directory so we can start fresh
            remove_dir_all(&self.output_path).chain_err(|| "Couldn't delete `public` directory")?;
        }

        Ok(())
    }

    /// Renders a single content page
    pub fn render_page(&self, page: &Page, section: Option<&Section>) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        // Copy the nesting of the content directory if we have sections for that page
        let mut current_path = self.output_path.to_path_buf();

        for component in page.path.split('/') {
            current_path.push(component);

            if !current_path.exists() {
                create_directory(&current_path)?;
            }
        }

        // Make sure the folder exists
        create_directory(&current_path)?;

        // Finally, create a index.html file there with the page rendered
        let output = page.render_html(&self.tera, &self.config, section)?;
        create_file(&current_path.join("index.html"), &self.inject_livereload(output))?;

        // Copy any asset we found previously into the same directory as the index.html
        for asset in &page.assets {
            let asset_path = asset.as_path();
            copy(&asset_path, &current_path.join(asset_path.file_name().unwrap()))?;
        }

        Ok(())
    }

    /// Deletes the `public` directory and builds the site
    pub fn build(&self) -> Result<()> {
        self.clean()?;
        // Render aliases first to allow overwriting
        self.render_aliases()?;
        self.render_sections()?;
        self.render_orphan_pages()?;
        self.render_sitemap()?;
        if self.config.generate_rss.unwrap() {
            self.render_rss_feed()?;
        }
        self.render_robots()?;
        // `render_categories` and `render_tags` will check whether the config allows
        // them to render or not
        self.render_categories()?;
        self.render_tags()?;

        self.copy_static_directory()
    }

    pub fn render_aliases(&self) -> Result<()> {
        for page in self.pages.values() {
            if let Some(ref aliases) = page.meta.aliases {
                for alias in aliases {
                    let mut output_path = self.output_path.to_path_buf();
                    for component in alias.split("/") {
                        output_path.push(&component);

                        if !output_path.exists() {
                            create_directory(&output_path)?;
                        }
                    }
                    create_file(&output_path.join("index.html"), &render_redirect_template(&page.permalink, &self.tera)?)?;
                }
            }
        }
        Ok(())
    }

    /// Renders robots.txt
    pub fn render_robots(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        create_file(
            &self.output_path.join("robots.txt"),
            &self.tera.render("robots.txt", &Context::new())?
        )
    }

    /// Renders all categories and the single category pages if there are some
    pub fn render_categories(&self) -> Result<()> {
        if let Some(ref categories) = self.categories {
            self.render_taxonomy(categories)?;
        }

        Ok(())
    }

    /// Renders all tags and the single tag pages if there are some
    pub fn render_tags(&self) -> Result<()> {
        if let Some(ref tags) = self.tags {
            self.render_taxonomy(tags)?;
        }

        Ok(())
    }

    fn render_taxonomy(&self, taxonomy: &Taxonomy) -> Result<()> {
        if taxonomy.items.is_empty() {
            return Ok(())
        }

        ensure_directory_exists(&self.output_path)?;

        let output_path = self.output_path.join(&taxonomy.get_list_name());
        let list_output = taxonomy.render_list(&self.tera, &self.config)?;
        create_directory(&output_path)?;
        create_file(&output_path.join("index.html"), &self.inject_livereload(list_output))?;

        for item in &taxonomy.items {
            let single_output = taxonomy.render_single_item(item, &self.tera, &self.config)?;

            create_directory(&output_path.join(&item.slug))?;
            create_file(
                &output_path.join(&item.slug).join("index.html"),
                &self.inject_livereload(single_output)
            )?;
        }

        Ok(())
    }

    /// What it says on the tin
    pub fn render_sitemap(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let mut context = Context::new();
        context.add("pages", &self.pages.values().collect::<Vec<&Page>>());
        context.add("sections", &self.sections.values().collect::<Vec<&Section>>());

        let mut categories = vec![];
        if let Some(ref c) = self.categories {
            let name = c.get_list_name();
            categories.push(self.config.make_permalink(&name));
            for item in &c.items {
                categories.push(
                    self.config.make_permalink(&format!("{}/{}", &name, item.slug))
                );
            }
        }
        context.add("categories", &categories);

        let mut tags = vec![];
        if let Some(ref t) = self.tags {
            let name = t.get_list_name();
            tags.push(self.config.make_permalink(&name));
            for item in &t.items {
                tags.push(
                    self.config.make_permalink(&format!("{}/{}", &name, item.slug))
                );
            }
        }
        context.add("tags", &tags);

        let sitemap = self.tera.render("sitemap.xml", &context)?;

        create_file(&self.output_path.join("sitemap.xml"), &sitemap)?;

        Ok(())
    }

    pub fn render_rss_feed(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let mut context = Context::new();
        let pages = self.pages.values()
            .filter(|p| p.meta.date.is_some())
            .take(self.config.rss_limit.unwrap()) // limit to the last n elements
            .cloned()
            .collect::<Vec<Page>>();

        // Don't generate a RSS feed if none of the pages has a date
        if pages.is_empty() {
            return Ok(());
        }
        context.add("last_build_date", &pages[0].meta.date);
        let (sorted_pages, _) = sort_pages(pages, SortBy::Date);
        context.add("pages", &sorted_pages);
        context.add("config", &self.config);

        let rss_feed_url = if self.config.base_url.ends_with('/') {
            format!("{}{}", self.config.base_url, "rss.xml")
        } else {
            format!("{}/{}", self.config.base_url, "rss.xml")
        };
        context.add("feed_url", &rss_feed_url);

        let sitemap = self.tera.render("rss.xml", &context)?;

        create_file(&self.output_path.join("rss.xml"), &sitemap)?;

        Ok(())
    }

    /// Create a hashmap of paths to section
    /// For example `content/posts/_index.md` key will be `posts`
    /// The index section will always be called `index` so don't use a path such as
    /// `content/index/_index.md` yourself
    fn get_sections_map(&self) -> HashMap<String, Section> {
        self.sections
            .values()
            .map(|s| (if s.is_index() { "index".to_string() } else { s.file.components.join("/") }, s.clone()))
            .collect()
    }

    /// Renders a single section
    pub fn render_section(&self, section: &Section, render_pages: bool) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let public = self.output_path.clone();

        let mut output_path = public.to_path_buf();
        for component in &section.file.components {
            output_path.push(component);

            if !output_path.exists() {
                create_directory(&output_path)?;
            }
        }

        if render_pages {
            for page in &section.pages {
                self.render_page(page, Some(section))?;
            }
        }

        if !section.meta.should_render() {
            return Ok(());
        }

        if section.meta.is_paginated() {
            self.render_paginated(&output_path, section)?;
        } else {
            let output = section.render_html(
                if section.is_index() { self.get_sections_map() } else { HashMap::new() },
                &self.tera,
                &self.config,
            )?;
            create_file(&output_path.join("index.html"), &self.inject_livereload(output))?;
        }

        Ok(())
    }

    pub fn render_index(&self) -> Result<()> {
        self.render_section(&self.sections[&self.base_path.join("content").join("_index.md")], false)
    }

    /// Renders all sections
    pub fn render_sections(&self) -> Result<()> {
        for section in self.sections.values() {
            self.render_section(section, true)?;
        }
        Ok(())
    }

    /// Renders all pages that do not belong to any sections
    pub fn render_orphan_pages(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        for page in self.get_all_orphan_pages() {
            self.render_page(page, None)?;
        }

        Ok(())
    }

    /// Renders a list of pages when the section/index is wanting pagination.
    fn render_paginated(&self, output_path: &Path, section: &Section) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let paginate_path = match section.meta.paginate_path {
            Some(ref s) => s.clone(),
            None => unreachable!()
        };

        let paginator = Paginator::new(&section.pages, section);
        for (i, pager) in paginator.pagers.iter().enumerate() {
            let folder_path = output_path.join(&paginate_path);
            let page_path = folder_path.join(&format!("{}", i + 1));
            create_directory(&folder_path)?;
            create_directory(&page_path)?;
            let output = paginator.render_pager(pager, self)?;
            if i > 0 {
                create_file(&page_path.join("index.html"), &self.inject_livereload(output))?;
            } else {
                create_file(&output_path.join("index.html"), &self.inject_livereload(output))?;
                create_file(&page_path.join("index.html"), &render_redirect_template(&section.permalink, &self.tera)?)?;
            }
        }

        Ok(())
    }
}


/// Resolves an internal link (of the `./posts/something.md#hey` sort) to its absolute link
pub fn resolve_internal_link(link: &str, permalinks: &HashMap<String, String>) -> Result<String> {
    // First we remove the ./ since that's gutenberg specific
    let clean_link = link.replacen("./", "", 1);
    // Then we remove any potential anchor
    // parts[0] will be the file path and parts[1] the anchor if present
    let parts = clean_link.split('#').collect::<Vec<_>>();
    match permalinks.get(parts[0]) {
        Some(p) => {
            if parts.len() > 1 {
                Ok(format!("{}#{}", p, parts[1]))
            } else {
                Ok(p.to_string())
            }
        },
        None => bail!(format!("Relative link {} not found.", link)),
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::resolve_internal_link;

    #[test]
    fn can_resolve_valid_internal_link() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("./pages/about.md", &permalinks).unwrap();
        assert_eq!(res, "https://vincent.is/about");
    }

    #[test]
    fn can_resolve_internal_links_with_anchors() {
        let mut permalinks = HashMap::new();
        permalinks.insert("pages/about.md".to_string(), "https://vincent.is/about".to_string());
        let res = resolve_internal_link("./pages/about.md#hello", &permalinks).unwrap();
        assert_eq!(res, "https://vincent.is/about#hello");
    }

    #[test]
    fn errors_resolve_inexistant_internal_link() {
        let res = resolve_internal_link("./pages/about.md#hello", &HashMap::new());
        assert!(res.is_err());
    }
}
