use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::{BuildMode, SITE_CONTENT, Site, feeds, minify, sitemap};
use content::{Page, Section, Taxonomy, TaxonomyTerm};
use errors::Result;
use fs_err as fs;
use rayon::prelude::*;
use relative_path::RelativePathBuf;
use render::{FeedInput, Paginator, Renderer};
use templates::render_redirect_template;
use utils::net::is_external_link;

#[derive(Debug, Clone, PartialEq)]
enum Feed<'a> {
    Site { lang: &'a str },
    Section { section: &'a Section, pages: Vec<&'a Page> },
    Taxonomy { taxonomy: &'a Taxonomy, term: &'a TaxonomyTerm, path: PathBuf },
}

#[derive(Debug, Clone, PartialEq)]
enum Job<'a> {
    Alias { from: &'a str, to: &'a str },
    Page(&'a Page),
    Section { section: &'a Section, path: PathBuf },
    Paginated { paginator_index: usize, pager_index: usize, path: PathBuf },
    TaxonomyList { taxonomy: &'a Taxonomy, path: PathBuf },
    TaxonomyTerm { taxonomy: &'a Taxonomy, term: &'a TaxonomyTerm, path: PathBuf },
    Feed(Feed<'a>),
    Sitemap,
    NotFound,
    Robots,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum OutputKind {
    Html,
    Xml,
    Text,
    Json,
    Css,
}

impl From<&str> for OutputKind {
    fn from(s: &str) -> Self {
        if s.ends_with(".html") {
            Self::Html
        } else if s.ends_with(".xml") {
            Self::Xml
        } else if s.ends_with(".json") {
            Self::Json
        } else if s.ends_with(".css") {
            Self::Css
        } else {
            Self::Text
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RenderedOutput {
    path: PathBuf,
    content: String,
    kind: OutputKind,
}

#[derive(Debug, Clone)]
pub struct Queue<'a> {
    site: &'a Site,
    jobs: Vec<Job<'a>>,
    /// Keeping them on the queue so we can use the index to avoid cloning them
    paginators: Vec<Paginator<'a>>,
}

impl<'a> Queue<'a> {
    fn add_section_jobs(&mut self, section: &'a Section, render_pages: bool) {
        let pages: Vec<_> =
            section.pages.iter().map(|k| self.site.library.pages.get(k).unwrap()).collect();
        if render_pages {
            for page in &pages {
                if page.meta.render {
                    self.jobs.push(Job::Page(page));
                }
            }
        }

        if section.meta.generate_feeds {
            self.jobs.push(Job::Feed(Feed::Section { pages, section }));
        }

        if !section.meta.render {
            return;
        }

        let mut base_path = PathBuf::new();
        if section.lang != self.site.config.default_language {
            base_path.push(&section.lang);
        }
        for component in &section.file.components {
            base_path.push(component);
        }

        if section.meta.is_paginated() {
            let paginator = Paginator::from_section(section, &self.site.library);
            let paginator_index = self.paginators.len();
            for pager_index in 0..paginator.pagers.len() {
                self.jobs.push(Job::Paginated {
                    paginator_index,
                    path: base_path.clone(),
                    pager_index,
                });
            }
            self.paginators.push(paginator);
        } else {
            // This will handle a normal section, which could also be a redirect
            self.jobs.push(Job::Section { section, path: base_path });
        }
    }

    pub fn single_page(site: &'a Site, page: &'a Page) -> Self {
        Self { jobs: vec![Job::Page(page)], site, paginators: vec![] }
    }

    pub fn single_section(site: &'a Site, section: &'a Section, render_pages: bool) -> Self {
        let mut queue = Self { jobs: vec![], site, paginators: vec![] };
        queue.add_section_jobs(section, render_pages);
        queue
    }

    pub fn full_build(site: &'a Site) -> Self {
        let mut queue =
            Queue { site, jobs: vec![Job::NotFound], paginators: Vec::with_capacity(100) };

        if site.config.generate_sitemap {
            queue.jobs.push(Job::Sitemap);
        }
        if site.config.generate_robots_txt {
            queue.jobs.push(Job::Robots);
        }

        // Aliases
        for (_, page) in &site.library.pages {
            for alias in &page.meta.aliases {
                queue.jobs.push(Job::Alias { from: alias, to: &page.permalink });
            }
        }
        for (_, section) in &site.library.sections {
            for alias in &section.meta.aliases {
                queue.jobs.push(Job::Alias { from: alias, to: &section.permalink });
            }
        }

        // Pages + sections
        for (_, section) in &site.library.sections {
            queue.add_section_jobs(section, true);
        }

        // Orphan pages
        for page in site.library.get_all_orphan_pages() {
            if page.meta.render {
                queue.jobs.push(Job::Page(page));
            }
        }

        // Taxonomies
        for taxonomy in &site.taxonomies {
            if !taxonomy.kind.render || taxonomy.items.is_empty() {
                continue;
            }

            let mut base_path = PathBuf::new();
            if taxonomy.lang != site.config.default_language {
                base_path.push(&taxonomy.lang);
            }
            if let Some(ref taxonomy_root) = site.config.taxonomy_root {
                base_path.push(taxonomy_root);
            }
            base_path.push(&taxonomy.slug);
            queue.jobs.push(Job::TaxonomyList { taxonomy, path: base_path.clone() });
            for term in &taxonomy.items {
                let term_path = base_path.join(&term.slug);

                if taxonomy.kind.is_paginated() {
                    let paginator =
                        Paginator::from_taxonomy(taxonomy, term, &site.library, &site.tera);
                    let paginator_index = queue.paginators.len();
                    for pager_index in 0..paginator.pagers.len() {
                        queue.jobs.push(Job::Paginated {
                            paginator_index,
                            path: term_path.clone(),
                            pager_index,
                        });
                    }
                    queue.paginators.push(paginator);
                } else {
                    queue.jobs.push(Job::TaxonomyTerm { taxonomy, term, path: term_path.clone() });
                }

                if taxonomy.kind.feed {
                    queue.jobs.push(Job::Feed(Feed::Taxonomy { taxonomy, term, path: term_path }));
                }
            }
        }

        // Whole site feed
        if site.config.generate_feeds {
            for (lang, lang_config) in &site.config.languages {
                if !lang_config.generate_feeds {
                    continue;
                }
                queue.jobs.push(Job::Feed(Feed::Site { lang }));
            }
        }

        queue
    }

    pub fn process(&self) -> Result<()> {
        self.jobs.par_iter().try_for_each(|job| {
            for output in self.execute_job(job)? {
                self.write_output(output)?;
            }

            match job {
                Job::Page(page) => {
                    let dest = self
                        .site
                        .output_path
                        .join(page.path.strip_prefix('/').unwrap_or(&page.path));
                    self.site.copy_assets(page.file.path.parent().unwrap(), &page.assets, &dest)?;
                }
                Job::Section { section, path } if section.meta.redirect_to.is_none() => {
                    let dest = self.site.output_path.join(path);
                    self.site.copy_assets(
                        section.file.path.parent().unwrap(),
                        &section.assets,
                        &dest,
                    )?;
                }
                _ => {}
            }
            Ok(())
        })
    }

    fn write_output(&self, output: RenderedOutput) -> Result<()> {
        let content = match output.kind {
            OutputKind::Html => {
                // TODO: move live reload injection to this file
                let mut c = self.site.inject_livereload(output.content);
                if self.site.config.minify_html {
                    c = minify::html(c)?;
                }
                c
            }
            _ => output.content,
        };
        let relative_path = output.path.strip_prefix("/").unwrap_or(&output.path);

        // First write to disk when needed
        match self.site.build_mode {
            BuildMode::Disk | BuildMode::Both => {
                let full_path = self.site.output_path.join(relative_path);
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&full_path, &content)?;
            }
            _ => (),
        }
        // Then in the in-memory map for zola serve
        match self.site.build_mode {
            BuildMode::Memory | BuildMode::Both => {
                let site_path = if relative_path.ends_with("index.html") {
                    RelativePathBuf::from_path(relative_path.parent().unwrap_or(relative_path))
                        .unwrap()
                } else {
                    RelativePathBuf::from_path(relative_path).unwrap()
                };

                SITE_CONTENT.write().unwrap().insert(site_path, content);
            }
            _ => (),
        }

        Ok(())
    }

    fn renderer(&self) -> Renderer<'_> {
        Renderer::new(&self.site.tera, &self.site.config, &self.site.library, &self.site.cache)
    }

    fn render_alias(&'a self, from: &'a str, to: &'a str) -> Result<RenderedOutput> {
        let mut path = PathBuf::from(from);
        if !from.ends_with(".html") {
            path.push("index.html");
        }
        let content = render_redirect_template(to, &self.site.tera)?;
        Ok(RenderedOutput { path, content, kind: OutputKind::Html })
    }

    fn render_page(&'a self, page: &'a Page) -> Result<RenderedOutput> {
        let content = self.renderer().render_page(page)?;
        let path = PathBuf::from(&page.path).join("index.html");
        Ok(RenderedOutput { path, content, kind: OutputKind::Html })
    }

    fn render_section(&self, section: &Section, path: &Path) -> Result<RenderedOutput> {
        if let Some(redirect_to) = &section.meta.redirect_to {
            let permalink: Cow<str> = if is_external_link(redirect_to) {
                Cow::Borrowed(redirect_to)
            } else {
                Cow::Owned(self.site.config.make_permalink(redirect_to))
            };
            let content = render_redirect_template(&permalink, &self.site.tera)?;
            Ok(RenderedOutput { path: path.join("index.html"), content, kind: OutputKind::Html })
        } else {
            let content = self.renderer().render_section(section)?;
            Ok(RenderedOutput { path: path.join("index.html"), content, kind: OutputKind::Html })
        }
    }

    fn render_paginated(
        &self,
        paginator_index: usize,
        pager_index: usize,
        path: &Path,
    ) -> Result<Vec<RenderedOutput>> {
        let paginator = &self.paginators[paginator_index];
        let pager = &paginator.pagers[pager_index];
        let mut res = Vec::new();

        let content = self.renderer().render_paginated(paginator, pager)?;

        if pager.index == 1 {
            // First page at root
            res.push(RenderedOutput {
                path: path.join("index.html"),
                content,
                kind: OutputKind::Html,
            });
            // Redirect from /page/1/ to root
            let redirect_path = path.join(&paginator.paginate_path).join("1").join("index.html");
            let redirect_content = render_redirect_template(&paginator.permalink, &self.site.tera)?;
            res.push(RenderedOutput {
                path: redirect_path,
                content: redirect_content,
                kind: OutputKind::Html,
            });
        } else {
            // /page/2/, /page/3/, etc.
            let output_path = path
                .join(&paginator.paginate_path)
                .join(pager.index.to_string())
                .join("index.html");
            res.push(RenderedOutput { path: output_path, content, kind: OutputKind::Html });
        }

        Ok(res)
    }

    fn render_taxonomy_list(&self, taxonomy: &Taxonomy, path: &Path) -> Result<RenderedOutput> {
        let content = self.renderer().render_taxonomy_list(taxonomy)?;
        Ok(RenderedOutput { path: path.join("index.html"), content, kind: OutputKind::Html })
    }

    fn render_taxonomy_term(
        &self,
        taxonomy: &Taxonomy,
        term: &TaxonomyTerm,
        path: &Path,
    ) -> Result<RenderedOutput> {
        let content = self.renderer().render_taxonomy_term(taxonomy, term)?;
        Ok(RenderedOutput { path: path.join("index.html"), content, kind: OutputKind::Html })
    }

    fn render_feed(&self, feed: &Feed) -> Result<Vec<RenderedOutput>> {
        let mut res = Vec::new();

        match feed {
            Feed::Site { lang } => {
                let pages: Vec<_> =
                    self.site.library.pages.values().filter(|p| p.lang == *lang).collect();
                let feed_data =
                    feeds::prepare_feed(&pages, self.site.config.feed_limit, &self.site.cache);
                for feed_filename in &self.site.config.languages[*lang].feed_filenames {
                    let feed_url = self.site.make_feed_url(None, feed_filename);
                    let input = FeedInput {
                        feed_filename,
                        lang,
                        feed_url: &feed_url,
                        pages: &feed_data.pages,
                        last_updated: feed_data.last_updated.as_deref(),
                    };
                    let content = self.renderer().render_feed(&input)?;
                    let path = if *lang == self.site.config.default_language {
                        PathBuf::from(feed_filename)
                    } else {
                        PathBuf::from(lang).join(feed_filename)
                    };
                    res.push(RenderedOutput {
                        path,
                        content,
                        kind: OutputKind::from(feed_filename.as_str()),
                    });
                }
            }
            Feed::Section { section, pages } => {
                let feed_data =
                    feeds::prepare_feed(pages, self.site.config.feed_limit, &self.site.cache);
                let base_path = PathBuf::from(&section.path[1..]);
                let cached_section = &self.site.cache.sections[&section.file.path].value;
                for feed_filename in &self.site.config.languages[&section.lang].feed_filenames {
                    let feed_url = self.site.make_feed_url(Some(&base_path), feed_filename);
                    let input = FeedInput {
                        feed_filename,
                        lang: &section.lang,
                        feed_url: &feed_url,
                        pages: &feed_data.pages,
                        last_updated: feed_data.last_updated.as_deref(),
                    };
                    let content =
                        self.renderer().render_section_feed(&input, cached_section.clone())?;
                    let path = base_path.join(feed_filename);
                    res.push(RenderedOutput {
                        path,
                        content,
                        kind: OutputKind::from(feed_filename.as_str()),
                    });
                }
            }
            Feed::Taxonomy { taxonomy, term, path } => {
                let pages: Vec<_> =
                    term.pages.iter().map(|p| self.site.library.pages.get(p).unwrap()).collect();
                let feed_data =
                    feeds::prepare_feed(&pages, self.site.config.feed_limit, &self.site.cache);
                let serialized_term = feeds::SerializedFeedTaxonomyItem::from_item(term);
                for feed_filename in &self.site.config.languages[&taxonomy.lang].feed_filenames {
                    let feed_url = self.site.make_feed_url(Some(path), feed_filename);
                    let input = FeedInput {
                        feed_filename,
                        lang: &taxonomy.lang,
                        feed_url: &feed_url,
                        pages: &feed_data.pages,
                        last_updated: feed_data.last_updated.as_deref(),
                    };
                    let content = self.renderer().render_taxonomy_feed(
                        &input,
                        &taxonomy.kind,
                        &serialized_term,
                    )?;
                    let path = path.join(feed_filename);
                    res.push(RenderedOutput {
                        path,
                        content,
                        kind: OutputKind::from(feed_filename.as_str()),
                    });
                }
            }
        }
        Ok(res)
    }

    fn render_sitemap(&self) -> Result<Vec<RenderedOutput>> {
        let mut res = Vec::new();
        let entries =
            sitemap::find_entries(&self.site.library, &self.site.taxonomies[..], &self.site.config);

        // We split sitemaps if there are too many entries.
        // 30k is IIRC an arbitrary number
        if entries.len() < 30000 {
            let content = self.renderer().render_sitemap(&entries)?;
            res.push(RenderedOutput {
                path: PathBuf::from("sitemap.xml"),
                content,
                kind: OutputKind::Xml,
            });
        } else {
            let mut sitemap_urls = Vec::new();
            for (i, chunk) in entries.chunks(30000).enumerate() {
                let content = self.renderer().render_sitemap(chunk)?;
                let file_name = format!("sitemap{}.xml", i + 1);
                res.push(RenderedOutput {
                    path: PathBuf::from(&file_name),
                    content,
                    kind: OutputKind::Xml,
                });
                let mut url = self.site.config.make_permalink(&file_name);
                url.pop();
                sitemap_urls.push(url);
            }
            let content = self.renderer().render_sitemap_index(&sitemap_urls)?;
            res.push(RenderedOutput {
                path: PathBuf::from("sitemap.xml"),
                content,
                kind: OutputKind::Xml,
            });
        }

        Ok(res)
    }

    fn render_404(&self) -> Result<RenderedOutput> {
        let content = self.renderer().render_404()?;
        Ok(RenderedOutput { path: PathBuf::from("404.html"), content, kind: OutputKind::Html })
    }

    fn render_robots(&self) -> Result<RenderedOutput> {
        let content = self.renderer().render_robots()?;
        Ok(RenderedOutput { path: PathBuf::from("robots.txt"), content, kind: OutputKind::Text })
    }

    fn execute_job(&self, job: &Job<'a>) -> Result<Vec<RenderedOutput>> {
        match job {
            Job::Alias { from, to } => Ok(vec![self.render_alias(from, to)?]),
            Job::Page(page) => Ok(vec![self.render_page(page)?]),
            Job::Section { section, path } => Ok(vec![self.render_section(section, path)?]),
            Job::Paginated { paginator_index, pager_index, path } => {
                self.render_paginated(*paginator_index, *pager_index, path)
            }
            Job::TaxonomyList { taxonomy, path } => {
                Ok(vec![self.render_taxonomy_list(taxonomy, path)?])
            }
            Job::TaxonomyTerm { taxonomy, term, path } => {
                Ok(vec![self.render_taxonomy_term(taxonomy, term, path)?])
            }
            Job::Feed(feed) => self.render_feed(feed),
            Job::Sitemap => self.render_sitemap(),
            Job::NotFound => Ok(vec![self.render_404()?]),
            Job::Robots => Ok(vec![self.render_robots()?]),
        }
    }
}
