/// A page, can be a blog post or a basic page
use std::collections::HashMap;
use std::default::Default;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;


use pulldown_cmark as cmark;
use regex::Regex;
use tera::{Tera, Value, Context};

use errors::{Result, ResultExt};
use config::Config;
use front_matter::parse_front_matter;


lazy_static! {
    static ref DELIM_RE: Regex = Regex::new(r"\+\+\+\s*\r?\n").unwrap();
}


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Page {
    // .md filepath, excluding the content/ bit
    pub filepath: String,
    // the name of the .md file
    pub filename: String,
    // the directories above our .md file are called sections
    // for example a file at content/kb/solutions/blabla.md will have 2 sections:
    // `kb` and `solutions`
    pub sections: Vec<String>,

    // <title> of the page
    pub title: String,
    // The page slug
    pub slug: String,
    // the actual content of the page
    pub raw_content: String,
    // the HTML rendered of the page
    pub content: String,

    // tags, not to be confused with categories
    pub tags: Vec<String>,
    // whether this page should be public or not
    pub is_draft: bool,
    // any extra parameter present in the front matter
    // it will be passed to the template context
    pub extra: HashMap<String, Value>,

    // the url the page appears at, overrides the slug if set
    pub url: Option<String>,
    // only one category allowed
    pub category: Option<String>,
    // optional date if we want to order pages (ie blog post)
    pub date: Option<String>,
    // optional layout, if we want to specify which tpl to render for that page
    pub layout: Option<String>,
    // description that appears when linked, e.g. on twitter
    pub description: Option<String>,
}


impl Default for Page {
    fn default() -> Page {
        Page {
            filepath: "".to_string(),
            filename: "".to_string(),
            sections: vec![],

            title: "".to_string(),
            slug: "".to_string(),
            raw_content: "".to_string(),
            content: "".to_string(),
            tags: vec![],
            is_draft: false,
            extra: HashMap::new(),

            url: None,
            category: None,
            date: None,
            layout: None,
            description: None,
        }
    }
}


impl Page {
    // Parse a page given the content of the .md file
    // Files without front matter or with invalid front matter are considered
    // erroneous
    pub fn from_str(filepath: &str, content: &str) -> Result<Page> {
        // 1. separate front matter from content
        if !DELIM_RE.is_match(content) {
            bail!("Couldn't find front matter in `{}`. Did you forget to add `+++`?", filepath);
        }

        // 2. extract the front matter and the content
        let splits: Vec<&str> = DELIM_RE.splitn(content, 2).collect();
        let front_matter = splits[0];
        let content = splits[1];

        // 2. create our page, parse front matter and assign all of that
        let mut page = Page::default();
        page.filepath = filepath.to_string();
        let path = Path::new(filepath);
        page.filename = path.file_stem().expect("Couldn't get file stem").to_string_lossy().to_string();

        // find out if we have sections
        for section in path.parent().unwrap().components() {
            page.sections.push(section.as_ref().to_string_lossy().to_string());
        }

        page.raw_content = content.to_string();
        parse_front_matter(front_matter, &mut page)
            .chain_err(|| format!("Error when parsing front matter of file `{}`", filepath))?;

        page.content = {
            let mut html = String::new();
            let parser = cmark::Parser::new(&page.raw_content);
            cmark::html::push_html(&mut html, parser);
            html
        };

        Ok(page)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Page> {
        let path = path.as_ref();

        let mut content = String::new();
        File::open(path)
            .chain_err(|| format!("Failed to open '{:?}'", path.display()))?
            .read_to_string(&mut content)?;

        // Remove the content string from name
        // Maybe get a path as an arg instead and use strip_prefix?
        Page::from_str(&path.strip_prefix("content").unwrap().to_string_lossy(), &content)
    }

    fn get_layout_name(&self) -> String {
        match self.layout {
            Some(ref l) => l.to_string(),
            None => "single.html".to_string()
        }
    }

    pub fn render_html(&mut self, tera: &Tera, config: &Config) -> Result<String> {
        let tpl = self.get_layout_name();
        let mut context = Context::new();
        context.add("site", config);
        context.add("page", self);

        tera.render(&tpl, context)
            .chain_err(|| "Error while rendering template")
    }
}


#[cfg(test)]
mod tests {
    use super::{Page};


    #[test]
    fn test_can_parse_a_valid_page() {
        let content = r#"
title = "Hello"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::from_str("post.md", content);
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.title, "Hello".to_string());
        assert_eq!(page.slug, "hello-world".to_string());
        assert_eq!(page.raw_content, "Hello world".to_string());
        assert_eq!(page.content, "<p>Hello world</p>\n".to_string());
    }

    #[test]
    fn test_can_find_one_parent_directory() {
        let content = r#"
title = "Hello"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::from_str("posts/intro.md", content);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.sections, vec!["posts".to_string()]);
    }

    #[test]
    fn test_can_find_multiplie_parent_directories() {
        let content = r#"
title = "Hello"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::from_str("posts/intro/start.md", content);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.sections, vec!["posts".to_string(), "intro".to_string()]);
    }

}
