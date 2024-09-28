use libs::image::imageops::FilterType::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterType {
    Lanczos3,
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
}

impl Into<libs::image::imageops::FilterType> for FilterType {
    fn into(self) -> libs::image::imageops::FilterType {
        match self {
            FilterType::Lanczos3 => Lanczos3,
            FilterType::Nearest => Nearest,
            FilterType::Gaussian => Gaussian,
            FilterType::Triangle => Triangle,
            FilterType::CatmullRom => CatmullRom,
        }
    }
}
