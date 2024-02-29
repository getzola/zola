use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use libs::tera::{from_value, to_value, Function as TeraFn, Result, Value};

use crate::global_fns::helpers::search_for_file;

#[derive(Debug)]
pub struct ResizeImage {
    /// The base path of the Zola site
    base_path: PathBuf,
    theme: Option<String>,
    imageproc: Arc<Mutex<imageproc::Processor>>,
    output_path: PathBuf,
}

impl ResizeImage {
    pub fn new(
        base_path: PathBuf,
        imageproc: Arc<Mutex<imageproc::Processor>>,
        theme: Option<String>,
        output_path: PathBuf,
    ) -> Self {
        Self { base_path, imageproc, theme, output_path }
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
        let resize_op = imageproc::ResizeOperation::from_args(&op, width, height)
            .map_err(|e| format!("`resize_image`: {}", e))?;
        let mut imageproc = self.imageproc.lock().unwrap();
        let (file_path, unified_path) =
            match search_for_file(&self.base_path, &path, &self.theme, &self.output_path)
                .map_err(|e| format!("`resize_image`: {}", e))?
            {
                Some(f) => f,
                None => {
                    return Err(format!("`resize_image`: Cannot find file: {}", path).into());
                }
            };

        let response = imageproc
            .enqueue(resize_op, unified_path, file_path, &format, quality)
            .map_err(|e| format!("`resize_image`: {}", e))?;

        to_value(response).map_err(Into::into)
    }
}

#[derive(Debug)]
pub struct GetImageMetadata {
    /// The base path of the Zola site
    base_path: PathBuf,
    theme: Option<String>,
    result_cache: Arc<Mutex<HashMap<String, Value>>>,
    output_path: PathBuf,
}

