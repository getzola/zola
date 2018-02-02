#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate image;
extern crate rayon;
extern crate twox_hash;

extern crate utils;
extern crate errors;

use std::path::{Path, PathBuf};
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::collections::hash_map::Entry as HEntry;
use std::fs::{self, File};

use regex::Regex;
use image::{GenericImage, FilterType};
use image::jpeg::JPEGEncoder;
use rayon::prelude::*;
use twox_hash::XxHash;

use utils::fs as ufs;
use errors::{Result, ResultExt};


static RESIZED_SUBDIR: &'static str = "_resized_images";
lazy_static!{
    pub static ref RESIZED_FILENAME: Regex = Regex::new(r#"([0-9a-f]{16})([0-9a-f]{2})[.]jpg"#).unwrap();
}

/// Describes the precise kind of a resize operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeOp {
    /// A simple scale operation that doesn't take aspect ratio into account
    Scale(u32, u32),
    /// Scales the image to a specified width with height computed such that aspect ratio is preserved
    FitWidth(u32),
    /// Scales the image to a specified height with width computed such that aspect ratio is preserved
    FitHeight(u32),
    /// Scales the image such that it fits within the specified width and height preserving aspect ratio.
    /// Either dimension may end up being smaller, but never larger than specified.
    Fit(u32, u32),
    /// Scales the image such that it fills the specified width and height. Output will always have the exact dimensions specified.
    /// The part of the image that doesn't fit in the thumbnail due to differing aspect ratio will be cropped away, if any.
    Fill(u32, u32),
}

