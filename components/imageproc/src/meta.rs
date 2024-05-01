use errors::{anyhow, Context, Result};
use libs::image::io::Reader as ImgReader;
use libs::image::ImageFormat;
use libs::jpegxl_rs::decoder_builder;
use libs::svg_metadata::Metadata as SvgMetadata;
use serde::Serialize;
use std::ffi::OsStr;
use std::path::Path;

/// Size and format read cheaply with `image`'s `Reader`.
#[derive(Debug)]
pub struct ImageMeta {
    /// (w, h)
    pub size: (u32, u32),
    pub format: Option<ImageFormat>,
}

impl ImageMeta {
    pub fn read(path: &Path) -> Result<Self> {
        if path.extension().is_some_and(|ext| ext == "jxl") {
            Self::read_jxl(path)
        } else {
            let reader = ImgReader::open(path).and_then(ImgReader::with_guessed_format)?;
            let format = reader.format();
            let size = reader.into_dimensions()?;

            Ok(Self { size, format })
        }
    }

    fn read_jxl(path: &Path) -> Result<Self> {
        let input = std::fs::read(path)?;
        let decoder = decoder_builder().build()?;
        let (meta, _) = decoder.decode(&input)?;
        Ok(ImageMeta { size: (meta.width, meta.height), format: None })
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
}

impl ImageMetaResponse {
    pub fn new_svg(width: u32, height: u32) -> Self {
        Self { width, height, format: Some("svg"), mime: Some("text/svg+xml") }
    }
    pub fn new_jxl(width: u32, height: u32) -> Self {
        Self { width, height, format: Some("jxl"), mime: Some("image/jxl") }
    }
}

impl From<ImageMeta> for ImageMetaResponse {
    fn from(im: ImageMeta) -> Self {
        Self {
            width: im.size.0,
            height: im.size.1,
            format: im.format.and_then(|f| f.extensions_str().first()).copied(),
            mime: im.format.map(|f| f.to_mime_type()),
        }
    }
}

/// Read image dimensions (cheaply), used in `get_image_metadata()`, supports SVG
pub fn read_image_metadata<P: AsRef<Path>>(path: P) -> Result<ImageMetaResponse> {
    let path = path.as_ref();
    let ext = path.extension().and_then(OsStr::to_str).unwrap_or("").to_lowercase();

    let err_context = || format!("Failed to read image: {}", path.display());

    match ext.as_str() {
        "svg" => {
            let img = SvgMetadata::parse_file(path).with_context(err_context)?;
            match (img.height(), img.width(), img.view_box()) {
                (Some(h), Some(w), _) => Ok((h, w)),
                (_, _, Some(view_box)) => Ok((view_box.height, view_box.width)),
                _ => Err(anyhow!("Invalid dimensions: SVG width/height and viewbox not set.")),
            }
            // this is not a typo, this returns the correct values for width and height.
            .map(|(h, w)| ImageMetaResponse::new_svg(w as u32, h as u32))
        }
        "jxl" => {
            let meta = ImageMeta::read(path)?;
            Ok(ImageMetaResponse::new_jxl(meta.size.0, meta.size.1))
        }
        _ => ImageMeta::read(path).map(ImageMetaResponse::from).with_context(err_context),
    }
}
