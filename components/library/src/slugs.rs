pub use slug::slugify;

pub fn quasi_slugify(s: &str) -> String {
    s.replace("#", "").replace("/", "")
}

pub fn maybe_slugify(s: &str, enabled: bool) -> String {
    if enabled == true {
        return slugify(s);
    }
    return quasi_slugify(s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maybe_slugify_enabled() {
        assert_eq!(maybe_slugify("héhé", true), "hehe");
    }

    #[test]
    fn maybe_slugify_disabled() {
        assert_eq!(maybe_slugify("héhé", false), "héhé");
    }

    #[test]
    fn quasi_slugify_strips_bad_symbols() {
        assert_eq!(quasi_slugify("test#test/test"), "testtesttest");
    }
}
