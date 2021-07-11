use std::{path::Path, sync::Arc};

use serde_derive::{Deserialize, Serialize};
use syntect::{
    highlighting::{Theme, ThemeSet},
    html::css_for_theme_with_class_style,
    parsing::{SyntaxSet, SyntaxSetBuilder},
};

use errors::{bail, Result};

use crate::highlighting::{CLASS_STYLE, THEME_SET};

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
    /// A list of directories to search for additional `.sublime-syntax` and `.tmTheme` files in.
    pub extra_syntaxes_and_themes: Vec<String>,
    /// The compiled extra syntaxes into a syntax set
    #[serde(skip_serializing, skip_deserializing)] // not a typo, 2 are need
    pub extra_syntax_set: Option<SyntaxSet>,
    /// The compiled extra themes into a theme set
    #[serde(skip_serializing, skip_deserializing)] // not a typo, 2 are need
    pub extra_theme_set: Arc<Option<ThemeSet>>,
}

impl Markdown {
    /// Gets the configured highlight theme from the THEME_SET or the config's extra_theme_set
    /// Returns None if the configured highlighting theme is set to use css
    pub fn get_highlight_theme(&self) -> Option<&Theme> {
        if self.highlight_theme == "css" {
            None
        } else {
            Some(self.get_highlight_theme_by_name(&self.highlight_theme))
        }
    }

    /// Gets an arbitrary theme from the THEME_SET or the extra_theme_set
    pub fn get_highlight_theme_by_name<'config>(&'config self, theme_name: &str) -> &'config Theme {
        (*self.extra_theme_set)
            .as_ref()
            .and_then(|ts| ts.themes.get(theme_name))
            .unwrap_or_else(|| &THEME_SET.themes[theme_name])
    }

    /// Attempt to load any extra syntaxes and themes found in the extra_syntaxes_and_themes folders
    pub fn load_extra_syntaxes_and_highlight_themes(
        &self,
        base_path: &Path,
    ) -> Result<(Option<SyntaxSet>, Option<ThemeSet>)> {
        if self.extra_syntaxes_and_themes.is_empty() {
            return Ok((None, None));
        }

        let mut ss = SyntaxSetBuilder::new();
        let mut ts = ThemeSet::new();
        for dir in &self.extra_syntaxes_and_themes {
            ss.add_from_folder(base_path.join(dir), true)?;
            ts.add_from_folder(base_path.join(dir))?;
        }
        let ss = ss.build();

        Ok((
            if ss.syntaxes().is_empty() { None } else { Some(ss) },
            if ts.themes.is_empty() { None } else { Some(ts) },
        ))
    }

    pub fn export_theme_css(&self, theme_name: &str) -> String {
        let theme = self.get_highlight_theme_by_name(theme_name);
        css_for_theme_with_class_style(theme, CLASS_STYLE)
    }

    // Initialise static once cells: EXTRA_SYNTAX_SET and EXTRA_THEME_SET
    // They should only be initialised once, when building a new site the existing values are reused
    pub fn init_extra_syntaxes_and_highlight_themes(&mut self, path: &Path) -> Result<()> {
        if self.highlight_theme == "css" {
            return Ok(());
        }

        let (loaded_extra_syntaxes, loaded_extra_highlight_themes) =
            self.load_extra_syntaxes_and_highlight_themes(path)?;

        if let Some(extra_syntax_set) = loaded_extra_syntaxes {
            self.extra_syntax_set = Some(extra_syntax_set);
        }
        if let Some(extra_theme_set) = loaded_extra_highlight_themes {
            self.extra_theme_set = Arc::new(Some(extra_theme_set));
        }

        // validate that the chosen highlight_theme exists in the loaded highlight theme sets
        if !THEME_SET.themes.contains_key(&self.highlight_theme) {
            if let Some(extra) = &*self.extra_theme_set {
                if !extra.themes.contains_key(&self.highlight_theme) {
                    bail!(
                        "Highlight theme {} not found in the extra theme set",
                        self.highlight_theme
                    )
                }
            } else {
                bail!("Highlight theme {} not available.\n\
                You can load custom themes by configuring `extra_syntaxes_and_themes` to include a list of folders containing '.tmTheme' files", self.highlight_theme)
            }
        }

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
            extra_syntaxes_and_themes: vec![],
            extra_syntax_set: None,
            extra_theme_set: Arc::new(None),
        }
    }
}
