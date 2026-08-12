use std::collections::BTreeMap;
use std::fs::File;
use std::ops::Deref;
use std::path::Path;

use serde::{Deserialize, Deserializer, Serialize};

use crate::error::GialloResult;

/// per vscode-textmate:
///  Allowed values:
///  * Scope Name, e.g. `source.ts`
///  * Top level scope reference, e.g. `source.ts#entity.name.class`
///  * Relative scope reference, e.g. `#entity.name.class`
///  * self, e.g. `$self`
///  * base, e.g. `$base`
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Reference {
    // Itself
    Self_,
    // The base grammar the user asked for even if we are in another grammar.
    // For example, if we are rendering Markdown and then switch to Python for the codeblock, if the
    // Python grammar include $base, it will actually include the root rule of Markdown grammar.
    Base,
    Local(String),
    OtherComplete(String),
    OtherSpecific(String, String),
}

impl Reference {
    pub fn is_local(&self) -> bool {
        matches!(self, Reference::Local(_))
    }
}

impl From<&str> for Reference {
    fn from(value: &str) -> Self {
        match value {
            "$self" => Self::Self_,
            "$base" => Self::Base,
            s if s.starts_with('#') => Self::Local(s[1..].to_string()),
            s if s.contains('#') => {
                let (scope, rule) = s.split_once('#').unwrap();
                Self::OtherSpecific(scope.to_string(), rule.to_string())
            }
            _ => Self::OtherComplete(value.to_string()),
        }
    }
}

/// Custom deserializer for the include field that parses string references into Reference enum
fn deserialize_reference<'de, D>(deserializer: D) -> Result<Option<Reference>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_string = Option::<String>::deserialize(deserializer)?;
    Ok(opt_string.map(|s| Reference::from(s.as_str())))
}

/// applyEndPatternLast is sometimes an integer or a bool
/// We only want them as bool
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum BoolOrNumber {
    Bool(bool),
    Number(u8),
}

/// Custom deserializer that handles both boolean and number (0/1) formats
/// This fixes compatibility with grammars that use numbers for boolean fields
fn bool_or_number<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match BoolOrNumber::deserialize(deserializer)? {
        BoolOrNumber::Bool(b) => Ok(b),
        BoolOrNumber::Number(0) => Ok(false),
        BoolOrNumber::Number(1) => Ok(true),
        BoolOrNumber::Number(x) => Err(serde::de::Error::custom(format!(
            "expected bool, 0, or 1, got {x}"
        ))),
    }
}

/// Captures in TM grammars is represented as a dict {[number: str]: Rule}. Sometimes the value is
/// array as well
/// {
///   "captures": {
///     "0": {"name": "entire.match"},
///     "1": {"name": "first.group"},
///     "2": {"name": "second.group"}
///   }
/// }
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Captures(pub(crate) BTreeMap<usize, RawRule>);

