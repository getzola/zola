use std::env;

use gutenberg::errors::Result;
use gutenberg::Site;


pub fn build(config_file: &str, clean_first: bool) -> Result<()> {
    let mut site = Site::new(env::current_dir().unwrap(), config_file)?;
    site.load()?;
    println!("-> Creating {} pages and {} sections", site.pages.len(), site.sections.len());
    if clean_first {
        site.clean()?;
    }
    site.build()
}
