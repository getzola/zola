use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use serde::Serialize;
use tera::Value;
use time::format_description::well_known::Rfc3339;

use config::Config;
use content::{Library, Page, Taxonomy};

/// The sitemap only needs links, potentially date and extra for pages in case of updates
/// for examples so we trim down all entries to only that
#[derive(Debug, Serialize)]
pub struct SitemapEntry<'a> {
    pub permalink: Cow<'a, str>,
    pub updated: Option<String>,
    pub extra: Option<Value>,
}

/// W3C Datetime for `<lastmod>`, preserving time if its present
fn lastmod(page: &Page) -> Option<String> {
    let (raw, dt) = page
        .meta
        .updated
        .as_deref()
        .zip(page.meta.updated_datetime)
        .or_else(|| page.meta.date.as_deref().zip(page.meta.datetime))?;
    Some(if raw.contains(':') {
        dt.format(&Rfc3339).ok()?
    } else {
        format!("{:04}-{:02}-{:02}", dt.year(), u8::from(dt.month()), dt.day())
    })
}

// Hash/Eq is not implemented for tera::Map but in our case we only care about the permalink
// when comparing/hashing so we implement it manually
impl<'a> Hash for SitemapEntry<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.permalink.hash(state);
    }
}
impl<'a> PartialEq for SitemapEntry<'a> {
    fn eq(&self, other: &SitemapEntry) -> bool {
        self.permalink == other.permalink
    }
}
impl<'a> Eq for SitemapEntry<'a> {}

impl<'a> SitemapEntry<'a> {
    pub fn new(permalink: Cow<'a, str>, updated: Option<String>) -> Self {
        SitemapEntry { permalink, updated, extra: None }
    }

    pub fn add_extra(&mut self, extra: Value) {
        self.extra = Some(extra);
    }
}

impl<'a> PartialOrd for SitemapEntry<'a> {
    fn partial_cmp(&self, other: &SitemapEntry) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for SitemapEntry<'a> {
    fn cmp(&self, other: &SitemapEntry) -> Ordering {
        self.permalink.as_ref().cmp(other.permalink.as_ref())
    }
}

/// Finds out all the links to put in a sitemap from the pages/sections/taxonomies
/// There are no duplicate permalinks in the output vec
pub fn find_entries<'a>(
    library: &'a Library,
    taxonomies: &'a [Taxonomy],
    config: &'a Config,
) -> Vec<SitemapEntry<'a>> {
    let mut entries = HashSet::new();

    for p in library.pages.values() {
        if !p.meta.render || p.hidden {
            continue;
        }

        let mut entry = SitemapEntry::new(Cow::Borrowed(&p.permalink), lastmod(p));
        entry.add_extra(p.meta.extra.clone());
        entries.insert(entry);
    }

    for s in library.sections.values() {
        if s.hidden {
            continue;
        }

        if s.meta.render {
            let mut entry = SitemapEntry::new(Cow::Borrowed(&s.permalink), None);
            entry.add_extra(s.meta.extra.clone());
            entries.insert(entry);
        }

        if let Some(paginate_by) = s.paginate_by()
            && !config.should_exclude_paginated_pages_in_sitemap()
        {
            let number_pagers = (s.pages.len() as f64 / paginate_by as f64).ceil() as isize;
            for i in 1..=number_pagers {
                let permalink = format!("{}{}/{}/", s.permalink, s.meta.paginate_path, i);
                entries.insert(SitemapEntry::new(Cow::Owned(permalink), None));
            }
        }
    }

    for taxonomy in taxonomies {
        if !taxonomy.kind.render {
            continue;
        }
        entries.insert(SitemapEntry::new(Cow::Borrowed(&taxonomy.permalink), None));

        for item in &taxonomy.items {
            entries.insert(SitemapEntry::new(Cow::Borrowed(&item.permalink), None));

            if taxonomy.kind.is_paginated() && !config.should_exclude_paginated_pages_in_sitemap() {
                let number_pagers = (item.pages.len() as f64
                    / taxonomy.kind.paginate_by.unwrap() as f64)
                    .ceil() as isize;
                for i in 1..=number_pagers {
                    let permalink = config.make_permalink(&format!(
                        "{}{}/{}/",
                        item.path,
                        taxonomy.kind.paginate_path(),
                        i
                    ));
                    entries.insert(SitemapEntry::new(Cow::Owned(permalink), None));
                }
            }
        }
    }

    let mut entries = entries.into_iter().collect::<Vec<_>>();
    entries.sort();
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://github.com/getzola/zola/issues/2335
    #[test]
    fn lastmod_preserves_time_and_adds_tz() {
        let inputs = vec![
            ((Some("2026-08-12"), None), Some("2026-08-12".to_string())),
            ((Some("2026-08-12T14:36:26"), None), Some("2026-08-12T14:36:26Z".to_string())),
            (
                (Some("2026-08-12T14:36:26-05:00"), None),
                Some("2026-08-12T14:36:26-05:00".to_string()),
            ),
            ((Some("2020-01-01"), Some("2026-08-12")), Some("2026-08-12".to_string())),
        ];

        for ((date, updated), expected) in inputs {
            let mut p = Page::default();
            p.meta.date = date.map(String::from);
            p.meta.updated = updated.map(String::from);
            p.meta.date_to_datetime();

            assert_eq!(lastmod(&p), expected);
        }
    }
}
