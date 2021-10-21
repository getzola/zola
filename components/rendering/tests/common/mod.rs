pub struct ShortCode {
    pub name: &'static str,
    pub output: &'static str,
    pub is_md: bool,
}

impl ShortCode {
    pub const fn new(name: &'static str, output: &'static str, is_md: bool) -> ShortCode {
        ShortCode { name, output, is_md }
    }

    /// Return filename for shortcode
    pub fn filename(&self) -> String {
        format!("{}.{}", self.name, if self.is_md { "md" } else { "html" })
    }
}
