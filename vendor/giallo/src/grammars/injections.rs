use std::fmt;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::scope::Scope;

/// Regex for tokenizing injection selectors (matches vscode-textmate exactly except for \* added)
static TOKEN_REGEX: LazyLock<onig::Regex> = LazyLock::new(|| {
    onig::Regex::new(r"([LR]:|[\w.:]+[\w\*.:\-]*|[,|\-()])").expect("Invalid selector regex")
});

// Only Left matters, Right is the same as no precedence. We keep both just for debug reasons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InjectionPrecedence {
    /// L: prefix
    Left,
    /// R: prefix
    Right,
}

/// A compiled injection selector matcher with priority
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledInjectionMatcher {
    matcher: SelectorMatcher,
    priority: Option<InjectionPrecedence>,
}

impl CompiledInjectionMatcher {
    #[inline]
    pub fn matches(&self, scope_stack: &[Scope]) -> bool {
        self.matcher.matches(scope_stack)
    }

    /// Returns the precedence (Left/Right) for this injection matcher
    #[inline]
    pub fn precedence(&self) -> InjectionPrecedence {
        self.priority.unwrap_or(InjectionPrecedence::Right)
    }
}

impl fmt::Debug for CompiledInjectionMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let selector = match self.priority {
            Some(InjectionPrecedence::Left) => format!("L:{}", self.matcher),
            Some(InjectionPrecedence::Right) => format!("R:{}", self.matcher),
            None => self.matcher.to_string(),
        };
        write!(f, "\"{}\"", selector)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectorMatcher {
    Scope(Scope),
    /// All matchers must succeed (space-separated)
    And(Vec<SelectorMatcher>),
    /// Any matcher can succeed (OR operation `|` or `,` separated)
    Or(Vec<SelectorMatcher>),
    /// Matcher must NOT succeed (NOT operation `-` prefix)
    Not(Box<SelectorMatcher>),
}

impl SelectorMatcher {
    /// Recursively evaluates this matcher against scope stack
    fn matches(&self, scope_stack: &[Scope]) -> bool {
        match self {
            SelectorMatcher::Scope(scope) => {
                // Single scope: check if ANY scope in stack is a match
                scope_stack
                    .iter()
                    .any(|&stack_scope| scope.is_prefix_of(stack_scope))
            }
            SelectorMatcher::And(matchers) => {
                // Sequential matching: each matcher must find a match at or after the previous match
                let mut start_index = 0;
                for matcher in matchers {
                    match matcher {
                        SelectorMatcher::Scope(scope) => {
                            // For individual scopes, check sequentially
                            let mut found = false;
                            for (i, scope_item) in scope_stack.iter().enumerate().skip(start_index)
                            {
                                if scope.is_prefix_of(*scope_item) {
                                    start_index = i + 1;
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                return false;
                            }
                        }
                        _ => {
                            // For compound matchers (OR, NOT), check against entire scope stack
                            if !matcher.matches(scope_stack) {
                                return false;
                            }
                            // For compound matchers, we don't advance position since they're global
                        }
                    }
                }
                true
            }
            SelectorMatcher::Or(matchers) => {
                // Any matcher can succeed
                matchers.iter().any(|m| m.matches(scope_stack))
            }
            SelectorMatcher::Not(matcher) => {
                // Matcher must NOT succeed
                !matcher.matches(scope_stack)
            }
        }
    }
}

impl fmt::Display for SelectorMatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectorMatcher::Scope(scope) => write!(f, "{}", scope),
            SelectorMatcher::And(matchers) => {
                let parts: Vec<String> = matchers.iter().map(|m| m.to_string()).collect();
                write!(f, "{}", parts.join(" "))
            }
            SelectorMatcher::Or(matchers) => {
                if matchers.len() == 1 {
                    // If there's only one item in Or, don't wrap in parentheses
                    write!(f, "{}", matchers[0])
                } else {
                    let parts: Vec<String> = matchers.iter().map(|m| m.to_string()).collect();
                    write!(f, "({})", parts.join(" | "))
                }
            }
            SelectorMatcher::Not(matcher) => {
                write!(f, "-{}", matcher)
            }
        }
    }
}

#[inline]
fn is_identifier(s: &str) -> bool {
    if s.is_empty() || s == "-" {
        return false;
    }

    s.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == ':' || c == '-' || c == '*'
    })
}

fn parse_inner_expression(tokens: &[&str], position: &mut usize) -> SelectorMatcher {
    let mut out = Vec::new();
    while let Some(m) = parse_conjunction(tokens, position) {
        out.push(m);
        if *position < tokens.len() && matches!(tokens[*position], "|" | ",") {
            *position += 1;
        } else {
            break;
        }
    }

    let mut deduplicated = Vec::new();
    for matcher in out {
        if !deduplicated.contains(&matcher) {
            deduplicated.push(matcher);
        }
    }

    if deduplicated.len() == 1 {
        deduplicated.pop().unwrap()
    } else {
        SelectorMatcher::Or(deduplicated)
    }
}

