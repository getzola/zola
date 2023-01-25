use errors::{anyhow, Result};

/// De-serialized & sanitized arguments of `resize_image`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResizeOperation {
    /// A simple scale operation that doesn't take aspect ratio into account
    Scale(u32, u32),
    /// Scales the image to a specified width with height computed such
    /// that aspect ratio is preserved
    FitWidth(u32),
    /// Scales the image to a specified height with width computed such
    /// that aspect ratio is preserved
    FitHeight(u32),
    /// If the image is larger than the specified width or height, scales the image such
    /// that it fits within the specified width and height preserving aspect ratio.
    /// Either dimension may end up being smaller, but never larger than specified.
    Fit(u32, u32),
    /// Scales the image such that it fills the specified width and height.
    /// Output will always have the exact dimensions specified.
    /// The part of the image that doesn't fit in the thumbnail due to differing
    /// aspect ratio will be cropped away, if any.
    Fill(u32, u32),
}

impl ResizeOperation {
    pub fn from_args(op: &str, width: Option<u32>, height: Option<u32>) -> Result<Self> {
        use ResizeOperation::*;

        // Validate args:
        match op {
            "fit_width" => {
                if width.is_none() {
                    return Err(anyhow!("op=\"fit_width\" requires a `width` argument"));
                }
            }
            "fit_height" => {
                if height.is_none() {
                    return Err(anyhow!("op=\"fit_height\" requires a `height` argument"));
                }
            }
            "scale" | "fit" | "fill" => {
                if width.is_none() || height.is_none() {
                    return Err(anyhow!("op={} requires a `width` and `height` argument", op));
                }
            }
            _ => return Err(anyhow!("Invalid image resize operation: {}", op)),
        };

        Ok(match op {
            "scale" => Scale(width.unwrap(), height.unwrap()),
            "fit_width" => FitWidth(width.unwrap()),
            "fit_height" => FitHeight(height.unwrap()),
            "fit" => Fit(width.unwrap(), height.unwrap()),
            "fill" => Fill(width.unwrap(), height.unwrap()),
            _ => unreachable!(),
        })
    }
}

/// Contains image crop/resize instructions for use by `Processor`
///
/// The `Processor` applies `crop` first, if any, and then `resize`, if any.
#[derive(Clone, PartialEq, Eq, Hash, Default, Debug)]
pub struct ResizeInstructions {
    pub crop_instruction: Option<(u32, u32, u32, u32)>, // x, y, w, h
    pub resize_instruction: Option<(u32, u32)>,         // w, h
}

impl ResizeInstructions {
    pub fn new(args: ResizeOperation, (orig_w, orig_h): (u32, u32)) -> Self {
        use ResizeOperation::*;

        let res = ResizeInstructions::default();

        match args {
            Scale(w, h) => res.resize((w, h)),
            FitWidth(w) => {
                let h = (orig_h as u64 * w as u64) / orig_w as u64;
                res.resize((w, h as u32))
            }
            FitHeight(h) => {
                let w = (orig_w as u64 * h as u64) / orig_h as u64;
                res.resize((w as u32, h))
            }
            Fit(w, h) => {
                if orig_w <= w && orig_h <= h {
                    return res; // ie. no-op
                }

                let orig_w_h = orig_w as u64 * h as u64;
                let orig_h_w = orig_h as u64 * w as u64;

                if orig_w_h > orig_h_w {
                    Self::new(FitWidth(w), (orig_w, orig_h))
                } else {
                    Self::new(FitHeight(h), (orig_w, orig_h))
                }
            }
            Fill(w, h) => {
                const RATIO_EPSILLION: f32 = 0.1;

                let factor_w = orig_w as f32 / w as f32;
                let factor_h = orig_h as f32 / h as f32;

                if (factor_w - factor_h).abs() <= RATIO_EPSILLION {
                    // If the horizontal and vertical factor is very similar,
                    // that means the aspect is similar enough that there's not much point
                    // in cropping, so just perform a simple scale in this case.
                    res.resize((w, h))
                } else {
                    // We perform the fill such that a crop is performed first
                    // and then resize_exact can be used, which should be cheaper than
                    // resizing and then cropping (smaller number of pixels to resize).
                    let (crop_w, crop_h) = if factor_w < factor_h {
                        (orig_w, (factor_w * h as f32).round() as u32)
                    } else {
                        ((factor_h * w as f32).round() as u32, orig_h)
                    };

                    let (offset_w, offset_h) = if factor_w < factor_h {
                        (0, (orig_h - crop_h) / 2)
                    } else {
                        ((orig_w - crop_w) / 2, 0)
                    };

                    res.crop((offset_w, offset_h, crop_w, crop_h)).resize((w, h))
                }
            }
        }
    }

    pub fn crop(mut self, crop: (u32, u32, u32, u32)) -> Self {
        self.crop_instruction = Some(crop);
        self
    }

    pub fn resize(mut self, size: (u32, u32)) -> Self {
        self.resize_instruction = Some(size);
        self
    }
}
