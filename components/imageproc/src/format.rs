use errors::{anyhow, Result};
use std::hash::{Hash, Hasher};

const QUALITY_MIN_JPEG: u8 = 1;
const QUALITY_MAX_JPEG: u8 = 100;
const QUALITY_MIN_WEBP: u8 = 0;
const QUALITY_MAX_WEBP: u8 = 100;
const QUALITY_MIN_AVIF: u8 = 1;
const QUALITY_MAX_AVIF: u8 = 100;
const SPEED_MIN_AVIF: u8 = 1;
const SPEED_MAX_AVIF: u8 = 10;

const DEFAULT_QUALITY_JPEG: u8 = 75;
const DEFAULT_QUALITY_AVIF: u8 = 80;
const DEFAULT_SPEED_AVIF: u8 = 5;

/// Thumbnail image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Jpeg { quality: u8 },
    Png,
    WebP { quality: Option<u8> }, // 'None' means lossless
    Avif { quality: u8, speed: u8 },
}

impl Format {
    pub fn from_args(
        is_lossy: bool,
        format: &str,
        quality: Option<u8>,
        speed: Option<u8>,
    ) -> Result<Format> {
        use Format::*;
        let format_from_auto = match (format, is_lossy) {
            ("auto", true) => "jpeg",
            ("auto", false) => "png",
            (other_format, _) => other_format,
        };
        match format_from_auto {
            "jpeg" | "jpg" => match quality.unwrap_or(DEFAULT_QUALITY_JPEG) {
                valid_quality @ QUALITY_MIN_JPEG..=QUALITY_MAX_JPEG => {
                    Ok(Jpeg { quality: valid_quality })
                }
                invalid_quality => Err(anyhow!(
                    "Quality for JPEG must be between {} and {} (inclusive); {} is not valid",
                    QUALITY_MIN_JPEG,
                    QUALITY_MAX_JPEG,
                    invalid_quality
                )),
            },
            "png" => Ok(Png),
            "webp" => match quality {
                Some(QUALITY_MIN_WEBP..=QUALITY_MAX_WEBP) | None => Ok(WebP { quality }),
                Some(invalid_quality) => Err(anyhow!(
                    "Quality for WebP must be between {} and {} (inclusive); {} is not valid",
                    QUALITY_MIN_WEBP,
                    QUALITY_MAX_WEBP,
                    invalid_quality
                )),
            },
            "avif" => {
                let q = match quality.unwrap_or(DEFAULT_QUALITY_AVIF) {
                    valid_quality @ QUALITY_MIN_AVIF..=QUALITY_MAX_AVIF => Ok(valid_quality),
                    invalid_quality => Err(anyhow!(
                        "Quality for AVIF must be between {} and {} (inclusive); {} is not valid",
                        QUALITY_MIN_AVIF,
                        QUALITY_MAX_AVIF,
                        invalid_quality
                    )),
                }?;
                let s = match speed.unwrap_or(DEFAULT_SPEED_AVIF) {
                    valid_speed @ SPEED_MIN_AVIF..=SPEED_MAX_AVIF => Ok(valid_speed),
                    invalid_speed => Err(anyhow!(
                        "Speed for AVIF must be between {} and {} (inclusive); {} is not valid",
                        SPEED_MIN_AVIF,
                        SPEED_MAX_AVIF,
                        invalid_speed
                    )),
                }?;
                Ok(Avif { quality: q, speed: s })
            }
            _ => Err(anyhow!("Invalid image format: {}", format)),
        }
    }

    pub fn extension(&self) -> &str {
        // Kept in sync with RESIZED_FILENAME and op_filename
        use Format::*;

        match *self {
            Png => "png",
            Jpeg { .. } => "jpg",
            WebP { .. } => "webp",
            Avif { .. } => "avif",
        }
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Format {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        use Format::*;

        let quality = match *self {
            Png => 0,
            Jpeg { quality } => quality,
            WebP { quality: None } => 0,
            WebP { quality: Some(quality) } => quality,
            Avif { quality, .. } => quality,
        };
        let speed = match *self {
            Png => 0,
            Jpeg { .. } => 0,
            WebP { .. } => 0,
            Avif { speed, .. } => speed,
        };

        hasher.write(self.extension().as_bytes());
        hasher.write_u8(quality);
        hasher.write_u8(speed);
    }
}
