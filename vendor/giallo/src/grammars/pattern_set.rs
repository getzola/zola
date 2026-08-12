use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicUsize, Ordering};

use onig::{RegSet, RegexOptions, SearchOptions};

use crate::grammars::GlobalRuleRef;

static NEXT_PATTERN_SET_ID: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    /// One `RegSet` per pattern set, per thread.
    ///
    /// `onig_regset_search` writes to the regset's internal region storage, so a
    /// single shared `RegSet` has to be locked and every thread highlighting the
    /// same language then queues behind that lock. Giving each thread its own
    /// compiled copy removes the contention instead of serialising around it; a
    /// copy is only compiled in a thread that actually searches that pattern set.
    static LOCAL_REGSETS: RefCell<HashMap<usize, RegSet>> = RefCell::new(HashMap::new());
}

fn compile(patterns: &[String]) -> Result<RegSet, String> {
    let pattern_strs: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();
    RegSet::with_options(&pattern_strs, RegexOptions::REGEX_OPTION_CAPTURE_GROUP).map_err(|e| {
        format!(
            "Failed to compile pattern set with {} patterns: {:?}",
            pattern_strs.len(),
            e
        )
    })
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PatternSetMatch {
    pub rule_ref: GlobalRuleRef,
    pub start: usize,
    pub end: usize,
    pub capture_pos: Vec<Option<(usize, usize)>>,
}

/// A pattern set for efficient batch regex matching using onig RegSet.
///
/// The compiled `RegSet` lives in thread-local storage rather than in the struct:
/// `onig_regset_search` mutates internal region storage, so one instance cannot
/// be searched concurrently. Each thread compiles its own copy the first time it
/// searches this set, which keeps highlighting parallel.
pub struct PatternSet {
    id: usize,
    rule_refs: Vec<GlobalRuleRef>,
    /// Kept so a thread that has not seen this set yet can compile its own copy.
    patterns: Vec<String>,
}

impl PatternSet {
    pub fn new(items: Vec<(GlobalRuleRef, String)>) -> Result<Self, String> {
        if items.is_empty() {
            return Ok(Self {
                id: usize::MAX,
                rule_refs: Vec::new(),
                patterns: Vec::new(),
            });
        }

        let (rule_refs, patterns): (Vec<_>, Vec<_>) = items.into_iter().unzip();
        let id = NEXT_PATTERN_SET_ID.fetch_add(1, Ordering::Relaxed);

        // Compile now so an invalid pattern is reported where the grammar is
        // loaded, not on the first line that happens to reach it — and keep the
        // result as this thread's copy rather than discarding it.
        let regset = compile(&patterns)?;
        LOCAL_REGSETS.with(|sets| sets.borrow_mut().insert(id, regset));

        Ok(Self {
            id,
            rule_refs,
            patterns,
        })
    }

    pub(crate) fn find_at(
        &self,
        text: &str,
        pos: usize,
        options: SearchOptions,
    ) -> Result<Option<PatternSetMatch>, String> {
        if self.patterns.is_empty() {
            return Ok(None);
        }

        LOCAL_REGSETS.with(|sets| {
            let mut sets = sets.borrow_mut();
            if !sets.contains_key(&self.id) {
                sets.insert(self.id, compile(&self.patterns)?);
            }
            let regset = &sets[&self.id];

            // We need to specify pos/text.len() because some regex might do lookbehind
            if let Some((pattern_index, captures)) = regset.captures_with_options(
                text,       // Full text (not sliced)
                pos,        // Start searching from this position
                text.len(), // Search to end of text
                onig::RegSetLead::Position,
                options,
            ) && let Some((match_start, match_end)) = captures.pos(0)
            {
                // Convert all capture positions (they're already absolute from captures_with_encoding)
                let capture_pos: Vec<Option<(usize, usize)>> =
                    (0..captures.len()).map(|i| captures.pos(i)).collect();

                return Ok(Some(PatternSetMatch {
                    rule_ref: self.rule_refs[pattern_index],
                    start: match_start,
                    end: match_end,
                    capture_pos,
                }));
            }

            Ok(None)
        })
    }
}

impl Debug for PatternSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PatternSet({} rules)", self.rule_refs.len())
    }
}
