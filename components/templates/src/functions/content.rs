use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use content::Library;
use tera::{Error, Function, Kwargs, State, TeraResult, Value};

fn add_lang_to_path(path: &str, lang: &str) -> TeraResult<Cow<'static, str>> {
    match path.rfind('.') {
        Some(period_offset) => {
            let prefix = path.get(0..period_offset);
            let suffix = path.get(period_offset..);
            if prefix.is_none() || suffix.is_none() {
                Err(Error::message(format!("Error adding language code to {}", path)))
            } else {
                Ok(Cow::Owned(format!("{}.{}{}", prefix.unwrap(), lang, suffix.unwrap())))
            }
        }
        None => Ok(Cow::Owned(format!("{}.{}", path, lang))),
    }
}

// TODO: fix me properly
fn get_path_with_lang<'a>(
    path: &'a str,
    lang: Option<&str>,
    default_lang: &str,
    supported_languages: &[String],
) -> TeraResult<Cow<'a, str>> {
    // Check if path already contains a language suffix
    if let Some(stem) = path.strip_suffix(".md") {
        for supported_lang in supported_languages {
            if stem.ends_with(&format!(".{}", supported_lang)) {
                // Path already has a language suffix, use as-is
                return Ok(Cow::Borrowed(path));
            }
        }
    }

    if supported_languages.contains(&default_lang.to_string()) {
        lang.as_ref().map_or_else(
            || Ok(Cow::Borrowed(path)),
            |&lang_code| match default_lang == lang_code {
                true => Ok(Cow::Borrowed(path)),
                false => {
                    // Need to convert Cow<'static, str> to Cow<'a, str>
                    add_lang_to_path(path, lang_code).map(|c| Cow::Owned(c.into_owned()))
                }
            },
        )
    } else {
        Err(Error::message(format!("Unsupported language {}", default_lang)))
    }
}

#[derive(Debug)]
pub struct GetPage {
    base_path: PathBuf,
    default_lang: String,
    supported_languages: Arc<Vec<String>>,
    library: Arc<Library>,
}

impl GetPage {
    pub fn new(
        base_path: PathBuf,
        default_lang: &str,
        supported_languages: Arc<Vec<String>>,
        library: Arc<Library>,
    ) -> Self {
        Self {
            base_path: base_path.join("content"),
            default_lang: default_lang.to_string(),
            supported_languages,
            library,
        }
    }
}

impl Default for GetPage {
    fn default() -> Self {
        Self {
            base_path: PathBuf::new(),
            default_lang: String::new(),
            supported_languages: Arc::new(Vec::new()),
            library: Arc::new(Library::default()),
        }
    }
}

impl Function<TeraResult<Value>> for GetPage {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let path: String = kwargs.must_get("path")?;
        let lang: Option<String> = state.get("lang")?;

        get_path_with_lang(&path, lang.as_deref(), &self.default_lang, &self.supported_languages)
            .and_then(|path_with_lang| {
                let full_path = self.base_path.join(path_with_lang.as_ref());

                match self.library.serialized_pages.get(&full_path) {
                    Some(p) => Ok(p.clone()),
                    None => match lang {
                        Some(lang_code) => Err(Error::message(format!(
                            "Page `{}` not found for language `{}`.",
                            path, lang_code
                        ))),
                        None => Err(Error::message(format!("Page `{}` not found.", path))),
                    },
                }
            })
    }
}

#[derive(Debug)]
pub struct GetSection {
    base_path: PathBuf,
    default_lang: String,
    supported_languages: Arc<Vec<String>>,
    library: Arc<Library>,
}

impl GetSection {
    pub fn new(
        base_path: PathBuf,
        default_lang: &str,
        supported_languages: Arc<Vec<String>>,
        library: Arc<Library>,
    ) -> Self {
        Self {
            base_path: base_path.join("content"),
            default_lang: default_lang.to_string(),
            supported_languages,
            library,
        }
    }
}

impl Default for GetSection {
    fn default() -> Self {
        Self {
            base_path: PathBuf::new(),
            default_lang: String::new(),
            supported_languages: Arc::new(Vec::new()),
            library: Arc::new(Library::default()),
        }
    }
}

impl Function<TeraResult<Value>> for GetSection {
    fn call(&self, kwargs: Kwargs, state: &State) -> TeraResult<Value> {
        let path: String = kwargs.must_get("path")?;
        let lang: Option<String> = state.get("lang")?;

        get_path_with_lang(&path, lang.as_deref(), &self.default_lang, &self.supported_languages)
            .and_then(|path_with_lang| {
                let full_path = self.base_path.join(path_with_lang.as_ref());

                match self.library.serialized_sections.get(&full_path) {
                    Some(s) => Ok(s.clone()),
                    None => match lang {
                        Some(lang_code) => Err(Error::message(format!(
                            "Section `{}` not found for language `{}`.",
                            path, lang_code
                        ))),
                        None => Err(Error::message(format!("Section `{}` not found.", path))),
                    },
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use content::{FileInfo, Page, Section, SortBy};
    use std::path::Path;
    use tera::{Context, Kwargs};

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
        let mut library = Library::default();
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
        library.pre_render();
        let base_path = "/test/base/path".into();
        let lang_list = vec!["en".to_string(), "fr".to_string()];

        let get_page = GetPage::new(base_path, "en", Arc::new(lang_list), Arc::new(library));

        // Find with lang in context
        let kwargs = Kwargs::from([("path", Value::from("wiki/recipes.md"))]);
        let ctx = make_context_with_lang("fr");
        let res = get_page.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recettes");

        // Find with lang in path for legacy support
        let kwargs = Kwargs::from([("path", Value::from("wiki/recipes.fr.md"))]);
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
        let mut library = Library::default();
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
        library.pre_render();
        let base_path = "/test/base/path".into();
        let lang_list = vec!["en".to_string(), "fr".to_string()];

        let get_section = GetSection::new(base_path, "en", Arc::new(lang_list), Arc::new(library));

        // Find with lang in context
        let kwargs = Kwargs::from([("path", Value::from("wiki/recipes/_index.md"))]);
        let ctx = make_context_with_lang("fr");
        let res = get_section.call(kwargs, &State::new(&ctx)).unwrap();
        let res_obj = res.as_map().unwrap();
        assert_eq!(res_obj.get(&"title".into()).unwrap().as_str().unwrap(), "Recettes");

        // Find with lang in path for legacy support
        let kwargs = Kwargs::from([("path", Value::from("wiki/recipes/_index.fr.md"))]);
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
    }
}
