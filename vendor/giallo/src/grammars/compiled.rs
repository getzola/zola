use std::collections::{BTreeMap, HashMap};
use std::ops::{Index, IndexMut};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::grammars::injections::{CompiledInjectionMatcher, parse_injection_selector};
use crate::grammars::raw::{Captures, RawGrammar, RawRule, Reference};
use crate::grammars::regex::Regex;
use crate::scope::Scope;

static CAPTURING_NAME_RE: LazyLock<onig::Regex> =
    LazyLock::new(|| onig::Regex::new(r"\$(\d+)|\${(\d+):\/(downcase|upcase)}").unwrap());

fn has_captures(pat: Option<&str>) -> bool {
    if let Some(p) = pat {
        CAPTURING_NAME_RE.find(p).is_some()
    } else {
        false
    }
}

fn has_backreferences(pattern: &str) -> bool {
    for i in 1..=9 {
        let backref = format!("\\{}", i);
        if pattern.contains(&backref) {
            return true;
        }
    }
    false
}

fn scopes_from_name(name: &Option<String>, name_is_capturing: bool) -> Vec<Scope> {
    if name_is_capturing {
        Vec::new()
    } else if let Some(name_ref) = name.as_ref() {
        Scope::new(name_ref)
    } else {
        Vec::new()
    }
}

pub fn replace_captures(
    original_name: &str,
    text: &str,
    captures_pos: &[Option<(usize, usize)>],
) -> String {
    CAPTURING_NAME_RE
        .replace_all(original_name, |caps: &onig::Captures| {
            let capture_num = caps
                .at(1)
                .or_else(|| caps.at(2))
                .unwrap_or("0")
                .parse::<usize>()
                .unwrap_or(0);
            let command = caps.at(3);

            if let Some(Some((start, end))) = captures_pos.get(capture_num) {
                // Remove leading dots that would make the selector invalid
                let result = text[*start..*end].trim_start_matches('.').to_string();
                match command {
                    Some("downcase") => result.to_lowercase(),
                    Some("upcase") => result.to_uppercase(),
                    _ => result,
                }
            } else if let Some(None) = captures_pos.get(capture_num) {
                // Capture exists but didn't match (None capture), replace with empty string
                String::new()
            } else {
                // Invalid capture bounds (index out of bounds), return original match
                caps.at(0).unwrap().to_string()
            }
        })
        .to_string()
}

