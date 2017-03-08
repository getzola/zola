use gutenberg::errors::Result;
use gutenberg::Site;


pub fn build() -> Result<()> {
    Site::new(false)?.build()
}
