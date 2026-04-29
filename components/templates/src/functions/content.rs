use std::path::PathBuf;
use std::sync::Arc;

use render::RenderCache;
use tera::{Error, Function, Kwargs, State, TeraResult, Value};

#[derive(Debug)]
pub struct GetPage {
    base_path: PathBuf,
    cache: Arc<RenderCache>,
    default_lang: String,
}

impl GetPage {
    pub fn new(base_path: PathBuf, default_lang: &str, cache: Arc<RenderCache>) -> Self {
        Self { base_path: base_path.join("content"), default_lang: default_lang.to_string(), cache }
    }
}

impl Default for GetPage {
    fn default() -> Self {
        Self {
            base_path: PathBuf::new(),
            default_lang: String::new(),
            cache: Arc::new(RenderCache::default()),
        }
    }
}

impl Function<TeraResult<Value>> for GetPage {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let path: String = kwargs.must_get("path")?;
        let lang: String = kwargs
            .get::<String>("lang")?
            .or_else(|| state.get::<String>("lang").ok().flatten())
            .unwrap_or_else(|| self.default_lang.clone());

        let full_path = self.base_path.join(&path);

        let cached = self
            .cache
            .pages
            .get(&full_path)
            .ok_or_else(|| Error::message(format!("Page `{}` not found.", path)))?;

        let file_path = self
            .cache
            .pages_by_canonical
            .get(&cached.canonical)
            .and_then(|by_lang| by_lang.get(&lang))
            .ok_or_else(|| {
                Error::message(format!("Page `{}` not found for language `{}`.", path, lang))
            })?;

        self.cache
            .pages
            .get(file_path)
            .map(|c| c.value.clone())
            .ok_or_else(|| Error::message(format!("Page `{}` not found.", path)))
    }
}

#[derive(Debug)]
pub struct GetSection {
    base_path: PathBuf,
    cache: Arc<RenderCache>,
    default_lang: String,
}

impl GetSection {
    pub fn new(base_path: PathBuf, default_lang: &str, cache: Arc<RenderCache>) -> Self {
        Self { base_path: base_path.join("content"), default_lang: default_lang.to_string(), cache }
    }
}

impl Default for GetSection {
    fn default() -> Self {
        Self {
            base_path: PathBuf::new(),
            default_lang: String::new(),
            cache: Arc::new(RenderCache::default()),
        }
    }
}

impl Function<TeraResult<Value>> for GetSection {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let path: String = kwargs.must_get("path")?;
        let lang: String = kwargs
            .get::<String>("lang")?
            .or_else(|| state.get::<String>("lang").ok().flatten())
            .unwrap_or_else(|| self.default_lang.clone());

        let full_path = self.base_path.join(&path);

        let cached = self
            .cache
            .sections
            .get(&full_path)
            .ok_or_else(|| Error::message(format!("Section `{}` not found.", path)))?;

        let file_path = self
            .cache
            .sections_by_canonical
            .get(&cached.canonical)
            .and_then(|by_lang| by_lang.get(&lang))
            .ok_or_else(|| {
                Error::message(format!("Section `{}` not found for language `{}`.", path, lang))
            })?;

