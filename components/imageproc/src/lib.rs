#[macro_use]
extern crate lazy_static;
extern crate image;
extern crate rayon;
extern crate regex;

extern crate errors;
extern crate utils;

use std::collections::hash_map::DefaultHasher;
use std::collections::hash_map::Entry as HEntry;
use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use image::jpeg::JPEGEncoder;
use image::png::PNGEncoder;
use image::{FilterType, GenericImageView};
use rayon::prelude::*;
use regex::Regex;

use errors::{Error, Result};
use utils::fs as ufs;

static RESIZED_SUBDIR: &'static str = "processed_images";

lazy_static! {
    pub static ref RESIZED_FILENAME: Regex =
        Regex::new(r#"([0-9a-f]{16})([0-9a-f]{2})[.](jpg|png)"#).unwrap();
}

/// Describes the precise kind of a resize operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeOp {
    /// A simple scale operation that doesn't take aspect ratio into account
    Scale(u32, u32),
    /// Scales the image to a specified width with height computed such
    /// that aspect ratio is preserved
    FitWidth(u32),
    /// Scales the image to a specified height with width computed such
    /// that aspect ratio is preserved
    FitHeight(u32),
    /// Scales the image such that it fits within the specified width and
    /// height preserving aspect ratio.
    /// Either dimension may end up being smaller, but never larger than specified.
    Fit(u32, u32),
    /// Scales the image such that it fills the specified width and height.
    /// Output will always have the exact dimensions specified.
    /// The part of the image that doesn't fit in the thumbnail due to differing
    /// aspect ratio will be cropped away, if any.
    Fill(u32, u32),
}

impl ResizeOp {
    pub fn from_args(op: &str, width: Option<u32>, height: Option<u32>) -> Result<ResizeOp> {
        use ResizeOp::*;

        // Validate args:
        match op {
            "fit_width" => {
                if width.is_none() {
                    return Err("op=\"fit_width\" requires a `width` argument".to_string().into());
                }
            }
            "fit_height" => {
                if height.is_none() {
                    return Err("op=\"fit_height\" requires a `height` argument"
                        .to_string()
                        .into());
                }
            }
            "scale" | "fit" | "fill" => {
                if width.is_none() || height.is_none() {
                    return Err(
                        format!("op={} requires a `width` and `height` argument", op).into()
                    );
                }
            }
            _ => return Err(format!("Invalid image resize operation: {}", op).into()),
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

    pub fn width(self) -> Option<u32> {
        use ResizeOp::*;

        match self {
            Scale(w, _) => Some(w),
            FitWidth(w) => Some(w),
            FitHeight(_) => None,
            Fit(w, _) => Some(w),
            Fill(w, _) => Some(w),
        }
    }

    pub fn height(self) -> Option<u32> {
        use ResizeOp::*;

        match self {
            Scale(_, h) => Some(h),
            FitWidth(_) => None,
            FitHeight(h) => Some(h),
            Fit(_, h) => Some(h),
            Fill(_, h) => Some(h),
        }
    }
}

impl From<ResizeOp> for u8 {
    fn from(op: ResizeOp) -> u8 {
        use ResizeOp::*;

        match op {
            Scale(_, _) => 1,
            FitWidth(_) => 2,
            FitHeight(_) => 3,
            Fit(_, _) => 4,
            Fill(_, _) => 5,
        }
    }
}

impl Hash for ResizeOp {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_u8(u8::from(*self));
        if let Some(w) = self.width() {
            hasher.write_u32(w);
        }
        if let Some(h) = self.height() {
            hasher.write_u32(h);
        }
    }
}

/// Thumbnail image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// JPEG, The `u8` argument is JPEG quality (in percent).
    Jpeg(u8),
    /// PNG
    Png,
}

impl Format {
    pub fn from_args(source: &str, format: &str, quality: u8) -> Result<Format> {
        use Format::*;

        assert!(quality > 0 && quality <= 100, "Jpeg quality must be within the range [1; 100]");

        match format {
            "auto" => match Self::is_lossy(source) {
                Some(true) => Ok(Jpeg(quality)),
                Some(false) => Ok(Png),
                None => Err(format!("Unsupported image file: {}", source).into()),
            },
            "jpeg" | "jpg" => Ok(Jpeg(quality)),
            "png" => Ok(Png),
            _ => Err(format!("Invalid image format: {}", format).into()),
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
        }
    }
}

impl Hash for Format {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        use Format::*;

        let q = match *self {
            Png => 0,
            Jpeg(q) => q,
        };

        hasher.write_u8(q);
    }
}

/// Holds all data needed to perform a resize operation
#[derive(Debug, PartialEq, Eq)]
pub struct ImageOp {
    source: String,
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
    pub fn new(source: String, op: ResizeOp, format: Format) -> ImageOp {
        let mut hasher = DefaultHasher::new();
        hasher.write(source.as_ref());
        op.hash(&mut hasher);
        format.hash(&mut hasher);
        let hash = hasher.finish();

        ImageOp { source, op, format, hash, collision_id: 0 }
    }

    pub fn from_args(
        source: String,
        op: &str,
        width: Option<u32>,
        height: Option<u32>,
        format: &str,
        quality: u8,
    ) -> Result<ImageOp> {
        let op = ResizeOp::from_args(op, width, height)?;
        let format = Format::from_args(&source, format, quality)?;
        Ok(Self::new(source, op, format))
    }

