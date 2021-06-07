use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tera::{from_value, to_value, Function as TeraFn, Result, Value};

use crate::global_fns::helpers::search_for_file;

#[derive(Debug)]
pub struct ResizeImage {
    /// The base path of the Zola site
    base_path: PathBuf,
    imageproc: Arc<Mutex<imageproc::Processor>>,
}

impl ResizeImage {
    pub fn new(base_path: PathBuf, imageproc: Arc<Mutex<imageproc::Processor>>) -> Self {
        Self { base_path, imageproc }
    }
}

static DEFAULT_OP: &str = "fill";
static DEFAULT_FMT: &str = "auto";

impl TeraFn for ResizeImage {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`resize_image` requires a `path` argument with a string value"
        );
        let width = optional_arg!(
            u32,
            args.get("width"),
            "`resize_image`: `width` must be a non-negative integer"
        );
        let height = optional_arg!(
            u32,
            args.get("height"),
            "`resize_image`: `height` must be a non-negative integer"
        );
        let op = optional_arg!(String, args.get("op"), "`resize_image`: `op` must be a string")
            .unwrap_or_else(|| DEFAULT_OP.to_string());

        let format =
            optional_arg!(String, args.get("format"), "`resize_image`: `format` must be a string")
                .unwrap_or_else(|| DEFAULT_FMT.to_string());

        let quality =
            optional_arg!(u8, args.get("quality"), "`resize_image`: `quality` must be a number");
        if let Some(quality) = quality {
            if quality == 0 || quality > 100 {
                return Err("`resize_image`: `quality` must be in range 1-100".to_string().into());
            }
        }

        let mut imageproc = self.imageproc.lock().unwrap();
        let file_path = match search_for_file(&self.base_path, &path) {
            Some(f) => f,
            None => {
                return Err(format!("`resize_image`: Cannot find file: {}", path).into());
            }
        };

        let response = imageproc
            .enqueue(file_path, &op, width, height, &format, quality)
            .map_err(|e| format!("`resize_image`: {}", e))?;

        to_value(response).map_err(Into::into)
    }
}

#[derive(Debug)]
pub struct GetImageMetadata {
    /// The base path of the Zola site
    base_path: PathBuf,
}

impl GetImageMetadata {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

impl TeraFn for GetImageMetadata {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`get_image_metadata` requires a `path` argument with a string value"
        );
        let allow_missing = optional_arg!(
            bool,
            args.get("allow_missing"),
            "`get_image_metadata`: `allow_missing` must be a boolean (true or false)"
        )
        .unwrap_or(false);
        let src_path = match search_for_file(&self.base_path, &path) {
            Some(f) => f,
            None => {
                if allow_missing {
                    println!("Image at path {} could not be found or loaded", path);
                    return Ok(Value::Null);
                }
                return Err(format!("`resize_image`: Cannot find path: {}", path).into());
            }
        };

        let response = imageproc::read_image_metadata(&src_path)
            .map_err(|e| format!("`resize_image`: {}", e))?;
        to_value(response).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::{GetImageMetadata, ResizeImage};

    use std::collections::HashMap;
    use std::fs::{copy, create_dir_all};

    use config::Config;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use tempfile::{tempdir, TempDir};
    use tera::{to_value, Function};

    fn create_dir_with_image() -> TempDir {
        let dir = tempdir().unwrap();
        create_dir_all(dir.path().join("content").join("gallery")).unwrap();
        create_dir_all(dir.path().join("static")).unwrap();
        copy("gutenberg.jpg", dir.path().join("content").join("gutenberg.jpg")).unwrap();
        copy("gutenberg.jpg", dir.path().join("content").join("gallery").join("asset.jpg"))
            .unwrap();
        copy("gutenberg.jpg", dir.path().join("static").join("gutenberg.jpg")).unwrap();
        dir
    }

    // https://github.com/getzola/zola/issues/788
    // https://github.com/getzola/zola/issues/1035
    #[test]
    fn can_resize_image() {
        let dir = create_dir_with_image();
        let imageproc = imageproc::Processor::new(dir.path().to_path_buf(), &Config::default());

        let static_fn = ResizeImage::new(dir.path().to_path_buf(), Arc::new(Mutex::new(imageproc)));
        let mut args = HashMap::new();
        args.insert("height".to_string(), to_value(40).unwrap());
        args.insert("width".to_string(), to_value(40).unwrap());

        // hashing is stable based on filename and params so we can compare with hashes

        // 1. resizing an image in static
        args.insert("path".to_string(), to_value("static/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        let static_path = Path::new("static").join("processed_images");

        assert_eq!(
            data["static_path"],
            to_value(&format!("{}", static_path.join("e49f5bd23ec5007c00.jpg").display())).unwrap()
        );
        assert_eq!(
            data["url"],
            to_value("http://a-website.com/processed_images/e49f5bd23ec5007c00.jpg").unwrap()
        );

        // 2. resizing an image in content with a relative path
        args.insert("path".to_string(), to_value("content/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(
            data["static_path"],
            to_value(&format!("{}", static_path.join("32454a1e0243976c00.jpg").display())).unwrap()
        );
        assert_eq!(
            data["url"],
            to_value("http://a-website.com/processed_images/32454a1e0243976c00.jpg").unwrap()
        );

        // 3. resizing with an absolute path is the same as the above
        args.insert("path".to_string(), to_value("/content/gutenberg.jpg").unwrap());
        assert_eq!(
            data["static_path"],
            to_value(&format!("{}", static_path.join("32454a1e0243976c00.jpg").display())).unwrap()
        );
        assert_eq!(
            data["url"],
            to_value("http://a-website.com/processed_images/32454a1e0243976c00.jpg").unwrap()
        );

        // 4. resizing an image in content starting with `@/` is the same as 2 and 3
        args.insert("path".to_string(), to_value("@/gutenberg.jpg").unwrap());
        assert_eq!(
            data["static_path"],
            to_value(&format!("{}", static_path.join("32454a1e0243976c00.jpg").display())).unwrap()
        );
        assert_eq!(
            data["url"],
            to_value("http://a-website.com/processed_images/32454a1e0243976c00.jpg").unwrap()
        );

        // 5. resizing an image with a relative path not starting with static or content
        args.insert("path".to_string(), to_value("gallery/asset.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(
            data["static_path"],
            to_value(&format!("{}", static_path.join("c8aaba7b0593a60b00.jpg").display())).unwrap()
        );
        assert_eq!(
            data["url"],
            to_value("http://a-website.com/processed_images/c8aaba7b0593a60b00.jpg").unwrap()
        );
    }

    // TODO: consider https://github.com/getzola/zola/issues/1161
    #[test]
    fn can_get_image_metadata() {
        let dir = create_dir_with_image();

        let static_fn = GetImageMetadata::new(dir.path().to_path_buf());

        // Let's test a few scenarii
        let mut args = HashMap::new();

        // 1. a call to something in `static` with a relative path
        args.insert("path".to_string(), to_value("static/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(data["height"], to_value(380).unwrap());
        assert_eq!(data["width"], to_value(300).unwrap());

        // 2. a call to something in `static` with an absolute path is handled currently the same as the above
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("/static/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(data["height"], to_value(380).unwrap());
        assert_eq!(data["width"], to_value(300).unwrap());

        // 3. a call to something in `content` with a relative path
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("content/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(data["height"], to_value(380).unwrap());
        assert_eq!(data["width"], to_value(300).unwrap());

        // 4. a call to something in `content` with a @/ path corresponds to
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("@/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(data["height"], to_value(380).unwrap());
        assert_eq!(data["width"], to_value(300).unwrap());
    }
}
