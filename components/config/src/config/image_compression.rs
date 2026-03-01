use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Jpeg,
    Webp,
    Avif,
}

impl ImageFormat {
    pub fn file_extension(&self) -> &str {
        match self {
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Webp => "webp",
            ImageFormat::Avif => "avif",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageCompression {
    /// Glob of files to run the compression on.
    pub glob: String,
    /// The codec to encode files to.
    pub format: ImageFormat,
    /// Target SSIM score.
    #[serde(default = "default_ssim")]
    pub target_ssim: f64,
    /// Number of iterations to try and reach target SSIM.
    #[serde(default = "default_iterations")]
    pub max_iterations: u8,
}

fn default_ssim() -> f64 {
    0.8295
}

fn default_iterations() -> u8 {
    5
}
