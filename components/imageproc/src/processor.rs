use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use ahash::{HashMap, HashSet};
use config::Config;
use errors::{Context, Result, anyhow};
use image::codecs::avif::AvifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageDecoder, ImageReader};
use image::{EncodableLayout, ImageEncoder};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use utils::fs as ufs;

use crate::format::Format;
use crate::helpers::get_processed_filename;
use crate::{ImageMeta, ResizeInstructions, ResizeOperation, fix_orientation};

pub const RESIZED_SUBDIR: &str = "processed_images";

/// Holds all data needed to perform a resize operation
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ImageOp {
    input_path: PathBuf,
    output_path: PathBuf,
    instr: ResizeInstructions,
    format: Format,
    /// Whether we actually want to perform that op.
    /// In practice we set it to true if the output file already
    /// exists and is not stale. We do need to keep the ImageOp around for pruning though.
    ignore: bool,
}

impl ImageOp {
    fn perform(&self) -> Result<()> {
        if self.ignore {
            return Ok(());
        }

        log::debug!("processing {} with instructions {:?}", self.input_path.display(), self.instr);
        let input_permissions = fs::metadata(&self.input_path)?.permissions();
        let reader =
            ImageReader::open(&self.input_path).and_then(ImageReader::with_guessed_format)?;
        let mut decoder = reader.into_decoder()?;
        let color_profile = decoder.icc_profile().ok().flatten();
        let metadata = decoder.exif_metadata()?.and_then(|raw_metadata| {
            exif::Reader::new()
                .read_raw(raw_metadata)
                .inspect_err(|e| {
                    eprintln!("Failed to parse exif for {}: {}", self.input_path.display(), e)
                })
                .ok()
        });
        let img = DynamicImage::from_decoder(decoder)?;

        let mut img = fix_orientation(&img, metadata.as_ref()).unwrap_or(img);

        let img = match self.instr.crop_instruction {
            Some((x, y, w, h)) => img.crop(x, y, w, h),
            None => img,
        };
        let img = match self.instr.resize_instruction {
            Some((w, h)) => img.resize_exact(w, h, FilterType::Lanczos3),
            None => img,
        };

        let tmp_output_file = match self.output_path.parent() {
            Some(parent) => Ok(NamedTempFile::new_in(parent)?),
            None => Err(anyhow!(
                "Image output path '{:?}' should contain a parent directory, but doesn't",
                self.output_path
            )),
        }?;
        let mut tmp_output_writer = BufWriter::new(&tmp_output_file);

        let has_color_profile = color_profile.is_some();
        let add_color_profile = |encoder: &mut dyn ImageEncoder| {
            if let Some(color_profile) = color_profile {
                let _ = encoder.set_icc_profile(color_profile).inspect_err(|_| log::warn!("processing {}: Image encoder for {} does not support color profiles, colors may be incorrect.", self.input_path.display(), self.format.extension()));
            }
        };

        match self.format {
            Format::Png => {
                let mut encoder = PngEncoder::new(&mut tmp_output_writer);
                add_color_profile(&mut encoder);
                img.write_with_encoder(encoder)?;
            }
            Format::Jpeg { quality } => {
                let mut encoder = JpegEncoder::new_with_quality(&mut tmp_output_writer, quality);
                add_color_profile(&mut encoder);
                encoder.encode_image(&img)?;
            }
            Format::WebP { quality } => {
                // use the `image` builtin encoder for lossless, as it supports color profiles
                if quality.is_none() && has_color_profile {
                    let mut encoder = WebPEncoder::new_lossless(&mut tmp_output_writer);
                    add_color_profile(&mut encoder);
                    img.write_with_encoder(encoder)?;
                } else {
                    if has_color_profile {
                        log::warn!(
                            "processing {}: Lossy WebP encoder does not support color profiles, colors may be incorrect.",
                            self.input_path.display()
                        );
                    }
                    let encoder = webp::Encoder::from_image(&img)
                        .map_err(|_| anyhow!("Unable to load this kind of image with webp"))?;
                    let memory = match quality {
                        Some(q) => encoder.encode(q as f32),
                        None => encoder.encode_lossless(),
                    };
                    tmp_output_writer.write_all(memory.as_bytes())?;
                }
            }
            Format::Avif { quality, speed } => {
                let mut encoder =
                    AvifEncoder::new_with_speed_quality(&mut tmp_output_writer, speed, quality);
                add_color_profile(&mut encoder);
                img.write_with_encoder(encoder)?;
            }
        };

        fs::set_permissions(&tmp_output_file, input_permissions)?;
        fs::rename(&tmp_output_file, &self.output_path)?;

        Ok(())
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
    fn new(
        url: String,
        static_path: PathBuf,
        meta: &ImageMeta,
        instr: &ResizeInstructions,
    ) -> Self {
        let static_path = static_path.to_string_lossy().into_owned();
        let (width, height) = instr.resize_instruction.unwrap_or(meta.size);
        let (orig_width, orig_height) = meta.size;

        Self { url, static_path, width, height, orig_width, orig_height }
    }
}

/// A struct into which image operations can be enqueued and then performed.
/// All output is written in a subdirectory in `static_path`,
/// taking care of file stale status based on timestamps
#[derive(Debug)]
pub struct Processor {
    base_url: String,
    output_dir: PathBuf,
    img_ops: HashSet<ImageOp>,
    /// We want to make sure we only ever get metadata for an image once
    meta_cache: HashMap<PathBuf, ImageMeta>,
}

impl Processor {
    pub fn new(base_path: PathBuf, config: &Config) -> Processor {
        Processor {
            output_dir: base_path.join("static").join(RESIZED_SUBDIR),
            base_url: config.make_permalink(RESIZED_SUBDIR),
            img_ops: HashSet::default(),
            meta_cache: HashMap::default(),
        }
    }

