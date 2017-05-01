use std::collections::{BTreeMap, HashMap};
use std::iter::FromIterator;
use std::fs::{remove_dir_all, copy, create_dir_all};
use std::path::{Path, PathBuf};

use glob::glob;
use tera::{Tera, Context};
use slug::slugify;
use walkdir::WalkDir;

use errors::{Result, ResultExt};
use config::{Config, get_config};
use page::{Page, populate_previous_and_next_pages, sort_pages};
use utils::{create_file, create_directory};
use section::{Section};
use filters;


lazy_static! {
    pub static ref GUTENBERG_TERA: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("rss.xml", include_str!("templates/rss.xml")),
            ("sitemap.xml", include_str!("templates/sitemap.xml")),
            ("robots.txt", include_str!("templates/robots.txt")),
            ("anchor-link.html", include_str!("templates/anchor-link.html")),

            ("shortcodes/youtube.html", include_str!("templates/shortcodes/youtube.html")),
            ("shortcodes/vimeo.html", include_str!("templates/shortcodes/vimeo.html")),
            ("shortcodes/gist.html", include_str!("templates/shortcodes/gist.html")),
        ]).unwrap();
        tera
    };
}


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
    pub sections: BTreeMap<PathBuf, Section>,
    pub index: Option<Section>,
    pub tera: Tera,
    live_reload: bool,
    output_path: PathBuf,
    static_path: PathBuf,
    pub tags: HashMap<String, Vec<PathBuf>>,
    pub categories: HashMap<String, Vec<PathBuf>>,
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
        tera.register_filter("markdown", filters::markdown);
        tera.register_filter("base64_encode", filters::base64_encode);
        tera.register_filter("base64_decode", filters::base64_decode);

        let site = Site {
            base_path: path.to_path_buf(),
            config: get_config(path, config_file),
            pages: HashMap::new(),
            sections: BTreeMap::new(),
            index: None,
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

        // TODO: make that parallel, that's the main bottleneck
        // `add_section` and `add_page` can't be used in the parallel version afaik
        for entry in glob(&content_glob).unwrap().filter_map(|e| e.ok()) {
            let path = entry.as_path();
            if path.file_name().unwrap() == "_index.md" {
                // Index section
                if path.parent().unwrap() == self.base_path.join("content") {
                    self.index = Some(Section::from_file(path, &self.config)?);
                } else {
                    // all the other sections
                    self.add_section(path)?;
                }
            } else {
                self.add_page(path)?;
            }
        }

        // A map of all .md files (section and pages) and their permalink
        // We need that if there are relative links in the content that need to be resolved
        let mut permalinks = HashMap::new();

        for page in self.pages.values() {
            permalinks.insert(page.relative_path.clone(), page.permalink.clone());
        }

        for section in self.sections.values() {
            permalinks.insert(section.relative_path.clone(), section.permalink.clone());
        }

        for page in self.pages.values_mut() {
            page.render_markdown(&permalinks, &self.tera, &self.config)?;
        }

        self.permalinks = permalinks;
        self.populate_sections();
        self.populate_tags_and_categories();

        Ok(())
    }

    /// Simple wrapper fn to avoid repeating that code in several places
    fn add_page(&mut self, path: &Path) -> Result<()> {
        let page = Page::from_file(&path, &self.config)?;
        self.pages.insert(page.file_path.clone(), page);
        Ok(())
    }

    /// Simple wrapper fn to avoid repeating that code in several places
    fn add_section(&mut self, path: &Path) -> Result<()> {
        let section = Section::from_file(path, &self.config)?;
        self.sections.insert(section.parent_path.clone(), section);
        Ok(())
    }

    /// Called in serve, add a page again updating permalinks and its content
    fn add_page_and_render(&mut self, path: &Path) -> Result<()> {
        self.add_page(path)?;
        let mut page = self.pages.get_mut(path).unwrap();
        self.permalinks.insert(page.relative_path.clone(), page.permalink.clone());
        page.render_markdown(&self.permalinks, &self.tera, &self.config)
    }

    /// Find out the direct subsections of each subsection if there are some
    /// as well as the pages for each section
    fn populate_sections(&mut self) {
        for page in self.pages.values() {
            if self.sections.contains_key(&page.parent_path) {
                self.sections.get_mut(&page.parent_path).unwrap().pages.push(page.clone());
            }
        }

        let mut grandparent_paths = HashMap::new();
        for section in self.sections.values() {
            let grand_parent = section.parent_path.parent().unwrap().to_path_buf();
            grandparent_paths.entry(grand_parent).or_insert_with(|| vec![]).push(section.clone());
        }

        for (parent_path, section) in &mut self.sections {
            // TODO: avoid this clone
            let (sorted_pages, _) = sort_pages(section.pages.clone(), Some(&section));
            section.pages = sorted_pages;

            match grandparent_paths.get(parent_path) {
                Some(paths) => section.subsections.extend(paths.clone()),
                None => continue,
            };
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

    pub fn rebuild_after_content_change(&mut self, path: &Path) -> Result<()> {
        let is_section = path.ends_with("_index.md");
        let is_index_section = if is_section {
            path.parent().unwrap() == self.base_path.join("content")
        } else {
            false
        };

        if path.exists() {
            // file exists, either a new one or updating content
            if is_section {
                if is_index_section {
                    self.index = Some(Section::from_file(path, &self.config)?);
                } else {
                    self.add_section(path)?;
                }
            } else {
                // probably just an update so just re-parse that page
                self.add_page_and_render(path)?;
            }
        } else {
            // File doesn't exist -> a deletion so we remove it from everything
            if is_section {
                if !is_index_section {
                    let relative_path = self.sections[path].relative_path.clone();
                    self.sections.remove(path);
                    self.permalinks.remove(&relative_path);
                } else {
                    self.index = None;
                }
            } else {
                let relative_path = self.pages[path].relative_path.clone();
                self.pages.remove(path);
                self.permalinks.remove(&relative_path);
            }
        }
        self.populate_sections();
        self.populate_tags_and_categories();
        self.build()
    }

    pub fn rebuild_after_template_change(&mut self, path: &Path) -> Result<()> {
        self.tera.full_reload()?;
        match path.file_name().unwrap().to_str().unwrap() {
            "sitemap.xml" => self.render_sitemap(),
            "rss.xml" => self.render_rss_feed(),
            _ => self.build_pages()
        }
    }

    pub fn render_page(&self, page: &Page) -> Result<()> {
        let public = self.output_path.clone();
        if !public.exists() {
            create_directory(&public)?;
        }

        // Copy the nesting of the content directory if we have sections for that page
        let mut current_path = public.to_path_buf();

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

    pub fn build_pages(&self) -> Result<()> {
        let public = self.output_path.clone();
        if !public.exists() {
            create_directory(&public)?;
        }

        // Sort the pages first
        // TODO: avoid the clone()
        let (mut sorted_pages, cannot_sort_pages) = sort_pages(self.pages.values().map(|p| p.clone()).collect(), self.index.as_ref());

        sorted_pages = populate_previous_and_next_pages(&sorted_pages);
        for page in &sorted_pages {
            self.render_page(page)?;
        }
        for page in &cannot_sort_pages {
            self.render_page(page)?;
        }

        // Outputting categories and pages
        if self.config.generate_categories_pages.unwrap() {
            self.render_categories_and_tags(RenderList::Categories)?;
        }
        if self.config.generate_tags_pages.unwrap() {
            self.render_categories_and_tags(RenderList::Tags)?;
        }

        // And finally the index page
        let mut context = Context::new();

        context.add("pages", &sorted_pages);
        context.add("sections", &self.sections.values().collect::<Vec<&Section>>());
        context.add("config", &self.config);
        context.add("current_url", &self.config.base_url);
        context.add("current_path", &"");
        let index = self.tera.render("index.html", &context)?;
        create_file(public.join("index.html"), &self.inject_livereload(index))?;

        Ok(())
    }

    /// Builds the site to the `public` directory after deleting it
    pub fn build(&self) -> Result<()> {
        self.clean()?;
        self.build_pages()?;
        self.render_sitemap()?;

        if self.config.generate_rss.unwrap() {
            self.render_rss_feed()?;
        }

        self.render_robots()?;

        self.render_sections()?;
        self.copy_static_directory()
    }

    fn render_robots(&self) -> Result<()> {
        create_file(
            self.output_path.join("robots.txt"),
            &self.tera.render("robots.txt", &Context::new())?
        )
    }

    /// Render the /{categories, list} pages and each individual category/tag page
    /// They are the same thing fundamentally, a list of pages with something in common
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
            let mut pages: Vec<&Page> = self.pages
                .iter()
                .filter(|&(path, _)| pages_paths.contains(path))
                .map(|(_, page)| page)
                .collect();
            // TODO: how to sort categories and tag content?
            // Have a setting in config.toml or a _category.md and _tag.md
            // The latter is more in line with the rest of Gutenberg but order ordering
            // doesn't really work across sections so default to partial ordering for now (date)
            pages.sort_by(|a, b| a.partial_cmp(b).unwrap());

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

    fn render_sitemap(&self) -> Result<()> {
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

    fn render_rss_feed(&self) -> Result<()> {
        let mut context = Context::new();
        let mut pages = self.pages.values()
            .filter(|p| p.meta.date.is_some())
            .take(15) // limit to the last 15 elements
            .collect::<Vec<&Page>>();

        // Don't generate a RSS feed if none of the pages has a date
        if pages.is_empty() {
            return Ok(());
        }

        pages.sort_by(|a, b| a.partial_cmp(b).unwrap());
        context.add("pages", &pages);
        context.add("last_build_date", &pages[0].meta.date);
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

    fn render_sections(&self) -> Result<()> {
        let public = self.output_path.clone();

        for section in self.sections.values() {
            let mut output_path = public.to_path_buf();
            for component in &section.components {
                output_path.push(component);

                if !output_path.exists() {
                    create_directory(&output_path)?;
                }
            }

            let output = section.render_html(&self.tera, &self.config)?;
            create_file(output_path.join("index.html"), &self.inject_livereload(output))?;
        }

        Ok(())
    }
}
