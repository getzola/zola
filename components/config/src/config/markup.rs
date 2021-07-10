use std::path::Path;

use serde_derive::{Deserialize, Serialize};
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder};

use errors::Result;

pub const DEFAULT_HIGHLIGHT_THEME: &str = "base16-ocean-dark";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeCss {
    /// Which theme are we generating the CSS from
    pub theme: String,
    /// In which file are we going to output the CSS
    pub filename: String,
}

impl Default for ThemeCss {
    fn default() -> ThemeCss {
        ThemeCss { theme: String::new(), filename: String::new() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Markdown {
    /// Whether to highlight all code blocks found in markdown files. Defaults to false
    pub highlight_code: bool,
    /// Which themes to use for code highlighting. See Readme for supported themes
    /// Defaults to "base16-ocean-dark"
    pub highlight_theme: String,
    /// Generate CSS files for Themes out of syntect
    pub highlight_themes_css: Vec<ThemeCss>,
    /// Whether to render emoji aliases (e.g.: :smile: => 😄) in the markdown files
    pub render_emoji: bool,
    /// Whether external links are to be opened in a new tab
    /// If this is true, a `rel="noopener"` will always automatically be added for security reasons
    pub external_links_target_blank: bool,
    /// Whether to set rel="nofollow" for all external links
    pub external_links_no_follow: bool,
    /// Whether to set rel="noreferrer" for all external links
    pub external_links_no_referrer: bool,
    /// Whether smart punctuation is enabled (changing quotes, dashes, dots etc in their typographic form)
    pub smart_punctuation: bool,

    /// A list of directories to search for additional `.sublime-syntax` files in.
    pub extra_syntaxes: Vec<String>,
    /// The compiled extra syntaxes into a syntax set
    #[serde(skip_serializing, skip_deserializing)] // not a typo, 2 are need
    pub extra_syntax_set: Option<SyntaxSet>,
}

impl Markdown {
    /// Attempt to load any extra syntax found in the extra syntaxes of the config
    pub fn load_extra_syntaxes(&mut self, base_path: &Path) -> Result<()> {
        if self.extra_syntaxes.is_empty() {
            return Ok(());
        }

        let mut ss = SyntaxSetBuilder::new();
        for dir in &self.extra_syntaxes {
            ss.add_from_folder(base_path.join(dir), true)?;
        }
        self.extra_syntax_set = Some(ss.build());

        Ok(())
    }

    pub fn has_external_link_tweaks(&self) -> bool {
        self.external_links_target_blank
            || self.external_links_no_follow
            || self.external_links_no_referrer
    }

    pub fn construct_external_link_tag(&self, url: &str, title: &str) -> String {
        let mut rel_opts = Vec::new();
        let mut target = "".to_owned();
        let title = if title.is_empty() { "".to_owned() } else { format!("title=\"{}\" ", title) };

        if self.external_links_target_blank {
            // Security risk otherwise
            rel_opts.push("noopener");
            target = "target=\"_blank\" ".to_owned();
        }
        if self.external_links_no_follow {
            rel_opts.push("nofollow");
        }
        if self.external_links_no_referrer {
            rel_opts.push("noreferrer");
        }
        let rel = if rel_opts.is_empty() {
            "".to_owned()
        } else {
            format!("rel=\"{}\" ", rel_opts.join(" "))
        };

        format!("<a {}{}{}href=\"{}\">", rel, target, title, url)
    }
}

impl Default for Markdown {
    fn default() -> Markdown {
        Markdown {
            highlight_code: false,
            highlight_theme: DEFAULT_HIGHLIGHT_THEME.to_owned(),
            highlight_themes_css: Vec::new(),
            render_emoji: false,
            external_links_target_blank: false,
            external_links_no_follow: false,
            external_links_no_referrer: false,
            smart_punctuation: false,
            extra_syntaxes: Vec::new(),
            extra_syntax_set: None,
        }
    }
}