    fn perform(&self, content_path: &Path, target_path: &Path) -> Result<()> {
        use ResizeOp::*;

        let src_path = content_path.join(&self.source);
        if !ufs::file_stale(&src_path, target_path) {
            return Ok(());
        }

        let mut img = image::open(&src_path)?;
        let (img_w, img_h) = img.dimensions();

        const RESIZE_FILTER: FilterType = FilterType::Lanczos3;
        const RATIO_EPSILLION: f32 = 0.1;

        let img = match self.op {
            Scale(w, h) => img.resize_exact(w, h, RESIZE_FILTER),
            FitWidth(w) => img.resize(w, u32::max_value(), RESIZE_FILTER),
            FitHeight(h) => img.resize(u32::max_value(), h, RESIZE_FILTER),
            Fit(w, h) => img.resize(w, h, RESIZE_FILTER),
            Fill(w, h) => {
                let factor_w = img_w as f32 / w as f32;
                let factor_h = img_h as f32 / h as f32;

                if (factor_w - factor_h).abs() <= RATIO_EPSILLION {
                    // If the horizontal and vertical factor is very similar,
                    // that means the aspect is similar enough that there's not much point
                    // in cropping, so just perform a simple scale in this case.
                    img.resize_exact(w, h, RESIZE_FILTER)
                } else {
                    // We perform the fill such that a crop is performed first
                    // and then resize_exact can be used, which should be cheaper than
                    // resizing and then cropping (smaller number of pixels to resize).
                    let (crop_w, crop_h) = if factor_w < factor_h {
                        (img_w, (factor_w * h as f32).round() as u32)
                    } else {
                        ((factor_h * w as f32).round() as u32, img_h)
                    };

                    let (offset_w, offset_h) = if factor_w < factor_h {
                        (0, (img_h - crop_h) / 2)
                    } else {
                        ((img_w - crop_w) / 2, 0)
                    };

                    img.crop(offset_w, offset_h, crop_w, crop_h).resize_exact(w, h, RESIZE_FILTER)
                }
            }
        };

        let mut f = File::create(target_path)?;
        let (img_w, img_h) = img.dimensions();

        match self.format {
            Format::Png => {
                let mut enc = PNGEncoder::new(&mut f);
                enc.encode(&img.raw_pixels(), img_w, img_h, img.color())?;
            }
            Format::Jpeg(q) => {
                let mut enc = JPEGEncoder::new_with_quality(&mut f, q);
                enc.encode(&img.raw_pixels(), img_w, img_h, img.color())?;
            }
        }

        Ok(())
    }
}

/// A strcture into which image operations can be enqueued and then performed.
/// All output is written in a subdirectory in `static_path`,
/// taking care of file stale status based on timestamps and possible hash collisions.
#[derive(Debug)]
pub struct Processor {
    content_path: PathBuf,
    resized_path: PathBuf,
    resized_url: String,
    /// A map of a ImageOps by their stored hash.
    /// Note that this cannot be a HashSet, because hashset handles collisions and we don't want that,
    /// we need to be aware of and handle collisions ourselves.
    img_ops: HashMap<u64, ImageOp>,
    /// Hash collisions go here:
    img_ops_collisions: Vec<ImageOp>,
}

impl Processor {
    pub fn new(content_path: PathBuf, static_path: &Path, base_url: &str) -> Processor {
        Processor {
            content_path,
            resized_path: static_path.join(RESIZED_SUBDIR),
            resized_url: Self::resized_url(base_url),
            img_ops: HashMap::new(),
            img_ops_collisions: Vec::new(),
        }
    }

    fn resized_url(base_url: &str) -> String {
        if base_url.ends_with('/') {
            format!("{}{}", base_url, RESIZED_SUBDIR)
        } else {
            format!("{}/{}", base_url, RESIZED_SUBDIR)
        }
    }

    pub fn set_base_url(&mut self, base_url: &str) {
        self.resized_url = Self::resized_url(base_url);
    }

    pub fn source_exists(&self, source: &str) -> bool {
        self.content_path.join(source).exists()
    }

    pub fn num_img_ops(&self) -> usize {
        self.img_ops.len() + self.img_ops_collisions.len()
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

    fn op_url(&self, hash: u64, collision_id: u32, format: Format) -> String {
        format!("{}/{}", &self.resized_url, Self::op_filename(hash, collision_id, format))
    }

    pub fn insert(&mut self, img_op: ImageOp) -> String {
        let hash = img_op.hash;
        let format = img_op.format;
        let collision_id = self.insert_with_collisions(img_op);
        self.op_url(hash, collision_id, format)
    }

    pub fn prune(&self) -> Result<()> {
        // Do not create folders if they don't exist
        if !self.resized_path.exists() {
            return Ok(());
        }

        ufs::ensure_directory_exists(&self.resized_path)?;
        let entries = fs::read_dir(&self.resized_path)?;
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

    pub fn do_process(&mut self) -> Result<()> {
        if !self.img_ops.is_empty() {
            ufs::ensure_directory_exists(&self.resized_path)?;
        }

        self.img_ops
            .par_iter()
            .map(|(hash, op)| {
                let target =
                    self.resized_path.join(Self::op_filename(*hash, op.collision_id, op.format));
                op.perform(&self.content_path, &target)
                    .map_err(|e| Error::chain(format!("Failed to process image: {}", op.source), e))
            })
            .collect::<Result<()>>()
    }
}
