use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
pub struct AvifConfiguration {
    /// The minimum quality to use when searching for the ideal quality to file-size compression ratio.
    pub min_quality: u8,
    /// The maximum quality to use when searching for the ideal quality to file-size compression ratio.
    pub max_quality: u8,
    /// How many rounds of encoding to use when searching for the ideal quality to file-size compression ratio.
    /// When set to 1 perceptual compression is disabled, and images are encoding at the set quality level.
    /// At any other setting, a binary search is performed over the rang of `min_quality` to  `max_quality`
    /// and the quality setting found that most closely matches the target is chosen.
    pub encode_iterations: u8,
}

impl Default for AvifConfiguration {
    fn default() -> Self {
        Self { min_quality: 1, max_quality: 100, encode_iterations: 5 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
pub struct JpegConfiguration {
    /// The minimum quality to use when searching for the ideal quality to file-size compression ratio.
    pub min_quality: u8,
    /// The maximum quality to use when searching for the ideal quality to file-size compression ratio.
    pub max_quality: u8,
    /// How many rounds of encoding to use when searching for the ideal quality to file-size compression ratio.
    /// When set to 1 perceptual compression is disabled, and images are encoding at the set quality level.
    /// At any other setting, a binary search is performed over the rang of `min_quality` to  `max_quality`
    /// and the quality setting found that most closely matches the target is chosen.
    pub encode_iterations: u8,
}

impl Default for JpegConfiguration {
    fn default() -> Self {
        Self { min_quality: 1, max_quality: 100, encode_iterations: 5 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(default)]
pub struct WebpConfiguration {
    /// The minimum quality to use when searching for the ideal quality to file-size compression ratio.
    pub min_quality: u8,
    /// The maximum quality to use when searching for the ideal quality to file-size compression ratio.
    pub max_quality: u8,
    /// How many rounds of encoding to use when searching for the ideal quality to file-size compression ratio.
    /// When set to 1 perceptual compression is disabled, and images are encoding at the set quality level.
    /// At any other setting, a binary search is performed over the rang of `min_quality` to  `max_quality`
    /// and the quality setting found that most closely matches the target is chosen.
    pub encode_iterations: u8,
}

impl Default for WebpConfiguration {
    fn default() -> Self {
        Self { min_quality: 0, max_quality: 100, encode_iterations: 5 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ImageEncoderConfig {
    pub avif: Option<AvifConfiguration>,
    pub jpeg: Option<JpegConfiguration>,
    pub webp: Option<WebpConfiguration>,
}

impl Default for ImageEncoderConfig {
    fn default() -> Self {
        Self { avif: None, jpeg: None, webp: None }
    }
}
