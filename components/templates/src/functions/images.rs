use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tera::{Error, Function, Kwargs, State, TeraResult, Value};

use crate::helpers::search_for_file;

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

impl Default for ResizeImage {
    fn default() -> Self {
        Self {
            base_path: PathBuf::new(),
            theme: None,
            imageproc: Arc::new(Mutex::new(imageproc::Processor::new(
                PathBuf::new(),
                &config::Config::default(),
            ))),
            output_path: PathBuf::new(),
        }
    }
}

const DEFAULT_OP: &str = "fill";
const DEFAULT_FMT: &str = "auto";

impl Function<TeraResult<Value>> for ResizeImage {
    fn call(&self, kwargs: Kwargs, _state: &State) -> TeraResult<Value> {
        let path: String = kwargs.must_get("path")?;
        let width: Option<u32> = kwargs.get("width")?;
        let height: Option<u32> = kwargs.get("height")?;
        let op: String = kwargs.get("op")?.unwrap_or_else(|| DEFAULT_OP.to_string());
        let format: String = kwargs.get("format")?.unwrap_or_else(|| DEFAULT_FMT.to_string());
        let quality: Option<u8> = kwargs.get("quality")?;
        let speed: Option<u8> = kwargs.get("speed")?;

        let resize_op = imageproc::ResizeOperation::from_args(&op, width, height)
            .map_err(|e| Error::message(format!("`resize_image`: {}", e)))?;

        let mut imageproc = self.imageproc.lock().unwrap();
        let file_path =
            match search_for_file(&self.base_path, &path, &self.theme, &self.output_path)
                .map_err(|e| Error::message(format!("`resize_image`: {}", e)))?
            {
                Some(f) => f,
                None => {
                    return Err(Error::message(format!(
                        "`resize_image`: Cannot find file: {}",
                        path
                    )));
                }
            };

        let response = imageproc
            .enqueue(resize_op, file_path, &format, quality, speed)
            .map_err(|e| Error::message(format!("`resize_image`: {}", e)))?;

        Ok(Value::from_serializable(&response))
    }
}

#[derive(Debug)]
pub struct GetImageMetadata {
    /// The base path of the Zola site
    base_path: PathBuf,
    theme: Option<String>,
    result_cache: Arc<Mutex<HashMap<PathBuf, Value>>>,
    output_path: PathBuf,
}

impl GetImageMetadata {
    pub fn new(base_path: PathBuf, theme: Option<String>, output_path: PathBuf) -> Self {
        Self { base_path, result_cache: Arc::new(Mutex::new(HashMap::new())), theme, output_path }
    }
}

impl Default for GetImageMetadata {
    fn default() -> Self {
        Self {
            base_path: PathBuf::new(),
            theme: None,
            result_cache: Arc::new(Mutex::new(HashMap::new())),
            output_path: PathBuf::new(),
        }
    }
}

