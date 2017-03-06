use errors::Result;
use site::Site;


pub fn build() -> Result<()> {
    Site::new(false)?.build()
}
