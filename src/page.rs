/// A page, can be a blog post or a basic page
use std::cmp::Ordering;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::result::Result as StdResult;


use pulldown_cmark as cmark;
use regex::Regex;
use tera::{Tera, Context};
use serde::ser::{SerializeStruct, self};
use slug::slugify;

use errors::{Result, ResultExt};
use config::Config;
use front_matter::{FrontMatter};


lazy_static! {
    static ref PAGE_RE: Regex = Regex::new(r"^\n?\+\+\+\n((?s).*(?-s))\+\+\+\n((?s).*(?-s))$").unwrap();
}


#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Page {
    /// .md filepath, excluding the content/ bit
    #[serde(skip_serializing)]
    pub filepath: String,
    /// The name of the .md file
    #[serde(skip_serializing)]
    pub filename: String,
    /// The directories above our .md file are called sections
    /// for example a file at content/kb/solutions/blabla.md will have 2 sections:
    /// `kb` and `solutions`
    #[serde(skip_serializing)]
    pub sections: Vec<String>,
    /// The actual content of the page, in markdown
    #[serde(skip_serializing)]
    pub raw_content: String,
    /// The HTML rendered of the page
    pub content: String,
    /// The front matter meta-data
    pub meta: FrontMatter,
    /// The previous page, by date
    pub previous: Option<Box<Page>>,
    /// The next page, by date
    pub next: Option<Box<Page>>,
}


impl Page {
    pub fn new(meta: FrontMatter) -> Page {
        Page {
            filepath: "".to_string(),
            filename: "".to_string(),
            sections: vec![],
            raw_content: "".to_string(),
            content: "".to_string(),
            meta: meta,
            previous: None,
            next: None,
        }
    }

    /// Get the slug for the page.
    /// First tries to find the slug in the meta and defaults to filename otherwise
    pub fn get_slug(&self) -> String {
        if let Some(ref slug) = self.meta.slug {
            slug.to_string()
        } else {
            slugify(self.filename.clone())
        }
    }

    pub fn get_url(&self) -> String {
        if let Some(ref u) = self.meta.url {
            return u.to_string();
        }

        if !self.sections.is_empty() {
            return format!("/{}/{}", self.sections.join("/"), self.get_slug());
        }

        format!("/{}", self.get_slug())
    }

    // Parse a page given the content of the .md file
    // Files without front matter or with invalid front matter are considered
    // erroneous
    pub fn parse(filepath: &str, content: &str) -> Result<Page> {
        // 1. separate front matter from content
        if !PAGE_RE.is_match(content) {
            bail!("Couldn't find front matter in `{}`. Did you forget to add `+++`?", filepath);
        }

        // 2. extract the front matter and the content
        let caps = PAGE_RE.captures(content).unwrap();
        // caps[0] is the full match
        let front_matter = &caps[1];
        let content = &caps[2];

        // 3. create our page, parse front matter and assign all of that
        let meta = FrontMatter::parse(&front_matter)
            .chain_err(|| format!("Error when parsing front matter of file `{}`", filepath))?;

        let mut page = Page::new(meta);
        page.filepath = filepath.to_string();
        page.raw_content = content.to_string();
        page.content = {
            let mut html = String::new();
            let parser = cmark::Parser::new(&page.raw_content);
            cmark::html::push_html(&mut html, parser);
            html
        };

        // 4. Find sections
        // Pages with custom urls exists outside of sections
        if page.meta.url.is_none() {
            let path = Path::new(filepath);
            page.filename = path.file_stem().expect("Couldn't get filename").to_string_lossy().to_string();

            // find out if we have sections
            for section in path.parent().unwrap().components() {
                page.sections.push(section.as_ref().to_string_lossy().to_string());
            }
        }

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
        Page::parse(&path.strip_prefix("content").unwrap().to_string_lossy(), &content)
    }

    fn get_layout_name(&self) -> String {
        match self.meta.layout {
            Some(ref l) => l.to_string(),
            None => "page.html".to_string()
        }
    }

    pub fn render_html(&self, tera: &Tera, config: &Config) -> Result<String> {
        let tpl = self.get_layout_name();
        let mut context = Context::new();
        context.add("site", config);
        context.add("page", self);

        tera.render(&tpl, &context)
            .chain_err(|| "Error while rendering template")
    }
}

impl ser::Serialize for Page {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error> where S: ser::Serializer {
        let mut state = serializer.serialize_struct("page", 10)?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("title", &self.meta.title)?;
        state.serialize_field("description", &self.meta.description)?;
        state.serialize_field("date", &self.meta.date)?;
        state.serialize_field("slug", &self.get_slug())?;
        state.serialize_field("url", &self.get_url())?;
        state.serialize_field("tags", &self.meta.tags)?;
        state.serialize_field("draft", &self.meta.draft)?;
        state.serialize_field("category", &self.meta.category)?;
        state.serialize_field("extra", &self.meta.extra)?;
        state.end()
    }
}

impl PartialOrd for Page {
    fn partial_cmp(&self, other: &Page) -> Option<Ordering> {
        if self.meta.date.is_none() {
            println!("No self data");
            return Some(Ordering::Less);
        }

        if other.meta.date.is_none() {
            println!("No other date");
            return Some(Ordering::Greater);
        }

        let this_date = self.meta.parse_date().unwrap();
        let other_date = other.meta.parse_date().unwrap();

        if this_date > other_date {
            return Some(Ordering::Less);
        }
        if this_date < other_date {
            return Some(Ordering::Greater);
        }

        Some(Ordering::Equal)
    }
}


#[cfg(test)]
mod tests {
    use super::{Page};


    #[test]
    fn test_can_parse_a_valid_page() {
        let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::parse("post.md", content);
        assert!(res.is_ok());
        let page = res.unwrap();

        assert_eq!(page.meta.title, "Hello".to_string());
        assert_eq!(page.meta.slug.unwrap(), "hello-world".to_string());
        assert_eq!(page.raw_content, "Hello world".to_string());
        assert_eq!(page.content, "<p>Hello world</p>\n".to_string());
    }

    #[test]
    fn test_can_find_one_parent_directory() {
        let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::parse("posts/intro.md", content);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.sections, vec!["posts".to_string()]);
    }

    #[test]
    fn test_can_find_multiple_parent_directories() {
        let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::parse("posts/intro/start.md", content);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.sections, vec!["posts".to_string(), "intro".to_string()]);
    }

    #[test]
    fn test_can_make_url_from_sections_and_slug() {
        let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::parse("posts/intro/start.md", content);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.get_url(), "/posts/intro/hello-world");
    }

    #[test]
    fn test_can_make_url_from_sections_and_slug_root() {
        let content = r#"
+++
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::parse("start.md", content);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.get_url(), "/hello-world");
    }

    #[test]
    fn test_errors_on_invalid_front_matter_format() {
        let content = r#"
title = "Hello"
description = "hey there"
slug = "hello-world"
+++
Hello world"#;
        let res = Page::parse("start.md", content);
        assert!(res.is_err());
    }

    #[test]
    fn test_can_make_slug_from_non_slug_filename() {
        let content = r#"
+++
title = "Hello"
description = "hey there"
+++
Hello world"#;
        let res = Page::parse("file with space.md", content);
        assert!(res.is_ok());
        let page = res.unwrap();
        assert_eq!(page.get_slug(), "file-with-space");
    }
}
