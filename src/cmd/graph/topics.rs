//! Topical merge + enrichment glue. [`merge_page_topics`] is the pure,
//! unit-tested core; [`enrich_one`] wraps the OpenRouter call for the
//! migrate/refresh drivers. Used by **both** commands (initial migrate +
//! stale-only refresh), so it must not depend on Firecrawl.

use errors::Result;

use super::openrouter::{TopicClient, TopicExtract, TopicInput};
use super::schema::{GraphStore, Relation, Topic};

/// Hand-rolled slug: lowercase, non-[a-z0-9] runs → `-`, trimmed.
/// ponytail: no `slug` crate dep. Ceiling = unicode labels collapse to ascii
/// runs; fine for SEO topic slugs which are short noun phrases.
pub fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_dash = true; // suppresses leading dashes
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            for lc in c.to_lowercase() {
                out.push(lc);
            }
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

/// Pure merge of one page's model output into the store. Idempotent: re-merging
/// the same extract does not duplicate topics, page_topic edges, or aliases.
/// Inter-topic relations are stored with kind `topic_topic` (the sub-type
/// related/broader/narrower is collapsed — ceiling; upgrade schema to preserve).
pub fn merge_page_topics(store: &mut GraphStore, page_url: &str, extract: &TopicExtract) {
    // resolve labels → topic ids, reusing existing store topics first, then
    // ones created earlier this call, then creating new. This is what makes
    // merge idempotent and case-insensitive across calls.
    let mut label_to_id: Vec<(String, String)> = Vec::new(); // (lowercase label, id)
    for spec in &extract.topics {
        let key = spec.label.to_ascii_lowercase();
        let id = if let Some(t) = store.topics.iter().find(|t| t.label.to_ascii_lowercase() == key)
        {
            t.id.clone()
        } else if let Some((_, id)) = label_to_id.iter().find(|(l, _)| *l == key) {
            id.clone()
        } else {
            let id = unique_topic_id(&store.topics, &spec.label, &label_to_id);
            label_to_id.push((key, id.clone()));
            store.topics.push(Topic {
                id: id.clone(),
                label: spec.label.clone(),
                aliases: spec.aliases.clone(),
                page_ids: vec![page_url.to_string()],
            });
            id
        };
        // attach page to topic (dedup)
        let topic = store.topics.iter_mut().find(|t| t.id == id).unwrap();
        if !topic.page_ids.iter().any(|u| u == page_url) {
            topic.page_ids.push(page_url.to_string());
        }
        // attach topic to page (dedup)
        if let Some(page) = store.pages.iter_mut().find(|p| p.url == page_url) {
            if !page.topic_ids.iter().any(|t| *t == id) {
                page.topic_ids.push(id.clone());
            }
        }
        upsert_relation(
            &mut store.relations,
            &Relation { from: page_url.into(), to: id, kind: "page_topic".into() },
        );
    }

    // inter-topic relations
    for rel in &extract.relations {
        let Some(from_id) = lookup_label(&label_to_id, &store.topics, &rel.from_label) else {
            continue;
        };
        let Some(to_id) = lookup_label(&label_to_id, &store.topics, &rel.to_label) else {
            continue;
        };
        if from_id == to_id {
            continue;
        }
        upsert_relation(
            &mut store.relations,
            &Relation { from: from_id, to: to_id, kind: "topic_topic".into() },
        );
    }
}

/// Run extraction for one page and merge. Returns true if the API was called
/// and the store mutated (false on dry-run). Network + max cap are the caller's
/// responsibility; this is the single API-touching step both drivers share.
pub fn enrich_one<C: TopicClient>(
    store: &mut GraphStore,
    page_url: &str,
    input: &TopicInput,
    client: &C,
    key: &str,
    dry_run: bool,
) -> Result<bool> {
    if dry_run {
        log::info!("topics [dry-run]: would enrich {page_url}");
        return Ok(false);
    }
    let extract = client.extract(input, key)?;
    merge_page_topics(store, page_url, &extract);
    Ok(true)
}

fn unique_topic_id(topics: &[Topic], label: &str, taken: &[(String, String)]) -> String {
    let base = slugify(label);
    if base.is_empty() {
        return "topic".into();
    }
    let id_exists =
        |id: &str| topics.iter().any(|t| t.id == id) || taken.iter().any(|(_, t)| t == id);
    if !id_exists(&base) {
        return base;
    }
    for n in 2.. {
        let cand = format!("{base}-{n}");
        if !id_exists(&cand) {
            return cand;
        }
    }
    unreachable!()
}

fn lookup_label(fresh: &[(String, String)], topics: &[Topic], label: &str) -> Option<String> {
    let key = label.to_ascii_lowercase();
    if let Some((_, id)) = fresh.iter().find(|(l, _)| *l == key) {
        return Some(id.clone());
    }
    topics.iter().find(|t| t.label.to_ascii_lowercase() == key).map(|t| t.id.clone())
}

