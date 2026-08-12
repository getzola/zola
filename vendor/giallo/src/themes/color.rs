use serde::{Deserialize, Serialize};

use crate::error::{Error, GialloResult};

/// RGBA color with 8-bit components
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

fn parse_hex_component(hex: &str, original: &str) -> GialloResult<u8> {
    u8::from_str_radix(hex, 16).map_err(|_| Error::InvalidHexColor {
        value: original.to_string(),
        reason: format!("invalid hex component '{}'", hex),
    })
}

impl Color {
    pub(crate) const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub(crate) const BLACK: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };

    /// Outputs the hex value for that colour.
    #[inline]
    pub fn as_hex(&self) -> String {
        if self.a < 255 {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        } else {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        }
    }

    #[inline]
    pub(crate) fn as_css_color_property(&self) -> String {
        format!("color: {};", self.as_hex())
    }

    #[inline]
    pub(crate) fn as_css_bg_color_property(&self) -> String {
        format!("background-color: {};", self.as_hex())
    }

    #[inline]
    pub(crate) fn as_css_light_dark_color_property(light: &Color, dark: &Color) -> String {
        format!("color: light-dark({}, {});", light.as_hex(), dark.as_hex())
    }

    /// Render as a foreground color ANSI escape code in the terminal
    pub(crate) fn as_ansi_fg(self, s: &mut String) {
        s.push_str(&format!("38;2;{};{};{}", self.r, self.g, self.b));
    }

    /// Render as a background color ANSI escape code in the terminal
    pub(crate) fn as_ansi_bg(self, s: &mut String) {
        s.push_str(&format!("48;2;{};{};{}", self.r, self.g, self.b));
    }

    #[inline]
    pub(crate) fn as_css_light_dark_bg_color_property(light: &Color, dark: &Color) -> String {
        format!(
            "background-color: light-dark({}, {});",
            light.as_hex(),
            dark.as_hex()
        )
    }

    /// Creates a Color from a string (in theory a hex but it can also be black/white).
    ///
    /// Errors if the string is not a valid hex colour.
    pub fn from_hex(hex: &str) -> GialloResult<Self> {
        let original = hex;
        let hex = hex.trim_start_matches('#');

        if hex == "white" {
            return Ok(Color::WHITE);
        } else if hex == "black" {
            return Ok(Color::BLACK);
        }
        // Parse based on length
        match hex.len() {
            // #RGB format (e.g., #F00 for red)
            3 => {
                let r = parse_hex_component(&hex[0..1], original)?;
                let g = parse_hex_component(&hex[1..2], original)?;
                let b = parse_hex_component(&hex[2..3], original)?;
                Ok(Color {
                    r: r * 17, // Convert 0xF to 0xFF
                    g: g * 17,
                    b: b * 17,
                    a: 255,
                })
            }
            // #RGBA format (e.g., #F00F for red with full opacity)
            4 => {
                let r = parse_hex_component(&hex[0..1], original)?;
                let g = parse_hex_component(&hex[1..2], original)?;
                let b = parse_hex_component(&hex[2..3], original)?;
                let a = parse_hex_component(&hex[3..4], original)?;
                Ok(Color {
                    r: r * 17,
                    g: g * 17,
                    b: b * 17,
                    a: a * 17,
                })
            }
            // #RRGGBB format (e.g., #FF0000 for red)
            6 => {
                let r = parse_hex_component(&hex[0..2], original)?;
                let g = parse_hex_component(&hex[2..4], original)?;
                let b = parse_hex_component(&hex[4..6], original)?;
                Ok(Color { r, g, b, a: 255 })
            }
            // #RRGGBBAA format (e.g., #FF0000FF for red with full opacity)
            8 => {
                let r = parse_hex_component(&hex[0..2], original)?;
                let g = parse_hex_component(&hex[2..4], original)?;
                let b = parse_hex_component(&hex[4..6], original)?;
                let a = parse_hex_component(&hex[6..8], original)?;
                Ok(Color { r, g, b, a })
            }
            _ => Err(Error::InvalidHexColor {
                value: original.to_string(),
                reason: format!("invalid length {}", hex.len()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_hex_colors() {
        let inputs = vec![
            // 3-digit RGB
            (
                "#F00",
                Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#0F0",
                Color {
                    r: 0,
                    g: 255,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#00F",
                Color {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 255,
                },
            ),
            (
                "#FFF",
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
            ),
            (
                "#000",
                Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#888",
                Color {
                    r: 136,
                    g: 136,
                    b: 136,
                    a: 255,
                },
            ),
            (
                "#369",
                Color {
                    r: 51,
                    g: 102,
                    b: 153,
                    a: 255,
                },
            ),
            // 4-digit RGBA
            (
                "#F00F",
                Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#0F0F",
                Color {
                    r: 0,
                    g: 255,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#00FF",
                Color {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 255,
                },
            ),
            (
                "#FFF0",
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 0,
                },
            ),
            (
                "#0008",
                Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 136,
                },
            ),
            (
                "#FFFA",
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 170,
                },
            ),
            // 6-digit RGB
            (
                "#FF0000",
                Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#00FF00",
                Color {
                    r: 0,
                    g: 255,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#0000FF",
                Color {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 255,
                },
            ),
            (
                "#FFFFFF",
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                },
            ),
            (
                "#000000",
                Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#808080",
                Color {
                    r: 128,
                    g: 128,
                    b: 128,
                    a: 255,
                },
            ),
            (
                "#FF00FF",
                Color {
                    r: 255,
                    g: 0,
                    b: 255,
                    a: 255,
                },
            ),
            (
                "#00FFFF",
                Color {
                    r: 0,
                    g: 255,
                    b: 255,
                    a: 255,
                },
            ),
            (
                "#FFFF00",
                Color {
                    r: 255,
                    g: 255,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#123456",
                Color {
                    r: 18,
                    g: 52,
                    b: 86,
                    a: 255,
                },
            ),
            (
                "#ABCDEF",
                Color {
                    r: 171,
                    g: 205,
                    b: 239,
                    a: 255,
                },
            ),
            // 8-digit RGBA
            (
                "#FF0000FF",
                Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#00FF00FF",
                Color {
                    r: 0,
                    g: 255,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#0000FFFF",
                Color {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 255,
                },
            ),
            (
                "#FFFFFF00",
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 0,
                },
            ),
            (
                "#00000000",
                Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                },
            ),
            (
                "#80808080",
                Color {
                    r: 128,
                    g: 128,
                    b: 128,
                    a: 128,
                },
            ),
            (
                "#FF00FF80",
                Color {
                    r: 255,
                    g: 0,
                    b: 255,
                    a: 128,
                },
            ),
            (
                "#00FFFFCC",
                Color {
                    r: 0,
                    g: 255,
                    b: 255,
                    a: 204,
                },
            ),
            (
                "#FFFF0033",
                Color {
                    r: 255,
                    g: 255,
                    b: 0,
                    a: 51,
                },
            ),
            // Without # prefix
            (
                "FF0000",
                Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "F00",
                Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "FF0000FF",
                Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            // Mixed case (should work with case-insensitive parsing)
            (
                "#ff0000",
                Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#Ff0000",
                Color {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            ),
            (
                "#aAbBcC",
                Color {
                    r: 170,
                    g: 187,
                    b: 204,
                    a: 255,
                },
            ),
            // And our defaults
            (
                "#333333",
                Color {
                    r: 51,
                    g: 51,
                    b: 51,
                    a: 255,
                },
            ),
            (
                "#fffffe",
                Color {
                    r: 255,
                    g: 255,
                    b: 254,
                    a: 255,
                },
            ),
            (
                "#bbbbbb",
                Color {
                    r: 187,
                    g: 187,
                    b: 187,
                    a: 255,
                },
            ),
            (
                "#1e1e1e",
                Color {
                    r: 30,
                    g: 30,
                    b: 30,
                    a: 255,
                },
            ),
        ];

        for (input, expected) in inputs {
            let color = Color::from_hex(input).unwrap();
            assert_eq!(color, expected);
        }
    }

    #[test]
    fn error_on_invalid_format() {
        assert!(Color::from_hex("#FF").is_err());
        assert!(Color::from_hex("#FFFFF").is_err());
        assert!(Color::from_hex("#GGGGGG").is_err());
    }
}
