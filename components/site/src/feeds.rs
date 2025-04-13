use std::cmp::Ordering;
use std::path::PathBuf;

use libs::rayon::prelude::*;
use libs::tera::Context;
use serde::Serialize;

use crate::Site;
use content::{Page, TaxonomyTerm};
use errors::Result;
use utils::templates::render_template;

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

pub fn render_feeds(
    site: &Site,
    all_pages: Vec<&Page>,
    lang: &str,
    base_path: Option<&PathBuf>,
    additional_context_fn: impl Fn(Context) -> Context,
) -> Result<Option<Vec<String>>> {
    let mut pages = all_pages.into_iter().filter(|p| p.meta.date.is_some()).collect::<Vec<_>>();

    pages.par_sort_unstable_by(|a, b| {
        let ord = b.meta.datetime.unwrap().cmp(&a.meta.datetime.unwrap());
        if ord == Ordering::Equal {
            a.permalink.cmp(&b.permalink)
        } else {
            ord
        }
    });

    let mut context = Context::new();
    if let Some(last_updated) = pages
        .iter()
        .filter_map(|page| page.meta.updated.as_ref())
        .chain(pages.first().and_then(|p| p.meta.date.as_ref()))
        .max()
    {
        context.insert("last_updated", &last_updated);
    }
    let library = site.library.read().unwrap();
    // limit to the last n elements if the limit is set; otherwise use all.
    let num_entries = site.config.feed_limit.unwrap_or(pages.len());
    let p = pages
        .iter()
        .take(num_entries)
        .map(|x| x.serialize_without_siblings(&library))
        .collect::<Vec<_>>();

    context.insert("pages", &p);
    context.insert("config", &site.config.serialize(lang));
    context.insert("lang", lang);

    let mut feeds = Vec::new();
    for feed_filename in &site.config.languages[lang].feed_filenames {
        let mut context = context.clone();

        let feed_url = if let Some(base) = base_path {
            site.config
                .make_permalink(&base.join(feed_filename).to_string_lossy().replace('\\', "/"))
        } else {
            site.config.make_permalink(feed_filename)
        };

        context.insert("feed_url", &feed_url);
        context = additional_context_fn(context);
        feeds.push(render_template(feed_filename, &site.tera, context, &site.config.theme)?);
    }

    Ok(Some(feeds))
}
