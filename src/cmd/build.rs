use std::env;

use gutenberg::errors::Result;
use gutenberg::Site;


pub fn build() -> Result<()> {
    let mut site = Site::new(env::current_dir().unwrap())?;
    site.load()?;
    site.build()
}
