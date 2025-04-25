use errors::{anyhow, Result};
use std::hash::{Hash, Hasher};

const DEFAULT_Q_JPG: u8 = 75;
const DEFAULT_Q_AVIF: u8 = 80;
const DEFAULT_S_AVIF: u8 = 5;

/// Thumbnail image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// JPEG, The `u8` argument is JPEG quality (in percent).
    Jpeg(u8),
    /// PNG
    Png,
    /// WebP, The `u8` argument is WebP quality (in percent), None meaning lossless.
    WebP(Option<u8>),
    /// AVIF, The first `u8` argument is AVIF quality (in percent). The second `u8` argument is AVIF speed.
    Avif(u8, u8),
}

impl Format {
    pub fn from_args(
        is_lossy: bool,
        format: &str,
        quality: Option<u8>,
        speed: Option<u8>,
    ) -> Result<Format> {
        use Format::*;
        if let Some(quality) = quality {
            assert!(quality > 0 && quality <= 100, "Quality must be within the range [1; 100]");
        }
        if let Some(speed) = speed {
            assert!(speed > 0 && speed <= 10, "Speed must be within the range [1; 10]");
        }
        let jpg_quality = quality.unwrap_or(DEFAULT_Q_JPG);
        match format {
            "auto" => {
                if is_lossy {
                    Ok(Jpeg(jpg_quality))
                } else {
                    Ok(Png)
                }
            }
            "jpeg" | "jpg" => Ok(Jpeg(jpg_quality)),
            "png" => Ok(Png),
            "webp" => Ok(WebP(quality)),
            "avif" => Ok(Avif(quality.unwrap_or(DEFAULT_Q_AVIF), speed.unwrap_or(DEFAULT_S_AVIF))),
            _ => Err(anyhow!("Invalid image format: {}", format)),
        }
    }

    pub fn extension(&self) -> &str {
        // Kept in sync with RESIZED_FILENAME and op_filename
        use Format::*;

        match *self {
            Png => "png",
            Jpeg(_) => "jpg",
            WebP(_) => "webp",
            Avif(_, _) => "avif",
        }
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Format {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        use Format::*;

        let quality = match *self {
            Png => 0,
            Jpeg(q) => q,
            WebP(None) => 0,
            WebP(Some(q)) => q,
            Avif(q, _) => q,
        };
        let speed = match *self {
            Png => 0,
            Jpeg(_) => 0,
            WebP(_) => 0,
            Avif(_, s) => s,
        };

        hasher.write(self.extension().as_bytes());
        hasher.write_u8(quality);
        hasher.write_u8(speed);
    }
}
