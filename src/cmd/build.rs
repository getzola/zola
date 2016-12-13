use std::fs::{create_dir, remove_dir_all};
use std::path::Path;

use glob::glob;
use tera::Tera;

use config:: Config;
use errors::{Result, ResultExt};
use page::Page;
use utils::create_file;



pub fn build(config: Config) -> Result<()> {
    if Path::new("public").exists() {
        // Delete current `public` directory so we can start fresh
        remove_dir_all("public").chain_err(|| "Couldn't delete `public` directory")?;
    }

    let tera = Tera::new("layouts/**/*").chain_err(|| "Error parsing templates")?;
    // let mut pages: Vec<Page> = vec![];

    // ok we got all the pages HTML, time to write them down to disk
    create_dir("public")?;
    let public = Path::new("public");

    // hardcoded pattern so can't error
    for entry in glob("content/**/*.md").unwrap().filter_map(|e| e.ok()) {
        let path = entry.as_path();
        let mut page = Page::from_file(&path)?;

        let mut current_path = public.clone().to_path_buf();
        for section in &page.sections {
            current_path.push(section);
            //current_path = current_path.join(section).as_path();
            if !current_path.exists() {
                println!("Creating {:?} folder", current_path);
                create_dir(&current_path)?;
                // TODO: create section index.html
                // create_file(current_path.join("index.html"), "");
            }
        }
        current_path.push(&page.filename);
        create_dir(&current_path)?;
        create_file(current_path.join("index.html"), &page.render_html(&tera, &config)?)?;
    }


    Ok(())
}
