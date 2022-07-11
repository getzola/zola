use std::collections::hash_map::Entry as HEntry;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::{collections::hash_map::DefaultHasher, io::Write};

use image::error::ImageResult;
use image::io::Reader as ImgReader;
use image::{imageops::FilterType, EncodableLayout};
use image::{ImageFormat, ImageOutputFormat};
use libs::image::DynamicImage;
use libs::{image, once_cell, rayon, regex, svg_metadata, webp};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use svg_metadata::Metadata as SvgMetadata;

use config::Config;
use errors::{anyhow, Context, Error, Result};
use utils::fs as ufs;

static RESIZED_SUBDIR: &str = "processed_images";
const DEFAULT_Q_JPG: u8 = 75;

static RESIZED_FILENAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"([0-9a-f]{16})([0-9a-f]{2})[.](jpg|png|webp)"#).unwrap());

/// Size and format read cheaply with `image`'s `Reader`.
#[derive(Debug)]
struct ImageMeta {
    size: (u32, u32),
    format: Option<ImageFormat>,
}

impl ImageMeta {
    fn read(path: &Path) -> ImageResult<Self> {
        let reader = ImgReader::open(path).and_then(ImgReader::with_guessed_format)?;
        let format = reader.format();
        let size = reader.into_dimensions()?;

        Ok(Self { size, format })
    }

    fn is_lossy(&self) -> bool {
        use ImageFormat::*;

        // We assume lossy by default / if unknown format
        let format = self.format.unwrap_or(Jpeg);
        !matches!(format, Png | Pnm | Tiff | Tga | Bmp | Ico | Hdr | Farbfeld)
    }
}

/// De-serialized & sanitized arguments of `resize_image`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeArgs {
    /// A simple scale operation that doesn't take aspect ratio into account
    Scale(u32, u32),
    /// Scales the image to a specified width with height computed such
    /// that aspect ratio is preserved
    FitWidth(u32),
    /// Scales the image to a specified height with width computed such
    /// that aspect ratio is preserved
    FitHeight(u32),
    /// If the image is larger than the specified width or height, scales the image such
    /// that it fits within the specified width and height preserving aspect ratio.
    /// Either dimension may end up being smaller, but never larger than specified.
    Fit(u32, u32),
    /// Scales the image such that it fills the specified width and height.
    /// Output will always have the exact dimensions specified.
    /// The part of the image that doesn't fit in the thumbnail due to differing
    /// aspect ratio will be cropped away, if any.
    Fill(u32, u32),
}

impl ResizeArgs {
    pub fn from_args(op: &str, width: Option<u32>, height: Option<u32>) -> Result<Self> {
        use ResizeArgs::*;

        // Validate args:
        match op {
            "fit_width" => {
                if width.is_none() {
                    return Err(anyhow!("op=\"fit_width\" requires a `width` argument"));
                }
            }
            "fit_height" => {
                if height.is_none() {
                    return Err(anyhow!("op=\"fit_height\" requires a `height` argument"));
                }
            }
            "scale" | "fit" | "fill" => {
                if width.is_none() || height.is_none() {
                    return Err(anyhow!("op={} requires a `width` and `height` argument", op));
                }
            }
            _ => return Err(anyhow!("Invalid image resize operation: {}", op)),
        };

        Ok(match op {
            "scale" => Scale(width.unwrap(), height.unwrap()),
            "fit_width" => FitWidth(width.unwrap()),
            "fit_height" => FitHeight(height.unwrap()),
            "fit" => Fit(width.unwrap(), height.unwrap()),
            "fill" => Fill(width.unwrap(), height.unwrap()),
            _ => unreachable!(),
        })
    }
}

/// Contains image crop/resize instructions for use by `Processor`
///
/// The `Processor` applies `crop` first, if any, and then `resize`, if any.
#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
struct ResizeOp {
    crop: Option<(u32, u32, u32, u32)>, // x, y, w, h
    resize: Option<(u32, u32)>,         // w, h
}