impl GetImageMetadata {
    pub fn new(base_path: PathBuf, theme: Option<String>, output_path: PathBuf) -> Self {
        Self { base_path, result_cache: Arc::new(Mutex::new(HashMap::new())), theme, output_path }
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

        let (src_path, unified_path) =
            match search_for_file(&self.base_path, &path, &self.theme, &self.output_path)
                .map_err(|e| format!("`get_image_metadata`: {}", e))?
            {
                Some((f, p)) => (f, p),
                None => {
                    if allow_missing {
                        return Ok(Value::Null);
                    }
                    return Err(format!("`get_image_metadata`: Cannot find path: {}", path).into());
                }
            };

        let mut cache = self.result_cache.lock().expect("result cache lock");
        if let Some(cached_result) = cache.get(&unified_path) {
            return Ok(cached_result.clone());
        }

        let response = imageproc::read_image_metadata(src_path)
            .map_err(|e| format!("`resize_image`: {}", e))?;
        let out = to_value(response).unwrap();
        cache.insert(unified_path, out.clone());

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::{GetImageMetadata, ResizeImage};

    use std::collections::HashMap;
    use std::fs::{copy, create_dir_all};

    use config::Config;
    use libs::tera::{to_value, Function};
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use tempfile::{tempdir, TempDir};

    fn create_dir_with_image() -> TempDir {
        let dir = tempdir().unwrap();
        create_dir_all(dir.path().join("content").join("gallery")).unwrap();
        create_dir_all(dir.path().join("static")).unwrap();
        create_dir_all(dir.path().join("themes").join("name").join("static")).unwrap();
        copy("gutenberg.jpg", dir.path().join("content").join("gutenberg.jpg")).unwrap();
        copy("gutenberg.jpg", dir.path().join("content").join("gallery").join("asset.jpg"))
            .unwrap();
        copy("gutenberg.jpg", dir.path().join("static").join("gutenberg.jpg")).unwrap();
        copy(
            "gutenberg.jpg",
            dir.path().join("themes").join("name").join("static").join("in-theme.jpg"),
        )
        .unwrap();
        dir
    }

    // https://github.com/getzola/zola/issues/788
    // https://github.com/getzola/zola/issues/1035
    #[test]
    fn can_resize_image() {
        let dir = create_dir_with_image();
        let imageproc = imageproc::Processor::new(dir.path().to_path_buf(), &Config::default());

        let static_fn = ResizeImage::new(
            dir.path().to_path_buf(),
            Arc::new(Mutex::new(imageproc)),
            Some("name".to_owned()),
            PathBuf::new(),
        );
        let mut args = HashMap::new();
        args.insert("height".to_string(), to_value(40).unwrap());
        args.insert("width".to_string(), to_value(40).unwrap());

        // hashing is stable based on filepath and params so we can compare with hashes

        // 1. resizing an image in static
        args.insert("path".to_string(), to_value("static/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        let static_path = Path::new("static").join("processed_images");

        assert_eq!(
            data["static_path"],
            to_value(&format!("{}", static_path.join("gutenberg.da10f4be4f1c441e.jpg").display()))
                .unwrap()
        );
        assert_eq!(
            data["url"],
            to_value("http://a-website.com/processed_images/gutenberg.da10f4be4f1c441e.jpg")
                .unwrap()
        );

        // 2. resizing an image in content with a relative path
        args.insert("path".to_string(), to_value("content/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(
            data["static_path"],
            to_value(&format!("{}", static_path.join("gutenberg.3301b37eed389d2e.jpg").display()))
                .unwrap()
        );
        assert_eq!(
            data["url"],
            to_value("http://a-website.com/processed_images/gutenberg.3301b37eed389d2e.jpg")
                .unwrap()
        );

        // 3. resizing with an absolute path is the same as the above
        args.insert("path".to_string(), to_value("/content/gutenberg.jpg").unwrap());
        let data2 = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(data, data2);

        // 4. resizing an image in content starting with `@/` is the same as 2 and 3
        args.insert("path".to_string(), to_value("@/gutenberg.jpg").unwrap());
        let data2 = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(data, data2);

        // 5. resizing an image with a relative path not starting with static or content
        args.insert("path".to_string(), to_value("gallery/asset.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(
            data["static_path"],
            to_value(&format!("{}", static_path.join("asset.d2fde9a750b68471.jpg").display()))
                .unwrap()
        );
        assert_eq!(
            data["url"],
            to_value("http://a-website.com/processed_images/asset.d2fde9a750b68471.jpg").unwrap()
        );

        // 6. Looking up a file in the theme
        args.insert("path".to_string(), to_value("in-theme.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(
            data["static_path"],
            to_value(&format!("{}", static_path.join("in-theme.9b0d29e07d588b60.jpg").display()))
                .unwrap()
        );
        assert_eq!(
            data["url"],
            to_value("http://a-website.com/processed_images/in-theme.9b0d29e07d588b60.jpg")
                .unwrap()
        );
    }

    // TODO: consider https://github.com/getzola/zola/issues/1161
    #[test]
    fn can_get_image_metadata() {
        let dir = create_dir_with_image();

        let static_fn = GetImageMetadata::new(dir.path().to_path_buf(), None, PathBuf::new());

        // Let's test a few scenarii
        let mut args = HashMap::new();

        // 1. a call to something in `static` with a relative path
        args.insert("path".to_string(), to_value("static/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(data["height"], to_value(380).unwrap());
        assert_eq!(data["width"], to_value(300).unwrap());
        assert_eq!(data["format"], to_value("jpg").unwrap());
        assert_eq!(data["mime"], to_value("image/jpeg").unwrap());

        // 2. a call to something in `static` with an absolute path is handled currently the same as the above
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("/static/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(data["height"], to_value(380).unwrap());
        assert_eq!(data["width"], to_value(300).unwrap());
        assert_eq!(data["format"], to_value("jpg").unwrap());
        assert_eq!(data["mime"], to_value("image/jpeg").unwrap());

        // 3. a call to something in `content` with a relative path
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("content/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(data["height"], to_value(380).unwrap());
        assert_eq!(data["width"], to_value(300).unwrap());
        assert_eq!(data["format"], to_value("jpg").unwrap());
        assert_eq!(data["mime"], to_value("image/jpeg").unwrap());

        // 4. a call to something in `content` with a @/ path corresponds to
        let mut args = HashMap::new();
        args.insert("path".to_string(), to_value("@/gutenberg.jpg").unwrap());
        let data = static_fn.call(&args).unwrap().as_object().unwrap().clone();
        assert_eq!(data["height"], to_value(380).unwrap());
        assert_eq!(data["width"], to_value(300).unwrap());
        assert_eq!(data["format"], to_value("jpg").unwrap());
        assert_eq!(data["mime"], to_value("image/jpeg").unwrap());
    }
}