    pub fn set_base_url(&mut self, config: &Config) {
        self.base_url = config.make_permalink(RESIZED_SUBDIR);
    }

    pub fn num_img_ops(&self) -> usize {
        self.img_ops.len()
    }

    pub fn enqueue(
        &mut self,
        op: ResizeOperation,
        input_src: String,
        input_path: PathBuf,
        format: &str,
        quality: Option<u8>,
        speed: Option<u8>,
    ) -> Result<EnqueueResponse> {
        // First we load metadata from the cache if possible, otherwise from the file itself
        if !self.meta_cache.contains_key(&input_path) {
            let meta = ImageMeta::read(&input_path)
                .with_context(|| format!("Failed to read image: {}", input_path.display()))?;
            self.meta_cache.insert(input_path.clone(), meta);
        }
        // We will have inserted it just above
        let meta = &self.meta_cache[&input_path];
        // We get the output format
        let format = Format::from_args(meta.is_lossy(), format, quality, speed)?;
        // Now we have all the data we need to generate the output filename and the response
        let filename = get_processed_filename(&input_path, &input_src, &op, &format);
        let url = format!("{}{}", self.base_url, filename);
        let static_path = Path::new("static").join(RESIZED_SUBDIR).join(&filename);
        let output_path = self.output_dir.join(&filename);
        let instr = ResizeInstructions::new(op, meta.size);
        let enqueue_response = EnqueueResponse::new(url, static_path, meta, &instr);
        let img_op = ImageOp {
            ignore: output_path.exists() && !ufs::file_stale(&input_path, &output_path),
            input_path,
            output_path,
            instr,
            format,
        };
        self.img_ops.insert(img_op);

        Ok(enqueue_response)
    }

    /// Run the enqueued image operations
    pub fn do_process(&mut self) -> Result<()> {
        if !self.img_ops.is_empty() {
            ufs::create_directory(&self.output_dir)?;
        }

        self.img_ops
            .par_iter()
            .map(|op| {
                op.perform().with_context(|| {
                    format!("Failed to process image: {}", op.input_path.display())
                })
            })
            .collect::<Result<()>>()
    }

    /// Remove stale processed images in the output directory
    pub fn prune(&self) -> Result<()> {
        // Do not create folders if they don't exist
        if !self.output_dir.exists() {
            return Ok(());
        }

        ufs::create_directory(&self.output_dir)?;
        let output_paths: HashSet<_> = self
            .img_ops
            .iter()
            .map(|o| o.output_path.file_name().unwrap().to_string_lossy())
            .collect();

        for entry in fs::read_dir(&self.output_dir)? {
            let entry_path = entry?.path();
            if entry_path.is_file() {
                let filename = entry_path.file_name().unwrap().to_string_lossy();
                if !output_paths.contains(&filename) {
                    fs::remove_file(&entry_path)?;
                }
            }
        }
        Ok(())
    }
}
