use std::borrow::Cow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::ResizeOperation;
use crate::format::Format;
use exif::Exif;
use image::DynamicImage;

#[derive(Debug)]
pub enum ExifError<'a> {
    MissingMetadata,
    NoSuchField {
        tag: exif::Tag,
        ifd_num: exif::In,
    },
    UnexpectedFieldType {
        tag: exif::Tag,
        ifd_num: exif::In,
        expected: &'static str,
        actual: &'a exif::Value,
    },
    EmptyValue {
        tag: exif::Tag,
        ifd_num: exif::In,
    },
}

impl<'a> std::fmt::Display for ExifError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExifError::MissingMetadata => f.write_str("Could not read exif metadata."),
            ExifError::NoSuchField { tag, ifd_num } => {
                write!(f, "Found no field for tag {} and ifd_num {}.", tag, ifd_num)
            }
            ExifError::UnexpectedFieldType { tag, ifd_num, expected, actual } => write!(
                f,
                "Expected field for tag {} and ifd_num {} to have type {}, but found {:?}.",
                tag, ifd_num, *expected, actual
            ),
            ExifError::EmptyValue { tag, ifd_num } => write!(
                f,
                "Expected the value of field for tag {} and ifd_num {} to have at least one element, but it was empty.",
                tag, ifd_num
            ),
        }
    }
}

/// Apply image rotation based on EXIF data
/// Returns `None` if no transformation is needed
pub fn fix_orientation<'a>(
    img: &DynamicImage,
    metadata: Option<&'a Exif>,
) -> Result<Option<DynamicImage>, ExifError<'a>> {
    Ok(match get_orientation(metadata)? {
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
    })
}

/// Adjusts the width and height of an image based on EXIF rotation data.
/// Returns `None` if no transformation is needed.
pub fn get_rotated_size(
    w: u32,
    h: u32,
    metadata: Option<&Exif>,
) -> Result<Option<(u32, u32)>, ExifError<'_>> {
    // See fix_orientation for the meaning of these values.
    Ok(match get_orientation(metadata)? {
        5..=8 => Some((h, w)),
        _ => None,
    })
}

fn get_field_value(
    metadata: Option<&Exif>,
    tag: exif::Tag,
    ifd_num: exif::In,
) -> Result<&exif::Value, ExifError<'_>> {
    let Some(metadata) = metadata else {
        return Err(ExifError::MissingMetadata);
    };
    let Some(field) = metadata.get_field(tag, ifd_num) else {
        return Err(ExifError::NoSuchField { tag, ifd_num });
    };
    Ok(&field.value)
}

fn get_orientation(metadata: Option<&Exif>) -> Result<u32, ExifError<'_>> {
    let tag = exif::Tag::Orientation;
    let ifd_num = exif::In::PRIMARY;

    let val = get_field_value(metadata, tag, ifd_num)?;
    let Some(orientation) = val.get_uint(0) else {
        return Err(ExifError::EmptyValue { tag, ifd_num });
    };
    Ok(orientation)
}

fn get_string_field(
    metadata: Option<&Exif>,
    tag: exif::Tag,
    ifd_num: exif::In,
) -> Result<String, ExifError<'_>> {
    let val = get_field_value(metadata, tag, ifd_num)?;
    let exif::Value::Ascii(bytes) = val else {
        return Err(ExifError::UnexpectedFieldType {
            tag,
            ifd_num,
            expected: &"exif::Value::Ascii",
            actual: val,
        });
    };

    if bytes.len() == 0 {
        return Err(ExifError::EmptyValue { tag, ifd_num });
    }

    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&*bytes[0]));
    Ok(s)
}

pub fn get_description(metadata: Option<&Exif>) -> Result<String, ExifError<'_>> {
    get_string_field(metadata, exif::Tag::ImageDescription, exif::In::PRIMARY)
}

pub fn get_created_datetime(metadata: Option<&Exif>) -> Result<String, ExifError<'_>> {
    get_string_field(metadata, exif::Tag::DateTimeOriginal, exif::In::PRIMARY)
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
