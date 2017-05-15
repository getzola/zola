use std::collections::{HashMap};
use std::iter::FromIterator;
use std::fs::{remove_dir_all, copy, create_dir_all};
use std::path::{Path, PathBuf};

use glob::glob;
use tera::{Tera, Context};
use slug::slugify;
use walkdir::WalkDir;

use errors::{Result, ResultExt};
use config::{Config, get_config};
use utils::{create_file, create_directory};
use content::{Page, Section, Paginator, SortBy, populate_previous_and_next_pages, sort_pages};
use templates::{GUTENBERG_TERA, global_fns, render_redirect_template};


#[derive(Debug, PartialEq)]
enum RenderList {
    Tags,
    Categories,
}

/// A tag or category
#[derive(Debug, Serialize, PartialEq)]
struct ListItem {
    name: String,
    slug: String,
    count: usize,
}

impl ListItem {
    pub fn new(name: &str, count: usize) -> ListItem {
        ListItem {
            name: name.to_string(),
            slug: slugify(name),
            count: count,
        }
    }
}

#[derive(Debug)]
pub struct Site {
    pub base_path: PathBuf,
    pub config: Config,
    pub pages: HashMap<PathBuf, Page>,
    pub sections: HashMap<PathBuf, Section>,
    pub tera: Tera,
    live_reload: bool,
    output_path: PathBuf,
    static_path: PathBuf,
    pub tags: HashMap<String, Vec<PathBuf>>,
    pub categories: HashMap<String, Vec<PathBuf>>,
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
            tags: HashMap::new(),
            categories: HashMap::new(),
            permalinks: HashMap::new(),
        };

        Ok(site)
    }

    /// What the function name says
    pub fn enable_live_reload(&mut self) {
        self.live_reload = true;
    }

    /// Gets the path of all ignored pages in the site
    /// Used for reporting them in the CLI
    pub fn get_ignored_pages(&self) -> Vec<PathBuf> {
        self.sections
            .values()
            .flat_map(|s| s.ignored_pages.iter().map(|p| p.file_path.clone()))
            .collect()
    }

    /// Get all the orphan (== without section) pages in the site
    pub fn get_all_orphan_pages(&self) -> Vec<&Page> {
        let mut pages_in_sections = vec![];
        let mut orphans = vec![];

        for s in self.sections.values() {
            pages_in_sections.extend(s.all_pages_path());
        }

        for page in self.pages.values() {
            if !pages_in_sections.contains(&page.file_path) {
                orphans.push(page);
            }
        }

        orphans
    }

    /// Finds the section that contains the page given if there is one
    pub fn find_parent_section(&self, page: &Page) -> Option<&Section> {
        for section in self.sections.values() {
            if section.is_child_page(page) {
                return Some(section)
            }
        }

        None
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
        // Insert a default index section so we don't need to create a _index.md to render
        // the index page
        let index_path = self.base_path.join("content").join("_index.md");
        if !self.sections.contains_key(&index_path) {
            let mut index_section = Section::default();
            index_section.permalink = self.config.make_permalink("");
            self.sections.insert(index_path, index_section);
        }

        // TODO: make that parallel
        for page in self.pages.values_mut() {
            page.render_markdown(&self.permalinks, &self.tera, &self.config)?;
        }
        // TODO: make that parallel
        for section in self.sections.values_mut() {
            section.render_markdown(&self.permalinks, &self.tera, &self.config)?;
        }

        self.populate_sections();
        self.populate_tags_and_categories();

        self.tera.register_global_function("get_page", global_fns::make_get_page(&self.pages));

        Ok(())
    }

    /// Add a page to the site
    /// The `render` parameter is used in the serve command, when rebuilding a page.
    /// If `true`, it will also render the markdown for that page
    /// Returns the previous page struct if there was one
    pub fn add_page(&mut self, path: &Path, render: bool) -> Result<Option<Page>> {
        let page = Page::from_file(&path, &self.config)?;
        self.permalinks.insert(page.relative_path.clone(), page.permalink.clone());
        let prev = self.pages.insert(page.file_path.clone(), page);

        if render {
            let mut page = self.pages.get_mut(path).unwrap();
            page.render_markdown(&self.permalinks, &self.tera, &self.config)?;
        }

        Ok(prev)
    }

    /// Add a section to the site
    /// The `render` parameter is used in the serve command, when rebuilding a page.
    /// If `true`, it will also render the markdown for that page
    /// Returns the previous page struct if there was one
    pub fn add_section(&mut self, path: &Path, render: bool) -> Result<Option<Section>> {
        let section = Section::from_file(path, &self.config)?;
        self.permalinks.insert(section.relative_path.clone(), section.permalink.clone());
        let prev = self.sections.insert(section.file_path.clone(), section);

        if render {
            let mut section = self.sections.get_mut(path).unwrap();
            section.render_markdown(&self.permalinks, &self.tera, &self.config)?;
        }

        Ok(prev)
    }

    /// Find out the direct subsections of each subsection if there are some
    /// as well as the pages for each section
    pub fn populate_sections(&mut self) {
        let mut grandparent_paths = HashMap::new();
        for section in self.sections.values_mut() {
            if let Some(grand_parent) = section.parent_path.parent() {
                grandparent_paths.entry(grand_parent.to_path_buf()).or_insert_with(|| vec![]).push(section.clone());
            }
            // Make sure the pages of a section are empty since we can call that many times on `serve`
            section.pages = vec![];
            section.ignored_pages = vec![];
        }

        for page in self.pages.values() {
            if self.sections.contains_key(&page.parent_path.join("_index.md")) {
                self.sections.get_mut(&page.parent_path.join("_index.md")).unwrap().pages.push(page.clone());
            }
        }

        for section in self.sections.values_mut() {
            match grandparent_paths.get(&section.parent_path) {
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

    /// Separated from `parse` for easier testing
    pub fn populate_tags_and_categories(&mut self) {
        for page in self.pages.values() {
            if let Some(ref category) = page.meta.category {
                self.categories
                    .entry(category.to_string())
                    .or_insert_with(|| vec![])
                    .push(page.file_path.clone());
            }

            if let Some(ref tags) = page.meta.tags {
                for tag in tags {
                    self.tags
                        .entry(tag.to_string())
                        .or_insert_with(|| vec![])
                        .push(page.file_path.clone());
                }
            }
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

    fn ensure_public_directory_exists(&self) -> Result<()> {
        let public = self.output_path.clone();
        if !public.exists() {
            create_directory(&public)?;
        }
        Ok(())
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
    pub fn render_page(&self, page: &Page) -> Result<()> {
        self.ensure_public_directory_exists()?;

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
        create_file(current_path.join("index.html"), &self.inject_livereload(output))?;

        // Copy any asset we found previously into the same directory as the index.html
        for asset in &page.assets {
            let asset_path = asset.as_path();
            copy(&asset_path, &current_path.join(asset_path.file_name().unwrap()))?;
        }

        Ok(())
    }

    /// Builds the site to the `public` directory after deleting it
    pub fn build(&self) -> Result<()> {
        self.clean()?;
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

    /// Renders robots.txt
    pub fn render_robots(&self) -> Result<()> {
        self.ensure_public_directory_exists()?;
        create_file(
            self.output_path.join("robots.txt"),
            &self.tera.render("robots.txt", &Context::new())?
        )
    }

    /// Renders all categories if the config allows it
    pub fn render_categories(&self) -> Result<()> {
        if self.config.generate_categories_pages.unwrap() {
            self.render_categories_and_tags(RenderList::Categories)
        } else {
            Ok(())
        }
    }

    /// Renders all tags if the config allows it
    pub fn render_tags(&self) -> Result<()> {
        if self.config.generate_tags_pages.unwrap() {
            self.render_categories_and_tags(RenderList::Tags)
        } else {
            Ok(())
        }
    }

    /// Render the /{categories, list} pages and each individual category/tag page
    /// They are the same thing fundamentally, a list of pages with something in common
    /// TODO: revisit this function, lots of things have changed since then
    fn render_categories_and_tags(&self, kind: RenderList) -> Result<()> {
        let items = match kind {
            RenderList::Categories => &self.categories,
            RenderList::Tags => &self.tags,
        };

        if items.is_empty() {
            return Ok(());
        }

        let (list_tpl_name, single_tpl_name, name, var_name) = if kind == RenderList::Categories {
            ("categories.html", "category.html", "categories", "category")
        } else {
            ("tags.html", "tag.html", "tags", "tag")
        };
        self.ensure_public_directory_exists()?;

        // Create the categories/tags directory first
        let public = self.output_path.clone();
        let mut output_path = public.to_path_buf();
        output_path.push(name);
        create_directory(&output_path)?;

        // Then render the index page for that kind.
        // We sort by number of page in that category/tag
        let mut sorted_items = vec![];
        for (item, count) in Vec::from_iter(items).into_iter().map(|(a, b)| (a, b.len())) {
            sorted_items.push(ListItem::new(item, count));
        }
        sorted_items.sort_by(|a, b| b.count.cmp(&a.count));
        let mut context = Context::new();
        context.add(name, &sorted_items);
        context.add("config", &self.config);
        context.add("current_url", &self.config.make_permalink(name));
        context.add("current_path", &format!("/{}", name));
        // And render it immediately
        let list_output = self.tera.render(list_tpl_name, &context)?;
        create_file(output_path.join("index.html"), &self.inject_livereload(list_output))?;

        // Now, each individual item
        for (item_name, pages_paths) in items.iter() {
            let pages: Vec<&Page> = self.pages
                .iter()
                .filter(|&(path, _)| pages_paths.contains(path))
                .map(|(_, page)| page)
                .collect();
            // TODO: how to sort categories and tag content?
            // Have a setting in config.toml or a _category.md and _tag.md
            // The latter is more in line with the rest of Gutenberg but order ordering
            // doesn't really work across sections.

            let mut context = Context::new();
            let slug = slugify(&item_name);
            context.add(var_name, &item_name);
            context.add(&format!("{}_slug", var_name), &slug);
            context.add("pages", &pages);
            context.add("config", &self.config);
            context.add("current_url", &self.config.make_permalink(&format!("{}/{}", name, slug)));
            context.add("current_path", &format!("/{}/{}", name, slug));
            let single_output = self.tera.render(single_tpl_name, &context)?;

            create_directory(&output_path.join(&slug))?;
            create_file(
                output_path.join(&slug).join("index.html"),
                &self.inject_livereload(single_output)
            )?;
        }

        Ok(())
    }

    /// What it says on the tin
    pub fn render_sitemap(&self) -> Result<()> {
        self.ensure_public_directory_exists()?;
        let mut context = Context::new();
        context.add("pages", &self.pages.values().collect::<Vec<&Page>>());
        context.add("sections", &self.sections.values().collect::<Vec<&Section>>());

        let mut categories = vec![];
        if self.config.generate_categories_pages.unwrap() && !self.categories.is_empty() {
            categories.push(self.config.make_permalink("categories"));
            for category in self.categories.keys() {
                categories.push(
                    self.config.make_permalink(&format!("categories/{}", slugify(category)))
                );
            }
        }
        context.add("categories", &categories);

        let mut tags = vec![];
        if self.config.generate_tags_pages.unwrap() && !self.tags.is_empty() {
            tags.push(self.config.make_permalink("tags"));
            for tag in self.tags.keys() {
                tags.push(
                    self.config.make_permalink(&format!("tags/{}", slugify(tag)))
                );
            }
        }
        context.add("tags", &tags);

        let sitemap = self.tera.render("sitemap.xml", &context)?;

        create_file(self.output_path.join("sitemap.xml"), &sitemap)?;

        Ok(())
    }

    pub fn render_rss_feed(&self) -> Result<()> {
        self.ensure_public_directory_exists()?;

        let mut context = Context::new();
        let pages = self.pages.values()
            .filter(|p| p.meta.date.is_some())
            .take(15) // limit to the last 15 elements
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

        create_file(self.output_path.join("rss.xml"), &sitemap)?;

        Ok(())
    }

    /// Create a hashmap of paths to section
    /// For example `content/posts/_index.md` key will be `posts`
    fn get_sections_map(&self) -> HashMap<String, Section> {
        self.sections
            .values()
            .map(|s| (s.components.join("/"), s.clone()))
            .collect()
    }

    /// Renders a single section
    pub fn render_section(&self, section: &Section, render_pages: bool) -> Result<()> {
        self.ensure_public_directory_exists()?;
        let public = self.output_path.clone();

        let mut output_path = public.to_path_buf();
        for component in &section.components {
            output_path.push(component);

            if !output_path.exists() {
                create_directory(&output_path)?;
            }
        }

        if render_pages {
            for page in &section.pages {
                self.render_page(page)?;
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
            create_file(output_path.join("index.html"), &self.inject_livereload(output))?;
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
        self.ensure_public_directory_exists()?;

        for page in self.get_all_orphan_pages() {
            self.render_page(page)?;
        }

        Ok(())
    }

    /// Renders a list of pages when the section/index is wanting pagination.
    fn render_paginated(&self, output_path: &Path, section: &Section) -> Result<()> {
        self.ensure_public_directory_exists()?;

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
                create_file(page_path.join("index.html"), &self.inject_livereload(output))?;
            } else {
                create_file(output_path.join("index.html"), &self.inject_livereload(output))?;
                create_file(page_path.join("index.html"), &render_redirect_template(&section.permalink, &self.tera)?)?;
            }
        }

        Ok(())
    }
}
