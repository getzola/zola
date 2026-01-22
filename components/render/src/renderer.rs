use config::Config;
use content::{Library, Page};
use errors::Result;
use tera::{Context, Tera};

use crate::RenderCache;

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
        todo!("Actually render template")
    }
}
