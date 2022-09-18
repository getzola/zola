use std::collections::HashMap;
use std::path::Path;

use serde::Serialize;

use crate::library::Library;
use crate::taxonomies::SerializedTaxonomy;
use crate::{Page, Section};
use libs::tera::{Map, Value};
use utils::table_of_contents::Heading;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BackLink<'a> {
    pub permalink: &'a str,
    pub title: &'a Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
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
    extra: &'a Map<String, Value>,
    path: &'a str,
    components: &'a [String],
    summary: &'a Option<String>,
    toc: &'a [Heading],
    word_count: Option<usize>,
    reading_time: Option<usize>,
    assets: &'a [String],
    draft: bool,
    lang: &'a str,
    lower: Option<Box<SerializingPage<'a>>>,
    higher: Option<Box<SerializingPage<'a>>>,
    translations: Vec<TranslatedContent<'a>>,
    backlinks: Vec<BackLink<'a>>,
}

impl<'a> SerializingPage<'a> {
    pub fn new(page: &'a Page, library: Option<&'a Library>, include_siblings: bool) -> Self {
        let mut year = None;
        let mut month = None;
        let mut day = None;
        if let Some(d) = page.meta.datetime_tuple {
            year = Some(d.0);
            month = Some(d.1);
            day = Some(d.2);
        }
        let mut lower = None;
        let mut higher = None;
        let mut translations = vec![];
        let mut backlinks = vec![];

        if let Some(lib) = library {
            translations = lib.find_translations(&page.file.canonical);

            if include_siblings {
                lower = page
                    .lower
                    .as_ref()
                    .map(|p| Box::new(Self::new(&lib.pages[p], Some(lib), false)));
                higher = page
                    .higher
                    .as_ref()
                    .map(|p| Box::new(Self::new(&lib.pages[p], Some(lib), false)));
            }

            backlinks = find_backlinks(&page.file.relative, lib);
        }

        Self {
            relative_path: &page.file.relative,
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
            path: &page.path,
            components: &page.components,
            summary: &page.summary,
            toc: &page.toc,
            word_count: page.word_count,
            reading_time: page.reading_time,
            assets: &page.serialized_assets,
            draft: page.meta.draft,
            lang: &page.lang,
            lower,
            higher,
            translations,
            backlinks,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SerializingSection<'a> {
    relative_path: &'a str,
    content: &'a str,
    permalink: &'a str,
    draft: bool,
    ancestors: &'a [String],
    title: &'a Option<String>,
    description: &'a Option<String>,
    extra: &'a Map<String, Value>,
    path: &'a str,
    components: &'a [String],
    toc: &'a [Heading],
    word_count: Option<usize>,
    reading_time: Option<usize>,
    lang: &'a str,
    assets: &'a [String],
    pages: Vec<SerializingPage<'a>>,
    subsections: Vec<&'a str>,
    translations: Vec<TranslatedContent<'a>>,
    backlinks: Vec<BackLink<'a>>,
    taxonomies: Vec<SerializedTaxonomy<'a>>,
}

#[derive(Debug)]
pub enum SectionSerMode<'a> {
    /// Just itself, no pages or subsections
    /// TODO: I believe we can get rid of it?
    ForMarkdown,
    /// Fetches subsections/ancestors/translations but not the pages
    MetadataOnly(&'a Library),
    /// Fetches everything
    Full(&'a Library),
}

impl<'a> SerializingSection<'a> {
    pub fn new(section: &'a Section, mode: SectionSerMode<'a>) -> Self {
        let mut pages = Vec::with_capacity(section.pages.len());
        let mut subsections = Vec::with_capacity(section.subsections.len());
        let mut translations = Vec::new();
        let mut backlinks = Vec::new();
        let mut taxonomies = Vec::new();

        match mode {
            SectionSerMode::ForMarkdown => {}
            SectionSerMode::MetadataOnly(lib) | SectionSerMode::Full(lib) => {
                taxonomies = section
                    .taxonomies
                    .iter()
                    .map(|t| SerializedTaxonomy::from_taxonomy(t, lib))
                    .collect();

                translations = lib.find_translations(&section.file.canonical);
                subsections = section
                    .subsections
                    .iter()
                    .map(|p| lib.sections[p].file.relative.as_str())
                    .collect();

                // Fetching pages on top
                if let SectionSerMode::Full(_) = mode {
                    for p in &section.pages {
                        pages.push(SerializingPage::new(&lib.pages[p], Some(lib), true));
                    }
                }

                backlinks = find_backlinks(&section.file.relative, lib);
            }
        }

        Self {
            relative_path: &section.file.relative,
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
            pages,
            subsections,
            translations,
            backlinks,
            taxonomies,
        }
    }
}
