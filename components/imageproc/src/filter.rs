use errors::{Result, anyhow};
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
