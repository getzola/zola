use std::collections::{BTreeMap, HashMap};
use std::iter::FromIterator;
use std::fs::{remove_dir_all, copy, remove_file};
use std::path::{Path, PathBuf};

use glob::glob;
use tera::{Tera, Context};
use slug::slugify;
use walkdir::WalkDir;

use errors::{Result, ResultExt};
use config::{Config, get_config};
use page::{Page};
use utils::{create_file, create_directory};
use section::{Section};


lazy_static! {
    static ref GUTENBERG_TERA: Tera = {
        let mut tera = Tera::default();
        tera.add_raw_templates(vec![
            ("rss.xml", include_str!("templates/rss.xml")),
            ("sitemap.xml", include_str!("templates/sitemap.xml")),
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
    pub templates: Tera,
    live_reload: bool,
    output_path: PathBuf,
}

impl Site {
    /// Parse a site at the given path. Defaults to the current dir
    /// Passing in a path is only used in tests
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Site> {
        let path = path.as_ref();

        let tpl_glob = format!("{}/{}", path.to_string_lossy().replace("\\", "/"), "templates/**/*");
        let mut tera = Tera::new(&tpl_glob).chain_err(|| "Error parsing templates")?;
        tera.extend(&GUTENBERG_TERA)?;

        let mut site = Site {
            base_path: path.to_path_buf(),
            config: get_config(path),
            pages: HashMap::new(),
            sections: BTreeMap::new(),
            templates: tera,
            live_reload: false,
            output_path: PathBuf::from("public"),
        };
        site.parse_site()?;

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

    /// Reads all .md files in the `content` directory and create pages
    /// out of them
    fn parse_site(&mut self) -> Result<()> {
        let path = self.base_path.to_string_lossy().replace("\\", "/");
        let content_glob = format!("{}/{}", path, "content/**/*.md");

        // parent_dir -> Section
        let mut sections = BTreeMap::new();

        // Glob is giving us the result order so _index will show up first
        // for each directory
        for entry in glob(&content_glob).unwrap().filter_map(|e| e.ok()) {
            let path = entry.as_path();

            if path.file_name().unwrap() == "_index.md" {
                let section = Section::from_file(&path, &self.config)?;
                sections.insert(section.parent_path.clone(), section);
            } else {
                let page = Page::from_file(&path, &self.config)?;
                if sections.contains_key(&page.parent_path) {
                    sections.get_mut(&page.parent_path).unwrap().pages.push(page.clone());
                }
                self.pages.insert(page.file_path.clone(), page);
            }
        }

        // Find out the direct subsections of each subsection if there are some
        let mut grandparent_paths = HashMap::new();
        for section in sections.values() {
            let grand_parent = section.parent_path.parent().unwrap().to_path_buf();
            grandparent_paths.entry(grand_parent).or_insert_with(|| vec![]).push(section.clone());
        }

        for (parent_path, section) in &mut sections {
            section.pages.sort_by(|a, b| a.partial_cmp(b).unwrap());

            match grandparent_paths.get(parent_path) {
                Some(paths) => section.subsections.extend(paths.clone()),
                None => continue,
            };
        }

        self.sections = sections;

        Ok(())
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

    /// Copy the content of the `static` folder into the `public` folder
    ///
    /// TODO: only copy one file if possible because that would be a waste
    /// to do re-copy the whole thing. Benchmark first to see if it's a big difference
    pub fn copy_static_directory(&self) -> Result<()> {
        let from = Path::new("static");
        let target = Path::new("public");

        for entry in WalkDir::new(from).into_iter().filter_map(|e| e.ok()) {
            let relative_path = entry.path().strip_prefix(&from).unwrap();
            let target_path = {
                let mut target_path = target.to_path_buf();
                target_path.push(relative_path);
                target_path
            };

            if entry.path().is_dir() {
                if !target_path.exists() {
                    create_directory(&target_path)?;
                }
            } else {
                if target_path.exists() {
                    remove_file(&target_path)?;
                }
                copy(entry.path(), &target_path)?;
            }
        }
        Ok(())
    }

    /// Deletes the `public` directory if it exists
    pub fn clean(&self) -> Result<()> {
        if Path::new("public").exists() {
            // Delete current `public` directory so we can start fresh
            remove_dir_all("public").chain_err(|| "Couldn't delete `public` directory")?;
        }

        Ok(())
    }

    pub fn rebuild_after_content_change(&mut self) -> Result<()> {
        self.parse_site()?;
        self.build()
    }

    pub fn rebuild_after_template_change(&mut self) -> Result<()> {
        self.templates.full_reload()?;
        self.build_pages()
    }

    pub fn build_pages(&self) -> Result<()> {
        let public = self.output_path.clone();
        if !public.exists() {
            create_directory(&public)?;
        }

        let mut pages = vec![];
        let mut category_pages: HashMap<String, Vec<&Page>> = HashMap::new();
        let mut tag_pages: HashMap<String, Vec<&Page>> = HashMap::new();

        // First we render the pages themselves
        for page in self.pages.values() {
            // Copy the nesting of the content directory if we have sections for that page
            let mut current_path = public.to_path_buf();

            for component in page.url.split('/') {
                current_path.push(component);

                if !current_path.exists() {
                    create_directory(&current_path)?;
                }
            }

            // Make sure the folder exists
            create_directory(&current_path)?;

            // Finally, create a index.html file there with the page rendered
            let output = page.render_html(&self.templates, &self.config)?;
            create_file(current_path.join("index.html"), &self.inject_livereload(output))?;

            // Copy any asset we found previously into the same directory as the index.html
            for asset in &page.assets {
                let asset_path = asset.as_path();
                copy(&asset_path, &current_path.join(asset_path.file_name().unwrap()))?;
            }

            pages.push(page);

            if let Some(ref category) = page.meta.category {
                category_pages.entry(category.to_string()).or_insert_with(|| vec![]).push(page);
            }

            if let Some(ref tags) = page.meta.tags {
                for tag in tags {
                    tag_pages.entry(tag.to_string()).or_insert_with(|| vec![]).push(page);
                }
            }
        }

        // Outputting categories and pages
        self.render_categories_and_tags(RenderList::Categories, &category_pages)?;
        self.render_categories_and_tags(RenderList::Tags, &tag_pages)?;

        // And finally the index page
        let mut context = Context::new();
        pages.sort_by(|a, b| a.partial_cmp(b).unwrap());
        context.add("pages", &pages);
        context.add("config", &self.config);
        let index = self.templates.render("index.html", &context)?;
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
        self.render_sections()?;
        self.copy_static_directory()
    }

    /// Render the /{categories, list} pages and each individual category/tag page
    fn render_categories_and_tags(&self, kind: RenderList, container: &HashMap<String, Vec<&Page>>) -> Result<()> {
        if container.is_empty() {
            return Ok(());
        }

        let (name, list_tpl_name, single_tpl_name, var_name) = if kind == RenderList::Categories {
            ("categories", "categories.html", "category.html", "category")
        } else {
            ("tags", "tags.html", "tag.html", "tag")
        };

        let public = self.output_path.clone();
        let mut output_path = public.to_path_buf();
        output_path.push(name);
        create_directory(&output_path)?;

        // First we render the list of categories/tags page
        let mut sorted_container = vec![];
        for (item, count) in Vec::from_iter(container).into_iter().map(|(a, b)| (a, b.len())) {
            sorted_container.push(ListItem::new(item, count));
        }
        sorted_container.sort_by(|a, b| b.count.cmp(&a.count));

        let mut context = Context::new();
        context.add(name, &sorted_container);
        context.add("config", &self.config);

        let list_output = self.templates.render(list_tpl_name, &context)?;
        create_file(output_path.join("index.html"), &self.inject_livereload(list_output))?;

        // and then each individual item
        for (item_name, mut pages) in container.clone() {
            let mut context = Context::new();
            pages.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let slug = slugify(&item_name);
            context.add(var_name, &item_name);
            context.add(&format!("{}_slug", var_name), &slug);
            context.add("pages", &pages);
            context.add("config", &self.config);
            let single_output = self.templates.render(single_tpl_name, &context)?;

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
        // TODO: add categories and tags pages
        let sitemap = self.templates.render("sitemap.xml", &context)?;

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
            format!("{}{}", self.config.base_url, "feed.xml")
        } else {
            format!("{}/{}", self.config.base_url, "feed.xml")
        };
        context.add("feed_url", &rss_feed_url);

        let sitemap = self.templates.render("rss.xml", &context)?;

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

            let output = section.render_html(&self.templates, &self.config)?;
            create_file(output_path.join("index.html"), &self.inject_livereload(output))?;
        }

        Ok(())
    }
}
