use syntect::dumps::from_binary;
use syntect::highlighting::ThemeSet;

lazy_static!{
    pub static ref THEME_SET: ThemeSet = from_binary(include_bytes!("../../sublime_themes/all.themedump"));
}
