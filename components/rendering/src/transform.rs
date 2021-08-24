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

    pub fn initial(&self) -> Range<usize> {
        self.span_start..self.initial_end
    }

    pub fn after(&self) -> Range<usize> {
        self.span_start..self.after_end
    }

    /// Based on whether this transform becomes bigger or shorter, either add onto the position or
    /// subtract from the position.
    ///
    /// If the position is smaller than the substraction value this function will return None.
    pub fn based_adjust(&self, position: usize) -> Option<usize> {
        if self.initial_end < self.after_end {
            Some(position + (self.after_end - self.initial_end))
        } else {
            let sub_difference = self.initial_end - self.after_end;

            if sub_difference > position {
                None
            } else {
                Some(position - sub_difference)
            }
        }
    }
}