impl ResizeOp {
    fn new(args: ResizeArgs, (orig_w, orig_h): (u32, u32)) -> Self {
        use ResizeArgs::*;

        let res = ResizeOp::default();

        match args {
            Scale(w, h) => res.resize((w, h)),
            FitWidth(w) => {
                let h = (orig_h as u64 * w as u64) / orig_w as u64;
                res.resize((w, h as u32))
            }
            FitHeight(h) => {
                let w = (orig_w as u64 * h as u64) / orig_h as u64;
                res.resize((w as u32, h))
            }
            Fit(w, h) => {
                if orig_w <= w && orig_h <= h {
                    return res; // ie. no-op
                }

                let orig_w_h = orig_w as u64 * h as u64;
                let orig_h_w = orig_h as u64 * w as u64;

                if orig_w_h > orig_h_w {
                    Self::new(FitWidth(w), (orig_w, orig_h))
                } else {
                    Self::new(FitHeight(h), (orig_w, orig_h))
                }
            }
            Fill(w, h) => {
                const RATIO_EPSILLION: f32 = 0.1;

                let factor_w = orig_w as f32 / w as f32;
                let factor_h = orig_h as f32 / h as f32;

                if (factor_w - factor_h).abs() <= RATIO_EPSILLION {
                    // If the horizontal and vertical factor is very similar,
                    // that means the aspect is similar enough that there's not much point
                    // in cropping, so just perform a simple scale in this case.
                    res.resize((w, h))
                } else {
                    // We perform the fill such that a crop is performed first
                    // and then resize_exact can be used, which should be cheaper than
                    // resizing and then cropping (smaller number of pixels to resize).
                    let (crop_w, crop_h) = if factor_w < factor_h {
                        (orig_w, (factor_w * h as f32).round() as u32)
                    } else {
                        ((factor_h * w as f32).round() as u32, orig_h)
                    };

                    let (offset_w, offset_h) = if factor_w < factor_h {
                        (0, (orig_h - crop_h) / 2)
                    } else {
                        ((orig_w - crop_w) / 2, 0)
                    };

                    res.crop((offset_w, offset_h, crop_w, crop_h)).resize((w, h))
                }
            }
        }
    }

    fn crop(mut self, crop: (u32, u32, u32, u32)) -> Self {
        self.crop = Some(crop);
        self
    }

    fn resize(mut self, size: (u32, u32)) -> Self {
        self.resize = Some(size);
        self
    }
}

/// Thumbnail image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// JPEG, The `u8` argument is JPEG quality (in percent).
    Jpeg(u8),
    /// PNG
    Png,
    /// WebP, The `u8` argument is WebP quality (in percent), None meaning lossless.
    WebP(Option<u8>),
}

impl Format {
    fn from_args(meta: &ImageMeta, format: &str, quality: Option<u8>) -> Result<Format> {
        use Format::*;
        if let Some(quality) = quality {
            assert!(quality > 0 && quality <= 100, "Quality must be within the range [1; 100]");
        }
        let jpg_quality = quality.unwrap_or(DEFAULT_Q_JPG);
        match format {
            "auto" => {
                if meta.is_lossy() {
                    Ok(Jpeg(jpg_quality))
                } else {
                    Ok(Png)
                }
            }
            "jpeg" | "jpg" => Ok(Jpeg(jpg_quality)),
            "png" => Ok(Png),
            "webp" => Ok(WebP(quality)),
            _ => Err(anyhow!("Invalid image format: {}", format)),
        }
    }

    /// Looks at file's extension and, if it's a supported image format, returns whether the format is lossless
    pub fn is_lossy<P: AsRef<Path>>(p: P) -> Option<bool> {
        p.as_ref()
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .map(|ext| match ext.to_lowercase().as_str() {
                "jpg" | "jpeg" => Some(true),
                "png" => Some(false),
                "gif" => Some(false),
                "bmp" => Some(false),
                // It is assumed that webp is lossy, but it can be both
                "webp" => Some(true),
                _ => None,
            })
            .unwrap_or(None)
    }

