use glob::glob;
use tera::Tera;

use config:: Config;
use errors::{Result, ResultExt};
use page::Page;



pub fn build(config: Config) -> Result<()> {
    let tera = Tera::new("layouts/**/*").chain_err(|| "Error parsing templates")?;
    let mut pages: Vec<Page> = vec![];

    // hardcoded pattern so can't error
    for entry in glob("content/**/*.md").unwrap().filter_map(|e| e.ok()) {
        let path = entry.as_path();
        // Remove the content string from name
        let filepath = path.to_string_lossy().replace("content/", "");
        pages.push(Page::from_file(&filepath)?);
    }

    for page in pages {
        let html = page.render_html(&tera, &config)
            .chain_err(|| format!("Failed to render '{}'", page.filepath))?;
    }

    Ok(())
}
