use std::collections::HashMap;
use std::fs::{create_dir, remove_dir_all};
use std::path::Path;

use glob::glob;
use tera::{Tera, Context};

use errors::{Result, ResultExt};
use config::{Config, get_config};
use page::Page;
use utils::create_file;


#[derive(Debug)]
pub struct Site {
    config: Config,
    pages: HashMap<String, Page>,
    sections: HashMap<String, Vec<String>>,
    templates: Tera,
}

impl Site {
    pub fn new() -> Result<Site> {
        let tera = Tera::new("templates/**/*").chain_err(|| "Error parsing templates")?;
        let mut site = Site {
            config: get_config(),
            pages: HashMap::new(),
            sections: HashMap::new(),
            templates: tera,
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
            let page = Page::from_file(&entry.as_path())?;

            for section in &page.sections {
                self.sections.entry(section.clone()).or_insert(vec![]).push(page.get_slug());
            }

            self.pages.insert(page.get_slug(), page);
        }

        Ok(())
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
            create_file(current_path.join("index.html"), &output)?;
            pages.push(page);
        }

        // Then the section pages
        // The folders have already been created in the page loop so no need to `create_dir` here
//        for (section, slugs) in &self.sections {
//            // TODO
//        }

        // And finally the index page
        let mut context = Context::new();
        context.add("pages", &pages);
        context.add("config", &self.config);
        create_file(public.join("index.html"), &self.templates.render("index.html", &context)?)?;

        Ok(())
    }
}