    fn extension(&self) -> &str {
        // Kept in sync with RESIZED_FILENAME and op_filename
        use Format::*;

        match *self {
            Png => "png",
            Jpeg(_) => "jpg",
            WebP(_) => "webp",
        }
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for Format {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        use Format::*;

        let q = match *self {
            Png => 0,
            Jpeg(q) => q,
            WebP(None) => 0,
            WebP(Some(q)) => q,
        };

        hasher.write_u8(q);
        hasher.write(self.extension().as_bytes());
    }
}

/// Holds all data needed to perform a resize operation
#[derive(Debug, PartialEq, Eq)]
pub struct ImageOp {
    /// This is the source input path string as passed in the template, we need this to compute the hash.
    /// Hashing the resolved `input_path` would include the absolute path to the image
    /// with all filesystem components.
    input_src: String,
    input_path: PathBuf,
    op: ResizeOp,
    format: Format,
    /// Hash of the above parameters
    hash: u64,
    /// If there is a hash collision with another ImageOp, this contains a sequential ID > 1
    /// identifying the collision in the order as encountered (which is essentially random).
    /// Therefore, ImageOps with collisions (ie. collision_id > 0) are always considered out of date.
    /// Note that this is very unlikely to happen in practice
    collision_id: u32,
}

impl ImageOp {
    const RESIZE_FILTER: FilterType = FilterType::Lanczos3;

    fn new(input_src: String, input_path: PathBuf, op: ResizeOp, format: Format) -> ImageOp {
        let mut hasher = DefaultHasher::new();
        hasher.write(input_src.as_ref());
        op.hash(&mut hasher);
        format.hash(&mut hasher);
        let hash = hasher.finish();

        ImageOp { input_src, input_path, op, format, hash, collision_id: 0 }
    }