fn parse_operand(tokens: &[&str], position: &mut usize) -> Option<SelectorMatcher> {
    if *position >= tokens.len() {
        return None;
    }

    match tokens[*position] {
        "-" => {
            *position += 1;
            let negated = parse_operand(tokens, position)?;
            Some(SelectorMatcher::Not(Box::new(negated)))
        }
        "(" => {
            *position += 1;
            let inner = parse_inner_expression(tokens, position);
            if *position < tokens.len() && tokens[*position] == ")" {
                *position += 1;
            }
            Some(inner)
        }
        _ => {
            let mut scopes = vec![];

            while *position < tokens.len() && is_identifier(tokens[*position]) {
                let token = tokens[*position];
                let scope = if let Some(pos) = token.find(".*") {
                    let base = &token[..pos];
                    Scope::new(base.trim_end_matches("."))[0]
                } else {
                    Scope::new(token)[0]
                };

                if !scopes.contains(&scope) {
                    scopes.push(scope);
                }
                *position += 1;
            }

            match scopes.len() {
                0 => None,
                1 => Some(SelectorMatcher::Scope(scopes.pop().unwrap())),
                _ => Some(SelectorMatcher::And(
                    scopes.into_iter().map(SelectorMatcher::Scope).collect(),
                )),
            }
        }
    }
}

fn parse_conjunction(tokens: &[&str], position: &mut usize) -> Option<SelectorMatcher> {
    let mut matchers = Vec::new();

    while let Some(m) = parse_operand(tokens, position) {
        matchers.push(m);
    }

    match matchers.len() {
        0 => None,
        1 => matchers.into_iter().next(),
        _ => Some(SelectorMatcher::And(matchers)),
    }
}

