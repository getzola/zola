use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use tera::{Tera, Context};
use serde::ser::{SerializeStruct, self};

use config::Config;
use front_matter::{SectionFrontMatter, split_section_content};
use errors::{Result, ResultExt};
use utils::{read_file, find_content_components};
use markdown::markdown_to_html;
use page::{Page};


#[derive(Clone, Debug, PartialEq)]
pub struct Section {
    /// The front matter meta-data
    pub meta: SectionFrontMatter,
    /// The _index.md full path
    pub file_path: PathBuf,
    /// The .md path, starting from the content directory, with / slashes
    pub relative_path: String,
    /// Path of the directory containing the _index.md file
    pub parent_path: PathBuf,
    /// The folder names from `content` to this section file
    pub components: Vec<String>,
    /// The URL path of the page
    pub path: String,
    /// The full URL for that page
    pub permalink: String,
    /// The actual content of the page, in markdown
    pub raw_content: String,
    /// The HTML rendered of the page
    pub content: String,
    /// All direct pages of that section
    pub pages: Vec<Page>,
    /// All pages that cannot be sorted in this section
    pub ignored_pages: Vec<Page>,
    /// All direct subsections
    pub subsections: Vec<Section>,
}

impl Section {
    pub fn new<P: AsRef<Path>>(file_path: P, meta: SectionFrontMatter) -> Section {
        let file_path = file_path.as_ref();

        Section {
            meta: meta,
            file_path: file_path.to_path_buf(),
            relative_path: "".to_string(),
            parent_path: file_path.parent().unwrap().to_path_buf(),
            components: vec![],
            path: "".to_string(),
            permalink: "".to_string(),
            raw_content: "".to_string(),
            content: "".to_string(),
            pages: vec![],
            ignored_pages: vec![],
            subsections: vec![],
        }
    }

    pub fn parse(file_path: &Path, content: &str, config: &Config) -> Result<Section> {
        let (meta, content) = split_section_content(file_path, content)?;
        let mut section = Section::new(file_path, meta);
        section.raw_content = content.clone();
        section.components = find_content_components(&section.file_path);
        section.path = section.components.join("/");
        section.permalink = config.make_permalink(&section.path);
        if section.components.is_empty() {
            // the index one
            section.relative_path = "_index.md".to_string();
        } else {
            section.relative_path = format!("{}/_index.md", section.components.join("/"));
        }

        Ok(section)
    }

    /// Read and parse a .md file into a Page struct
    pub fn from_file<P: AsRef<Path>>(path: P, config: &Config) -> Result<Section> {
        let path = path.as_ref();
        let content = read_file(path)?;

        Section::parse(path, &content, config)
    }

    pub fn get_template_name(&self) -> String {
        match self.meta.template {
            Some(ref l) => l.to_string(),
            None => {
                if self.is_index() {
                    return "index.html".to_string();
                }
                "section.html".to_string()
            },
        }
    }

    /// We need access to all pages url to render links relative to content
    /// so that can't happen at the same time as parsing
    pub fn render_markdown(&mut self, permalinks: &HashMap<String, String>, tera: &Tera, config: &Config) -> Result<()> {
        self.content = markdown_to_html(&self.raw_content, permalinks, tera, config)?;
        Ok(())
    }

    /// Renders the page using the default layout, unless specified in front-matter
    pub fn render_html(&self, sections: HashMap<String, Section>, tera: &Tera, config: &Config) -> Result<String> {
        let tpl_name = self.get_template_name();

        let mut context = Context::new();
        context.add("config", config);
        context.add("section", self);
        context.add("current_url", &self.permalink);
        context.add("current_path", &self.path);
        if self.is_index() {
            context.add("sections", &sections);
        }

        tera.render(&tpl_name, &context)
            .chain_err(|| format!("Failed to render section '{}'", self.file_path.display()))
    }

    /// Is this the index section?
    pub fn is_index(&self) -> bool {
        self.components.is_empty()
    }

    /// Returns all the paths for the pages belonging to that section
    pub fn all_pages_path(&self) -> Vec<PathBuf> {
        let mut paths = vec![];
        paths.extend(self.pages.iter().map(|p| p.file_path.clone()));
        paths.extend(self.ignored_pages.iter().map(|p| p.file_path.clone()));
        paths
    }
}

impl ser::Serialize for Section {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error> where S: ser::Serializer {
        let mut state = serializer.serialize_struct("section", 7)?;
        state.serialize_field("content", &self.content)?;
        state.serialize_field("title", &self.meta.title)?;
        state.serialize_field("description", &self.meta.description)?;
        state.serialize_field("path", &format!("/{}", self.path))?;
        state.serialize_field("permalink", &self.permalink)?;
        state.serialize_field("pages", &self.pages)?;
        state.serialize_field("subsections", &self.subsections)?;
        state.end()
    }
}

impl Default for Section {
    /// Used to create a default index section if there is no _index.md in the root content directory
    fn default() -> Section {
        Section {
            meta: SectionFrontMatter::default(),
            file_path: PathBuf::new(),
            relative_path: "".to_string(),
            parent_path: PathBuf::new(),
            components: vec![],
            path: "".to_string(),
            permalink: "".to_string(),
            raw_content: "".to_string(),
            content: "".to_string(),
            pages: vec![],
            ignored_pages: vec![],
            subsections: vec![],
        }
    }
}