    fn perform(&self, target_path: &Path) -> Result<()> {
        if !ufs::file_stale(&self.input_path, target_path) {
            return Ok(());
        }

        let mut img = image::open(&self.input_path)?;

        let img = match self.op.crop {
            Some((x, y, w, h)) => img.crop(x, y, w, h),
            None => img,
        };
        let img = match self.op.resize {
            Some((w, h)) => img.resize_exact(w, h, Self::RESIZE_FILTER),
            None => img,
        };

        let img = fix_orientation(&img, &self.input_path).unwrap_or(img);

        let mut f = File::create(target_path)?;

        match self.format {
            Format::Png => {
                img.write_to(&mut f, ImageOutputFormat::Png)?;
            }
            Format::Jpeg(q) => {
                img.write_to(&mut f, ImageOutputFormat::Jpeg(q))?;
            }
            Format::WebP(q) => {
                let encoder = webp::Encoder::from_image(&img)
                    .map_err(|_| anyhow!("Unable to load this kind of image with webp"))?;
                let memory = match q {
                    Some(q) => encoder.encode(q as f32),
                    None => encoder.encode_lossless(),
                };
                f.write_all(memory.as_bytes())?;
            }
        }

        Ok(())
    }
}

/// Apply image rotation based on EXIF data
/// Returns `None` if no transformation is needed
pub fn fix_orientation(img: &DynamicImage, path: &Path) -> Option<DynamicImage> {
    let file = std::fs::File::open(path).ok()?;
    let mut buf_reader = std::io::BufReader::new(&file);
    let exif_reader = exif::Reader::new();
    let exif = exif_reader.read_from_container(&mut buf_reader).ok()?;
    let orientation =
        exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?.value.get_uint(0)?;
    match orientation {
        // Values are taken from the page 30 of
        // https://www.cipa.jp/std/documents/e/DC-008-2012_E.pdf
        // For more details check http://sylvana.net/jpegcrop/exif_orientation.html
        1 => None,
        2 => Some(img.fliph()),
        3 => Some(img.rotate180()),
        4 => Some(img.flipv()),
        5 => Some(img.fliph().rotate270()),
        6 => Some(img.rotate90()),
        7 => Some(img.fliph().rotate90()),
        8 => Some(img.rotate270()),
        _ => None,
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnqueueResponse {
    /// The final URL for that asset
    pub url: String,
    /// The path to the static asset generated
    pub static_path: String,
    /// New image width
    pub width: u32,
    /// New image height
    pub height: u32,
    /// Original image width
    pub orig_width: u32,
    /// Original image height
    pub orig_height: u32,
}

impl EnqueueResponse {
    fn new(url: String, static_path: PathBuf, meta: &ImageMeta, op: &ResizeOp) -> Self {
        let static_path = static_path.to_string_lossy().into_owned();
        let (width, height) = op.resize.unwrap_or(meta.size);
        let (orig_width, orig_height) = meta.size;

        Self { url, static_path, width, height, orig_width, orig_height }
    }
}

/// A struct into which image operations can be enqueued and then performed.
/// All output is written in a subdirectory in `static_path`,
/// taking care of file stale status based on timestamps and possible hash collisions.
#[derive(Debug)]
pub struct Processor {
    base_url: String,
    output_dir: PathBuf,
    /// A map of a ImageOps by their stored hash.
    /// Note that this cannot be a HashSet, because hashset handles collisions and we don't want that,
    /// we need to be aware of and handle collisions ourselves.
    img_ops: HashMap<u64, ImageOp>,
    /// Hash collisions go here:
    img_ops_collisions: Vec<ImageOp>,
}

impl Processor {
    pub fn new(base_path: PathBuf, config: &Config) -> Processor {
        Processor {
            output_dir: base_path.join("static").join(RESIZED_SUBDIR),
            base_url: config.make_permalink(RESIZED_SUBDIR),
            img_ops: HashMap::new(),
            img_ops_collisions: Vec::new(),
        }
    }

    pub fn set_base_url(&mut self, config: &Config) {
        self.base_url = config.make_permalink(RESIZED_SUBDIR);
    }

    pub fn num_img_ops(&self) -> usize {
        self.img_ops.len() + self.img_ops_collisions.len()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn enqueue(
        &mut self,
        input_src: String,
        input_path: PathBuf,
        op: &str,
        width: Option<u32>,
        height: Option<u32>,
        format: &str,
        quality: Option<u8>,
    ) -> Result<EnqueueResponse> {
        let meta = ImageMeta::read(&input_path)
            .with_context(|| format!("Failed to read image: {}", input_path.display()))?;

        let args = ResizeArgs::from_args(op, width, height)?;
        let op = ResizeOp::new(args, meta.size);
        let format = Format::from_args(&meta, format, quality)?;
        let img_op = ImageOp::new(input_src, input_path, op.clone(), format);
        let (static_path, url) = self.insert(img_op);

        Ok(EnqueueResponse::new(url, static_path, &meta, &op))
    }

    fn insert_with_collisions(&mut self, mut img_op: ImageOp) -> u32 {
        match self.img_ops.entry(img_op.hash) {
            HEntry::Occupied(entry) => {
                if *entry.get() == img_op {
                    return 0;
                }
            }
            HEntry::Vacant(entry) => {
                entry.insert(img_op);
                return 0;
            }
        }

        // If we get here, that means a hash collision.
        // This is detected when there is an ImageOp with the same hash in the `img_ops`
        // map but which is not equal to this one.
        // To deal with this, all collisions get a (random) sequential ID number.

        // First try to look up this ImageOp in `img_ops_collisions`, maybe we've
        // already seen the same ImageOp.
        // At the same time, count IDs to figure out the next free one.
        // Start with the ID of 2, because we'll need to use 1 for the ImageOp
        // already present in the map:
        let mut collision_id = 2;
        for op in self.img_ops_collisions.iter().filter(|op| op.hash == img_op.hash) {
            if *op == img_op {
                // This is a colliding ImageOp, but we've already seen an equal one
                // (not just by hash, but by content too), so just return its ID:
                return collision_id;
            } else {
                collision_id += 1;
            }
        }

        // If we get here, that means this is a new colliding ImageOp and
        // `collision_id` is the next free ID
        if collision_id == 2 {
            // This is the first collision found with this hash, update the ID
            // of the matching ImageOp in the map.
            self.img_ops.get_mut(&img_op.hash).unwrap().collision_id = 1;
        }
        img_op.collision_id = collision_id;
        self.img_ops_collisions.push(img_op);
        collision_id
    }

    fn op_filename(hash: u64, collision_id: u32, format: Format) -> String {
        // Please keep this in sync with RESIZED_FILENAME
        assert!(collision_id < 256, "Unexpectedly large number of collisions: {}", collision_id);
        format!("{:016x}{:02x}.{}", hash, collision_id, format.extension())
    }

    /// Adds the given operation to the queue but do not process it immediately.
    /// Returns (path in static folder, final URL).
    fn insert(&mut self, img_op: ImageOp) -> (PathBuf, String) {
        let hash = img_op.hash;
        let format = img_op.format;
        let collision_id = self.insert_with_collisions(img_op);
        let filename = Self::op_filename(hash, collision_id, format);
        let url = format!("{}{}", self.base_url, filename);
        (Path::new("static").join(RESIZED_SUBDIR).join(filename), url)
    }

    /// Remove stale processed images in the output directory
    pub fn prune(&self) -> Result<()> {
        // Do not create folders if they don't exist
        if !self.output_dir.exists() {
            return Ok(());
        }

        ufs::ensure_directory_exists(&self.output_dir)?;
        let entries = fs::read_dir(&self.output_dir)?;
        for entry in entries {
            let entry_path = entry?.path();
            if entry_path.is_file() {
                let filename = entry_path.file_name().unwrap().to_string_lossy();
                if let Some(capts) = RESIZED_FILENAME.captures(filename.as_ref()) {
                    let hash = u64::from_str_radix(capts.get(1).unwrap().as_str(), 16).unwrap();
                    let collision_id =
                        u32::from_str_radix(capts.get(2).unwrap().as_str(), 16).unwrap();

                    if collision_id > 0 || !self.img_ops.contains_key(&hash) {
                        fs::remove_file(&entry_path)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Run the enqueued image operations
    pub fn do_process(&mut self) -> Result<()> {
        if !self.img_ops.is_empty() {
            ufs::ensure_directory_exists(&self.output_dir)?;
        }

        self.img_ops
            .par_iter()
            .map(|(hash, op)| {
                let target =
                    self.output_dir.join(Self::op_filename(*hash, op.collision_id, op.format));

                op.perform(&target).with_context(|| {
                    format!("Failed to process image: {}", op.input_path.display())
                })
            })
            .collect::<Result<()>>()
    }
}

#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct ImageMetaResponse {
    pub width: u32,
    pub height: u32,
    pub format: Option<&'static str>,
}

impl ImageMetaResponse {
    pub fn new_svg(width: u32, height: u32) -> Self {
        Self { width, height, format: Some("svg") }
    }
}

impl From<ImageMeta> for ImageMetaResponse {
    fn from(im: ImageMeta) -> Self {
        Self {
            width: im.size.0,
            height: im.size.1,
            format: im.format.and_then(|f| f.extensions_str().get(0)).copied(),
        }
    }
}

impl From<webp::WebPImage> for ImageMetaResponse {
    fn from(img: webp::WebPImage) -> Self {
        Self { width: img.width(), height: img.height(), format: Some("webp") }
    }
}

/// Read image dimensions (cheaply), used in `get_image_metadata()`, supports SVG
pub fn read_image_metadata<P: AsRef<Path>>(path: P) -> Result<ImageMetaResponse> {
    let path = path.as_ref();
    let ext = path.extension().and_then(OsStr::to_str).unwrap_or("").to_lowercase();

    let err_context = || format!("Failed to read image: {}", path.display());

    match ext.as_str() {
        "svg" => {
            let img = SvgMetadata::parse_file(&path).with_context(err_context)?;
            match (img.height(), img.width(), img.view_box()) {
                (Some(h), Some(w), _) => Ok((h, w)),
                (_, _, Some(view_box)) => Ok((view_box.height, view_box.width)),
                _ => Err(anyhow!("Invalid dimensions: SVG width/height and viewbox not set.")),
            }
            //this is not a typo, this returns the correct values for width and height.
            .map(|(h, w)| ImageMetaResponse::new_svg(w as u32, h as u32))
        }
        "webp" => {
            // Unfortunately we have to load the entire image here, unlike with the others :|
            let data = fs::read(path).with_context(err_context)?;
            let decoder = webp::Decoder::new(&data[..]);
            decoder.decode().map(ImageMetaResponse::from).ok_or_else(|| {
                Error::msg(format!("Failed to decode WebP image: {}", path.display()))
            })
        }
        _ => ImageMeta::read(path).map(ImageMetaResponse::from).with_context(err_context),
    }
}

/// Assert that `address` matches `prefix` + RESIZED_FILENAME regex + "." + `extension`,
/// this is useful in test so that we don't need to hardcode hash, which is annoying.
pub fn assert_processed_path_matches(path: &str, prefix: &str, extension: &str) {
    let filename = path
        .strip_prefix(prefix)
        .unwrap_or_else(|| panic!("Path `{}` doesn't start with `{}`", path, prefix));

    let suffix = format!(".{}", extension);
    assert!(filename.ends_with(&suffix), "Path `{}` doesn't end with `{}`", path, suffix);

    assert!(
        RESIZED_FILENAME.is_match_at(filename, 0),
        "In path `{}`, file stem `{}` doesn't match the RESIZED_FILENAME regex",
        path,
        filename
    );
}