impl ResizeOp {
    pub fn from_args(op: &str, width: Option<u32>, height: Option<u32>) -> Result<ResizeOp> {
        use ResizeOp::*;

        // Validate args:
        match op {
            "fitwidth" => if width.is_none() { return Err(format!("op=fitwidth requires a `width` argument").into()) },
            "fitheight" => if height.is_none() { return Err(format!("op=fitwidth requires a `height` argument").into()) },
            "scale" | "fit" | "fill" => if width.is_none() || height.is_none() {
                return Err(format!("op={} requires a `width` and `height` argument", op).into())
            },
            _ => return Err(format!("Invalid image resize operation: {}", op).into())
        };

        Ok(match op {
            "scale" => Scale(width.unwrap(), height.unwrap()),
            "fitwidth" => FitWidth(width.unwrap()),
            "fitheight" => FitHeight(height.unwrap()),
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
        if let Some(w) = self.width()  { hasher.write_u32(w); }
        if let Some(h) = self.height() { hasher.write_u32(h); }
    }
}

/// Holds all data needed to perform a resize operation
#[derive(Debug, PartialEq, Eq)]
pub struct ImageOp {
    source: String,
    op: ResizeOp,
    quality: u8,
    hash: u64,
    collision: Option<u32>,
}

impl ImageOp {
    pub fn new(source: String, op: ResizeOp, quality: u8) -> ImageOp {
        let mut hasher = XxHash::with_seed(0);
        hasher.write(source.as_ref());
        op.hash(&mut hasher);
        hasher.write_u8(quality);
        let hash = hasher.finish();

        ImageOp { source, op, quality, hash, collision: None }
    }

    pub fn from_args(source: String, op: &str, width: Option<u32>, height: Option<u32>, quality: u8) -> Result<ImageOp> {
        let op = ResizeOp::from_args(op, width, height)?;
        Ok(Self::new(source, op, quality))
    }

    fn num_colli(&self) -> u32 { self.collision.unwrap_or(0) }

    fn perform(&self, content_path: &Path, target_path: &Path) -> Result<()> {
        use ResizeOp::*;

        let src_path = content_path.join(&self.source);
        if !ufs::file_stale(&src_path, target_path) {
            return Ok(())
        }

        let mut img = image::open(&src_path)?;
        let (img_w, img_h) = img.dimensions();

        const RESIZE_FILTER: FilterType = FilterType::Gaussian;
        const RATIO_EPSILLION: f32 = 0.1;

        let img = match self.op {
            Scale(w, h) => img.resize_exact(w, h, RESIZE_FILTER),
            FitWidth(w) => img.resize(w, u32::max_value(), RESIZE_FILTER),
            FitHeight(h) => img.resize(u32::max_value(), h, RESIZE_FILTER),
            Fit(w, h) => img.resize(w, h, RESIZE_FILTER),
            Fill(w, h) => {
                let fw = img_w as f32 / w as f32;
                let fh = img_h as f32 / h as f32;

                if (fw - fh).abs() <= RATIO_EPSILLION {
                    // The aspect is similar enough that there's not much point in cropping
                    img.resize_exact(w, h, RESIZE_FILTER)
                } else {
                    // We perform the fill such that a crop is performed first and then resize_exact can be used,
                    // which should be cheaper than resizing and then cropping (smaller number of pixels to resize).
                    let (crop_w, crop_h) = match fw < fh {
                        true  => (img_w, (fw * h as f32).round() as u32),
                        false => ((fh * w as f32).round() as u32, img_h),
                    };
                    let (off_w, off_h) = match fw < fh {
                        true  => (0, (img_h - crop_h) / 2),
                        false => ((img_w - crop_w) / 2, 0),
                    };
                    img.crop(off_w, off_h, crop_w, crop_h).resize_exact(w, h, RESIZE_FILTER)
                }
            },
        };

        let mut f = File::create(target_path)?;
        let mut enc = JPEGEncoder::new_with_quality(&mut f, self.quality);
        let (img_w, img_h) = img.dimensions();
        enc.encode(&img.raw_pixels(), img_w, img_h, img.color())?;
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
    img_ops: HashMap<u64, ImageOp>,
    // Hash collisions go here:
    img_ops_colls: Vec<ImageOp>,
}

impl Processor {
    pub fn new(content_path: PathBuf, static_path: &Path, base_url: &str) -> Processor {
        Processor {
            content_path,
            resized_path: static_path.join(RESIZED_SUBDIR),
            resized_url: Self::resized_url(base_url),
            img_ops: HashMap::new(),
            img_ops_colls: Vec::new(),
        }
    }

    fn resized_url(base_url: &str) -> String {
        match base_url.ends_with('/') {
            true  => format!("{}{}", base_url, RESIZED_SUBDIR),
            false => format!("{}/{}", base_url, RESIZED_SUBDIR),
        }
    }

    pub fn set_base_url(&mut self, base_url: &str) {
        self.resized_url = Self::resized_url(base_url);
    }

    pub fn source_exists(&self, source: &str) -> bool {
        self.content_path.join(source).exists()
    }

    pub fn num_img_ops(&self) -> usize {
        self.img_ops.len() + self.img_ops_colls.len()
    }

    fn insert_with_colls(&mut self, mut img_op: ImageOp) -> u32 {
        match self.img_ops.entry(img_op.hash) {
            HEntry::Occupied(entry) => if *entry.get() == img_op { return 0; },
            HEntry::Vacant(entry) => {
                entry.insert(img_op);
                return 0;
            },
        }

        // If we get here, that means a hash collision.
        let mut num = 1;
        for op in self.img_ops_colls.iter().filter(|op| op.hash == img_op.hash) {
            if *op == img_op {
                return num;
            } else {
                num += 1;
            }
        }

        if num == 1 {
            self.img_ops.get_mut(&img_op.hash).unwrap().collision = Some(0);
        }
        img_op.collision = Some(num);
        self.img_ops_colls.push(img_op);
        num
    }

    fn op_filename(hash: u64, colli_num: u32) -> String {
        // Please keep this in sync with RESIZED_FILENAME
        assert!(colli_num < 256, "Unexpectedly large number of collisions: {}", colli_num);
        format!("{:016x}{:02x}.jpg", hash, colli_num)
    }

    fn op_url(&self, hash: u64, colli_num: u32) -> String {
        format!("{}/{}", &self.resized_url, Self::op_filename(hash, colli_num))
    }

    pub fn insert(&mut self, img_op: ImageOp) -> String {
        let hash = img_op.hash;
        let num_colli = self.insert_with_colls(img_op);
        self.op_url(hash, num_colli)
    }

    pub fn prune(&self) -> Result<()> {
        ufs::ensure_directory_exists(&self.resized_path)?;
        let entries = fs::read_dir(&self.resized_path)?;
        for entry in entries {
            let entry_path = entry?.path();
            if entry_path.is_file() {
                let filename = entry_path.file_name().unwrap().to_string_lossy();
                if let Some(capts) = RESIZED_FILENAME.captures(filename.as_ref()) {
                    let hash = u64::from_str_radix(capts.get(1).unwrap().as_str(), 16).unwrap();
                    let num_colli = u32::from_str_radix(capts.get(2).unwrap().as_str(), 16).unwrap();
                    if num_colli > 0 || !self.img_ops.contains_key(&hash) {
                        fs::remove_file(&entry_path)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn do_process(&mut self) -> Result<()> {
        self.img_ops.par_iter().map(|(hash, op)| {
            let target = self.resized_path.join(Self::op_filename(*hash, op.num_colli()));
            op.perform(&self.content_path, &target)
                .chain_err(|| format!("Failed to process image: {}", op.source))
        })
        .fold(|| Ok(()), Result::and)
        .reduce(|| Ok(()), Result::and)
    }
}


/// Looks at file's extension and returns whether it's a supported image format
pub fn file_is_img<P: AsRef<Path>>(p: P) -> bool {
    p.as_ref().extension().and_then(|s| s.to_str()).map(|ext| {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => true,
            "png" => true,
            "gif" => true,
            "bmp" => true,
            _ => false,
        }
    }).unwrap_or(false)
}
