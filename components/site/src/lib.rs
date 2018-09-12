extern crate tera;
extern crate rayon;
extern crate glob;
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
extern crate search;
extern crate imageproc;

#[cfg(test)]
extern crate tempfile;

use std::collections::HashMap;
use std::fs::{create_dir_all, remove_dir_all, copy};
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use glob::glob;
use tera::{Tera, Context};
use sass_rs::{Options as SassOptions, OutputStyle, compile_file};

use errors::{Result, ResultExt};
use config::{Config, get_config};
use utils::fs::{create_file, copy_directory, create_directory, ensure_directory_exists};
use utils::templates::{render_template, rewrite_theme_paths};
use utils::net::get_available_port;
use content::{Page, Section, populate_siblings, sort_pages};
use templates::{GUTENBERG_TERA, global_fns, render_redirect_template};
use front_matter::{SortBy, InsertAnchor};
use taxonomies::{Taxonomy, find_taxonomies};
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
    imageproc: Arc<Mutex<imageproc::Processor>>,
    // the live reload port to be used if there is one
    pub live_reload: Option<u16>,
    pub output_path: PathBuf,
    content_path: PathBuf,
    pub static_path: PathBuf,
    pub taxonomies: Vec<Taxonomy>,
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

        let content_path = path.join("content");
        let static_path = path.join("static");
        let imageproc = imageproc::Processor::new(content_path.clone(), &static_path, &config.base_url);

        let site = Site {
            base_path: path.to_path_buf(),
            config,
            tera,
            pages: HashMap::new(),
            sections: HashMap::new(),
            imageproc: Arc::new(Mutex::new(imageproc)),
            live_reload: None,
            output_path: path.join("public"),
            content_path,
            static_path,
            taxonomies: Vec::new(),
            permalinks: HashMap::new(),
        };

        Ok(site)
    }

    /// The index section is ALWAYS at that path
    pub fn index_section_path(&self) -> PathBuf {
        self.content_path.join("_index.md")
    }

    pub fn enable_live_reload(&mut self) {
        self.live_reload = get_available_port();
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

    pub fn set_base_url(&mut self, base_url: String) {
        let mut imageproc = self.imageproc.lock().unwrap();
        imageproc.set_base_url(&base_url);
        self.config.base_url = base_url;
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
        // a _index.md to render the index page at the root of the site
        let index_path = self.index_section_path();
        if let Some(ref index_section) = self.sections.get(&index_path) {
            if self.config.build_search_index && !index_section.meta.in_search_index {
                bail!(
                    "You have enabled search in the config but disabled it in the index section: \
                    either turn off the search in the config or remote `in_search_index = true` from the \
                    section front-matter."
                )
            }
        }
        // Not in else because of borrow checker
        if !self.sections.contains_key(&index_path) {
            let mut index_section = Section::default();
            index_section.permalink = self.config.make_permalink("");
            index_section.file.parent = self.content_path.clone();
            index_section.file.relative = "_index.md".to_string();
            self.sections.insert(index_path, index_section);
        }

        let mut pages_insert_anchors = HashMap::new();
        for page in pages {
            let p = page?;
            pages_insert_anchors.insert(p.file.path.clone(), self.find_parent_section_insert_anchor(&p.file.parent.clone()));
            self.add_page(p, false)?;
        }

        self.register_early_global_fns();
        self.render_markdown()?;
        self.populate_sections();
        self.populate_taxonomies()?;
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
        let base_path = &self.base_path;

        // TODO: avoid the duplication with function above for that part
        // This is needed in the first place because of silly borrow checker
        let mut pages_insert_anchors = HashMap::new();
        for (_, p) in &self.pages {
            pages_insert_anchors.insert(p.file.path.clone(), self.find_parent_section_insert_anchor(&p.file.parent.clone()));
        }

        self.pages.par_iter_mut()
            .map(|(_, page)| {
                let insert_anchor = pages_insert_anchors[&page.file.path];
                page.render_markdown(permalinks, tera, config, base_path, insert_anchor)
            })
            .fold(|| Ok(()), Result::and)
            .reduce(|| Ok(()), Result::and)?;

        self.sections.par_iter_mut()
            .map(|(_, section)| section.render_markdown(permalinks, tera, config, base_path))
            .fold(|| Ok(()), Result::and)
            .reduce(|| Ok(()), Result::and)?;

        Ok(())
    }

    /// Adds global fns that are to be available to shortcodes while rendering markdown
    pub fn register_early_global_fns(&mut self) {
        self.tera.register_global_function(
            "get_url", global_fns::make_get_url(self.permalinks.clone(), self.config.clone()),
        );
        self.tera.register_global_function(
            "resize_image", global_fns::make_resize_image(self.imageproc.clone()),
        );
    }

    pub fn register_tera_global_fns(&mut self) {
        self.tera.register_global_function("trans", global_fns::make_trans(self.config.clone()));
        self.tera.register_global_function("get_page", global_fns::make_get_page(&self.pages));
        self.tera.register_global_function("get_section", global_fns::make_get_section(&self.sections));
        self.tera.register_global_function(
            "get_taxonomy",
            global_fns::make_get_taxonomy(self.taxonomies.clone()),
        );
        self.tera.register_global_function(
            "get_taxonomy_url",
            global_fns::make_get_taxonomy_url(self.taxonomies.clone()),
        );
        self.tera.register_global_function("load_data", global_fns::make_load_data(self.content_path.clone()));
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
            page.render_markdown(&self.permalinks, &self.tera, &self.config, &self.base_path, insert_anchor)?;
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
            section.render_markdown(&self.permalinks, &self.tera, &self.config, &self.base_path)?;
        }

        Ok(prev)
    }

    /// Finds the insert_anchor for the parent section of the directory at `path`.
    /// Defaults to `AnchorInsert::None` if no parent section found
    pub fn find_parent_section_insert_anchor(&self, parent_path: &PathBuf) -> InsertAnchor {
        match self.sections.get(&parent_path.join("_index.md")) {
            Some(s) => s.meta.insert_anchor_links,
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
                    .sort_by(|a, b| a.meta.weight.cmp(&b.meta.weight));
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
            let (sorted_pages, cannot_be_sorted_pages) = sort_pages(pages, section.meta.sort_by);
            section.pages = populate_siblings(&sorted_pages, section.meta.sort_by);
            section.ignored_pages = cannot_be_sorted_pages;
        }
    }

    /// Find all the tags and categories if it's asked in the config
    pub fn populate_taxonomies(&mut self) -> Result<()> {
        if self.config.taxonomies.is_empty() {
            return Ok(());
        }

        self.taxonomies = find_taxonomies(
            &self.config,
            self.pages
                .values()
                .filter(|p| !p.is_draft())
                .cloned()
                .collect::<Vec<_>>()
                .as_slice(),
        )?;

        Ok(())
    }

    /// Inject live reload script tag if in live reload mode
    fn inject_livereload(&self, html: String) -> String {
        if let Some(port) = self.live_reload {
            return html.replace(
                "</body>",
                &format!(r#"<script src="/livereload.js?port={}&mindelay=10"></script></body>"#, port),
            );
        }

        html
    }

    /// Copy the main `static` folder and the theme `static` folder if a theme is used
    pub fn copy_static_directories(&self) -> Result<()> {
        // The user files will overwrite the theme files
        if let Some(ref theme) = self.config.theme {
            copy_directory(
                &self.base_path.join("themes").join(theme).join("static"),
                &self.output_path,
            )?;
        }
        // We're fine with missing static folders
        if self.static_path.exists() {
            copy_directory(&self.static_path, &self.output_path)?;
        }

        Ok(())
    }

    pub fn num_img_ops(&self) -> usize {
        let imageproc = self.imageproc.lock().unwrap();
        imageproc.num_img_ops()
    }

    pub fn process_images(&self) -> Result<()> {
        let mut imageproc = self.imageproc.lock().unwrap();
        imageproc.prune()?;
        imageproc.do_process()
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
            self.render_rss_feed(None, None)?;
        }
        self.render_404()?;
        self.render_robots()?;
        self.render_taxonomies()?;

        if let Some(ref theme) = self.config.theme {
            let theme_path = self.base_path.join("themes").join(theme);
            if theme_path.join("sass").exists() {
                self.compile_sass(&theme_path)?;
            }
        }

        if self.config.compile_sass {
            self.compile_sass(&self.base_path)?;
        }

        self.process_images()?;
        self.copy_static_directories()?;

        if self.config.build_search_index {
            self.build_search_index()?;
        }

        Ok(())
    }

    pub fn build_search_index(&self) -> Result<()> {
        // index first
        create_file(
            &self.output_path.join(&format!("search_index.{}.js", self.config.default_language)),
            &format!(
                "window.searchIndex = {};",
                search::build_index(&self.sections, &self.config.default_language)?
            ),
        )?;

        // then elasticlunr.min.js
        create_file(
            &self.output_path.join("elasticlunr.min.js"),
            search::ELASTICLUNR_JS,
        )?;

        Ok(())
    }

    pub fn compile_sass(&self, base_path: &Path) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let sass_path = {
            let mut sass_path = PathBuf::from(base_path);
            sass_path.push("sass");
            sass_path
        };

        let mut options = SassOptions::default();
        options.output_style = OutputStyle::Compressed;
        let mut compiled_paths = self.compile_sass_glob(&sass_path, "scss", options.clone())?;

        options.indented_syntax = true;
        compiled_paths.extend(self.compile_sass_glob(&sass_path, "sass", options)?);

        compiled_paths.sort();
        for window in compiled_paths.windows(2) {
            if window[0].1 == window[1].1 {
                bail!(
                    "SASS path conflict: \"{}\" and \"{}\" both compile to \"{}\"",
                    window[0].0.display(),
                    window[1].0.display(),
                    window[0].1.display(),
                );
            }
        }

        Ok(())
    }

    fn compile_sass_glob(&self, sass_path: &Path, extension: &str, options: SassOptions) -> Result<Vec<(PathBuf, PathBuf)>> {
        let glob_string = format!("{}/**/*.{}", sass_path.display(), extension);
        let files = glob(&glob_string)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|entry| !entry.as_path().file_name().unwrap().to_string_lossy().starts_with('_'))
            .collect::<Vec<_>>();

        let mut compiled_paths = Vec::new();
        for file in files {
            let css = compile_file(&file, options.clone())?;

            let path_inside_sass = file.strip_prefix(&sass_path).unwrap();
            let parent_inside_sass = path_inside_sass.parent();
            let css_output_path = self.output_path.join(path_inside_sass).with_extension("css");

            if parent_inside_sass.is_some() {
                create_dir_all(&css_output_path.parent().unwrap())?;
            }

            create_file(&css_output_path, &css)?;
            compiled_paths.push((path_inside_sass.to_owned(), css_output_path));
        }

        Ok(compiled_paths)
    }

    pub fn render_aliases(&self) -> Result<()> {
        for page in self.pages.values() {
            for alias in &page.meta.aliases {
                let mut output_path = self.output_path.to_path_buf();
                let mut split = alias.split('/').collect::<Vec<_>>();

                // If the alias ends with an html file name, use that instead of mapping
                // as a path containing an `index.html`
                let page_name = match split.pop() {
                    Some(part) if part.ends_with(".html") => part,
                    Some(part) => {
                        split.push(part);
                        "index.html"
                    }
                    None => "index.html"
                };

                for component in split {
                    output_path.push(&component);

                    if !output_path.exists() {
                        create_directory(&output_path)?;
                    }
                }
                create_file(&output_path.join(page_name), &render_redirect_template(&page.permalink, &self.tera)?)?;
            }
        }
        Ok(())
    }

    /// Renders 404.html
    pub fn render_404(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        let mut context = Context::new();
        context.insert("config", &self.config);
        create_file(
            &self.output_path.join("404.html"),
            &render_template("404.html", &self.tera, &context, &self.config.theme)?,
        )
    }

    /// Renders robots.txt
    pub fn render_robots(&self) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;
        create_file(
            &self.output_path.join("robots.txt"),
            &render_template("robots.txt", &self.tera, &Context::new(), &self.config.theme)?,
        )
    }

    /// Renders all taxonomies with at least one non-draft post
    pub fn render_taxonomies(&self) -> Result<()> {
        // TODO: make parallel?
        for taxonomy in &self.taxonomies {
            self.render_taxonomy(taxonomy)?;
        }

        Ok(())
    }

    fn render_taxonomy(&self, taxonomy: &Taxonomy) -> Result<()> {
        if taxonomy.items.is_empty() {
            return Ok(());
        }

        ensure_directory_exists(&self.output_path)?;
        let output_path = self.output_path.join(&taxonomy.kind.name);
        let list_output = taxonomy.render_all_terms(&self.tera, &self.config)?;
        create_directory(&output_path)?;
        create_file(&output_path.join("index.html"), &self.inject_livereload(list_output))?;

        taxonomy
            .items
            .par_iter()
            .map(|item| {
                if taxonomy.kind.rss {
                    // TODO: can we get rid of `clone()`?
                    self.render_rss_feed(
                        Some(item.pages.clone()),
                        Some(&PathBuf::from(format!("{}/{}", taxonomy.kind.name, item.slug))),
                    )?;
                }

                if taxonomy.kind.is_paginated() {
                    self.render_paginated(&output_path, &Paginator::from_taxonomy(&taxonomy, item))
                } else {
                    let single_output = taxonomy.render_term(item, &self.tera, &self.config)?;
                    let path = output_path.join(&item.slug);
                    create_directory(&path)?;
                    create_file(
                        &path.join("index.html"),
                        &self.inject_livereload(single_output),
                    )
                }
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

        let mut taxonomies = vec![];
        for taxonomy in &self.taxonomies {
            let name = &taxonomy.kind.name;
            let mut terms = vec![];
            terms.push(SitemapEntry::new(self.config.make_permalink(name), None));
            for item in &taxonomy.items {
                terms.push(SitemapEntry::new(self.config.make_permalink(&format!("{}/{}", &name, item.slug)), None));
            }
            terms.sort_by(|a, b| a.permalink.cmp(&b.permalink));
            taxonomies.push(terms);
        }
        context.add("taxonomies", &taxonomies);

        context.add("config", &self.config);

        let sitemap = &render_template("sitemap.xml", &self.tera, &context, &self.config.theme)?;

        create_file(&self.output_path.join("sitemap.xml"), sitemap)?;

        Ok(())
    }

    /// Renders a RSS feed for the given path and at the given path
    /// If both arguments are `None`, it will render only the RSS feed for the whole
    /// site at the root folder.
    pub fn render_rss_feed(&self, all_pages: Option<Vec<Page>>, base_path: Option<&PathBuf>) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let mut context = Context::new();
        let pages = all_pages
            // TODO: avoid that cloned().
            // It requires having `sort_pages` take references of Page
            .unwrap_or_else(|| self.pages.values().cloned().collect::<Vec<_>>())
            .into_iter()
            .filter(|p| p.meta.date.is_some() && !p.is_draft())
            .collect::<Vec<_>>();

        // Don't generate a RSS feed if none of the pages has a date
        if pages.is_empty() {
            return Ok(());
        }

        let (sorted_pages, _) = sort_pages(pages, SortBy::Date);
        context.add("last_build_date", &sorted_pages[0].meta.date.clone().map(|d| d.to_string()));
        // limit to the last n elements
        context.add("pages", &sorted_pages.iter().take(self.config.rss_limit).collect::<Vec<_>>());
        context.add("config", &self.config);

        let rss_feed_url = if let Some(ref base) = base_path {
            self.config.make_permalink(&base.join("rss.xml").to_string_lossy().replace('\\', "/"))
        } else {
            self.config.make_permalink("rss.xml")
        };

        context.add("feed_url", &rss_feed_url);

        let feed = &render_template("rss.xml", &self.tera, &context, &self.config.theme)?;

        if let Some(ref base) = base_path {
            let mut output_path = self.output_path.clone().to_path_buf();
            for component in base.components() {
                output_path.push(component);
                if !output_path.exists() {
                    create_directory(&output_path)?;
                }
            }
            create_file(&output_path.join("rss.xml"), feed)?;
        } else {
            create_file(&self.output_path.join("rss.xml"), feed)?;
        }

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

        // Copy any asset we found previously into the same directory as the index.html
        for asset in &section.assets {
            let asset_path = asset.as_path();
            copy(&asset_path, &output_path.join(asset_path.file_name().unwrap()))?;
        }

        if render_pages {
            section
                .pages
                .par_iter()
                .map(|p| self.render_page(p))
                .fold(|| Ok(()), Result::and)
                .reduce(|| Ok(()), Result::and)?;
        }

        if !section.meta.render {
            return Ok(());
        }

        if let Some(ref redirect_to) = section.meta.redirect_to {
            let permalink = self.config.make_permalink(redirect_to);
            create_file(&output_path.join("index.html"), &render_redirect_template(&permalink, &self.tera)?)?;
            return Ok(());
        }

        if section.meta.is_paginated() {
            self.render_paginated(&output_path, &Paginator::from_section(&section.pages, section))?;
        } else {
            let output = section.render_html(&self.tera, &self.config)?;
            create_file(&output_path.join("index.html"), &self.inject_livereload(output))?;
        }

        Ok(())
    }

    /// Used only on reload
    pub fn render_index(&self) -> Result<()> {
        self.render_section(
            &self.sections[&self.content_path.join("_index.md")],
            false,
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
    pub fn render_paginated(&self, output_path: &Path, paginator: &Paginator) -> Result<()> {
        ensure_directory_exists(&self.output_path)?;

        let folder_path = output_path.join(&paginator.paginate_path);
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
                    create_file(&page_path.join("index.html"), &render_redirect_template(&paginator.permalink, &self.tera)?)?;
                }
                Ok(())
            })
            .fold(|| Ok(()), Result::and)
            .reduce(|| Ok(()), Result::and)
    }
}
