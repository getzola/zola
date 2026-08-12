use serde::{Deserialize, Serialize};

use crate::scope::Scope;

/// Represents a parent scope requirement in a theme selector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Parent {
    /// Parent scope that can appear anywhere up the scope stack
    /// `Anywhere(source.js)` from "source.js meta.function" - can have scopes between
    Anywhere(Scope),
    /// Parent scope that must be the immediate parent (child combinator `>`)
    /// `Direct(meta.function)` from "meta.function > string" - must be immediate parent
    Direct(Scope),
}

/// A parsed theme selector that is used to match against scope stacks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeSelector {
    /// The target scope to match (rightmost in the selector string)
    pub target_scope: Scope,
    /// Required parent scopes from right to left of the selector
    pub parent_scopes: Vec<Parent>,
}

impl ThemeSelector {
    pub fn new(target_scope: Scope, parent_scopes: Vec<Parent>) -> Self {
        Self {
            target_scope,
            parent_scopes,
        }
    }

    /// Checks if this selector matches the given scope stack.
    ///
    /// This implements the VSCode-TextMate scope matching algorithm:
    /// 1. The target scope must match the last scope in the stack
    /// 2. All parent scope requirements must be satisfied walking up the stack
    /// 3. `Parent::Anywhere` can skip intermediate scopes
    /// 4. `Parent::Direct` requires immediate parent relationship
    pub fn matches(&self, scope_stack: &[Scope]) -> bool {
        // Empty stack cannot match anything
        if scope_stack.is_empty() {
            return false;
        }

        // Check if target scope matches the innermost scope (last in stack)
        let (last, mut rest) = scope_stack.split_last().unwrap();
        if !self.target_scope.is_prefix_of(*last) {
            return false;
        }

        // If no parent requirements, we're done
        if self.parent_scopes.is_empty() {
            return true;
        }

        for (parent_idx, required_parent) in self.parent_scopes.iter().enumerate() {
            let is_last_parent = parent_idx == self.parent_scopes.len() - 1;

            match required_parent {
                Parent::Direct(parent_scope) => {
                    // Direct parent must match the last remaining parent
                    match rest.split_last() {
                        Some((last, r)) if parent_scope.is_prefix_of(*last) => {
                            rest = r;
                        }
                        _ => return false, // No match or no more parents
                    }
                }
                Parent::Anywhere(parent_scope) => {
                    // Find this parent anywhere in remaining parents (from end to start)
                    match rest
                        .iter()
                        .rposition(|&scope| parent_scope.is_prefix_of(scope))
                    {
                        Some(pos) => {
                            // Consume all parents up to and including this match
                            rest = &rest[..pos];
                        }
                        None => return false, // Required parent not found
                    }
                }
            }

            // Check if we have more requirements but no more parents
            if rest.is_empty() && !is_last_parent {
                return false;
            }
        }

        true
    }
}

