use config::Config;
use config::TaxonomyConfig;
use content::{Library, Page, Section, SerializingPage, Taxonomy, TaxonomyTerm};
use errors::{Context as _, Result};
use serde::Serialize;
use tera::{Context, Tera};

/// Common parameters for rendering feeds
pub struct FeedInput<'a> {
    pub feed_filename: &'a str,
    pub lang: &'a str,
    pub feed_url: &'a str,
    pub pages: &'a [SerializingPage<'a>],
    pub last_updated: Option<&'a str>,
}

use crate::pagination::{Pager, PaginationRoot, Paginator};
use crate::{RenderCache, render_template};

pub struct Renderer<'a> {
    pub tera: &'a Tera,
    pub config: &'a Config,
    pub library: &'a Library,
    pub cache: &'a RenderCache,
}

impl<'a> Renderer<'a> {
    pub fn new(
        tera: &'a Tera,
        config: &'a Config,
        library: &'a Library,
        cache: &'a RenderCache,
    ) -> Self {
        Self { tera, config, library, cache }
    }

    pub fn render_page(&self, page: &Page) -> Result<String> {
        let mut context = Context::new();

        context.insert_value("config", self.cache.configs.get(&page.lang).unwrap().clone());
        context.insert_value("page", self.cache.pages.get(&page.file.path).unwrap().clone());
        context.insert("current_url", &page.permalink);
        context.insert("current_path", &page.path);
        context.insert("lang", &page.lang);

        let tpl_name = page.meta.template.as_deref().unwrap_or("page.html");
        render_template(tpl_name, self.tera, context)
            .with_context(|| format!("Failed to render section '{}'", page.file.path.display()))
    }

    pub fn render_section(&self, section: &Section) -> Result<String> {
        let mut context = Context::new();

        context.insert_value("config", self.cache.configs.get(&section.lang).unwrap().clone());
        context
            .insert_value("section", self.cache.sections.get(&section.file.path).unwrap().clone());
        context.insert("current_url", &section.permalink);
        context.insert("current_path", &section.path);
        context.insert("lang", &section.lang);

        let tpl_name = section.get_template_name();
        render_template(tpl_name, self.tera, context)
            .with_context(|| format!("Failed to render section '{}'", section.file.path.display()))
    }

    pub fn render_taxonomy_term(&self, taxonomy: &Taxonomy, term: &TaxonomyTerm) -> Result<String> {
        let cached = self.cache.get_taxonomy(&taxonomy.lang, &taxonomy.slug).unwrap();
        let mut context = Context::new();

        context.insert_value("config", self.cache.configs.get(&taxonomy.lang).unwrap().clone());
        context.insert_value("term", cached.terms.get(&term.slug).unwrap().clone());
        context.insert("taxonomy", &taxonomy.kind);
        context.insert("lang", &taxonomy.lang);
        context.insert("current_url", &term.permalink);
        context.insert("current_path", &term.path);

        let tpl = cached.single_template.as_deref().unwrap_or("taxonomy_single.html");
        render_template(tpl, self.tera, context)
            .with_context(|| format!("Failed to render taxonomy term '{}'", term.slug))
    }

    pub fn render_taxonomy_list(&self, taxonomy: &Taxonomy) -> Result<String> {
        // Get cached taxonomy (contains value, terms, and template names)
        let cached = self.cache.get_taxonomy(&taxonomy.lang, &taxonomy.slug).unwrap();

        let mut context = Context::new();

        context.insert_value("config", self.cache.configs.get(&taxonomy.lang).unwrap().clone());
        context.insert_value("terms", cached.value.clone());
        context.insert("taxonomy", &taxonomy.kind);
        context.insert("lang", &taxonomy.lang);
        context.insert("current_url", &taxonomy.permalink);
        context.insert("current_path", &taxonomy.path);

        let tpl = cached.list_template.as_deref().unwrap_or("taxonomy_list.html");
        render_template(tpl, self.tera, context)
            .with_context(|| format!("Failed to render taxonomy list '{}'", taxonomy.slug))
    }

