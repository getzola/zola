use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::ResizeOperation;
use crate::format::Format;
use image::DynamicImage;

/// Apply image rotation based on EXIF data
/// Returns `None` if no transformation is needed
pub fn fix_orientation(img: &DynamicImage, raw_metadata: Option<Vec<u8>>) -> Option<DynamicImage> {
    match get_orientation(raw_metadata)? {
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

/// Adjusts the width and height of an image based on EXIF rotation data.
/// Returns `None` if no transformation is needed.
pub fn get_rotated_size(w: u32, h: u32, raw_metadata: Option<Vec<u8>>) -> Option<(u32, u32)> {
    // See fix_orientation for the meaning of these values.
    match get_orientation(raw_metadata)? {
        5..=8 => Some((h, w)),
        _ => None,
    }
}

fn get_orientation(raw_metadata: Option<Vec<u8>>) -> Option<u32> {
    match raw_metadata {
        Some(metadata) => {
            let exif = exif::Reader::new().read_raw(metadata).ok()?;
            Some(exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?.value.get_uint(0)?)
        }
        None => None,
    }
}

/// We only use the input_path to get the file stem.
/// Hashing the resolved `input_path` would include the absolute path to the image
/// with all filesystem components.
pub fn get_processed_filename(
    input_path: &Path,
    input_src: &str,
    op: &ResizeOperation,
    format: &Format,
) -> String {
    let mut hasher = DefaultHasher::new();
    hasher.write(input_src.as_ref());
    op.hash(&mut hasher);
    format.hash(&mut hasher);
    let hash = hasher.finish();
    let filename = input_path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_else(|| Cow::Borrowed("unknown"));

    format!("{}.{:016x}.{}", filename, hash, format.extension())
}
