use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use serde::Serialize;

use config::Config;
use content::{Library, Taxonomy};
use libs::tera::{Map, Value};
use std::cmp::Ordering;

/// The sitemap only needs links, potentially date and extra for pages in case of updates
/// for examples so we trim down all entries to only that
#[derive(Debug, Serialize)]
pub struct SitemapEntry<'a> {
    pub permalink: Cow<'a, str>,
    pub updated: &'a Option<String>,
    pub extra: Option<&'a Map<String, Value>>,
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
    pub fn new(permalink: Cow<'a, str>, updated: &'a Option<String>) -> Self {
        SitemapEntry { permalink, updated, extra: None }
    }

    pub fn add_extra(&mut self, extra: &'a Map<String, Value>) {
        self.extra = Some(extra);
    }
}

impl<'a> PartialOrd for SitemapEntry<'a> {
    fn partial_cmp(&self, other: &SitemapEntry) -> Option<Ordering> {
        Some(self.permalink.as_ref().cmp(other.permalink.as_ref()))
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
        let mut entry = SitemapEntry::new(
            Cow::Borrowed(&p.permalink),
            if p.meta.updated.is_some() { &p.meta.updated } else { &p.meta.date },
        );
        entry.add_extra(&p.meta.extra);
        entries.insert(entry);
    }

    for s in library.sections.values() {
        if s.meta.render {
            let mut entry = SitemapEntry::new(Cow::Borrowed(&s.permalink), &None);
            entry.add_extra(&s.meta.extra);
            entries.insert(entry);
        }

        if let Some(paginate_by) = s.paginate_by() {
            let number_pagers = (s.pages.len() as f64 / paginate_by as f64).ceil() as isize;
            for i in 1..=number_pagers {
                let permalink = format!("{}{}/{}/", s.permalink, s.meta.paginate_path, i);
                entries.insert(SitemapEntry::new(Cow::Owned(permalink), &None));
            }
        }
    }

    for taxonomy in taxonomies {
        if !taxonomy.kind.render {
            continue;
        }
        entries.insert(SitemapEntry::new(Cow::Borrowed(&taxonomy.permalink), &None));

        for item in &taxonomy.items {
            entries.insert(SitemapEntry::new(Cow::Borrowed(&item.permalink), &None));

            if taxonomy.kind.is_paginated() {
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
                    entries.insert(SitemapEntry::new(Cow::Owned(permalink), &None));
                }
            }
        }
    }

    let mut entries = entries.into_iter().collect::<Vec<_>>();
    entries.sort();
    entries
}
