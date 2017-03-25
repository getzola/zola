use std::env;

use gutenberg::errors::Result;
use gutenberg::Site;


pub fn build(config_file: &str) -> Result<()> {
    let mut site = Site::new(env::current_dir().unwrap(), config_file)?;
    site.load()?;
    site.build()
}
