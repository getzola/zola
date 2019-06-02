use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use config::Config;
use library::{Library, Taxonomy};
use std::cmp::Ordering;
use tera::{Map, Value};

/// The sitemap only needs links, potentially date and extra for pages in case of updates
/// for examples so we trim down all entries to only that
#[derive(Debug, Serialize)]
pub struct SitemapEntry<'a> {
    permalink: Cow<'a, str>,
    date: Option<String>,
    extra: Option<&'a Map<String, Value>>,
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
    pub fn new(permalink: Cow<'a, str>, date: Option<String>) -> Self {
        SitemapEntry { permalink, date, extra: None }
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
    let pages = library
        .pages_values()
        .iter()
        .filter(|p| !p.is_draft())
        .map(|p| {
            let date = match p.meta.date {
                Some(ref d) => Some(d.to_string()),
                None => None,
            };
            let mut entry = SitemapEntry::new(Cow::Borrowed(&p.permalink), date);
            entry.add_extra(&p.meta.extra);
            entry
        })
        .collect::<Vec<_>>();

    let mut sections = library
        .sections_values()
        .iter()
        .filter(|s| s.meta.render)
        .map(|s| SitemapEntry::new(Cow::Borrowed(&s.permalink), None))
        .collect::<Vec<_>>();

    for section in library.sections_values().iter().filter(|s| s.meta.paginate_by.is_some()) {
        let number_pagers =
            (section.pages.len() as f64 / section.meta.paginate_by.unwrap() as f64).ceil() as isize;
        for i in 1..=number_pagers {
            let permalink = format!("{}{}/{}/", section.permalink, section.meta.paginate_path, i);
            sections.push(SitemapEntry::new(Cow::Owned(permalink), None))
        }
    }

    let mut taxonomies_entries = vec![];
    for taxonomy in taxonomies {
        let name = &taxonomy.kind.name;
        let mut terms = vec![];
        terms.push(SitemapEntry::new(Cow::Owned(config.make_permalink(name)), None));
        for item in &taxonomy.items {
            terms.push(SitemapEntry::new(
                Cow::Owned(config.make_permalink(&format!("{}/{}", name, item.slug))),
                None,
            ));

            if taxonomy.kind.is_paginated() {
                let number_pagers = (item.pages.len() as f64
                    / taxonomy.kind.paginate_by.unwrap() as f64)
                    .ceil() as isize;
                for i in 1..=number_pagers {
                    let permalink = config.make_permalink(&format!(
                        "{}/{}/{}/{}",
                        name,
                        item.slug,
                        taxonomy.kind.paginate_path(),
                        i
                    ));
                    terms.push(SitemapEntry::new(Cow::Owned(permalink), None))
                }
            }
        }

        taxonomies_entries.push(terms);
    }

    let mut all_sitemap_entries = HashSet::new();
    for p in pages {
        all_sitemap_entries.insert(p);
    }
    for s in sections {
        all_sitemap_entries.insert(s);
    }
    for terms in taxonomies_entries {
        for term in terms {
            all_sitemap_entries.insert(term);
        }
    }

    all_sitemap_entries.into_iter().collect::<Vec<_>>()
}
