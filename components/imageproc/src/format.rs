use errors::{anyhow, Result};
use std::hash::{Hash, Hasher};

const DEFAULT_Q_JPG: u8 = 75;

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
    pub fn from_args(is_lossy: bool, format: &str, quality: Option<u8>) -> Result<Format> {
        use Format::*;
        if let Some(quality) = quality {
            assert!(quality > 0 && quality <= 100, "Quality must be within the range [1; 100]");
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
        }
    }
}

#[allow(clippy::derived_hash_with_manual_eq)]
impl Hash for Format {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        use Format::*;

        let q = match *self {
            Png => 0,
            Jpeg(q) => 1001 + q as u16,
            WebP(None) => 2000,
            WebP(Some(q)) => 2001 + q as u16,
        };

        hasher.write_u16(q);
        hasher.write(self.extension().as_bytes());
    }
}