    pub fn render_404(&self) -> Result<String> {
        let mut context = Context::new();
        context.insert_value(
            "config",
            self.cache.configs.get(&self.config.default_language).unwrap().clone(),
        );
        context.insert("lang", &self.config.default_language);
        render_template("404.html", self.tera, context).with_context(|| "Failed to render 404.html")
    }

    pub fn render_robots(&self) -> Result<String> {
        let mut context = Context::new();
        context.insert_value(
            "config",
            self.cache.configs.get(&self.config.default_language).unwrap().clone(),
        );
        context.insert("lang", &self.config.default_language);
        render_template("robots.txt", self.tera, context)
            .with_context(|| "Failed to render robots.txt")
    }

    pub fn render_paginated(&self, paginator: &Paginator, pager: &Pager) -> Result<String> {
        let mut context = Context::new();
        context.insert("current_url", &pager.permalink);
        context.insert("current_path", &pager.path);
        context.insert_value("paginator", paginator.build_context(pager, self.cache));

        match paginator.root {
            PaginationRoot::Section(s) => {
                context.insert_value(
                    "section",
                    self.cache.sections.get(&s.file.path).unwrap().clone(),
                );
                context.insert("lang", &s.lang);
                context.insert_value("config", self.cache.configs.get(&s.lang).unwrap().clone());
            }
            PaginationRoot::Taxonomy(t, term) => {
                let cached = self.cache.get_taxonomy(&t.lang, &t.slug).unwrap();
                context.insert("taxonomy", &t.kind);
                context.insert_value("term", cached.terms.get(&term.slug).unwrap().clone());
                context.insert("lang", &t.lang);
                context.insert_value("config", self.cache.configs.get(&t.lang).unwrap().clone());
            }
        }

        render_template(&paginator.template, self.tera, context)
            .with_context(|| format!("Failed to render pager {}", pager.index))
    }

    pub fn render_sitemap<T: Serialize>(&self, entries: &[T]) -> Result<String> {
        let mut context = Context::new();
        context.insert("entries", entries);
        context.insert_value(
            "config",
            self.cache.configs.get(&self.config.default_language).unwrap().clone(),
        );
        render_template("sitemap.xml", self.tera, context)
            .with_context(|| "Failed to render sitemap.xml")
    }

    pub fn render_sitemap_index(&self, sitemap_urls: &[String]) -> Result<String> {
        let mut context = Context::new();
        context.insert("sitemaps", sitemap_urls);
        context.insert_value(
            "config",
            self.cache.configs.get(&self.config.default_language).unwrap().clone(),
        );
        render_template("split_sitemap_index.xml", self.tera, context)
            .with_context(|| "Failed to render sitemap index")
    }

    pub fn render_feed(&self, input: &FeedInput<'_>) -> Result<String> {
        let context = self.build_feed_context(input);
        render_template(input.feed_filename, self.tera, context)
            .with_context(|| format!("Failed to render feed '{}'", input.feed_filename))
    }

    pub fn render_section_feed<S: Serialize>(
        &self,
        input: &FeedInput<'_>,
        section: &S,
    ) -> Result<String> {
        let mut context = self.build_feed_context(input);
        context.insert("section", section);
        render_template(input.feed_filename, self.tera, context)
            .with_context(|| format!("Failed to render feed '{}'", input.feed_filename))
    }

    pub fn render_taxonomy_feed<T: Serialize>(
        &self,
        input: &FeedInput<'_>,
        taxonomy: &TaxonomyConfig,
        term: &T,
    ) -> Result<String> {
        let mut context = self.build_feed_context(input);
        context.insert("taxonomy", taxonomy);
        context.insert("term", term);
        render_template(input.feed_filename, self.tera, context)
            .with_context(|| format!("Failed to render feed '{}'", input.feed_filename))
    }

    fn build_feed_context(&self, input: &FeedInput<'_>) -> Context {
        let mut context = Context::new();
        context.insert("pages", input.pages);
        context.insert_value("config", self.cache.configs.get(input.lang).unwrap().clone());
        context.insert("lang", input.lang);
        context.insert("feed_url", input.feed_url);
        if let Some(updated) = input.last_updated {
            context.insert("last_updated", updated);
        }
        context
    }
}