/// Parse injection selector string into compiled matchers.
/// A selector can correspond to multiple matcher, each with their own optional priority
pub fn parse_injection_selector(selector: &str) -> Vec<CompiledInjectionMatcher> {
    let selector = selector.trim();
    if selector.is_empty() {
        return Vec::new();
    }

    let tokens: Vec<_> = TOKEN_REGEX
        .find_iter(selector)
        .map(|(start, end)| &selector[start..end])
        .filter(|s| !s.is_empty())
        .collect();

    let mut position = 0;
    let mut res = Vec::new();

    let mut priority = None;
    while position < tokens.len() {
        let token = tokens[position];

        match token {
            "L:" => {
                priority = Some(InjectionPrecedence::Left);
                position += 1;
                continue;
            }
            "R:" => {
                priority = Some(InjectionPrecedence::Right);
                position += 1;
                continue;
            }
            _ => (),
        };

        if let Some(matcher) = parse_conjunction(&tokens, &mut position) {
            res.push(CompiledInjectionMatcher { matcher, priority });
            priority = None;
            if position < tokens.len() && tokens[position] == "," {
                position += 1;
            } else {
                break;
            }
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::{assert_debug_snapshot, with_settings};

    #[test]
    fn test_parse_injection_selector_snapshots() {
        let test_cases = vec![
            // Simple patterns
            "L:text.html.markdown", // mermaid.json
            "L:text.html -comment", // angular-template.json
            "text.html",
            "L:meta.decorator.ts -comment -text.html", // angular-inline-template.json
            // Complex negations
            "L:text.pug -comment -string.comment, L:text.html.derivative -comment.block", // vue-interpolations.json
            "L:meta.tag -meta.attribute -meta.ng-binding -entity.name.tag.pug", // vue-directives.json
            // Parenthesized groups with OR operators
            "L:(meta.script.svelte | meta.style.svelte) (meta.lang.js | meta.lang.javascript) - (meta source)", // svelte.json
            "L:(source.ts, source.js, source.coffee)", // svelte.json
            // Specific contexts
            "L:meta.script.svelte - meta.lang - (meta source)", // svelte.json
            "L:(meta.script.astro) (meta.lang.json) - (meta source)", // astro.json
            "text.html.php.blade - (meta.embedded | meta.tag | comment.block.blade), L:(text.html.php.blade meta.tag - (comment.block.blade | meta.embedded.block.blade)), L:(source.js.embedded.html - (comment.block.blade | meta.embedded.block.blade))", // blade.json
            "R:text.html - (comment.block, text.html meta.embedded, meta.tag.*.*.html, meta.tag.*.*.*.html, meta.tag.*.*.*.*.html)",
            "L:source.css -comment, L:source.postcss -comment, L:source.sass -comment, L:source.stylus -comment",
            // es-tag-css.json
            "L:source.js -comment -string, L:source.js -comment -string, L:source.jsx -comment -string,  L:source.js.jsx -comment -string, L:source.ts -comment -string, L:source.tsx -comment -string, L:source.rescript -comment -string, L:source.vue -comment -string, L:source.svelte -comment -string, L:source.php -comment -string, L:source.rescript -comment -string",
        ];

        for (i, test_case) in test_cases.into_iter().enumerate() {
            let result = parse_injection_selector(test_case);
            with_settings!({description => test_case}, {
                assert_debug_snapshot!(format!("injection_{i}"), result);
            })
        }
    }

    #[test]
    fn can_match_scopes() {
        // (selector, scope_names, expected)
        let test_cases = vec![
            // === Simple Scope Matching ===
            ("text.html", vec!["text.html"], true),
            ("text.html", vec!["text.html.markdown"], true), // prefix match
            ("text.html", vec!["source.js"], false),
            ("text.html", vec!["source.js", "text.html"], true), // found in stack
            ("comment", vec!["comment.line.double-slash"], true), // prefix match
            // === Sequential Scope Matching (AND) ===
            ("text.html meta.tag", vec!["text.html", "meta.tag"], true),
            (
                "text.html meta.tag",
                vec!["text.html", "meta.function", "meta.tag"],
                true,
            ), // with intermediate
            ("text.html meta.tag", vec!["text.html"], false), // missing meta.tag
            ("text.html meta.tag", vec!["meta.tag"], false),  // missing text.html
            ("text.html meta.tag", vec!["meta.tag", "text.html"], false), // wrong order
            (
                "source.js comment",
                vec!["source.js", "meta.function", "comment.line"],
                true,
            ),
            // === NOT Operations ===
            ("text.html -comment", vec!["text.html"], true),
            (
                "text.html -comment",
                vec!["text.html", "comment.block"],
                false,
            ),
            ("text.html -comment", vec!["source.js"], false), // no text.html
            ("comment -comment.block", vec!["comment.line"], true),
            ("comment -comment.block", vec!["comment.block"], false),
            // === Parenthesized Groups ===
            ("(meta.script | meta.style)", vec!["meta.script"], true),
            ("(meta.script | meta.style)", vec!["meta.style"], true),
            ("(meta.script | meta.style)", vec!["meta.tag"], false),
            (
                "(source.js | source.ts) comment",
                vec!["source.js", "comment.line"],
                true,
            ),
            (
                "(source.js | source.ts) comment",
                vec!["source.py", "comment.line"],
                false,
            ),
            // === Complex Boolean Logic ===
            (
                "L:(meta.script.svelte | meta.style.svelte) (meta.lang.js | meta.lang.javascript) - (meta source)",
                vec!["meta.script.svelte", "meta.lang.js"],
                true,
            ),
            (
                "L:(meta.script.svelte | meta.style.svelte) (meta.lang.js | meta.lang.javascript) - (meta source)",
                vec!["meta.style.svelte", "meta.lang.javascript"],
                true,
            ),
            (
                "L:(meta.script.svelte | meta.style.svelte) (meta.lang.js | meta.lang.javascript) - (meta source)",
                vec![
                    "meta.script.svelte",
                    "meta.lang.js",
                    "meta.embedded",
                    "source.js",
                ],
                false, // has "source" which contains "meta source"
            ),
            (
                "L:(meta.script.svelte | meta.style.svelte) (meta.lang.js | meta.lang.javascript) - (meta source)",
                vec!["meta.tag", "meta.lang.js"],
                false, // missing script.svelte or style.svelte
            ),
            // === Precedence Prefixes ===
            ("L:text.html", vec!["text.html"], true),
            ("R:text.html", vec!["text.html"], true),
            ("L:source.js -comment", vec!["source.js"], true),
            (
                "R:source.js -comment",
                vec!["source.js", "comment.block"],
                false,
            ),
            // === Real-world Examples from Snapshot Tests ===
            ("L:text.html.markdown", vec!["text.html.markdown"], true),
            ("L:text.html -comment", vec!["text.html"], true),
            (
                "L:text.html -comment",
                vec!["text.html", "comment.line"],
                false,
            ),
            (
                "L:meta.decorator.ts -comment -text.html",
                vec!["meta.decorator.ts"],
                true,
            ),
            (
                "L:meta.decorator.ts -comment -text.html",
                vec!["meta.decorator.ts", "comment.block"],
                false,
            ),
            (
                "L:meta.decorator.ts -comment -text.html",
                vec!["meta.decorator.ts", "text.html"],
                false,
            ),
            // === Edge Cases ===
            ("text.html", vec![], false), // empty scope stack
        ];

        for (selector_str, scope_names, expected) in test_cases {
            let matchers = parse_injection_selector(selector_str);
            let scope_stack: Vec<Scope> =
                scope_names.iter().map(|name| Scope::new(name)[0]).collect();

            println!("{selector_str} {matchers:?}");

            let result = matchers.iter().any(|x| x.matches(&scope_stack));

            assert_eq!(
                result, expected,
                "Selector '{}' with scopes {:?}: expected {}, got {}",
                selector_str, scope_names, expected, result
            );
        }
    }
}
