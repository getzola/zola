use std::path::{Path, PathBuf};
use std::result::Result as StdResult;

use tera::{Tera, Context};
use serde::ser::{SerializeStruct, self};

use config::Config;
use front_matter::{FrontMatter, split_content};
use errors::{Result, ResultExt};
use utils::{read_file, find_content_components};
use page::Page;


#[derive(Clone, Debug, PartialEq)]
pub struct Section {
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
    /// The front matter meta-data
    pub meta: FrontMatter,
    /// All direct pages of that section
    pub pages: Vec<Page>,
    /// All direct subsections
    pub subsections: Vec<Section>,
}

impl Section {
    pub fn new(file_path: &Path, meta: FrontMatter) -> Section {
        Section {
            file_path: file_path.to_path_buf(),
            relative_path: "".to_string(),
            parent_path: file_path.parent().unwrap().to_path_buf(),
            components: vec![],
            path: "".to_string(),
            permalink: "".to_string(),
            meta: meta,
            pages: vec![],
            subsections: vec![],
        }
    }

    pub fn parse(file_path: &Path, content: &str, config: &Config) -> Result<Section> {
        let (meta, _) = split_content(file_path, content)?;
        let mut section = Section::new(file_path, meta);
        section.components = find_content_components(&section.file_path);
        section.path = section.components.join("/");
        section.permalink = config.make_permalink(&section.path);
        section.relative_path = format!("{}/_index.md", section.components.join("/"));


        Ok(section)
    }

    /// Read and parse a .md file into a Page struct
    pub fn from_file<P: AsRef<Path>>(path: P, config: &Config) -> Result<Section> {
        let path = path.as_ref();
        let content = read_file(path)?;

        Section::parse(path, &content, config)
    }

    /// Renders the page using the default layout, unless specified in front-matter
    pub fn render_html(&self, tera: &Tera, config: &Config) -> Result<String> {
        let tpl_name = match self.meta.template {
            Some(ref l) => l.to_string(),
            None => "section.html".to_string()
        };

        // TODO: create a helper to create context to ensure all contexts
        // have the same names
        let mut context = Context::new();
        context.add("config", config);
        context.add("section", self);
        context.add("current_url", &self.permalink);
        context.add("current_path", &self.path);

        tera.render(&tpl_name, &context)
            .chain_err(|| format!("Failed to render section '{}'", self.file_path.display()))
    }
}

impl ser::Serialize for Section {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error> where S: ser::Serializer {
        let mut state = serializer.serialize_struct("section", 6)?;
        state.serialize_field("title", &self.meta.title)?;
        state.serialize_field("description", &self.meta.description)?;
        state.serialize_field("path", &format!("/{}", self.path))?;
        state.serialize_field("permalink", &self.permalink)?;
        state.serialize_field("pages", &self.pages)?;
        state.serialize_field("subsections", &self.subsections)?;
        state.end()
    }
}
