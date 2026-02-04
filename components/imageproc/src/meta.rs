use avif_parse::read_avif;
use errors::{Context, Result, anyhow};
use image::{ImageDecoder, ImageReader};
use image::{ImageFormat, ImageResult};
use serde::Serialize;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use svg_metadata::Metadata as SvgMetadata;

use crate::get_rotated_size;
use crate::helpers::{get_created_datetime, get_description};

/// Size and format read cheaply with `image`'s `Reader`.
#[derive(Debug)]
pub struct ImageMeta {
    /// (w, h)
    pub size: (u32, u32),
    pub format: Option<ImageFormat>,
    pub description: Option<String>,
    pub created: Option<String>,
}

impl ImageMeta {
    pub fn read(path: &Path) -> ImageResult<Self> {
        let reader = ImageReader::open(path).and_then(ImageReader::with_guessed_format)?;
        let format = reader.format();
        let mut decoder = reader.into_decoder()?;
        let mut size = decoder.dimensions();
        let metadata = decoder.exif_metadata()?.and_then(|raw_metadata| {
            exif::Reader::new()
                .read_raw(raw_metadata)
                .inspect_err(|e| eprintln!("Failed to parse exif for {}: {}", path.display(), e))
                .ok()
        });
        let metadata = metadata.as_ref();

        if let Some((w, h)) = get_rotated_size(size.0, size.1, metadata) {
            size = (w, h);
        }

        let description = get_description(metadata);
        let created = get_created_datetime(metadata);

        Ok(Self { size, format, description, created })
    }

    pub fn is_lossy(&self) -> bool {
        use ImageFormat::*;

        // We assume lossy by default / if unknown format
        let format = self.format.unwrap_or(Jpeg);
        !matches!(format, Png | Pnm | Tiff | Tga | Bmp | Ico | Hdr | Farbfeld)
    }
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct ImageMetaResponse {
    pub width: u32,
    pub height: u32,
    pub format: Option<&'static str>,
    pub mime: Option<&'static str>,
    pub description: Option<String>,
    pub created: Option<String>,
}

impl ImageMetaResponse {
    fn new_svg(path: &Path) -> Result<Self> {
        let img = SvgMetadata::parse_file(path)?;
        let (w, h) = match (img.width(), img.height(), img.view_box()) {
            (Some(w), Some(h), _) => Ok((w, h)),
            (_, _, Some(view_box)) => Ok((view_box.width, view_box.height)),
            _ => Err(anyhow!("Invalid dimensions: SVG width/height and viewbox not set.")),
        }?;
        Ok(Self {
            width: w as u32,
            height: h as u32,
            format: Some("svg"),
            mime: Some("text/svg+xml"),
            // SVG files have these fields, but we'd need a more comprehensive parser to read them.
            description: None,
            created: None,
        })
    }

    fn new_avif(path: &Path) -> Result<Self> {
        let avif_data = read_avif(&mut BufReader::new(File::open(path)?))?;
        let meta = avif_data.primary_item_metadata()?;
        Ok(Self {
            width: meta.max_frame_width.get(),
            height: meta.max_frame_height.get(),
            format: Some("avif"),
            mime: Some("image/avif"),
            // AVIF files have exif metadata to read these fields from, but we'd  need to enable image's avif-native feature to support decoding it.
            description: None,
            created: None,
        })
    }
}

impl From<ImageMeta> for ImageMetaResponse {
    fn from(im: ImageMeta) -> Self {
        Self {
            width: im.size.0,
            height: im.size.1,
            format: im.format.and_then(|f| f.extensions_str().first()).copied(),
            mime: im.format.map(|f| f.to_mime_type()),
            description: im.description,
            created: im.created,
        }
    }
}

/// Read image dimensions (cheaply), used in `get_image_metadata()`, supports SVG
pub fn read_image_metadata<P: AsRef<Path>>(path: P) -> Result<ImageMetaResponse> {
    let path = path.as_ref();
    let ext = path.extension().and_then(OsStr::to_str).unwrap_or("").to_lowercase();

    let err_context = || {
        format!(
            "Failed to read image (ext: {}) metadata: {}",
            if ext.is_empty() { "?" } else { ext.as_str() },
            path.display()
        )
    };

    match ext.as_str() {
        "svg" => ImageMetaResponse::new_svg(path),
        "avif" => ImageMetaResponse::new_avif(path),
        _ => ImageMeta::read(path).map(ImageMetaResponse::from).map_err(Into::into),
    }
    .with_context(err_context)
}
