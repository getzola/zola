use std::ops::Range;

#[derive(Debug, PartialEq)]
/// Returns the relationship between two ranges.
pub enum RangeRelation {
    /// `x ~ y = StartsIn <=> i ∈ x && j ∉ x` with `x = a..b` and `y = i..j`
    StartsIn,
    /// `x ~ y = EndsIn <=> i ∉ x && j ∈ x` with `x = a..b` and `y = i..j`
    EndsIn,
    /// `x ~ y = Within <=> i ∈ x && j ∈ x` with `x = a..b` and `y = i..j`
    Within,
    /// `x ~ y = Before <=> y < x` with `x = a..b` and `y = i..j`
    Before,
    /// `x ~ y = Before <=> y > x` with `x = a..b` and `y = i..j`
    After,
    /// `x ~ y = Before <=> i < x && j > x` with `x = a..b` and `y = i..j`
    Around,
}

impl RangeRelation {
    /// Gives the relation of `span` to `original_span`
    pub fn new<T>(original_span: &Range<T>, span: &Range<T>) -> RangeRelation
    where
        T: PartialOrd<T>,
    {
        use RangeRelation::*;

        match (
            original_span.contains(&span.start),
            span.end > original_span.start && span.end <= original_span.end,
        ) {
            // First we test whether the boundries of `span` are within `original_span`
            (true, true) => Within,
            (true, false) => StartsIn,
            (false, true) => EndsIn,

            // Now we check whether the boundries of `original_span` are within `span`
            _ => match (span.end <= original_span.start, span.start >= original_span.end) {
                // The `(true, true)` should never happen since that is not how ranges work.
                (true, _) => Before,
                (_, true) => After,
                _ => {
                    // Debug assert that we are actually around the original span
                    debug_assert!(span.start < original_span.start);
                    debug_assert!(span.end > original_span.end);

                    Around
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_rel {
        ($orig:expr, $span:expr => $res:ident) => {
            assert_eq!(RangeRelation::new(&$orig, &$span), RangeRelation::$res);
        };
    }

    #[test]
    fn basic() {
        assert_rel!(10..20, 2..8 => Before);
        assert_rel!(10..20, 2..18 => EndsIn);
        assert_rel!(10..20, 12..18 => Within);
        assert_rel!(10..20, 12..28 => StartsIn);
        assert_rel!(10..20, 22..28 => After);
        assert_rel!(10..20, 8..22 => Around);

        assert_rel!(10..20, 2..10 => Before);
        assert_rel!(10..20, 10..22 => StartsIn);
        assert_rel!(10..20, 8..20 => EndsIn);
        assert_rel!(10..20, 10..20 => Within);
    }
}