fn upsert_relation(rels: &mut Vec<Relation>, rel: &Relation) {
    let exists = rels.iter().any(|r| r.from == rel.from && r.to == rel.to && r.kind == rel.kind);
    if !exists {
        rels.push(rel.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::graph::openrouter::{TopicClient, TopicExtract, TopicInput, TopicSpec};

    fn store_with_page(url: &str) -> GraphStore {
        GraphStore {
            pages: vec![super::super::schema::Page {
                url: url.into(),
                path: "p/index.md".into(),
                title: "T".into(),
                summary: String::new(),
                content_hash: "h".into(),
                topic_ids: vec![],
            }],
            ..Default::default()
        }
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Applicant Tracking!"), "applicant-tracking");
        assert_eq!(slugify("  --hi--  "), "hi");
        assert_eq!(slugify("Über"), "ber"); // non-ascii stripped — documented ceiling
    }

    #[test]
    fn merge_creates_topics_and_edges() {
        let mut store = store_with_page("https://x/a");
        let extract = TopicExtract {
            topics: vec![
                TopicSpec { label: "Hiring".into(), aliases: vec!["Recruiting".into()] },
                TopicSpec { label: "ATS".into(), aliases: vec![] },
            ],
            relations: vec![],
        };
        merge_page_topics(&mut store, "https://x/a", &extract);
        assert_eq!(store.topics.len(), 2);
        assert_eq!(store.topics[0].id, "hiring");
        assert_eq!(store.topics[0].page_ids, vec!["https://x/a".to_string()]);
        assert_eq!(store.topics[1].id, "ats");
        let page = &store.pages[0];
        assert_eq!(page.topic_ids, vec!["hiring".to_string(), "ats".to_string()]);
        let pt: Vec<_> = store.relations.iter().filter(|r| r.kind == "page_topic").collect();
        assert_eq!(pt.len(), 2);
    }

    #[test]
    fn merge_is_idempotent() {
        let mut store = store_with_page("https://x/a");
        let extract = TopicExtract {
            topics: vec![TopicSpec { label: "Hiring".into(), aliases: vec![] }],
            relations: vec![],
        };
        merge_page_topics(&mut store, "https://x/a", &extract);
        merge_page_topics(&mut store, "https://x/a", &extract); // re-merge same
        assert_eq!(store.topics.len(), 1, "no duplicate topic");
        assert_eq!(store.topics[0].page_ids.len(), 1, "page not double-added");
        assert_eq!(store.relations.len(), 1, "no duplicate edge");
    }

    #[test]
    fn merge_dedupes_case_insensitive_label() {
        let mut store = store_with_page("https://x/a");
        let ex1 = TopicExtract {
            topics: vec![TopicSpec { label: "Hiring".into(), aliases: vec![] }],
            relations: vec![],
        };
        let ex2 = TopicExtract {
            topics: vec![TopicSpec { label: "hiring".into(), aliases: vec![] }],
            relations: vec![],
        };
        merge_page_topics(&mut store, "https://x/a", &ex1);
        merge_page_topics(&mut store, "https://x/a", &ex2);
        assert_eq!(store.topics.len(), 1, "Hiring/hiring same topic");
    }

    #[test]
    fn merge_inter_topic_relation() {
        let mut store = store_with_page("https://x/a");
        let extract = TopicExtract {
            topics: vec![
                TopicSpec { label: "Hiring".into(), aliases: vec![] },
                TopicSpec { label: "ATS".into(), aliases: vec![] },
            ],
            relations: vec![super::super::openrouter::TopicRelationSpec {
                from_label: "Hiring".into(),
                to_label: "ATS".into(),
                kind: "related".into(),
            }],
        };
        merge_page_topics(&mut store, "https://x/a", &extract);
        let tt: Vec<_> = store.relations.iter().filter(|r| r.kind == "topic_topic").collect();
        assert_eq!(tt.len(), 1);
        assert_eq!(tt[0].from, "hiring");
        assert_eq!(tt[0].to, "ats");
    }

    /// Mock that returns a fixed extract regardless of input.
    struct FixedClient;
    impl TopicClient for FixedClient {
        fn extract(&self, _input: &TopicInput, _key: &str) -> Result<TopicExtract> {
            Ok(TopicExtract {
                topics: vec![TopicSpec { label: "Mocked".into(), aliases: vec![] }],
                relations: vec![],
            })
        }
    }

    #[test]
    fn enrich_one_merges_and_reports_called() {
        let mut store = store_with_page("https://x/a");
        let input =
            TopicInput { title: "T".into(), description: String::new(), body: "body".into() };
        let called =
            enrich_one(&mut store, "https://x/a", &input, &FixedClient, "k", false).unwrap();
        assert!(called);
        assert_eq!(store.topics.len(), 1);
        assert_eq!(store.topics[0].label, "Mocked");
    }

    #[test]
    fn enrich_one_dry_run_does_not_mutate() {
        let mut store = store_with_page("https://x/a");
        let input =
            TopicInput { title: "T".into(), description: String::new(), body: "body".into() };
        let called =
            enrich_one(&mut store, "https://x/a", &input, &FixedClient, "k", true).unwrap();
        assert!(!called);
        assert!(store.topics.is_empty());
    }
}