impl Function<TeraResult<Value>> for GetImageMetadata {
    fn call(&self, kwargs: Kwargs, _state: &State) -> TeraResult<Value> {
        let path: String = kwargs.must_get("path")?;
        let allow_missing: bool = kwargs.get("allow_missing")?.unwrap_or(false);

        let src_path = match search_for_file(&self.base_path, &path, &self.theme, &self.output_path)
            .map_err(|e| Error::message(format!("`get_image_metadata`: {}", e)))?
        {
            Some(f) => f,
            None => {
                if allow_missing {
                    return Ok(Value::none());
                }
                return Err(Error::message(format!(
                    "`get_image_metadata`: Cannot find path: {}",
                    path
                )));
            }
        };

        let mut cache = self.result_cache.lock().expect("result cache lock");
        if let Some(cached_result) = cache.get(&src_path) {
            return Ok(cached_result.clone());
        }

        let response = imageproc::read_image_metadata(&src_path)
            .map_err(|e| Error::message(format!("`get_image_metadata`: {}", e)))?;
        let out = Value::from_serializable(&response);
        cache.insert(src_path, out.clone());

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::{GetImageMetadata, ResizeImage};

    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    use config::Config;
    use fs_err as fs;
    use tempfile::{TempDir, tempdir};
    use tera::{Context, Function, Kwargs, State, Value};

    fn create_dir_with_image() -> TempDir {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("content").join("gallery")).unwrap();
        fs::create_dir_all(dir.path().join("static")).unwrap();
        fs::create_dir_all(dir.path().join("themes").join("name").join("static")).unwrap();
        fs::copy("gutenberg.jpg", dir.path().join("content").join("gutenberg.jpg")).unwrap();
        fs::copy("gutenberg.jpg", dir.path().join("content").join("gallery").join("asset.jpg"))
            .unwrap();
        fs::copy("gutenberg.jpg", dir.path().join("static").join("gutenberg.jpg")).unwrap();
        fs::copy(
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
        let ctx = Context::new();
        let state = State::new(&ctx);

        // hashing is stable based on filepath and params so we can compare with hashes

        // 1. resizing an image in static
        let kwargs = Kwargs::from([
            ("height", Value::from(40)),
            ("width", Value::from(40)),
            ("path", Value::from("static/gutenberg.jpg")),
        ]);
        let data = static_fn.call(kwargs, &state).unwrap();
        let data = data.as_map().unwrap();
        let static_path = Path::new("static").join("processed_images");

        assert_eq!(
            data.get(&"static_path".into()).unwrap(),
            &Value::from(format!(
                "{}",
                static_path.join("gutenberg.9786ef7a62f75bc4.jpg").display()
            ))
        );
        assert_eq!(
            data.get(&"url".into()).unwrap(),
            &Value::from("http://a-website.com/processed_images/gutenberg.9786ef7a62f75bc4.jpg")
        );

        // 2. resizing an image in content with a relative path
        let kwargs = Kwargs::from([
            ("height", Value::from(40)),
            ("width", Value::from(40)),
            ("path", Value::from("content/gutenberg.jpg")),
        ]);
        let data = static_fn.call(kwargs, &state).unwrap();
        let data = data.as_map().unwrap();
        assert_eq!(
            data.get(&"static_path".into()).unwrap(),
            &Value::from(format!(
                "{}",
                static_path.join("gutenberg.9786ef7a62f75bc4.jpg").display()
            ))
        );
        assert_eq!(
            data.get(&"url".into()).unwrap(),
            &Value::from("http://a-website.com/processed_images/gutenberg.9786ef7a62f75bc4.jpg")
        );

        // 3. resizing with an absolute path is the same as the above
        let kwargs = Kwargs::from([
            ("height", Value::from(40)),
            ("width", Value::from(40)),
            ("path", Value::from("/content/gutenberg.jpg")),
        ]);
        let data2 = static_fn.call(kwargs, &state).unwrap();
        assert_eq!(data2.as_map().unwrap(), data);

        // 4. resizing an image in content starting with `@/` is the same as 2 and 3
        let kwargs = Kwargs::from([
            ("height", Value::from(40)),
            ("width", Value::from(40)),
            ("path", Value::from("@/gutenberg.jpg")),
        ]);
        let data2 = static_fn.call(kwargs, &state).unwrap();
        assert_eq!(data2.as_map().unwrap(), data);

        // 5. resizing an image with a relative path not starting with static or content
        let kwargs = Kwargs::from([
            ("height", Value::from(40)),
            ("width", Value::from(40)),
            ("path", Value::from("gallery/asset.jpg")),
        ]);
        let data = static_fn.call(kwargs, &state).unwrap();
        let data = data.as_map().unwrap();
        assert_eq!(
            data.get(&"static_path".into()).unwrap(),
            &Value::from(format!("{}", static_path.join("asset.9786ef7a62f75bc4.jpg").display()))
        );
        assert_eq!(
            data.get(&"url".into()).unwrap(),
            &Value::from("http://a-website.com/processed_images/asset.9786ef7a62f75bc4.jpg")
        );

        // 6. Looking up a file in the theme
        let kwargs = Kwargs::from([
            ("height", Value::from(40)),
            ("width", Value::from(40)),
            ("path", Value::from("in-theme.jpg")),
        ]);
        let data = static_fn.call(kwargs, &state).unwrap();
        let data = data.as_map().unwrap();
        assert_eq!(
            data.get(&"static_path".into()).unwrap(),
            &Value::from(format!(
                "{}",
                static_path.join("in-theme.9786ef7a62f75bc4.jpg").display()
            ))
        );
        assert_eq!(
            data.get(&"url".into()).unwrap(),
            &Value::from("http://a-website.com/processed_images/in-theme.9786ef7a62f75bc4.jpg")
        );
    }

    // TODO: consider https://github.com/getzola/zola/issues/1161
    #[test]
    fn can_get_image_metadata() {
        let dir = create_dir_with_image();

        let static_fn = GetImageMetadata::new(dir.path().to_path_buf(), None, PathBuf::new());
        let ctx = Context::new();
        let state = State::new(&ctx);

        // Let's test a few scenarii

        // 1. a call to something in `static` with a relative path
        let kwargs = Kwargs::from([("path", Value::from("static/gutenberg.jpg"))]);
        let data = static_fn.call(kwargs, &state).unwrap();
        let data = data.as_map().unwrap();
        assert_eq!(data.get(&"height".into()).unwrap(), &Value::from(380));
        assert_eq!(data.get(&"width".into()).unwrap(), &Value::from(300));
        assert_eq!(data.get(&"format".into()).unwrap(), &Value::from("jpg"));
        assert_eq!(data.get(&"mime".into()).unwrap(), &Value::from("image/jpeg"));

        // 2. a call to something in `static` with an absolute path is handled currently the same as the above
        let kwargs = Kwargs::from([("path", Value::from("/static/gutenberg.jpg"))]);
        let data = static_fn.call(kwargs, &state).unwrap();
        let data = data.as_map().unwrap();
        assert_eq!(data.get(&"height".into()).unwrap(), &Value::from(380));
        assert_eq!(data.get(&"width".into()).unwrap(), &Value::from(300));
        assert_eq!(data.get(&"format".into()).unwrap(), &Value::from("jpg"));
        assert_eq!(data.get(&"mime".into()).unwrap(), &Value::from("image/jpeg"));

        // 3. a call to something in `content` with a relative path
        let kwargs = Kwargs::from([("path", Value::from("content/gutenberg.jpg"))]);
        let data = static_fn.call(kwargs, &state).unwrap();
        let data = data.as_map().unwrap();
        assert_eq!(data.get(&"height".into()).unwrap(), &Value::from(380));
        assert_eq!(data.get(&"width".into()).unwrap(), &Value::from(300));
        assert_eq!(data.get(&"format".into()).unwrap(), &Value::from("jpg"));
        assert_eq!(data.get(&"mime".into()).unwrap(), &Value::from("image/jpeg"));

        // 4. a call to something in `content` with a @/ path corresponds to
        let kwargs = Kwargs::from([("path", Value::from("@/gutenberg.jpg"))]);
        let data = static_fn.call(kwargs, &state).unwrap();
        let data = data.as_map().unwrap();
        assert_eq!(data.get(&"height".into()).unwrap(), &Value::from(380));
        assert_eq!(data.get(&"width".into()).unwrap(), &Value::from(300));
        assert_eq!(data.get(&"format".into()).unwrap(), &Value::from("jpg"));
        assert_eq!(data.get(&"mime".into()).unwrap(), &Value::from("image/jpeg"));
    }
}
