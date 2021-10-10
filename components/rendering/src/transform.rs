use std::ops::Range;

/// We will need to update the spans of the contexts which haven't yet been handled because their
/// spans are still based on the untransformed version.
#[derive(Debug, PartialEq)]
pub struct Transform {
    pub span_start: usize,
    pub initial_end: usize,
    pub after_end: usize,
}

impl Transform {
    pub fn new(original_span: &Range<usize>, new_len: usize) -> Transform {
        Transform {
            span_start: original_span.start,
            initial_end: original_span.end,
            after_end: new_len + original_span.start,
        }
    }
}
