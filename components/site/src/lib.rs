extern crate tera;
extern crate rayon;
extern crate glob;
extern crate walkdir;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate sass_rs;

#[macro_use]
extern crate errors;
extern crate config;
extern crate utils;
extern crate front_matter;
extern crate templates;
extern crate pagination;
extern crate taxonomies;
extern crate content;

#[cfg(test)]
extern crate tempdir;

use std::collections::HashMap;
use std::fs::{remove_dir_all, copy, create_dir_all};
use std::mem;
use std::path::{Path, PathBuf};

use glob::glob;
use tera::{Tera, Context};
use walkdir::WalkDir;
use sass_rs::{Options, OutputStyle, compile_file};

use errors::{Result, ResultExt};
use config::{Config, get_config};
use utils::fs::{create_file, create_directory, ensure_directory_exists};
use utils::templates::{render_template, rewrite_theme_paths};
use content::{Page, Section, populate_previous_and_next_pages, sort_pages};
use templates::{GUTENBERG_TERA, global_fns, render_redirect_template};
use front_matter::{SortBy, InsertAnchor};
use taxonomies::Taxonomy;
use pagination::Paginator;

use rayon::prelude::*;


/// The sitemap only needs links and potentially date so we trim down
/// all pages to only that
#[derive(Debug, Serialize)]
struct SitemapEntry {
    permalink: String,
    date: Option<String>,
}

