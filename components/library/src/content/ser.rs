//! What we are sending to the templates when rendering them
use std::collections::HashMap;

use tera::{Map, Value};

use content::{Page, Section};
use library::Library;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TranslatedContent<'a> {
    lang: &'a str,
    permalink: &'a str,
    title: &'a Option<String>,
}

impl<'a> TranslatedContent<'a> {
    // copypaste eh, not worth creating an enum imo
    pub fn find_all_sections(section: &'a Section, library: &'a Library) -> Vec<Self> {
        let mut translations = vec![];

        for key in &section.translations {
            let other = library.get_section_by_key(*key);
            translations.push(TranslatedContent {
                lang: &other.lang,
                permalink: &other.permalink,
                title: &other.meta.title,
            });
        }

        translations
    }

    pub fn find_all_pages(page: &'a Page, library: &'a Library) -> Vec<Self> {
        let mut translations = vec![];

        for key in &page.translations {
            let other = library.get_page_by_key(*key);
            translations.push(TranslatedContent {
                lang: &other.lang,
                permalink: &other.permalink,
                title: &other.meta.title,
            });
        }

        translations
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SerializingPage<'a> {
    relative_path: &'a str,
    content: &'a str,
    permalink: &'a str,
    slug: &'a str,
    ancestors: Vec<String>,
    title: &'a Option<String>,
    description: &'a Option<String>,
    date: &'a Option<String>,
    year: Option<i32>,
    month: Option<u32>,
    day: Option<u32>,
    taxonomies: &'a HashMap<String, Vec<String>>,
    extra: &'a Map<String, Value>,
    path: &'a str,
    components: &'a [String],
    summary: &'a Option<String>,
    word_count: Option<usize>,
    reading_time: Option<usize>,
    assets: &'a [String],
    draft: bool,
    lang: &'a str,
    lighter: Option<Box<SerializingPage<'a>>>,
    heavier: Option<Box<SerializingPage<'a>>>,
    earlier: Option<Box<SerializingPage<'a>>>,
    later: Option<Box<SerializingPage<'a>>>,
    translations: Vec<TranslatedContent<'a>>,
}

impl<'a> SerializingPage<'a> {
    /// Grabs all the data from a page, including sibling pages
    pub fn from_page(page: &'a Page, library: &'a Library) -> Self {
        let mut year = None;
        let mut month = None;
        let mut day = None;
        if let Some(d) = page.meta.datetime_tuple {
            year = Some(d.0);
            month = Some(d.1);
            day = Some(d.2);
        }
        let pages = library.pages();
        let lighter = page
            .lighter
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let heavier = page
            .heavier
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let earlier = page
            .earlier
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let later = page
            .later
            .map(|k| Box::new(Self::from_page_basic(pages.get(k).unwrap(), Some(library))));
        let ancestors = page
            .ancestors
            .iter()
            .map(|k| library.get_section_by_key(*k).file.relative.clone())
            .collect();

        let translations = TranslatedContent::find_all_pages(page, library);

        SerializingPage {
            relative_path: &page.file.relative,
            ancestors,
            content: &page.content,
            permalink: &page.permalink,
            slug: &page.slug,
            title: &page.meta.title,
            description: &page.meta.description,
            extra: &page.meta.extra,
            date: &page.meta.date,
            year,
            month,
            day,
            taxonomies: &page.meta.taxonomies,
            path: &page.path,
            components: &page.components,
            summary: &page.summary,
            word_count: page.word_count,
            reading_time: page.reading_time,
            assets: &page.serialized_assets,
            draft: page.is_draft(),
            lang: &page.lang,
            lighter,
            heavier,
            earlier,
            later,
            translations,
        }
    }

