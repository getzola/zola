use std::cmp::Ordering;

use rayon::prelude::*;
use serde::Serialize;

use content::{Library, Page, SerializingPage, TaxonomyTerm};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SerializedFeedTaxonomyItem<'a> {
    name: &'a str,
    slug: &'a str,
    permalink: &'a str,
}

impl<'a> SerializedFeedTaxonomyItem<'a> {
    pub fn from_item(item: &'a TaxonomyTerm) -> Self {
        SerializedFeedTaxonomyItem {
            name: &item.name,
            slug: &item.slug,
            permalink: &item.permalink,
        }
    }
}

pub struct FeedData<'a> {
    pub pages: Vec<SerializingPage<'a>>,
    pub last_updated: Option<String>,
}

/// Prepares feed data by filtering, sorting, and serializing pages.
/// Returns the serialized pages and the last_updated timestamp.
pub fn prepare_feed<'a>(
    all_pages: Vec<&'a Page>,
    feed_limit: Option<usize>,
    library: &'a Library,
) -> FeedData<'a> {
    let mut pages = all_pages.into_iter().filter(|p| p.meta.date.is_some()).collect::<Vec<_>>();

    pages.par_sort_unstable_by(|a, b| {
        let ord = b.meta.datetime.unwrap().cmp(&a.meta.datetime.unwrap());
        if ord == Ordering::Equal { a.permalink.cmp(&b.permalink) } else { ord }
    });

    let last_updated = pages
        .iter()
        .filter_map(|page| page.meta.updated.as_ref())
        .chain(pages.first().and_then(|p| p.meta.date.as_ref()))
        .max()
        .cloned();

    // limit to the last n elements if the limit is set; otherwise use all.
    let num_entries = feed_limit.unwrap_or(pages.len());
    let serialized_pages = pages
        .iter()
        .take(num_entries)
        .map(|x| x.serialize_without_siblings(library))
        .collect::<Vec<_>>();

    FeedData { pages: serialized_pages, last_updated }
}
