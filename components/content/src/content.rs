use std::path::{Path, PathBuf};

use crate::FileInfo;
use crate::front_matter::FrontMatter;
use ahash::AHashMap;
use utils::table_of_contents::Heading;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Page {
    pub file: FileInfo,
    pub meta: FrontMatter,
    /// The list of parent sections relative paths
    pub ancestors: Vec<String>,
    /// The actual content of the page, in markdown
    pub raw_content: String,
    /// All the non-md files we found next to the .md file
    pub assets: Vec<PathBuf>,
    /// All the non-md files we found next to the .md file
    pub serialized_assets: Vec<String>,
    /// The HTML rendered of the page
    pub content: String,
    /// The slug of that page.
    /// First tries to find the slug in the meta and defaults to filename otherwise
    pub slug: String,
    /// The URL path of the page, always starting with a slash
    pub path: String,
    /// The components of the path of the page
    pub components: Vec<String>,
    /// The full URL for that page
    pub permalink: String,
    /// The summary for the article, defaults to None
    /// When <!-- more --> is found in the text, will take the content up to that part
    /// as summary
    pub summary: Option<String>,
    /// The previous page when sorting: later/later_updated/lighter/prev
    pub lower: Option<PathBuf>,
    /// The next page when sorting: earlier/earlier_updated/heavier/next
    pub higher: Option<PathBuf>,
    /// Toc made from the headings of the markdown file
    pub toc: Vec<Heading>,
    /// How many words in the raw content
    pub word_count: Option<usize>,
    /// How long would it take to read the raw content.
    /// See `get_reading_analytics` on how it is calculated
    pub reading_time: Option<usize>,
    /// The language of that page. Equal to the default lang if the user doesn't setup `languages` in config.
    /// Corresponds to the lang in the {slug}.{lang}.md file scheme
    pub lang: String,
    /// Contains all the translated version of that page
    pub translations: Vec<PathBuf>,
    /// The list of all internal links (as path to markdown file), with optional anchor fragments.
    /// We can only check the anchor after all pages have been built and their ToC compiled.
    /// The page itself should exist otherwise it would have errored before getting there.
    pub internal_links: Vec<(String, Option<String>)>,
    /// The list of all links to external webpages. They can be validated by the `link_checker`.
    pub external_links: Vec<String>,
    /// For each asset: (asset path -> (language-less md path, relative path from md file)
    pub colocated_assets: AHashMap<String, (String, String)>,
    /// Computed visibility
    pub hidden: bool,

    // Those below only apply to sections and will be empty on pages
    /// All direct pages of that section
    pub pages: Vec<PathBuf>,
    /// All direct hidden pages of that section that are hidden
    /// Not listed directly anywhere but we still render them in the queue
    pub hidden_pages: Vec<PathBuf>,
    /// All pages that cannot be sorted in this section
    pub ignored_pages: Vec<PathBuf>,
    /// All direct subsections
    pub subsections: Vec<PathBuf>,
    /// All subsections that cannot be sorted in this section
    pub ignored_subsections: Vec<PathBuf>,
}

impl Page {
    pub fn new<P: AsRef<Path>>(file_path: P, meta: FrontMatter, base_path: &Path) -> Self {
        let file_path = file_path.as_ref();

        Self { file: FileInfo::new(file_path, base_path), meta, ..Self::default() }
    }
}
