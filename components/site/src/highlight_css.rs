use std::path::{Path, PathBuf};

use errors::Result;
use utils::fs::ensure_directory_exists;

use config::highlighting::THEME_SET;
use config::ThemeCss;
use std::fs::File;
use std::io::{BufWriter, Write};
use syntect::html::{css_for_theme_with_class_style, ClassStyle};

pub fn generate_highlighting_themes(
    output_path: &PathBuf,
    highlighting_themes_css: &[ThemeCss],
) -> Result<()> {
    ensure_directory_exists(output_path)?;
    for css_theme in highlighting_themes_css {
        let theme = &THEME_SET.themes[&css_theme.theme];
        let css_file = File::create(Path::new(&output_path.join(&css_theme.filename)))?;
        let mut css_writer = BufWriter::new(&css_file);

        let css = css_for_theme_with_class_style(theme, ClassStyle::Spaced);
        writeln!(css_writer, "{}", css)?;
    }
    Ok(())
}
