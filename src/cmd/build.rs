use std::env;

use gutenberg::errors::Result;
use gutenberg::Site;


pub fn build() -> Result<()> {
    Site::new(env::current_dir().unwrap())?.build()
}