    /// Same as from_page but does not fill sibling pages
    pub fn from_page_basic(page: &'a Page, library: Option<&'a Library>) -> Self {
        let mut year = None;
        let mut month = None;
        let mut day = None;
        if let Some(d) = page.meta.datetime_tuple {
            year = Some(d.0);
            month = Some(d.1);
            day = Some(d.2);
        }
        let ancestors = if let Some(ref lib) = library {
            page.ancestors
                .iter()
                .map(|k| lib.get_section_by_key(*k).file.relative.clone())
                .collect()
        } else {
            vec![]
        };

        let translations = if let Some(ref lib) = library {
            TranslatedContent::find_all_pages(page, lib)
        } else {
            vec![]
        };

        SerializingPage {
            relative_path: &page.file.relative,
            ancestors,
            content: &page.content,
            permalink: &page.permalink,
            slug: &page.slug,
            title: &page.meta.title,
            description: &page.meta.description,
            extra: &page.meta.extra,
            date: &page.meta.date,
            year,
            month,
            day,
            taxonomies: &page.meta.taxonomies,
            path: &page.path,
            components: &page.components,
            summary: &page.summary,
            word_count: page.word_count,
            reading_time: page.reading_time,
            assets: &page.serialized_assets,
            draft: page.is_draft(),
            lang: &page.lang,
            lighter: None,
            heavier: None,
            earlier: None,
            later: None,
            translations,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SerializingSection<'a> {
    relative_path: &'a str,
    content: &'a str,
    permalink: &'a str,
    ancestors: Vec<String>,
    title: &'a Option<String>,
    description: &'a Option<String>,
    extra: &'a HashMap<String, Value>,
    path: &'a str,
    components: &'a [String],
    word_count: Option<usize>,
    reading_time: Option<usize>,
    lang: &'a str,
    assets: &'a [String],
    pages: Vec<SerializingPage<'a>>,
    subsections: Vec<&'a str>,
    translations: Vec<TranslatedContent<'a>>,
}

impl<'a> SerializingSection<'a> {
    pub fn from_section(section: &'a Section, library: &'a Library) -> Self {
        let mut pages = Vec::with_capacity(section.pages.len());
        let mut subsections = Vec::with_capacity(section.subsections.len());

        for k in &section.pages {
            pages.push(library.get_page_by_key(*k).to_serialized_basic(library));
        }

        for k in &section.subsections {
            subsections.push(library.get_section_path_by_key(*k));
        }

        let ancestors = section
            .ancestors
            .iter()
            .map(|k| library.get_section_by_key(*k).file.relative.clone())
            .collect();
        let translations = TranslatedContent::find_all_sections(section, library);

        SerializingSection {
            relative_path: &section.file.relative,
            ancestors,
            content: &section.content,
            permalink: &section.permalink,
            title: &section.meta.title,
            description: &section.meta.description,
            extra: &section.meta.extra,
            path: &section.path,
            components: &section.components,
            word_count: section.word_count,
            reading_time: section.reading_time,
            assets: &section.serialized_assets,
            lang: &section.lang,
            pages,
            subsections,
            translations,
        }
    }

    /// Same as from_section but doesn't fetch pages and sections
    pub fn from_section_basic(section: &'a Section, library: Option<&'a Library>) -> Self {
        let ancestors = if let Some(ref lib) = library {
            section
                .ancestors
                .iter()
                .map(|k| lib.get_section_by_key(*k).file.relative.clone())
                .collect()
        } else {
            vec![]
        };

        let translations = if let Some(ref lib) = library {
            TranslatedContent::find_all_sections(section, lib)
        } else {
            vec![]
        };

        SerializingSection {
            relative_path: &section.file.relative,
            ancestors,
            content: &section.content,
            permalink: &section.permalink,
            title: &section.meta.title,
            description: &section.meta.description,
            extra: &section.meta.extra,
            path: &section.path,
            components: &section.components,
            word_count: section.word_count,
            reading_time: section.reading_time,
            assets: &section.serialized_assets,
            lang: &section.lang,
            pages: vec![],
            subsections: vec![],
            translations,
        }
    }
}
