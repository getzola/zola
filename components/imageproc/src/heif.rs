use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use errors::{Result, anyhow};
use fs_err as fs;
use heifer::FileKind;
use image::{DynamicImage, RgbImage, RgbaImage};

use crate::ImageMeta;
use crate::helpers::{get_created_datetime, get_description};

/// Returns `true` when `header` starts with an ISO-BMFF `ftyp` box for HEIC/HEIF
/// (excluding AVIF/AVIS, which Zola handles through `avif-parse` and `image`).
fn is_heif_header(header: &[u8]) -> bool {
    if header.len() < 16 || &header[4..8] != b"ftyp" {
        return false;
    }
    let box_size = u32::from_be_bytes([header[0], header[1], header[2], header[3]]) as usize;
    let end = box_size.clamp(16, 256).min(header.len());

    let is_avif_brand = |offset: usize| matches!(&header[offset..offset + 4], b"avif" | b"avis");
    if is_avif_brand(8) || (16..end.saturating_sub(3)).step_by(4).any(is_avif_brand) {
        return false;
    }

    let is_hevc_brand = |offset: usize| {
        matches!(&header[offset..offset + 4], b"heic" | b"heix" | b"hevc" | b"hevx")
    };
    is_hevc_brand(8)
        || matches!(&header[8..12], b"mif1" | b"mif2")
        || (16..end.saturating_sub(3)).step_by(4).any(is_hevc_brand)
}

/// Returns `true` when `path` starts with a HEIC/HEIF `ftyp` header.
pub(crate) fn is_heif_file(path: &Path) -> Result<bool> {
    let mut file = fs::File::open(path)?;
    let mut buf = [0u8; 256];
    let mut len = 0;
    while len < buf.len() {
        let n = file.read(&mut buf[len..])?;
        if n == 0 {
            break;
        }
        len += n;
    }
    Ok(is_heif_header(&buf[..len]))
}

/// Reads display dimensions and EXIF text fields from a HEIC/HEIF file without decoding pixels.
pub(crate) fn read_meta(path: &Path) -> Result<(ImageMeta, &'static str, &'static str)> {
    let mut reader = BufReader::new(fs::File::open(path)?);
    let info = heifer::read_info_reader(&mut reader)
        .map_err(|e| anyhow!("Failed to read HEIF info: {e}"))?;
    reader.seek(SeekFrom::Start(0))?;
    let metadata = heifer::read_metadata_reader(&mut reader)
        .map_err(|e| anyhow!("Failed to read HEIF metadata: {e}"))?;

    let (description, created) = match metadata.exif {
        Some(raw_exif) => match exif::Reader::new().read_raw(raw_exif) {
            Ok(exif) => (get_description(&exif).ok(), get_created_datetime(&exif).ok()),
            Err(e) => {
                log::warn!("Failed to parse exif for {}: {}", path.display(), e);
                (None, None)
            }
        },
        None => (None, None),
    };

    let (format, mime) = match info.kind {
        FileKind::Heic => ("heic", "image/heic"),
        _ => ("heif", "image/heif"),
    };

    // Do not call `get_rotated_size` here: `heifer::read_info_reader` already applies container
    // transforms (`clap`/`irot`/`imir`), and HEIC files commonly keep the camera's EXIF
    // Orientation tag alongside `irot`.
    Ok((
        ImageMeta { size: (info.width, info.height), format: None, description, created },
        format,
        mime,
    ))
}

/// Decodes the primary image of a HEIC/HEIF file (with container transforms already applied)
/// and returns its embedded ICC profile if any.
pub(crate) fn decode(path: &Path) -> Result<(DynamicImage, Option<Vec<u8>>)> {
    let mut reader = BufReader::new(fs::File::open(path)?);
    let icc_profile = heifer::read_metadata_reader(&mut reader)
        .map_err(|e| anyhow!("Failed to read HEIF metadata: {e}"))?
        .icc_profile;

    reader.seek(SeekFrom::Start(0))?;
    let decoded = if icc_profile.is_some() {
        // Keep pixels in the embedded ICC profile's color space so downstream encoders can
        // attach the original profile unchanged.
        heifer::decode_reader_rgba8(&mut reader, &heifer::Options::new())
    } else {
        let options = heifer::Options::new().with_fallback_to_coded_color(true);
        heifer::decode_reader_srgb_rgba8(&mut reader, &options)
    }
    .map_err(|e| anyhow!("Failed to decode HEIF image: {e}"))?;

    let (width, height) = (decoded.width, decoded.height);
    let img = if decoded.has_alpha {
        let rgba = RgbaImage::from_raw(width, height, decoded.data)
            .ok_or_else(|| anyhow!("Invalid decoded HEIF image dimensions"))?;
        DynamicImage::ImageRgba8(rgba)
    } else {
        let rgb = RgbImage::from_raw(width, height, decoded.into_rgb8())
            .ok_or_else(|| anyhow!("Invalid decoded HEIF image dimensions"))?;
        DynamicImage::ImageRgb8(rgb)
    };

    Ok((img, icc_profile))
}
