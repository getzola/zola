pub fn maybe_slugify(s: &str, slugify: bool) -> String {
    if slugify {
        slug::slugify(s)
    } else {
        s.to_string()
    }
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
}