        self.cache
            .sections
            .get(file_path)
            .map(|c| c.value.clone())
            .ok_or_else(|| Error::message(format!("Section `{}` not found.", path)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::Config;
    use content::{FileInfo, Library, Page, Section, SortBy};
    use render::RenderCache;
    use std::path::Path;
    use tera::{Context, Kwargs, Tera};

    fn create_page(title: &str, file_path: &str, lang: &str) -> Page {
        let mut page = Page { lang: lang.to_owned(), ..Page::default() };
        page.file = FileInfo::new_page(
            Path::new(format!("/test/base/path/{}", file_path).as_str()),
            &PathBuf::new(),
        );
        page.meta.title = Some(title.to_string());
        page.meta.weight = Some(1);
        page.file.find_language("en", &["fr"]).unwrap();
        page
    }

    fn make_context_with_lang(lang: &str) -> Context {
        let mut ctx = Context::new();
        ctx.insert("lang", &lang);
        ctx
    }

    #[test]
    fn can_get_page() {
        let config = Config::default_for_test();
        let mut library = Library::new(&config);
        let pages = vec![
            ("Homepage", "content/homepage.md", "en"),
            ("Page D'Accueil", "content/homepage.fr.md", "fr"),
            ("Blog", "content/blog.md", "en"),
            ("Wiki", "content/wiki.md", "en"),
            ("Wiki", "content/wiki.fr.md", "fr"),
            ("Recipes", "content/wiki/recipes.md", "en"),
            ("Recettes", "content/wiki/recipes.fr.md", "fr"),
            ("Programming", "content/wiki/programming.md", "en"),
            ("La Programmation", "content/wiki/programming.fr.md", "fr"),
            ("Novels", "content/novels.md", "en"),
            ("Des Romans", "content/novels.fr.md", "fr"),
        ];
        for (t, f, l) in pages.clone() {
            library.insert_page(create_page(t, f, l));
        }
        let tera = Tera::default();
        let mut cache = RenderCache::new(&config);
        cache.build(&library, &[], &tera);
        let base_path = "/test/base/path".into();

        let get_page = GetPage::new(base_path, "en", Arc::new(cache));

        // Find with lang in context
        let kwargs = Kwargs::from([("path", Value::from("wiki/recipes.md"))]);
        let ctx = make_context_with_lang("fr");
        let res = get_page.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recettes");

        // Find with lang kwarg (takes precedence over context)
        let kwargs =
            Kwargs::from([("path", Value::from("wiki/recipes.md")), ("lang", Value::from("fr"))]);
        let ctx = Context::new();
        let res = get_page.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recettes");

        // Find with default lang (no lang in context)
        let kwargs = Kwargs::from([("path", Value::from("wiki/recipes.md"))]);
        let ctx = Context::new();
        let res = get_page.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recipes");

        // Find with default lang when default lang in context
        let kwargs = Kwargs::from([("path", Value::from("wiki/recipes.md"))]);
        let ctx = make_context_with_lang("en");
        let res = get_page.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recipes");

        // Error: non-existent path
        let kwargs = Kwargs::from([("path", Value::from("nonexistent.md"))]);
        let ctx = Context::new();
        let res = get_page.call(kwargs, &State::new(&ctx));
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Page `nonexistent.md` not found"));

        // Error: path exists but requested lang translation doesn't
        let kwargs = Kwargs::from([("path", Value::from("blog.md")), ("lang", Value::from("fr"))]);
        let ctx = Context::new();
        let res = get_page.call(kwargs, &State::new(&ctx));
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("not found for language `fr`"));
    }

    fn create_section(title: &str, file_path: &str, lang: &str) -> Section {
        let mut section = Section { lang: lang.to_owned(), ..Section::default() };
        section.file = FileInfo::new_section(
            Path::new(format!("/test/base/path/{}", file_path).as_str()),
            &PathBuf::new(),
        );
        section.meta.title = Some(title.to_string());
        section.meta.weight = 1;
        section.meta.transparent = false;
        section.meta.sort_by = SortBy::None;
        section.meta.page_template = Some("new_page.html".to_owned());
        section.file.find_language("en", &["fr"]).unwrap();
        section
    }

    #[test]
    fn can_get_section() {
        let config = Config::default_for_test();
        let mut library = Library::new(&config);
        let sections = vec![
            ("Homepage", "content/_index.md", "en"),
            ("Page D'Accueil", "content/_index.fr.md", "fr"),
            ("Blog", "content/blog/_index.md", "en"),
            ("Wiki", "content/wiki/_index.md", "en"),
            ("Wiki", "content/wiki/_index.fr.md", "fr"),
            ("Recipes", "content/wiki/recipes/_index.md", "en"),
            ("Recettes", "content/wiki/recipes/_index.fr.md", "fr"),
            ("Programming", "content/wiki/programming/_index.md", "en"),
            ("La Programmation", "content/wiki/programming/_index.fr.md", "fr"),
            ("Novels", "content/novels/_index.md", "en"),
            ("Des Romans", "content/novels/_index.fr.md", "fr"),
        ];
        for (t, f, l) in sections.clone() {
            library.insert_section(create_section(t, f, l));
        }
        let tera = Tera::default();
        let mut cache = RenderCache::new(&config);
        cache.build(&library, &[], &tera);
        let base_path = "/test/base/path".into();

        let get_section = GetSection::new(base_path, "en", Arc::new(cache));

        // Find with lang in context
        let kwargs = Kwargs::from([("path", Value::from("wiki/recipes/_index.md"))]);
        let ctx = make_context_with_lang("fr");
        let res = get_section.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recettes");

        // Find with lang kwarg (takes precedence over context)
        let kwargs = Kwargs::from([
            ("path", Value::from("wiki/recipes/_index.md")),
            ("lang", Value::from("fr")),
        ]);
        let ctx = Context::new();
        let res = get_section.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recettes");

        // Find with default lang (no lang in context)
        let kwargs = Kwargs::from([("path", Value::from("wiki/recipes/_index.md"))]);
        let ctx = Context::new();
        let res = get_section.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recipes");

        // Find with default lang when default lang in context
        let kwargs = Kwargs::from([("path", Value::from("wiki/recipes/_index.md"))]);
        let ctx = make_context_with_lang("en");
        let res = get_section.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recipes");

        // Error: non-existent path
        let kwargs = Kwargs::from([("path", Value::from("nonexistent/_index.md"))]);
        let ctx = Context::new();
        let res = get_section.call(kwargs, &State::new(&ctx));
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Section `nonexistent/_index.md` not found"));

        // Error: path exists but requested lang translation doesn't
        let kwargs =
            Kwargs::from([("path", Value::from("blog/_index.md")), ("lang", Value::from("fr"))]);
        let ctx = Context::new();
        let res = get_section.call(kwargs, &State::new(&ctx));
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("not found for language `fr`"));
    }
}
