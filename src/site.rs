use std::collections::HashMap;
use std::iter::FromIterator;
use std::fs::{create_dir, remove_dir_all, copy, remove_file};
use std::path::Path;

use glob::glob;
use tera::{Tera, Context};
use slug::slugify;
use walkdir::WalkDir;

use errors::{Result, ResultExt};
use config::{Config, get_config};
use page::Page;
use utils::create_file;


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
    config: Config,
    pages: HashMap<String, Page>,
    sections: HashMap<String, Vec<String>>,
    templates: Tera,
    live_reload: bool,
}

impl Site {
    pub fn new(livereload: bool) -> Result<Site> {
        let mut tera = Tera::new("templates/**/*")
            .chain_err(|| "Error parsing templates")?;
        tera.extend(&GUTENBERG_TERA)?;

        let mut site = Site {
            config: get_config(),
            pages: HashMap::new(),
            sections: HashMap::new(),
            templates: tera,
            live_reload: livereload,
        };
        site.parse_site()?;

        Ok(site)
    }

    /// Reads all .md files in the `content` directory and create pages
    /// out of them
    fn parse_site(&mut self) -> Result<()> {
        // First step: do all the articles and group article by sections
        // hardcoded pattern so can't error
        for entry in glob("content/**/*.md").unwrap().filter_map(|e| e.ok()) {
            let page = Page::from_file(&entry.as_path(), &self.config)?;

            for section in &page.sections {
                self.sections.entry(section.clone()).or_insert(vec![]).push(page.slug.clone());
            }

            self.pages.insert(page.slug.clone(), page);
        }

        Ok(())
    }

    // Inject live reload script tag if in live reload mode
    fn inject_livereload(&self, html: String) -> String {
        if self.live_reload {
            return html.replace(
                "</body>",
                r#"<script src="/livereload.js?port=1112&mindelay=10"></script></body>"#
            );
        }

        html
    }

    /// Reload the Tera templates
    pub fn reload_templates(&mut self) -> Result<()> {
        Ok(())
    }

    /// Copy the content of the `static` folder into the `public` folder
    ///
    /// TODO: only copy one file if possible because that would be a waster
    /// to do re-copy the whole thing
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
                    create_dir(&target_path)?;
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

    /// Re-parse and re-generate the site
    /// Very dumb for now, ideally it would only rebuild what changed
    pub fn rebuild(&mut self) -> Result<()> {
        self.parse_site()?;
        self.templates.full_reload()?;
        self.build()
    }

    /// Builds the site to the `public` directory after deleting it
    pub fn build(&self) -> Result<()> {
        if Path::new("public").exists() {
            // Delete current `public` directory so we can start fresh
            remove_dir_all("public").chain_err(|| "Couldn't delete `public` directory")?;
        }

        // Start from scratch
        create_dir("public")?;
        let public = Path::new("public");

        let mut pages = vec![];
        let mut category_pages: HashMap<String, Vec<&Page>> = HashMap::new();
        let mut tag_pages: HashMap<String, Vec<&Page>> = HashMap::new();
        // First we render the pages themselves
        for page in self.pages.values() {
            // Copy the nesting of the content directory if we have sections for that page
            let mut current_path = public.to_path_buf();

            // This loop happens when the page doesn't have a set URL
            for section in &page.sections {
                current_path.push(section);

                if !current_path.exists() {
                    create_dir(&current_path)?;
                }
            }

            // if we have a url already set, use that as base
            if let Some(ref url) = page.meta.url {
                current_path.push(url);
            }

            // Make sure the folder exists
            create_dir(&current_path)?;
            // Finally, create a index.html file there with the page rendered
            let output = page.render_html(&self.templates, &self.config)?;
            create_file(current_path.join("index.html"), &self.inject_livereload(output))?;
            pages.push(page);

            if let Some(ref category) = page.meta.category {
                category_pages.entry(category.to_string()).or_insert(vec![]).push(page);
            }
            if let Some(ref tags) = page.meta.tags {
                for tag in tags {
                    tag_pages.entry(tag.to_string()).or_insert(vec![]).push(page);
                }
            }
        }

        // Outputting categories and pages
        self.render_categories_and_tags(RenderList::Categories, &category_pages)?;
        self.render_categories_and_tags(RenderList::Tags, &tag_pages)?;

        self.render_sitemap()?;
        self.render_rss_feed()?;

        // And finally the index page
        let mut context = Context::new();
        pages.sort_by(|a, b| a.partial_cmp(b).unwrap());
        context.add("pages", &pages);
        context.add("config", &self.config);
        let index = self.templates.render("index.html", &context)?;
        create_file(public.join("index.html"), &self.inject_livereload(index))?;

        self.copy_static_directory()?;

        Ok(())
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

        let public = Path::new("public");
        let mut output_path = public.to_path_buf();
        output_path.push(name);
        create_dir(&output_path)?;

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

            create_dir(&output_path.join(&slug))?;
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
        let sitemap = self.templates.render("sitemap.xml", &context)?;

        let public = Path::new("public");
        create_file(public.join("sitemap.xml"), &sitemap)?;

        Ok(())
    }

    fn get_rss_feed_url(&self) -> String {
        if self.config.base_url.ends_with("/") {
            format!("{}{}", self.config.base_url, "feed.xml")
        } else {
            format!("{}/{}", self.config.base_url, "feed.xml")
        }
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
        context.add("feed_url", &self.get_rss_feed_url());

        let sitemap = self.templates.render("rss.xml", &context)?;

        let public = Path::new("public");
        create_file(public.join("rss.xml"), &sitemap)?;

        Ok(())
    }
}
