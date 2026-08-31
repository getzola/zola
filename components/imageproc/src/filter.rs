use errors::{Result, anyhow};
use fast_image_resize as fir;
use image::imageops::FilterType;

pub(crate) fn filter_type_from_name(filter: &str) -> Result<FilterType> {
    use FilterType::*;

    Ok(match filter {
        "nearest" => Nearest,
        "triangle" => Triangle,
        "catmull_rom" => CatmullRom,
        "gaussian" => Gaussian,
        "lanczos3" => Lanczos3,
        _ => {
            return Err(anyhow!(
                "Invalid filter type : {filter}. Valid values: nearest, triangle, catmull_rom, gaussian, lanczos3"
            ));
        }
    })
}

pub(crate) fn fir_resize_alg_from_filter(filter: FilterType) -> fir::ResizeAlg {
    match filter {
        FilterType::Nearest => fir::ResizeAlg::Nearest,
        FilterType::Triangle => fir::ResizeAlg::Convolution(fir::FilterType::Bilinear),
        FilterType::CatmullRom => fir::ResizeAlg::Convolution(fir::FilterType::CatmullRom),
        FilterType::Gaussian => fir::ResizeAlg::Convolution(fir::FilterType::Gaussian),
        FilterType::Lanczos3 => fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3),
    }
}
