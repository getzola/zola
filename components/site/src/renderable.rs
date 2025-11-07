use errors::Result;
use libs::tera::{Context, Tera};

use config::Config;
use content::{Library, Page, Pager, Paginator, Section, Taxonomy, TaxonomyTerm};
use templates::render_redirect_template;
use utils::templates::render_template;

use super::middleware::ContentMetadata;
use super::sitemap::SitemapEntry;

/// Trait for content that can be rendered through the pipeline
pub trait Renderable {
    /// Render the content using the provided Tera instance and configuration
    fn render(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String>;

    /// Get metadata about this renderable content
    fn metadata(&self) -> ContentMetadata;
}

/// Implementation for Page
impl Renderable for Page {
    fn render(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        self.render_html(tera, config, library)
    }

    fn metadata(&self) -> ContentMetadata {
        let template_name = self.meta.template.as_deref().unwrap_or("page.html");

        ContentMetadata::builder()
            .path(self.file.path.clone())
            .components(self.components.clone())
            .template_name(template_name)
            .language(&self.lang)
            .permalink(&self.permalink)
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}

/// Implementation for Section
impl Renderable for Section {
    fn render(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        self.render_html(tera, config, library)
    }

    fn metadata(&self) -> ContentMetadata {
        let template_name = self.get_template_name();

        ContentMetadata::builder()
            .path(self.file.path.clone())
            .components(self.components.clone())
            .template_name(template_name)
            .language(&self.lang)
            .permalink(&self.permalink)
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}

/// Wrapper for a redirect page
pub struct RedirectRenderable {
    pub target_url: String,
    pub components: Vec<String>,
    pub filename: String,
}

impl Renderable for RedirectRenderable {
    fn render(&self, tera: &Tera, _config: &Config, _library: &Library) -> Result<String> {
        render_redirect_template(&self.target_url, tera)
    }

    fn metadata(&self) -> ContentMetadata {
        ContentMetadata::builder()
            .components(self.components.clone())
            .filename(&self.filename)
            .template_name("__redirect")
            .permalink(&self.target_url)
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}

/// Wrapper for a 404 page
pub struct NotFoundRenderable {
    pub lang: String,
}

impl Renderable for NotFoundRenderable {
    fn render(&self, tera: &Tera, config: &Config, _library: &Library) -> Result<String> {
        let mut context = Context::new();
        context.insert("config", &config.serialize(&self.lang));
        context.insert("lang", &self.lang);
        render_template("404.html", tera, context, &config.theme)
    }

    fn metadata(&self) -> ContentMetadata {
        ContentMetadata::builder()
            .filename("404.html")
            .template_name("404.html")
            .language(&self.lang)
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}

/// Wrapper for a single pager in a paginated section
pub struct PagerRenderable<'a> {
    pub pager: &'a Pager<'a>,
    pub paginator: &'a Paginator<'a>,
    pub components: Vec<String>,
    pub is_first: bool,
}

impl<'a> Renderable for PagerRenderable<'a> {
    fn render(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        self.paginator.render_pager(self.pager, config, tera, library)
    }

    fn metadata(&self) -> ContentMetadata {
        // Pagination uses the same template as the section/taxonomy
        // The template name is handled internally by render_pager
        ContentMetadata::builder()
            .components(self.components.clone())
            .template_name("index.html")
            .permalink(&self.pager.permalink)
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}

/// Wrapper for a taxonomy list page (all terms)
pub struct TaxonomyListRenderable<'a> {
    pub taxonomy: &'a Taxonomy,
    pub components: Vec<String>,
}

impl<'a> Renderable for TaxonomyListRenderable<'a> {
    fn render(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        self.taxonomy.render_all_terms(tera, config, library)
    }

    fn metadata(&self) -> ContentMetadata {
        ContentMetadata::builder()
            .components(self.components.clone())
            .template_name(format!("{}.html", self.taxonomy.kind.name))
            .language(&self.taxonomy.lang)
            .permalink(&self.taxonomy.permalink)
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}

/// Wrapper for a single taxonomy term page
pub struct TaxonomyTermRenderable<'a> {
    pub taxonomy: &'a Taxonomy,
    pub term: &'a TaxonomyTerm,
    pub components: Vec<String>,
}

impl<'a> Renderable for TaxonomyTermRenderable<'a> {
    fn render(&self, tera: &Tera, config: &Config, library: &Library) -> Result<String> {
        self.taxonomy.render_term(self.term, tera, config, library)
    }

    fn metadata(&self) -> ContentMetadata {
        // Template selection is handled by render_term (uses {taxname}/single.html or taxonomy_single.html)
        ContentMetadata::builder()
            .components(self.components.clone())
            .template_name(format!("{}/single.html", self.taxonomy.kind.name))
            .language(&self.taxonomy.lang)
            .permalink(&self.term.permalink)
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}

/// Wrapper for a sitemap
pub struct SitemapRenderable<'a> {
    pub entries: &'a [SitemapEntry<'a>],
    pub filename: String,
}

impl<'a> Renderable for SitemapRenderable<'a> {
    fn render(&self, tera: &Tera, config: &Config, _library: &Library) -> Result<String> {
        let mut context = Context::new();
        context.insert("entries", &self.entries);
        render_template("sitemap.xml", tera, context, &config.theme)
    }

    fn metadata(&self) -> ContentMetadata {
        ContentMetadata::builder()
            .filename(&self.filename)
            .template_name("sitemap.xml")
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}

/// Wrapper for a split sitemap index
pub struct SitemapIndexRenderable {
    pub sitemap_urls: Vec<String>,
}

impl Renderable for SitemapIndexRenderable {
    fn render(&self, tera: &Tera, config: &Config, _library: &Library) -> Result<String> {
        let mut context = Context::new();
        context.insert("sitemaps", &self.sitemap_urls);
        render_template("split_sitemap_index.xml", tera, context, &config.theme)
    }

    fn metadata(&self) -> ContentMetadata {
        ContentMetadata::builder()
            .filename("sitemap.xml")
            .template_name("split_sitemap_index.xml")
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}

/// Wrapper for robots.txt
pub struct RobotsTxtRenderable {
    pub lang: String,
}

impl Renderable for RobotsTxtRenderable {
    fn render(&self, tera: &Tera, config: &Config, _library: &Library) -> Result<String> {
        let mut context = Context::new();
        context.insert("config", &config.serialize(&self.lang));
        render_template("robots.txt", tera, context, &config.theme)
    }

    fn metadata(&self) -> ContentMetadata {
        ContentMetadata::builder()
            .filename("robots.txt")
            .template_name("robots.txt")
            .language(&self.lang)
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}

/// Wrapper for feed files (atom.xml, rss.xml, etc.)
pub struct FeedRenderable {
    pub feed_filename: String,
    pub feed_url: String,
    pub context: Context,
    pub lang: String,
    pub components: Vec<String>,
}

impl Renderable for FeedRenderable {
    fn render(&self, tera: &Tera, config: &Config, _library: &Library) -> Result<String> {
        render_template(&self.feed_filename, tera, self.context.clone(), &config.theme)
    }

    fn metadata(&self) -> ContentMetadata {
        ContentMetadata::builder()
            .components(self.components.clone())
            .filename(&self.feed_filename)
            .template_name(&self.feed_filename)
            .language(&self.lang)
            .permalink(&self.feed_url)
            .build()
            .expect("ContentMetadata builder should not fail")
    }
}
