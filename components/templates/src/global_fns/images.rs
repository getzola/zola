use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use image::GenericImageView;
use svg_metadata as svg;
use tera::{from_value, to_value, Error, Function as TeraFn, Result, Value};

#[derive(Debug)]
pub struct ResizeImage {
    imageproc: Arc<Mutex<imageproc::Processor>>,
}
impl ResizeImage {
    pub fn new(imageproc: Arc<Mutex<imageproc::Processor>>) -> Self {
        Self { imageproc }
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
        if !imageproc.source_exists(&path) {
            return Err(format!("`resize_image`: Cannot find path: {}", path).into());
        }

        let imageop = imageproc::ImageOp::from_args(path, &op, width, height, &format, quality)
            .map_err(|e| format!("`resize_image`: {}", e))?;
        let url = imageproc.insert(imageop);

        to_value(url).map_err(|err| err.into())
    }
}

#[derive(Debug)]
pub struct GetImageMeta {
    content_path: PathBuf,
}

impl GetImageMeta {
    pub fn new(content_path: PathBuf) -> Self {
        Self { content_path }
    }
}

impl TeraFn for GetImageMeta {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let path = required_arg!(
            String,
            args.get("path"),
            "`get_image_metadata` requires a `path` argument with a string value"
        );
        let src_path = self.content_path.join(&path);
        if !src_path.exists() {
            return Err(format!("`get_image_metadata`: Cannot find path: {}", path).into());
        }
        let (height, width) = image_dimensions(&src_path)?;
        let mut map = tera::Map::new();
        map.insert(String::from("height"), Value::Number(tera::Number::from(height)));
        map.insert(String::from("width"), Value::Number(tera::Number::from(width)));
        Ok(Value::Object(map))
    }
}

// Try to read the image dimensions for a given image
fn image_dimensions(path: &PathBuf) -> Result<(u32, u32)> {
    if let Some("svg") = path.extension().and_then(OsStr::to_str) {
        let img = svg::Metadata::parse_file(&path)
            .map_err(|e| Error::chain(format!("Failed to process SVG: {}", path.display()), e))?;
        match (img.height(), img.width(), img.view_box()) {
            (Some(h), Some(w), _) => Ok((h as u32, w as u32)),
            (_, _, Some(view_box)) => Ok((view_box.height as u32, view_box.width as u32)),
            _ => Err("Invalid dimensions: SVG width/height and viewbox not set.".into()),
        }
    } else {
        let img = image::open(&path)
            .map_err(|e| Error::chain(format!("Failed to process image: {}", path.display()), e))?;
        Ok((img.height(), img.width()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO
}
