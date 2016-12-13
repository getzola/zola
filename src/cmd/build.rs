use std::collections::HashMap;
use std::fs::{create_dir, remove_dir_all};
use std::path::Path;

use glob::glob;
use tera::{Tera, Context};

use config:: Config;
use errors::{Result, ResultExt};
use page::{Page, order_pages};
use utils::create_file;



pub fn build(config: Config) -> Result<()> {
    if Path::new("public").exists() {
        // Delete current `public` directory so we can start fresh
        remove_dir_all("public").chain_err(|| "Couldn't delete `public` directory")?;
    }

    let tera = Tera::new("layouts/**/*").chain_err(|| "Error parsing templates")?;

    // ok we got all the pages HTML, time to write them down to disk
    create_dir("public")?;
    let public = Path::new("public");
    let mut pages: Vec<Page> = vec![];
    let mut sections: HashMap<String, Vec<Page>> = HashMap::new();

    // First step: do all the articles and group article by sections
    // hardcoded pattern so can't error
    for entry in glob("content/**/*.md").unwrap().filter_map(|e| e.ok()) {
        let path = entry.as_path();
        let mut page = Page::from_file(&path)?;

        let mut current_path = public.clone().to_path_buf();

        for section in &page.sections {
            current_path.push(section);

            if !current_path.exists() {
                create_dir(&current_path)?;
            }

            let str_path = current_path.as_path().to_string_lossy().to_string();
            if sections.contains_key(&str_path) {
                sections.get_mut(&str_path).unwrap().push(page.clone());
            } else {
                sections.insert(str_path, vec![page.clone()]);
            }

        }

        current_path.push(&page.filename);
        create_dir(&current_path)?;
        create_file(current_path.join("index.html"), &page.render_html(&tera, &config)?)?;
        pages.push(page);
    }

    for (section, pages) in sections {
        render_section_index(section, pages, &tera, &config)?;
    }


    Ok(())
}


fn render_section_index(section: String, pages: Vec<Page>, tera: &Tera, config: &Config) -> Result<()> {
    let path = Path::new(&section);
    let mut context = Context::new();
    context.add("pages", &order_pages(pages));
    context.add("config", &config);

    let section_name = match path.components().into_iter().last() {
        Some(s) => s.as_ref().to_string_lossy().to_string(),
        None => bail!("Couldn't find a section name in {:?}", path.display())
    };

    create_file(path.join("index.html"), &tera.render(&format!("{}.html", section_name), context)?)
}