impl Deref for Captures {
    type Target = BTreeMap<usize, RawRule>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Helper enum for deserializing captures in both object and array formats
#[derive(Deserialize)]
#[serde(untagged)]
enum CapturesFormat {
    Object(BTreeMap<String, RawRule>),
    Array(Vec<RawRule>),
}

impl<'de> Deserialize<'de> for Captures {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut out = BTreeMap::new();
        // Try to deserialize as our supported formats, but handle the case where it might be empty/null
        match CapturesFormat::deserialize(deserializer) {
            Ok(captures_format) => {
                match captures_format {
                    CapturesFormat::Object(string_map) => {
                        for (key, value) in string_map {
                            // anything not a number is a bug, just skip them
                            // currently only for XML syntax https://github.com/microsoft/vscode/pull/269766
                            if let Ok(idx) = key.parse::<usize>() {
                                out.insert(idx, value);
                            }
                        }
                    }
                    CapturesFormat::Array(array) => {
                        for (idx, value) in array.into_iter().enumerate() {
                            out.insert(idx, value);
                        }
                    }
                }

                Ok(Captures(out))
            }
            Err(_) => {
                // If deserialization fails, just return an empty Captures
                // This handles cases like null, empty strings, or unexpected formats
                Ok(Captures(out))
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawRuleValue {
    Vec(Vec<RawRule>),
    Single(Box<RawRule>),
}

/// Custom deserializer for repository HashMap that handles values that might be single rules or arrays of rules
fn deserialize_repository_map<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, RawRule>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_map = BTreeMap::<String, RawRuleValue>::deserialize(deserializer)?;
    let mut result = BTreeMap::new();

    let default = RawRule::default();

    for (key, val) in raw_map {
        let mut rule = match val {
            RawRuleValue::Vec(rules) => RawRule {
                patterns: rules,
                ..Default::default()
            },
            RawRuleValue::Single(rule) => *rule,
        };

        // Some grammars have empty patterns but still decide to put [{}] in there, which messes
        // up our logic later so we filter empty patterns out
        // eg berry
        //     "comment-block": {
        //       "begin": "#-",
        //       "end": "-#",
        //       "name": "comment.berry",
        //       "patterns": [
        //         {
        //         }
        //       ]
        //     },
        rule.patterns.retain(|x| x != &default);
        result.insert(key, rule);
    }

    Ok(result)
}

/// Unified rule structure that represents all possible TextMate grammar patterns
/// This will be split when compiling the grammar
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RawRule {
    #[serde(deserialize_with = "deserialize_reference")]
    pub include: Option<Reference>,

    pub name: Option<String>,
    pub content_name: Option<String>,

    #[serde(rename = "match")]
    pub match_: Option<String>,
    pub captures: Captures,

    pub begin: Option<String>,
    pub begin_captures: Captures,

    pub end: Option<String>,
    pub end_captures: Captures,

    #[serde(rename = "while")]
    pub while_: Option<String>,
    pub while_captures: Captures,

    pub patterns: Vec<RawRule>,
    #[serde(deserialize_with = "deserialize_repository_map")]
    pub repository: BTreeMap<String, RawRule>,

    #[serde(deserialize_with = "bool_or_number")]
    pub apply_end_pattern_last: bool,
}

/// Top-level structure representing a complete TextMate JSON grammar
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct RawGrammar {
    /// Human-readable name of the language
    /// Example: "JavaScript", "TypeScript", "Rust"
    pub name: String,
    /// Optional alternative display name
    /// Example: "JavaScript (ES6)", "TypeScript React"
    #[serde(default)]
    pub display_name: Option<String>,
    /// File extensions this grammar applies to
    /// Example: ["js", "jsx", "mjs"] for JavaScript
    #[serde(default)]
    pub file_types: Vec<String>,
    /// Unique identifier for this grammar's scope
    /// Example: "source.js", "text.html.markdown", "source.rust"
    pub scope_name: String,
    /// Named pattern definitions that can be referenced by includes
    /// Key is the repository name, value is the pattern(s)
    #[serde(default, deserialize_with = "deserialize_repository_map")]
    pub repository: BTreeMap<String, RawRule>,
    /// Root patterns that define the top-level structure
    /// These patterns are applied first when tokenizing
    #[serde(default)]
    pub patterns: Vec<RawRule>,
    /// Language injection patterns for embedding languages
    /// Maps selectors to patterns for injecting this grammar into others
    #[serde(default)]
    pub injections: BTreeMap<String, RawRule>,
    /// CSS selector defining where injections should occur
    /// Example: "source.js meta.embedded.block.sql"
    #[serde(default)]
    pub injection_selector: Option<String>,
    /// Restrict injections to those grammars
    #[serde(default)]
    pub inject_to: Vec<String>,
}

impl RawGrammar {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> GialloResult<Self> {
        let file = File::open(&path)?;
        let raw_grammar = serde_json::from_reader(&file)?;
        Ok(raw_grammar)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn can_parse_all_grammars() {
        let entries = fs::read_dir("grammars-themes/packages/tm-grammars/grammars")
            .expect("Failed to read grammars directory");

        for entry in entries {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            assert!(RawGrammar::load_from_file(&path).is_ok());
        }
    }

    #[test]
    fn can_parse_references() {
        let test_cases = vec![
            // Local references - most common pattern
            ("#value", Reference::Local("value".to_string())),
            ("#expressions", Reference::Local("expressions".to_string())),
            ("#comments", Reference::Local("comments".to_string())),
            ("#objectkey", Reference::Local("objectkey".to_string())),
            (
                "#stringcontent",
                Reference::Local("stringcontent".to_string()),
            ),
            // Local references with dots - found in applescript.json, lua.json
            ("#blocks.tell", Reference::Local("blocks.tell".to_string())),
            (
                "#blocks.repeat",
                Reference::Local("blocks.repeat".to_string()),
            ),
            (
                "#emmydoc.type",
                Reference::Local("emmydoc.type".to_string()),
            ),
            (
                "#built-in.constant",
                Reference::Local("built-in.constant".to_string()),
            ),
            (
                "#attributes.considering-ignoring",
                Reference::Local("attributes.considering-ignoring".to_string()),
            ),
            (
                "#comments.nested",
                Reference::Local("comments.nested".to_string()),
            ),
            // Self and base references
            ("$self", Reference::Self_),
            ("$base", Reference::Base),
            // Complete scope references - should be OtherComplete
            (
                "source.js",
                Reference::OtherComplete("source.js".to_string()),
            ),
            (
                "source.java",
                Reference::OtherComplete("source.java".to_string()),
            ),
            (
                "source.json",
                Reference::OtherComplete("source.json".to_string()),
            ),
            (
                "text.html.basic",
                Reference::OtherComplete("text.html.basic".to_string()),
            ),
            (
                "source.tsx",
                Reference::OtherComplete("source.tsx".to_string()),
            ),
            (
                "source.css",
                Reference::OtherComplete("source.css".to_string()),
            ),
            // Specific scope references - scope#rule pattern
            (
                "source.tsx#template-substitution-element",
                Reference::OtherSpecific(
                    "source.tsx".to_string(),
                    "template-substitution-element".to_string(),
                ),
            ),
            (
                "source.ts#expression",
                Reference::OtherSpecific("source.ts".to_string(), "expression".to_string()),
            ),
            (
                "text.html.basic#core-minus-invalid",
                Reference::OtherSpecific(
                    "text.html.basic".to_string(),
                    "core-minus-invalid".to_string(),
                ),
            ),
            (
                "source.css#property-names",
                Reference::OtherSpecific("source.css".to_string(), "property-names".to_string()),
            ),
            (
                "source.json#value",
                Reference::OtherSpecific("source.json".to_string(), "value".to_string()),
            ),
            // Edge cases
            ("", Reference::OtherComplete("".to_string())),
            ("simple", Reference::OtherComplete("simple".to_string())),
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                Reference::from(input),
                expected,
                "Failed to parse reference: {}",
                input
            );
        }
    }
}