fn process_scope_name(
    name: &Option<String>,
    is_capturing: bool,
    input: &str,
    captures_pos: &[Option<(usize, usize)>],
) -> Option<String> {
    match name {
        Some(name) if is_capturing => Some(replace_captures(name, input, captures_pos)),
        Some(name) => Some(name.clone()),
        None => None,
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GrammarId(pub u16);

impl GrammarId {
    pub(crate) fn as_index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RuleId(pub u16);

impl RuleId {
    pub(crate) fn as_index(self) -> usize {
        self.0 as usize
    }
}

pub const ROOT_RULE_ID: RuleId = RuleId(0);
pub const END_RULE_ID: RuleId = RuleId(u16::MAX);
const TEMP_RULE_ID: RuleId = RuleId(u16::MAX - 1);

/// A rule reference that works across the whole registry.
/// We use that to be able to refer to multiple grammars while tokenizing (eg HTML -> JS)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct GlobalRuleRef {
    pub(crate) grammar: GrammarId,
    pub(crate) rule: RuleId,
}

pub const NO_OP_GLOBAL_RULE_REF: GlobalRuleRef = GlobalRuleRef {
    grammar: GrammarId(u16::MAX - 1),
    rule: TEMP_RULE_ID,
};

pub const BASE_GLOBAL_RULE_REF: GlobalRuleRef = GlobalRuleRef {
    grammar: GrammarId(u16::MAX - 2),
    rule: ROOT_RULE_ID,
};

pub const PRE_CROSS_LINKING_RULE_REF: GlobalRuleRef = GlobalRuleRef {
    grammar: GrammarId(u16::MAX - 3),
    rule: TEMP_RULE_ID,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RegexId(u16);

impl RegexId {
    pub(crate) fn as_index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RepositoryId(u16);

impl RepositoryId {
    pub(crate) fn as_index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Repository(BTreeMap<String, RuleId>);

impl Repository {
    /// Look up a rule by name in this repository
    pub fn get(&self, name: &str) -> Option<&RuleId> {
        self.0.get(name)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct RepositoryStack {
    // Biggest stack seem to be 4 but it's cheap so let's double it
    stack: [Option<RepositoryId>; 8],
    len: u8,
}

impl RepositoryStack {
    pub fn push(mut self, id: RepositoryId) -> Self {
        self.stack[self.len as usize] = Some(id);
        self.len += 1;
        self
    }

    pub fn pop(mut self) -> (RepositoryId, Self) {
        let popped = self.stack[self.len as usize - 1].take().unwrap();
        self.len -= 1;
        (popped, self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Match {
    pub id: GlobalRuleRef,
    // some match only care about the captures and thus don't have a name themselves
    pub name: Option<String>,
    pub name_is_capturing: bool,
    pub scopes: Vec<Scope>,
    /// The regex ID for this match rule.
    /// None for scope-only rules (e.g., capture groups that only assign scopes like
    /// punctuation.definition.string.begin without their own pattern to match)
    pub regex_id: Option<RegexId>,
    pub captures: Vec<Option<GlobalRuleRef>>,
    pub repository_stack: RepositoryStack,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct IncludeOnly {
    pub id: GlobalRuleRef,
    pub name: Option<String>,
    pub name_is_capturing: bool,
    pub scopes: Vec<Scope>,
    pub content_name: Option<String>,
    pub content_name_is_capturing: bool,
    pub content_scopes: Vec<Scope>,
    pub repository_stack: RepositoryStack,
    pub patterns: Vec<GlobalRuleRef>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BeginEnd {
    pub id: GlobalRuleRef,
    pub name: Option<String>,
    pub name_is_capturing: bool,
    pub scopes: Vec<Scope>,
    pub content_name: Option<String>,
    pub content_name_is_capturing: bool,
    pub content_scopes: Vec<Scope>,
    pub begin: RegexId,
    pub begin_captures: Vec<Option<GlobalRuleRef>>,
    pub end: RegexId,
    pub end_has_backrefs: bool,
    pub end_captures: Vec<Option<GlobalRuleRef>>,
    pub apply_end_pattern_last: bool,
    pub patterns: Vec<GlobalRuleRef>,
    pub repository_stack: RepositoryStack,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BeginWhile {
    pub id: GlobalRuleRef,
    pub name: Option<String>,
    pub name_is_capturing: bool,
    pub scopes: Vec<Scope>,
    pub content_name: Option<String>,
    pub content_name_is_capturing: bool,
    pub content_scopes: Vec<Scope>,
    pub begin: RegexId,
    pub begin_captures: Vec<Option<GlobalRuleRef>>,
    pub while_: RegexId,
    pub while_has_backrefs: bool,
    pub while_captures: Vec<Option<GlobalRuleRef>>,
    pub patterns: Vec<GlobalRuleRef>,
    pub repository_stack: RepositoryStack,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Rule {
    Match(Match),
    IncludeOnly(IncludeOnly),
    BeginEnd(BeginEnd),
    BeginWhile(BeginWhile),
    /// Used at compile time to indicate rules that were removed because not found
    Noop,
}

impl Rule {
    pub fn original_name(&self) -> Option<&str> {
        match self {
            Rule::Match(m) => m.name.as_deref(),
            Rule::IncludeOnly(m) => m.name.as_deref(),
            Rule::BeginEnd(m) => m.name.as_deref(),
            Rule::BeginWhile(m) => m.name.as_deref(),
            Rule::Noop => None,
        }
    }

    pub fn name(&self, input: &str, captures_pos: &[Option<(usize, usize)>]) -> Option<String> {
        let (name, is_capturing) = match self {
            Rule::Match(m) => (&m.name, m.name_is_capturing),
            Rule::IncludeOnly(i) => (&i.name, i.name_is_capturing),
            Rule::BeginEnd(b) => (&b.name, b.name_is_capturing),
            Rule::BeginWhile(b) => (&b.name, b.name_is_capturing),
            Rule::Noop => return None,
        };

        process_scope_name(name, is_capturing, input, captures_pos)
    }

    pub fn content_name(
        &self,
        input: &str,
        captures_pos: &[Option<(usize, usize)>],
    ) -> Option<String> {
        let (content_name, is_capturing) = match self {
            Rule::IncludeOnly(i) => (&i.content_name, i.content_name_is_capturing),
            Rule::BeginEnd(b) => (&b.content_name, b.content_name_is_capturing),
            Rule::BeginWhile(b) => (&b.content_name, b.content_name_is_capturing),
            _ => return None,
        };

        process_scope_name(content_name, is_capturing, input, captures_pos)
    }

    pub fn apply_end_pattern_last(&self) -> bool {
        match self {
            Rule::BeginEnd(b) => b.apply_end_pattern_last,
            _ => false,
        }
    }

    pub fn end_has_backrefs(&self) -> bool {
        match self {
            Rule::BeginEnd(b) => b.end_has_backrefs,
            Rule::BeginWhile(b) => b.while_has_backrefs,
            _ => false,
        }
    }

    fn patterns(&self) -> &[GlobalRuleRef] {
        match self {
            Rule::BeginEnd(b) => &b.patterns,
            Rule::BeginWhile(b) => &b.patterns,
            Rule::IncludeOnly(b) => &b.patterns,
            Rule::Match(_) | Rule::Noop => &[],
        }
    }

    pub fn has_patterns(&self) -> bool {
        !self.patterns().is_empty()
    }

    fn repository_stack(&self) -> RepositoryStack {
        match self {
            Rule::BeginEnd(b) => b.repository_stack,
            Rule::BeginWhile(b) => b.repository_stack,
            Rule::IncludeOnly(i) => i.repository_stack,
            Rule::Match(b) => b.repository_stack,
            Rule::Noop => RepositoryStack::default(),
        }
    }

    fn replace_pattern(&mut self, position: usize, rule_ref: GlobalRuleRef) {
        match self {
            Rule::BeginEnd(b) => b.patterns[position] = rule_ref,
            Rule::BeginWhile(b) => b.patterns[position] = rule_ref,
            Rule::IncludeOnly(b) => b.patterns[position] = rule_ref,
            Rule::Match(_) => (),
            Rule::Noop => (),
        }
    }

    fn has_only_missing_patterns(&self) -> bool {
        let patterns = self.patterns();
        if patterns.is_empty() {
            false
        } else {
            patterns.iter().all(|p| *p == NO_OP_GLOBAL_RULE_REF)
        }
    }

    /// Get name scopes, either pre-compiled or computed from captures
    pub fn get_name_scopes(
        &self,
        input: &str,
        captures_pos: &[Option<(usize, usize)>],
    ) -> Vec<Scope> {
        let (name_is_capturing, scopes) = match self {
            Rule::Match(m) => (m.name_is_capturing, &m.scopes),
            Rule::IncludeOnly(i) => (i.name_is_capturing, &i.scopes),
            Rule::BeginEnd(b) => (b.name_is_capturing, &b.scopes),
            Rule::BeginWhile(bw) => (bw.name_is_capturing, &bw.scopes),
            Rule::Noop => return Vec::new(),
        };

        if name_is_capturing {
            if let Some(name) = self.name(input, captures_pos) {
                Scope::new(&name)
            } else {
                Vec::new()
            }
        } else {
            scopes.clone()
        }
    }

    /// Get content scopes, either pre-compiled or computed from captures
    pub fn get_content_scopes(
        &self,
        input: &str,
        captures_pos: &[Option<(usize, usize)>],
    ) -> Vec<Scope> {
        let (content_name_is_capturing, content_scopes) = match self {
            Rule::IncludeOnly(i) => (i.content_name_is_capturing, &i.content_scopes),
            Rule::BeginEnd(b) => (b.content_name_is_capturing, &b.content_scopes),
            Rule::BeginWhile(bw) => (bw.content_name_is_capturing, &bw.content_scopes),
            Rule::Match(_) | Rule::Noop => return Vec::new(),
        };

        if content_name_is_capturing {
            if let Some(content_name) = self.content_name(input, captures_pos) {
                Scope::new(&content_name)
            } else {
                Vec::new()
            }
        } else {
            content_scopes.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct RefToReplace {
    rule_id: RuleId,
    index: usize,
    reference: Reference,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledGrammar {
    pub id: GrammarId,
    pub name: String,
    pub display_name: Option<String>,
    pub aliases: Vec<String>,
    pub scope_name: String,
    pub scope: Scope,
    pub file_types: Vec<String>,
    pub regexes: Vec<Regex>,
    pub rules: Vec<Rule>,
    pub repositories: Vec<Repository>,
    pub injections: Vec<(Vec<CompiledInjectionMatcher>, GlobalRuleRef)>,
    references: Vec<RefToReplace>,
    // The fields below are only set for injection grammars, eg grammars that are not meant to be
    // used by themselves
    pub injection_selector: Vec<CompiledInjectionMatcher>,
    pub inject_to: Vec<String>,
}

impl CompiledGrammar {
    pub(crate) fn from_raw_grammar(raw: RawGrammar, id: GrammarId) -> Self {
        let mut grammar = Self {
            id,
            name: raw.name,
            display_name: raw.display_name,
            aliases: Vec::new(),
            scope_name: raw.scope_name.clone(),
            scope: Scope::new(&raw.scope_name)[0],
            file_types: raw.file_types,
            regexes: Vec::new(),
            rules: Vec::new(),
            repositories: Vec::new(),
            injections: Vec::new(),
            injection_selector: raw
                .injection_selector
                .map(|x| parse_injection_selector(&x))
                .unwrap_or_default(),
            inject_to: raw.inject_to,
            references: Vec::new(),
        };

        let root_rule = RawRule {
            patterns: raw.patterns,
            repository: raw.repository,
            ..Default::default()
        };
        let root_rule_id = grammar.compile_rule(root_rule, RepositoryStack::default());
        assert_eq!(root_rule_id.as_index(), 0);

        // Compile injections
        for (selector, raw_rule) in raw.injections {
            let matchers = parse_injection_selector(&selector);
            let mut repo_stack = RepositoryStack::default();
            if !grammar.repositories.is_empty() {
                repo_stack = repo_stack.push(RepositoryId(0));
            }
            let rule_id = grammar.compile_rule(raw_rule, repo_stack);

            grammar.injections.push((
                matchers,
                GlobalRuleRef {
                    grammar: id,
                    rule: rule_id,
                },
            ));
        }

        // Resolve all Local references after compilation is complete
        grammar.resolve_local_references();

        grammar
    }

    fn compile_rule(&mut self, raw_rule: RawRule, repository_stack: RepositoryStack) -> RuleId {
        let local_id = RuleId(self.rules.len() as u16);
        let global_id = GlobalRuleRef {
            grammar: self.id,
            rule: local_id,
        };

        // push a no-op to reserve its spot
        self.rules.push(Rule::Noop);
        let name = raw_rule.name;

        // https://github.com/microsoft/vscode-textmate/blob/f03a6a8790af81372d0e81facae75554ec5e97ef/src/rule.ts#L389-L447
        let rule = if let Some(pat) = raw_rule.match_ {
            // vscode-textmate does filter regex with empty match patterns
            // eg in haxe
            //  {
            //    "match": "",
            //    "name": "constant.character.escape.hx"
            //  },
            if pat.is_empty() {
                Rule::Noop
            } else {
                let name_is_capturing = has_captures(name.as_deref());
                let scopes = scopes_from_name(&name, name_is_capturing);
                Rule::Match(Match {
                    id: global_id,
                    name_is_capturing,
                    name,
                    scopes,
                    regex_id: Some(self.compile_regex(pat).0),
                    captures: self.compile_captures(raw_rule.captures, repository_stack),
                    repository_stack,
                })
            }
        } else if let Some(begin_pat) = raw_rule.begin {
            let content_name = raw_rule.content_name;
            let apply_end_pattern_last = raw_rule.apply_end_pattern_last;
            if let Some(while_pat) = raw_rule.while_ {
                let (while_, while_has_backrefs) = self.compile_regex(while_pat);
                let patterns = self.compile_patterns(local_id, raw_rule.patterns, repository_stack);
                let name_is_capturing = has_captures(name.as_deref());
                let content_name_is_capturing = has_captures(content_name.as_deref());
                let scopes = scopes_from_name(&name, name_is_capturing);
                let content_scopes = scopes_from_name(&content_name, content_name_is_capturing);
                Rule::BeginWhile(BeginWhile {
                    id: global_id,
                    name_is_capturing,
                    name,
                    scopes,
                    content_name_is_capturing,
                    content_name,
                    content_scopes,
                    begin: self.compile_regex(begin_pat).0,
                    begin_captures: self.compile_captures(
                        // Some grammars use "captures" instead of "beginCaptures" for BeginEnd/BeginWhile rules
                        if !raw_rule.begin_captures.is_empty() {
                            raw_rule.begin_captures
                        } else {
                            raw_rule.captures.clone()
                        },
                        repository_stack,
                    ),
                    while_,
                    while_has_backrefs,
                    while_captures: self.compile_captures(
                        if !raw_rule.while_captures.is_empty() {
                            raw_rule.while_captures
                        } else {
                            raw_rule.captures
                        },
                        repository_stack,
                    ),
                    patterns,
                    repository_stack,
                })
            } else {
                // For rules with begin but no explicit end, use "\u{FFFF}" as default end pattern
                let end_pat = match raw_rule.end.as_deref() {
                    None => "\u{FFFF}",
                    Some(pattern) => pattern,
                };
                let (end, end_has_backrefs) = self.compile_regex(end_pat.to_string());
                let patterns = self.compile_patterns(local_id, raw_rule.patterns, repository_stack);
                let name_is_capturing = has_captures(name.as_deref());
                let content_name_is_capturing = has_captures(content_name.as_deref());
                let scopes = scopes_from_name(&name, name_is_capturing);
                let content_scopes = scopes_from_name(&content_name, content_name_is_capturing);
                Rule::BeginEnd(BeginEnd {
                    id: global_id,
                    name_is_capturing,
                    name,
                    scopes,
                    content_name_is_capturing,
                    content_name,
                    content_scopes,
                    begin: self.compile_regex(begin_pat).0,
                    begin_captures: self.compile_captures(
                        // Some grammars use "captures" instead of "beginCaptures" for BeginEnd/BeginWhile rules
                        if !raw_rule.begin_captures.is_empty() {
                            raw_rule.begin_captures
                        } else {
                            raw_rule.captures.clone()
                        },
                        repository_stack,
                    ),
                    end,
                    end_has_backrefs,
                    end_captures: self.compile_captures(
                        if !raw_rule.end_captures.is_empty() {
                            raw_rule.end_captures
                        } else {
                            raw_rule.captures
                        },
                        repository_stack,
                    ),
                    patterns,
                    apply_end_pattern_last,
                    repository_stack,
                })
            }
        } else {
            let repository_stack = if raw_rule.repository.is_empty() {
                repository_stack
            } else {
                let repo_id = self.compile_repository(raw_rule.repository, repository_stack);
                repository_stack.push(repo_id)
            };

            // Check if this is a scope-only rule (like a capture with just a name)
            if name.is_some() && raw_rule.patterns.is_empty() && raw_rule.include.is_none() {
                // This is a scope-only rule - create a Match rule with no regex
                // This handles captures that only assign scopes
                let name_is_capturing = has_captures(name.as_deref());
                let scopes = scopes_from_name(&name, name_is_capturing);
                Rule::Match(Match {
                    id: global_id,
                    name_is_capturing,
                    name,
                    scopes,
                    regex_id: None, // Scope-only rule (e.g., capture that only assigns scope)
                    captures: vec![],
                    repository_stack,
                })
            } else {
                // vscode-textmate does something funny here:
                // - if patterns are NOT present and includes are, it moves includes to patterns;
                // - however, if patterns ARE present, includes are ignored
                // https://github.com/microsoft/vscode-textmate/blob/f03a6a8790af81372d0e81facae75554ec5e97ef/src/rule.ts#L404
                let patterns = if raw_rule.patterns.is_empty() {
                    if let Some(reference) = raw_rule.include {
                        vec![RawRule {
                            include: Some(reference),
                            ..Default::default()
                        }]
                    } else {
                        raw_rule.patterns
                    }
                } else {
                    raw_rule.patterns
                };

                if patterns.is_empty() {
                    Rule::Noop
                } else {
                    let compiled_patterns =
                        self.compile_patterns(local_id, patterns, repository_stack);
                    let name_is_capturing = has_captures(name.as_deref());
                    let content_name_is_capturing = has_captures(raw_rule.content_name.as_deref());
                    let scopes = if !name_is_capturing {
                        if let Some(name_ref) = name.as_ref() {
                            Scope::new(name_ref)
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };
                    let content_scopes =
                        scopes_from_name(&raw_rule.content_name, content_name_is_capturing);

                    Rule::IncludeOnly(IncludeOnly {
                        id: global_id,
                        name_is_capturing,
                        name,
                        scopes,
                        content_name_is_capturing,
                        content_name: raw_rule.content_name,
                        content_scopes,
                        repository_stack,
                        patterns: compiled_patterns,
                    })
                }
            }
        };

        self.rules[local_id] = rule;
        local_id
    }

    fn compile_regex(&mut self, pattern: String) -> (RegexId, bool) {
        let regex_id = RegexId(self.regexes.len() as u16);
        let has_backrefs = has_backreferences(&pattern);
        let re = Regex::new(pattern);
        self.regexes.push(re);

        (regex_id, has_backrefs)
    }

    fn compile_repository(
        &mut self,
        raw_repository: BTreeMap<String, RawRule>,
        repository_stack: RepositoryStack,
    ) -> RepositoryId {
        let repo_id = RepositoryId(self.repositories.len() as u16);

        self.repositories.push(Repository::default());
        let stack = repository_stack.push(repo_id);

        let mut rules = BTreeMap::new();

        for (name, raw_rule) in raw_repository {
            rules.insert(name, self.compile_rule(raw_rule, stack));
        }

        self.repositories[repo_id] = Repository(rules);

        repo_id
    }

    fn compile_captures(
        &mut self,
        captures: Captures,
        repository_stack: RepositoryStack,
    ) -> Vec<Option<GlobalRuleRef>> {
        if captures.is_empty() {
            return Vec::new();
        }

        // mdc.json syntax has actually a 912 backref
        let max_capture = captures.keys().max().copied().unwrap_or_default();
        let mut out: Vec<_> = vec![None; max_capture + 1];

        for (key, rule) in captures.0 {
            let local_id = self.compile_rule(rule, repository_stack);
            let global_id = GlobalRuleRef {
                grammar: self.id,
                rule: local_id,
            };
            out[key] = Some(global_id);
        }

        out
    }

    /// Resolve all Local references after self compilation is complete.
    /// This must be called after all repositories are fully compiled.
    fn resolve_local_references(&mut self) {
        let references = std::mem::take(&mut self.references);
        let (local, external) = references.into_iter().partition(|x| x.reference.is_local());
        self.references = external;

        for rep in local {
            let rule = &mut self.rules[rep.rule_id];
            let stack = rule.repository_stack();
            let mut found = false;

            for repo_id in stack.stack.iter().filter(|x| x.is_some()).rev() {
                let repo = &self.repositories[repo_id.unwrap()];

                if let Reference::Local(name) = &rep.reference
                    && let Some(rule_id) = repo.get(name)
                {
                    found = true;
                    let global_ref = GlobalRuleRef {
                        grammar: self.id,
                        rule: *rule_id,
                    };
                    rule.replace_pattern(rep.index, global_ref);
                    break;
                }
            }

            if !found {
                #[cfg(feature = "debug")]
                log::warn!(
                    "Local reference '{:?}' not found in grammar {}",
                    rep.reference,
                    self.name
                );
                rule.replace_pattern(rep.index, NO_OP_GLOBAL_RULE_REF);
            }
        }

        self.remove_empty_rules();
    }

    /// We match the logic from vscode-textmate
    pub(crate) fn remove_empty_rules(&mut self) {
        loop {
            let mut empty_rules = Vec::new();
            for (i, rule) in self.rules.iter().enumerate() {
                if matches!(rule, Rule::Noop | Rule::Match(_)) {
                    continue;
                }

                if rule.has_only_missing_patterns() {
                    empty_rules.push(i);
                    continue;
                }

                let patterns = rule.patterns();
                if patterns.is_empty() {
                    continue;
                }

                let num_patterns = patterns.len();
                let mut num_noop = 0;
                for p in patterns {
                    if p.rule == TEMP_RULE_ID || p.grammar != self.id {
                        break;
                    }
                    if matches!(self.rules[p.rule], Rule::Noop) {
                        num_noop += 1;
                    }
                }
                if num_noop == num_patterns {
                    empty_rules.push(i);
                }
            }

            for i in &empty_rules {
                self.rules[*i] = Rule::Noop;
            }

            if empty_rules.is_empty() {
                break;
            }
        }
    }

    fn compile_patterns(
        &mut self,
        rule_id: RuleId,
        rules: Vec<RawRule>,
        repository_stack: RepositoryStack,
    ) -> Vec<GlobalRuleRef> {
        let mut out = vec![];

        for (index, r) in rules.into_iter().enumerate() {
            if let Some(reference) = r.include {
                // vscode ignores other rule contents is there's an include
                // https://github.com/microsoft/vscode-textmate/blob/f03a6a8790af81372d0e81facae75554ec5e97ef/src/rule.ts#L495

                match reference {
                    Reference::Base => out.push(BASE_GLOBAL_RULE_REF),
                    Reference::Self_ => {
                        out.push(GlobalRuleRef {
                            grammar: self.id,
                            rule: RuleId(0),
                        });
                    }
                    Reference::Local(_) => {
                        out.push(GlobalRuleRef {
                            grammar: self.id,
                            rule: TEMP_RULE_ID,
                        });
                        self.references.push(RefToReplace {
                            rule_id,
                            index,
                            reference,
                        });
                    }
                    Reference::OtherComplete(_) | Reference::OtherSpecific(_, _) => {
                        out.push(PRE_CROSS_LINKING_RULE_REF);
                        self.references.push(RefToReplace {
                            rule_id,
                            index,
                            reference,
                        });
                    }
                }
            } else {
                let local_id = self.compile_rule(r, repository_stack);
                out.push(GlobalRuleRef {
                    grammar: self.id,
                    rule: local_id,
                });
            }
        }

        out
    }

    #[cfg(feature = "debug")]
    pub(crate) fn get_original_rule_name(&self, rule_id: RuleId) -> Option<&str> {
        self.rules[rule_id.as_index()].original_name()
    }
}

/// Resolves `grammars[index]` external references after all grammar compilations
/// are complete.
/// This is called by the registry, not by the grammar itself.
pub(crate) fn resolve_external_references(
    index: usize,
    grammars: &mut [CompiledGrammar],
    grammar_mapping: &HashMap<String, GrammarId>,
) {
    // This is called after local are resolved so there should be only external refs here
    let references = std::mem::take(&mut grammars[index].references);

    for rep in references {
        let (grammar_name, repo_name) = match &rep.reference {
            Reference::OtherComplete(f) => (f, None),
            Reference::OtherSpecific(f, s) => (f, Some(s)),
            _ => unreachable!(),
        };

        let mut resolved = NO_OP_GLOBAL_RULE_REF;
        if let Some(g_id) = grammar_mapping.get(grammar_name).copied()
            && let Some(grammar) = grammars.get(g_id.as_index())
        {
            if let Some(repo_name) = repo_name {
                for repo in &grammar.repositories {
                    if let Some(r_id) = repo.get(repo_name) {
                        resolved = GlobalRuleRef {
                            grammar: g_id,
                            rule: *r_id,
                        };
                        break;
                    }
                }
                #[cfg(feature = "debug")]
                if resolved == NO_OP_GLOBAL_RULE_REF {
                    log::warn!(
                        "External grammar '{grammar_name}' found in registry but repository {repo_name} not found in it."
                    );
                }
            } else {
                resolved = GlobalRuleRef {
                    grammar: g_id,
                    rule: ROOT_RULE_ID,
                };
            }
        } else {
            #[cfg(feature = "debug")]
            log::warn!("External grammar '{grammar_name}' not found in registry.");
        }

        grammars[index].rules[rep.rule_id].replace_pattern(rep.index, resolved);
    }

    grammars[index].remove_empty_rules();
}

// Index trait implementations for type-safe array access
impl Index<GrammarId> for Vec<CompiledGrammar> {
    type Output = CompiledGrammar;

    fn index(&self, index: GrammarId) -> &Self::Output {
        &self[index.as_index()]
    }
}

impl IndexMut<GrammarId> for Vec<CompiledGrammar> {
    fn index_mut(&mut self, index: GrammarId) -> &mut Self::Output {
        &mut self[index.as_index()]
    }
}

impl Index<RuleId> for Vec<Rule> {
    type Output = Rule;

    fn index(&self, index: RuleId) -> &Self::Output {
        &self[index.as_index()]
    }
}

impl IndexMut<RuleId> for Vec<Rule> {
    fn index_mut(&mut self, index: RuleId) -> &mut Self::Output {
        &mut self[index.as_index()]
    }
}

impl Index<RegexId> for Vec<Regex> {
    type Output = Regex;

    fn index(&self, index: RegexId) -> &Self::Output {
        &self[index.as_index()]
    }
}

impl IndexMut<RegexId> for Vec<Regex> {
    fn index_mut(&mut self, index: RegexId) -> &mut Self::Output {
        &mut self[index.as_index()]
    }
}

impl Index<RepositoryId> for Vec<Repository> {
    type Output = Repository;

    fn index(&self, index: RepositoryId) -> &Self::Output {
        &self[index.as_index()]
    }
}

impl IndexMut<RepositoryId> for Vec<Repository> {
    fn index_mut(&mut self, index: RepositoryId) -> &mut Self::Output {
        &mut self[index.as_index()]
    }
}

#[cfg(test)]
mod tests {
    use super::{has_captures, replace_captures};
    use crate::grammars::raw::RawGrammar;
    use crate::grammars::{CompiledGrammar, GrammarId};
    use std::fs;

    #[test]
    fn can_find_captures() {
        let inputs = vec![
            (None, false),
            (Some("source"), false),
            (Some("source.hey"), false),
            (Some("source.hey.$1"), true),
            (Some("keyword.operator.logical.$1.media.css"), true),
            (Some("keyword.control.at-rule.${3:/downcase}.css"), true),
        ];

        for (input, expected) in inputs {
            assert_eq!(has_captures(input), expected);
        }
    }

    #[test]
    fn can_replace_captures() {
        let test_cases = vec![
            // (original_name, text, captures_pos, expected)

            // No captures - should return original
            (
                "source.hey",
                "hello world",
                vec![(0, 11), (0, 5), (6, 11)],
                "source.hey",
            ),
            // Simple capture replacements
            (
                "prefix.$1.suffix",
                "hello",
                vec![(0, 5), (0, 5)],
                "prefix.hello.suffix",
            ),
            (
                "$1_$2",
                "hello world",
                vec![(0, 11), (0, 5), (6, 11)],
                "hello_world",
            ),
            (
                "keyword.operator.logical.$1.media.css",
                "and",
                vec![(0, 3), (0, 3)],
                "keyword.operator.logical.and.media.css",
            ),
            // Transformation captures
            ("${1:/downcase}", "HELLO", vec![(0, 5), (0, 5)], "hello"),
            ("${1:/upcase}", "world", vec![(0, 5), (0, 5)], "WORLD"),
            (
                "keyword.control.at-rule.${1:/downcase}.css",
                "MEDIA",
                vec![(0, 5), (0, 5)],
                "keyword.control.at-rule.media.css",
            ),
            // Mixed simple and transformation captures
            (
                "$1.${2:/upcase}",
                "hello world",
                vec![(0, 11), (0, 5), (6, 11)],
                "hello.WORLD",
            ),
            (
                "${1:/downcase}_$2_${3:/upcase}",
                "Hello big WORLD",
                vec![(0, 15), (0, 5), (6, 9), (10, 15)],
                "hello_big_WORLD",
            ),
            // Leading dots removal
            ("scope.$1", "..method", vec![(0, 8), (0, 8)], "scope.method"),
            (
                "prefix.${1:/downcase}",
                "...CLASS",
                vec![(0, 8), (0, 8)],
                "prefix.class",
            ),
            // Missing captures - should return original capture reference
            ("$1 $5", "hello", vec![(0, 5), (0, 5)], "hello $5"),
            (
                "${3:/downcase}",
                "hello",
                vec![(0, 5), (0, 5)],
                "${3:/downcase}",
            ),
            // Full match (capture 0)
            (
                "match: $0",
                "hello world",
                vec![(0, 11), (0, 5), (6, 11)],
                "match: hello world",
            ),
            // Multiple occurrences of same capture
            (
                "$1.$1.suffix",
                "hello",
                vec![(0, 5), (0, 5)],
                "hello.hello.suffix",
            ),
            (
                "${1:/downcase}.${1:/upcase}",
                "Hello",
                vec![(0, 5), (0, 5)],
                "hello.HELLO",
            ),
        ];

        for (original_name, text, captures_pos, expected) in test_cases {
            // Convert Vec<(usize, usize)> to Vec<Option<(usize, usize)>>
            let option_captures: Vec<Option<(usize, usize)>> =
                captures_pos.into_iter().map(Some).collect();
            let result = replace_captures(original_name, text, &option_captures);
            assert_eq!(
                result, expected,
                "Failed for input: '{}' with text: '{}' - expected: '{}', got: '{}'",
                original_name, text, expected, result
            );
        }
    }

    #[test]
    fn can_compile_all_shiki_grammars() {
        let entries = fs::read_dir("grammars-themes/packages/tm-grammars/grammars")
            .expect("Failed to read grammars directory");

        for entry in entries {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            let raw_grammar = RawGrammar::load_from_file(&path).unwrap();

            println!(">> {path:#?}");
            let _ = CompiledGrammar::from_raw_grammar(raw_grammar, GrammarId(0));
        }
    }
}
