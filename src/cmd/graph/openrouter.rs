//! OpenRouter topical extraction (ADR-003: `openai/gpt-4o-mini`, JSON-object
//! response). Same transport contract as `cmd::translate` — the call is behind
//! the [`TopicClient`] trait so tests mock it without a network or key.

use std::time::Duration;

use errors::{Result, anyhow, bail};
use serde_json::{Value, json};

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
/// Umbrella ADR-003: gpt-4o-mini only.
const MODEL: &str = "openai/gpt-4o-mini";
const MAX_TOKENS: u32 = 4096;
/// Body chars sent to the model — caps tokens per page.
const BODY_CHAR_CAP: usize = 6000;

/// Page fields the model sees.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopicInput {
    pub title: String,
    pub description: String,
    pub body: String,
}

/// One extracted topic (label + aliases).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopicSpec {
    pub label: String,
    pub aliases: Vec<String>,
}

/// One inter-topic relation. `kind` ∈ `related` | `broader` | `narrower`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopicRelationSpec {
    pub from_label: String,
    pub to_label: String,
    pub kind: String,
}

/// Model output for one page.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TopicExtract {
    pub topics: Vec<TopicSpec>,
    pub relations: Vec<TopicRelationSpec>,
}

/// Extraction client. Trait so tests inject a mock.
pub trait TopicClient {
    fn extract(&self, input: &TopicInput, key: &str) -> Result<TopicExtract>;
    /// Pillar overview (134–167 words). Default: unsupported.
    fn overview(&self, title: &str, body: &str, key: &str) -> Result<String> {
        let _ = (title, body, key);
        bail!("overview not implemented")
    }
}

/// Live OpenRouter client (blocking reqwest).
pub struct OpenRouterTopicClient;

impl TopicClient for OpenRouterTopicClient {
    fn extract(&self, input: &TopicInput, key: &str) -> Result<TopicExtract> {
        let body = truncate(&input.body, BODY_CHAR_CAP);
        let payload = json!({
            "model": MODEL,
            "response_format": {"type": "json_object"},
            "max_tokens": MAX_TOKENS,
            "messages": [
                {"role": "system", "content":
                    "You extract a concise topical knowledge graph from a web page for SEO. \
                     Return a single JSON object with exactly these keys: \
                     topics (array of {label, aliases}) and \
                     relations (array of {from_label, to_label, kind}). \
                     label = short lowercase noun phrase. aliases = up to 3 synonyms. \
                     kind in {related, broader, narrower}. \
                     At most 8 topics. No prose."},
                {"role": "user", "content": json!({
                    "title": input.title,
                    "description": input.description,
                    "body": body,
                }).to_string()},
            ],
        });
        let content = chat_content(key, payload)?;
        parse_extract(&content)
    }

    fn overview(&self, title: &str, body: &str, key: &str) -> Result<String> {
        let body = truncate(body, 4000);
        let user = format!(
            "Write 134 to 167 words, inclusive, plain prose, no heading, no bullets.\n\
             Describe this Curriculo page for an AI citation. Product is one platform:\n\
             ATS for recruiters and a free resume builder. Do not invent metrics.\n\
             Title: {title}\n\
             Body: {body}"
        );
        let payload = json!({
            "model": MODEL,
            "max_tokens": MAX_TOKENS,
            "messages": [
                {"role": "user", "content": user},
            ],
        });
        let content = chat_content(key, payload)?;
        Ok(content.trim().to_string())
    }
}

fn chat_content(key: &str, payload: Value) -> Result<String> {
    let bytes = serde_json::to_vec(&payload)?;
    let resp = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?
        .post(OPENROUTER_URL)
        .bearer_auth(key)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(bytes)
        .send()?;
    let status = resp.status();
    let text = resp.text()?;
    if !status.is_success() {
        bail!("OpenRouter HTTP {status}: {}", take200(&text));
    }
    let data: Value =
        serde_json::from_str(&text).map_err(|e| anyhow!("OpenRouter non-JSON response: {e}"))?;
    data["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("OpenRouter: unexpected shape: {}", take160(&data.to_string())))
}

/// Parse the model's JSON-object content into [`TopicExtract`]. Pure — used by
/// tests and the live client.
pub fn parse_extract(content: &str) -> Result<TopicExtract> {
    let v: Value = serde_json::from_str(content)
        .map_err(|_| anyhow!("model returned non-JSON: {}", take160(content)))?;
    let mut topics = Vec::new();
    if let Some(arr) = v.get("topics").and_then(|t| t.as_array()) {
        for t in arr {
            let label = t.get("label").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
            if label.is_empty() {
                continue;
            }
            let aliases = t
                .get("aliases")
                .and_then(|a| a.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.trim().to_string()))
                        .filter(|s| !s.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            topics.push(TopicSpec { label, aliases });
        }
    }
    let mut relations = Vec::new();
    if let Some(arr) = v.get("relations").and_then(|t| t.as_array()) {
        for r in arr {
            let from_label =
                r.get("from_label").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
            let to_label =
                r.get("to_label").and_then(|x| x.as_str()).unwrap_or("").trim().to_string();
            let kind =
                r.get("kind").and_then(|x| x.as_str()).unwrap_or("related").trim().to_string();
            if from_label.is_empty() || to_label.is_empty() {
                continue;
            }
            relations.push(TopicRelationSpec { from_label, to_label, kind });
        }
    }
    Ok(TopicExtract { topics, relations })
}

fn truncate(s: &str, cap: usize) -> String {
    if s.len() <= cap {
        s.to_string()
    } else {
        // ponytail: byte-cap on a char boundary near `cap`; fine for SEO input.
        let mut end = cap;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

fn take200(s: &str) -> String {
    s.chars().take(200).collect()
}
fn take160(s: &str) -> String {
    s.chars().take(160).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_extract_normal() {
        let content = r#"{"topics":[{"label":"hiring","aliases":["recruiting"]},{"label":"ats"}],"relations":[{"from_label":"hiring","to_label":"ats","kind":"related"}]}"#;
        let ex = parse_extract(content).unwrap();
        assert_eq!(ex.topics.len(), 2);
        assert_eq!(ex.topics[0].label, "hiring");
        assert_eq!(ex.topics[0].aliases, vec!["recruiting".to_string()]);
        assert_eq!(ex.relations.len(), 1);
        assert_eq!(ex.relations[0].kind, "related");
    }

    #[test]
    fn parse_extract_drops_empty_labels() {
        let content = r#"{"topics":[{"label":""},{"label":"ats"}]}"#;
        let ex = parse_extract(content).unwrap();
        assert_eq!(ex.topics.len(), 1);
    }

    #[test]
    fn parse_extract_non_json_errors() {
        assert!(parse_extract("not json").is_err());
    }

    #[test]
    fn parse_extract_empty_is_default() {
        let ex = parse_extract(r#"{"topics":[],"relations":[]}"#).unwrap();
        assert!(ex.topics.is_empty());
        assert!(ex.relations.is_empty());
    }

    #[test]
    fn truncate_on_boundary() {
        let s = "abcdef";
        assert_eq!(truncate(s, 3), "abc…");
        assert_eq!(truncate(s, 100), s);
    }
}