/// Parses a theme selector string into a structured ThemeSelector.
///
/// # Selector Format
/// - Scopes are separated by whitespace: `"source.js meta.function string"`
/// - Child combinator `>` creates direct parent requirement: `"parent > child"`
/// - Target scope is always the rightmost non-`>` token
/// - Parent scopes are processed left to right
///
/// Returns `None` if the selector string is invalid or empty
pub fn parse_selector(input: &str) -> Option<ThemeSelector> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let (last, rest) = parts.split_last().unwrap();
    if *last == ">" {
        return None;
    }
    let target_scope = Scope::new(last)[0];

    let mut parents = Vec::new();
    let mut is_direct = false;
    for part in rest.iter().rev() {
        if *part == ">" {
            is_direct = true;
            continue;
        }
        let parent_scope = Scope::new(part)[0];
        parents.push(if is_direct {
            Parent::Direct(parent_scope)
        } else {
            Parent::Anywhere(parent_scope)
        });
        is_direct = false;
    }

    Some(ThemeSelector::new(target_scope, parents))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_selector() {
        let test_cases = vec![
            (
                "comment",
                ThemeSelector {
                    target_scope: Scope::new("comment").into_iter().next().unwrap(),
                    parent_scopes: vec![],
                },
            ),
            (
                "source.js meta.function string",
                ThemeSelector {
                    target_scope: Scope::new("string").into_iter().next().unwrap(),
                    parent_scopes: vec![
                        Parent::Anywhere(Scope::new("meta.function").into_iter().next().unwrap()), // deepest parent
                        Parent::Anywhere(Scope::new("source.js").into_iter().next().unwrap()), // shallowest parent
                    ],
                },
            ),
            (
                "meta.function > string",
                ThemeSelector {
                    target_scope: Scope::new("string").into_iter().next().unwrap(),
                    parent_scopes: vec![Parent::Direct(
                        Scope::new("meta.function").into_iter().next().unwrap(),
                    )],
                },
            ),
            (
                "source.js meta.function > string.quoted",
                ThemeSelector {
                    target_scope: Scope::new("string.quoted").into_iter().next().unwrap(),
                    parent_scopes: vec![
                        Parent::Direct(Scope::new("meta.function").into_iter().next().unwrap()), // deepest parent
                        Parent::Anywhere(Scope::new("source.js").into_iter().next().unwrap()), // shallowest parent
                    ],
                },
            ),
            (
                "source > meta > string",
                ThemeSelector {
                    target_scope: Scope::new("string").into_iter().next().unwrap(),
                    parent_scopes: vec![
                        Parent::Direct(Scope::new("meta").into_iter().next().unwrap()), // deepest parent
                        Parent::Direct(Scope::new("source").into_iter().next().unwrap()), // shallowest parent
                    ],
                },
            ),
            (
                "  source.js   meta.function  >   string  ",
                ThemeSelector {
                    target_scope: Scope::new("string").into_iter().next().unwrap(),
                    parent_scopes: vec![
                        Parent::Direct(Scope::new("meta.function").into_iter().next().unwrap()), // deepest parent
                        Parent::Anywhere(Scope::new("source.js").into_iter().next().unwrap()), // shallowest parent
                    ],
                },
            ),
        ];

        for (input, expected) in test_cases {
            let result = parse_selector(input).unwrap();
            assert_eq!(result, expected, "Mismatch for input: '{}'", input);
        }
    }

    fn create_scope_stack(scope_names: &[&str]) -> Vec<Scope> {
        scope_names
            .iter()
            .map(|name| Scope::new(name).into_iter().next().unwrap())
            .collect()
    }

    #[test]
    fn test_selector_matches() {
        let test_cases = vec![
            // (selector_string, scope_stack, expected_match)

            // Simple selector tests
            ("comment", vec!["source.js", "comment.line"], true),
            ("comment", vec!["source.js", "string.quoted"], false),
            ("comment", vec!["comment"], true),
            (
                "comment.line",
                vec!["source.js", "comment.line.double-slash"],
                true,
            ),
            // Parent selector tests (Anywhere)
            (
                "source.js string",
                vec!["source.js", "meta.function", "string.quoted"],
                true,
            ),
            ("source.js string", vec!["source.js", "string.quoted"], true),
            (
                "source.js string",
                vec!["source.py", "string.quoted"],
                false,
            ),
            (
                "source string",
                vec!["source.js", "meta.function", "string.quoted"],
                true,
            ),
            (
                "meta.function string",
                vec!["source.js", "meta.function.arrow", "string.quoted"],
                true,
            ),
            // Child combinator tests (Direct)
            (
                "meta.function > string",
                vec!["source.js", "meta.function", "string.quoted"],
                true,
            ),
            (
                "meta.function > string",
                vec!["source.js", "meta.function", "punctuation", "string.quoted"],
                false,
            ),
            (
                "meta > string",
                vec!["source.js", "meta.function", "string.quoted"],
                true,
            ),
            // Mixed combinator tests
            (
                "source.js meta.function > string.quoted",
                vec!["source.js", "meta.function", "string.quoted"],
                true,
            ),
            (
                "source.js meta.function > string.quoted",
                vec!["source.js", "meta.function", "punctuation", "string.quoted"],
                false,
            ),
            (
                "source.js meta.function > string.quoted",
                vec!["source.py", "meta.function", "string.quoted"],
                false,
            ),
            // Multiple direct parents
            (
                "source > meta > string",
                vec!["source.js", "meta.function", "string.quoted"],
                true,
            ),
            (
                "source > meta > string",
                vec!["source.js", "punctuation", "meta.function", "string.quoted"],
                false,
            ),
            // Edge cases
            ("comment", vec![], false),                      // empty stack
            ("source.js comment", vec!["source.js"], false), // target not found
            ("meta.function > string", vec!["meta.function"], false), // no target found
            // Complex nesting
            (
                "source.js meta string",
                vec![
                    "source.js",
                    "meta.function",
                    "meta.parameter",
                    "string.quoted",
                ],
                true,
            ),
            (
                "source.js meta.function string",
                vec!["source.js", "meta.class", "meta.function", "string.quoted"],
                true,
            ),
        ];

        for (selector_str, scope_names, expected) in test_cases {
            let selector = parse_selector(selector_str)
                .unwrap_or_else(|| panic!("Failed to parse selector: '{}'", selector_str));
            let scope_stack = create_scope_stack(&scope_names);
            let result = selector.matches(&scope_stack);

            assert_eq!(
                result, expected,
                "Selector '{}' matching scope stack {:?}: expected {}, got {}",
                selector_str, scope_names, expected, result
            );
        }
    }
}