impl SitemapEntry {
    pub fn new(permalink: String, date: Option<String>) -> SitemapEntry {
        SitemapEntry { permalink, date }
    }
}

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
    pub static_path: PathBuf,
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
        let mut config = get_config(path, config_file);

        let tpl_glob = format!("{}/{}", path.to_string_lossy().replace("\\", "/"), "templates/**/*.*ml");
        // Only parsing as we might be extending templates from themes and that would error
        // as we haven't loaded them yet
        let mut tera = Tera::parse(&tpl_glob).chain_err(|| "Error parsing templates")?;

        if let Some(theme) = config.theme.clone() {
            // Grab data from the extra section of the theme
            config.merge_with_theme(&path.join("themes").join(&theme).join("theme.toml"))?;

            // Test that the templates folder exist for that theme
            let theme_path = path.join("themes").join(&theme);
            if !theme_path.join("templates").exists() {
                bail!("Theme `{}` is missing a templates folder", theme);
            }

            let theme_tpl_glob = format!("{}/{}", path.to_string_lossy().replace("\\", "/"), "themes/**/*.html");
            let mut tera_theme = Tera::parse(&theme_tpl_glob).chain_err(|| "Error parsing templates from themes")?;
            rewrite_theme_paths(&mut tera_theme, &theme);
            tera_theme.build_inheritance_chains()?;
            tera.extend(&tera_theme)?;
        }
        tera.extend(&GUTENBERG_TERA)?;
        // the `extend` above already does it but hey
        tera.build_inheritance_chains()?;

        let site = Site {
            base_path: path.to_path_buf(),
            config: config,
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

    pub fn set_output_path<P: AsRef<Path>>(&mut self, path: P) {
        self.output_path = path.as_ref().to_path_buf();
    }

    /// Reads all .md files in the `content` directory and create pages/sections
    /// out of them
    pub fn load(&mut self) -> Result<()> {
        let base_path = self.base_path.to_string_lossy().replace("\\", "/");
        let content_glob = format!("{}/{}", base_path, "content/**/*.md");

        let (section_entries, page_entries): (Vec<_>, Vec<_>) = glob(&content_glob)
            .unwrap()
            .filter_map(|e| e.ok())
            .partition(|entry| entry.as_path().file_name().unwrap() == "_index.md");

        let sections = {
            let config = &self.config;

            section_entries
                .into_par_iter()
                .filter(|entry| entry.as_path().file_name().unwrap() == "_index.md")
                .map(|entry| {
                    let path = entry.as_path();
                    Section::from_file(path, config)
                })
                .collect::<Vec<_>>()
        };

        let pages = {
            let config = &self.config;

            page_entries
                .into_par_iter()
                .filter(|entry| entry.as_path().file_name().unwrap() != "_index.md")
                .map(|entry| {
                    let path = entry.as_path();
                    Page::from_file(path, config)
                })
                .collect::<Vec<_>>()
        };

        // Kinda duplicated code for add_section/add_page but necessary to do it that
        // way because of the borrow checker
        for section in sections {
            let s = section?;
            self.add_section(s, false)?;
        }

        // Insert a default index section if necessary so we don't need to create
        // a _index.md to render the index page
        let index_path = self.base_path.join("content").join("_index.md");
        if !self.sections.contains_key(&index_path) {
            let mut index_section = Section::default();
            index_section.permalink = self.config.make_permalink("");
            index_section.file.parent = self.base_path.join("content");
            index_section.file.relative = "_index.md".to_string();
            self.sections.insert(index_path, index_section);
        }

        let mut pages_insert_anchors = HashMap::new();
        for page in pages {
            let p = page?;
            pages_insert_anchors.insert(p.file.path.clone(), self.find_parent_section_insert_anchor(&p.file.parent.clone()));
            self.add_page(p, false)?;
        }

        self.render_markdown()?;
        self.populate_sections();
        self.populate_tags_and_categories();

        self.register_tera_global_fns();

        Ok(())
    }

    /// Render the markdown of all pages/sections
    /// Used in a build and in `serve` if a shortcode has changed
    pub fn render_markdown(&mut self) -> Result<()> {
        // Another silly thing needed to not borrow &self in parallel and
        // make the borrow checker happy
        let permalinks = &self.permalinks;
        let tera = &self.tera;
        let config = &self.config;

        // TODO: avoid the duplication with function above for that part
        // This is needed in the first place because of silly borrow checker
        let mut pages_insert_anchors = HashMap::new();
        for (_, p) in &self.pages {
            pages_insert_anchors.insert(p.file.path.clone(), self.find_parent_section_insert_anchor(&p.file.parent.clone()));
        }

        self.pages.par_iter_mut()
            .map(|(_, page)| {
                let insert_anchor = pages_insert_anchors[&page.file.path];
                page.render_markdown(permalinks, tera, config, insert_anchor)
            })
            .fold(|| Ok(()), Result::and)
            .reduce(|| Ok(()), Result::and)?;

        self.sections.par_iter_mut()
            .map(|(_, section)| section.render_markdown(permalinks, tera, config))
            .fold(|| Ok(()), Result::and)
            .reduce(|| Ok(()), Result::and)?;

        Ok(())
    }

    pub fn register_tera_global_fns(&mut self) {
        self.tera.register_global_function("trans", global_fns::make_trans(self.config.clone()));
        self.tera.register_global_function("get_page", global_fns::make_get_page(&self.pages));
        self.tera.register_global_function("get_section", global_fns::make_get_section(&self.sections));
        self.tera.register_global_function(
            "get_taxonomy_url",
            global_fns::make_get_taxonomy_url(self.tags.clone(), self.categories.clone())
        );
        self.tera.register_global_function(
            "get_url",
            global_fns::make_get_url(self.permalinks.clone(), self.config.clone())
        );
    }

    /// Add a page to the site
    /// The `render` parameter is used in the serve command, when rebuilding a page.
    /// If `true`, it will also render the markdown for that page
    /// Returns the previous page struct if there was one at the same path
    pub fn add_page(&mut self, page: Page, render: bool) -> Result<Option<Page>> {
        let path = page.file.path.clone();
        self.permalinks.insert(page.file.relative.clone(), page.permalink.clone());
        let prev = self.pages.insert(page.file.path.clone(), page);

        if render {
            let insert_anchor = self.find_parent_section_insert_anchor(&self.pages[&path].file.parent);
            let page = self.pages.get_mut(&path).unwrap();
            page.render_markdown(&self.permalinks, &self.tera, &self.config, insert_anchor)?;
        }

        Ok(prev)
    }

    /// Add a section to the site
    /// The `render` parameter is used in the serve command, when rebuilding a page.
    /// If `true`, it will also render the markdown for that page
    /// Returns the previous section struct if there was one at the same path
    pub fn add_section(&mut self, section: Section, render: bool) -> Result<Option<Section>> {
        let path = section.file.path.clone();
        self.permalinks.insert(section.file.relative.clone(), section.permalink.clone());
        let prev = self.sections.insert(section.file.path.clone(), section);

        if render {
            let section = self.sections.get_mut(&path).unwrap();
            section.render_markdown(&self.permalinks, &self.tera, &self.config)?;
        }

        Ok(prev)
    }

    /// Finds the insert_anchor for the parent section of the directory at `path`.
    /// Defaults to `AnchorInsert::None` if no parent section found
    pub fn find_parent_section_insert_anchor(&self, parent_path: &PathBuf) -> InsertAnchor {
        match self.sections.get(&parent_path.join("_index.md")) {
            Some(s) => s.meta.insert_anchor_links.unwrap(),
            None => InsertAnchor::None
        }
    }

    /// Find out the direct subsections of each subsection if there are some
    /// as well as the pages for each section
    pub fn populate_sections(&mut self) {
        let mut grandparent_paths: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        for section in self.sections.values_mut() {
            if let Some(ref grand_parent) = section.file.grand_parent {
                grandparent_paths
                    .entry(grand_parent.to_path_buf())
                    .or_insert_with(|| vec![])
                    .push(section.file.path.clone());
            }
            // Make sure the pages of a section are empty since we can call that many times on `serve`
            section.pages = vec![];
            section.ignored_pages = vec![];
        }

        for page in self.pages.values() {
            let parent_section_path = page.file.parent.join("_index.md");
            if self.sections.contains_key(&parent_section_path) {
                // TODO: use references instead of cloning to avoid having to call populate_section on
                // content change
                self.sections.get_mut(&parent_section_path).unwrap().pages.push(page.clone());
            }
        }

        self.sort_sections_pages(None);
        // TODO: remove this clone
        let sections = self.sections.clone();

        for section in self.sections.values_mut() {
            if let Some(paths) = grandparent_paths.get(&section.file.parent) {
                section.subsections = paths
                    .iter()
                    .map(|p| sections[p].clone())
                    .collect::<Vec<_>>();
                section.subsections
                    .sort_by(|a, b| a.meta.weight.unwrap().cmp(&b.meta.weight.unwrap()));
            }
        }
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
            let pages = mem::replace(&mut section.pages, vec![]);
            let (sorted_pages, cannot_be_sorted_pages) = sort_pages(pages, section.meta.sort_by());
            section.pages = populate_previous_and_next_pages(&sorted_pages);
            section.ignored_pages = cannot_be_sorted_pages;
        }
    }

    /// Find all the tags and categories if it's asked in the config
    pub fn populate_tags_and_categories(&mut self) {
        let generate_tags_pages = self.config.generate_tags_pages;
        let generate_categories_pages = self.config.generate_categories_pages;
        if !generate_tags_pages && !generate_categories_pages {
            return;
        }

        // TODO: can we pass a reference?
        let (tags, categories) = Taxonomy::find_tags_and_categories(
            &self.config,
            self.pages
                .values()
                .filter(|p| !p.is_draft())
                .cloned()
                .collect::<Vec<_>>()
                .as_slice()
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

    /// Copy the file at the given path into the public folder
    pub fn copy_static_file<P: AsRef<Path>>(&self, path: P, base_path: &PathBuf) -> Result<()> {
        let relative_path = path.as_ref().strip_prefix(base_path).unwrap();
        let target_path = self.output_path.join(relative_path);
        if let Some(parent_directory) = target_path.parent() {
            create_dir_all(parent_directory)?;
        }
        copy(path.as_ref(), &target_path)?;
        Ok(())
    }

    /// Copy the content of the given folder into the `public` folder
    fn copy_static_directory(&self, path: &PathBuf) -> Result<()> {
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let relative_path = entry.path().strip_prefix(path).unwrap();
            let target_path = self.output_path.join(relative_path);
            if entry.path().is_dir() {
                if !target_path.exists() {
                    create_directory(&target_path)?;
                }
            } else {
                let entry_fullpath = self.base_path.join(entry.path());
                self.copy_static_file(entry_fullpath, path)?;
            }
        }
        Ok(())
    }

    /// Copy the main `static` folder and the theme `static` folder if a theme is used
    pub fn copy_static_directories(&self) -> Result<()> {
        // The user files will overwrite the theme files
        if let Some(ref theme) = self.config.theme {
            self.copy_static_directory(
                &self.base_path.join("themes").join(theme).join("static")
            )?;
        }
        // We're fine with missing static folders
        if self.static_path.exists() {
            self.copy_static_directory(&self.static_path)?;
        }

        Ok(())
    }

    /// Deletes the `public` directory if it exists
    pub fn clean(&self) -> Result<()> {
        if self.output_path.exists() {
            // Delete current `public` directory so we can start fresh
            remove_dir_all(&self.output_path).chain_err(|| "Couldn't delete output directory")?;
        }

        Ok(())
    }

    /// Renders a single content page
    pub fn render_page(&self, page: &Page) -> Result<()> {
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
        let output = page.render_html(&self.tera, &self.config)?;
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
        if self.config.generate_rss {
            self.render_rss_feed()?;
        }
        self.render_robots()?;
        // `render_categories` and `render_tags` will check whether the config allows
        // them to render or not
        self.render_categories()?;
        self.render_tags()?;

        if let Some(ref theme) = self.config.theme {
            let theme_path = self.base_path.join("themes").join(theme);
            if theme_path.join("sass").exists() {
                self.compile_sass(&theme_path)?;
            }
        }

        if self.config.compile_sass {
            self.compile_sass(&self.base_path)?;
        }

        self.copy_static_directories()
    }

    pub fn compile_sass(&self, base_path: &Path) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let sass_path = {
            let mut sass_path = PathBuf::from(base_path);
            sass_path.push("sass");
            sass_path
        };

        let sass_glob = format!("{}/**/*.scss", sass_path.display());
        let files = glob(&sass_glob)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|entry| !entry.as_path().file_name().unwrap().to_string_lossy().starts_with('_'))
            .collect::<Vec<_>>();

        let mut sass_options = Options::default();
        sass_options.output_style = OutputStyle::Compressed;
        for file in files {
            let css = compile_file(&file, sass_options.clone())?;

            let path_inside_sass = file.strip_prefix(&sass_path).unwrap();
            let parent_inside_sass = path_inside_sass.parent();
            let css_output_path = self.output_path.join(path_inside_sass).with_extension("css");

            if parent_inside_sass.is_some() {
                create_dir_all(&css_output_path.parent().unwrap())?;
            }
            create_file(&css_output_path, &css)?;
        }

        Ok(())
    }

    pub fn render_aliases(&self) -> Result<()> {
        for page in self.pages.values() {
            if let Some(ref aliases) = page.meta.aliases {
                for alias in aliases {
                    let mut output_path = self.output_path.to_path_buf();
                    for component in alias.split('/') {
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
            &render_template("robots.txt", &self.tera, &Context::new(), self.config.theme.clone())?
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

        taxonomy
            .items
            .par_iter()
            .map(|item| {
                let single_output = taxonomy.render_single_item(item, &self.tera, &self.config)?;
                create_directory(&output_path.join(&item.slug))?;
                create_file(
                    &output_path.join(&item.slug).join("index.html"),
                    &self.inject_livereload(single_output)
                )
            })
            .fold(|| Ok(()), Result::and)
            .reduce(|| Ok(()), Result::and)
    }

    /// What it says on the tin
    pub fn render_sitemap(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let mut context = Context::new();

        let mut pages = self.pages
            .values()
            .filter(|p| !p.is_draft())
            .map(|p| {
                let date = match p.meta.date {
                    Some(ref d) => Some(d.to_string()),
                    None => None,
                };
                SitemapEntry::new(p.permalink.clone(), date)
            })
            .collect::<Vec<_>>();
        pages.sort_by(|a, b| a.permalink.cmp(&b.permalink));
        context.add("pages", &pages);

        let mut sections = self.sections
                .values()
                .map(|s| SitemapEntry::new(s.permalink.clone(), None))
                .collect::<Vec<_>>();
        sections.sort_by(|a, b| a.permalink.cmp(&b.permalink));
        context.add("sections", &sections);

        let mut categories = vec![];
        if let Some(ref c) = self.categories {
            let name = c.get_list_name();
            categories.push(SitemapEntry::new(self.config.make_permalink(&name), None));
            for item in &c.items {
                categories.push(
                    SitemapEntry::new(self.config.make_permalink(&format!("{}/{}", &name, item.slug)), None),
                );
            }
        }
        categories.sort_by(|a, b| a.permalink.cmp(&b.permalink));
        context.add("categories", &categories);

        let mut tags = vec![];
        if let Some(ref t) = self.tags {
            let name = t.get_list_name();
            tags.push(SitemapEntry::new(self.config.make_permalink(&name), None));
            for item in &t.items {
                tags.push(
                    SitemapEntry::new(self.config.make_permalink(&format!("{}/{}", &name, item.slug)), None),
                );
            }
        }
        tags.sort_by(|a, b| a.permalink.cmp(&b.permalink));
        context.add("tags", &tags);
        context.add("config", &self.config);

        let sitemap = &render_template("sitemap.xml", &self.tera, &context, self.config.theme.clone())?;

        create_file(&self.output_path.join("sitemap.xml"), sitemap)?;

        Ok(())
    }

    pub fn render_rss_feed(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let mut context = Context::new();
        let pages = self.pages.values()
            .filter(|p| p.meta.date.is_some() && !p.is_draft())
            .cloned()
            .collect::<Vec<Page>>();

        // Don't generate a RSS feed if none of the pages has a date
        if pages.is_empty() {
            return Ok(());
        }

        let (sorted_pages, _) = sort_pages(pages, SortBy::Date);
        context.add("last_build_date", &sorted_pages[0].meta.date.clone().map(|d| d.to_string()));
         // limit to the last n elements)
        context.add("pages", &sorted_pages.iter().take(self.config.rss_limit).collect::<Vec<_>>());
        context.add("config", &self.config);

        let rss_feed_url = if self.config.base_url.ends_with('/') {
            format!("{}{}", self.config.base_url, "rss.xml")
        } else {
            format!("{}/{}", self.config.base_url, "rss.xml")
        };
        context.add("feed_url", &rss_feed_url);

        let feed = &render_template("rss.xml", &self.tera, &context, self.config.theme.clone())?;

        create_file(&self.output_path.join("rss.xml"), feed)?;

        Ok(())
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
            section
                .pages
                .par_iter()
                .map(|p| self.render_page(p))
                .fold(|| Ok(()), Result::and)
                .reduce(|| Ok(()), Result::and)?;
        }

        if !section.meta.should_render() {
            return Ok(());
        }

        if let Some(ref redirect_to) = section.meta.redirect_to {
            let permalink = self.config.make_permalink(redirect_to);
            create_file(&output_path.join("index.html"), &render_redirect_template(&permalink, &self.tera)?)?;
            return Ok(());
        }

        if section.meta.is_paginated() {
            self.render_paginated(&output_path, section)?;
        } else {
            let output = section.render_html(&self.tera, &self.config)?;
            create_file(&output_path.join("index.html"), &self.inject_livereload(output))?;
        }

        Ok(())
    }

    /// Used only on reload
    pub fn render_index(&self) -> Result<()> {
        self.render_section(
            &self.sections[&self.base_path.join("content").join("_index.md")],
            false
        )
    }

    /// Renders all sections
    pub fn render_sections(&self) -> Result<()> {
        self.sections
            .values()
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|s| self.render_section(s, true))
            .fold(|| Ok(()), Result::and)
            .reduce(|| Ok(()), Result::and)
    }

    /// Renders all pages that do not belong to any sections
    pub fn render_orphan_pages(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        for page in self.get_all_orphan_pages() {
            self.render_page(page)?;
        }

        Ok(())
    }

    /// Renders a list of pages when the section/index is wanting pagination.
    pub fn render_paginated(&self, output_path: &Path, section: &Section) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let paginate_path = match section.meta.paginate_path {
            Some(ref s) => s.clone(),
            None => unreachable!()
        };

        let paginator = Paginator::new(&section.pages, section);
        let folder_path = output_path.join(&paginate_path);
        create_directory(&folder_path)?;

        paginator
            .pagers
            .par_iter()
            .enumerate()
            .map(|(i, pager)| {
                let page_path = folder_path.join(&format!("{}", i + 1));
                create_directory(&page_path)?;
                let output = paginator.render_pager(pager, &self.config, &self.tera)?;
                if i > 0 {
                    create_file(&page_path.join("index.html"), &self.inject_livereload(output))?;
                } else {
                    create_file(&output_path.join("index.html"), &self.inject_livereload(output))?;
                    create_file(&page_path.join("index.html"), &render_redirect_template(&section.permalink, &self.tera)?)?;
                }
                Ok(())
            })
            .fold(|| Ok(()), Result::and)
            .reduce(|| Ok(()), Result::and)
    }
}
