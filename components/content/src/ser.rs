use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

use crate::library::Library;
use crate::{Page, Section};
use tera::Value;
use utils::table_of_contents::Heading;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BackLink<'a> {
    pub permalink: &'a str,
    pub title: &'a Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TranslatedContent<'a> {
    pub lang: &'a str,
    pub permalink: &'a str,
    pub title: &'a Option<String>,
    /// The path to the markdown file
    pub path: &'a Path,
}

fn find_backlinks<'a>(relative_path: &str, library: &'a Library) -> Vec<BackLink<'a>> {
    let mut backlinks = Vec::new();
    if let Some(b) = library.backlinks.get(relative_path) {
        for backlink in b {
            if let Some(p) = library.pages.get(backlink) {
                backlinks.push(BackLink { permalink: &p.permalink, title: &p.meta.title });
            }
            if let Some(s) = library.sections.get(backlink) {
                backlinks.push(BackLink { permalink: &s.permalink, title: &s.meta.title });
            }
        }
        backlinks.sort_by_key(|b| b.permalink);
    }
    backlinks
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SerializingPage<'a> {
    relative_path: &'a str,
    colocated_path: &'a Option<String>,
    content: &'a str,
    permalink: &'a str,
    slug: &'a str,
    ancestors: &'a [String],
    pub(crate) title: &'a Option<String>,
    description: &'a Option<String>,
    updated: &'a Option<String>,
    date: &'a Option<String>,
    year: Option<i32>,
    month: Option<u8>,
    day: Option<u8>,
    taxonomies: &'a HashMap<String, Vec<String>>,
    authors: &'a [String],
    extra: &'a Value,
    path: &'a str,
    components: &'a [String],
    summary: &'a Option<String>,
    toc: &'a [Heading],
    word_count: Option<usize>,
    reading_time: Option<usize>,
    assets: &'a [String],
    draft: bool,
    lang: &'a str,
    lower: Option<Value>,
    higher: Option<Value>,
    translations: Vec<TranslatedContent<'a>>,
    backlinks: Vec<BackLink<'a>>,
}

impl<'a> SerializingPage<'a> {
    pub fn new(page: &'a Page, library: &'a Library) -> Self {
        let mut year = None;
        let mut month = None;
        let mut day = None;
        if let Some(d) = page.meta.datetime_tuple {
            year = Some(d.0);
            month = Some(d.1);
            day = Some(d.2);
        }
        let translations = library.find_translations(&page.file.canonical);
        let backlinks = find_backlinks(&page.file.relative, library);

        Self {
            relative_path: &page.file.relative,
            colocated_path: &page.file.colocated_path,
            ancestors: &page.ancestors,
            content: &page.content,
            permalink: &page.permalink,
            slug: &page.slug,
            title: &page.meta.title,
            description: &page.meta.description,
            extra: &page.meta.extra,
            updated: &page.meta.updated,
            date: &page.meta.date,
            year,
            month,
            day,
            taxonomies: &page.meta.taxonomies,
            authors: &page.meta.authors,
            path: &page.path,
            components: &page.components,
            summary: &page.summary,
            toc: &page.toc,
            word_count: page.word_count,
            reading_time: page.reading_time,
            assets: &page.serialized_assets,
            draft: page.meta.draft,
            lang: &page.lang,
            lower: None,
            higher: None,
            translations,
            backlinks,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SerializingSection<'a> {
    relative_path: &'a str,
    colocated_path: &'a Option<String>,
    content: &'a str,
    permalink: &'a str,
    draft: bool,
    ancestors: &'a [String],
    title: &'a Option<String>,
    description: &'a Option<String>,
    extra: &'a Value,
    path: &'a str,
    components: &'a [String],
    toc: &'a [Heading],
    word_count: Option<usize>,
    reading_time: Option<usize>,
    lang: &'a str,
    assets: &'a [String],
    pages: Vec<Value>,
    subsections: Vec<&'a str>,
    translations: Vec<TranslatedContent<'a>>,
    backlinks: Vec<BackLink<'a>>,
    generate_feeds: bool,
    transparent: bool,
    paginate_by: &'a Option<usize>,
    paginate_reversed: bool,
}

impl<'a> SerializingSection<'a> {
    pub fn new(section: &'a Section, library: &'a Library, pages: Vec<Value>) -> Self {
        let translations = library.find_translations(&section.file.canonical);
        let subsections: Vec<&str> = section
            .subsections
            .iter()
            .map(|p| library.sections[p].file.relative.as_str())
            .collect();
        let backlinks = find_backlinks(&section.file.relative, library);

        Self {
            relative_path: &section.file.relative,
            colocated_path: &section.file.colocated_path,
            ancestors: &section.ancestors,
            draft: section.meta.draft,
            content: &section.content,
            permalink: &section.permalink,
            title: &section.meta.title,
            description: &section.meta.description,
            extra: &section.meta.extra,
            path: &section.path,
            components: &section.components,
            toc: &section.toc,
            word_count: section.word_count,
            reading_time: section.reading_time,
            assets: &section.serialized_assets,
            lang: &section.lang,
            generate_feeds: section.meta.generate_feeds,
            transparent: section.meta.transparent,
            pages,
            subsections,
            translations,
            backlinks,
            paginate_by: &section.meta.paginate_by,
            paginate_reversed: section.meta.paginate_reversed,
        }
    }
}
