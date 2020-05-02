use std::str::FromStr;
use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc, NaiveDateTime, TimeZone};
use elasticlunr::{Index, Language};
use lazy_static::lazy_static;

use errors::{bail, Result, Error};
use library::{Library, Section};

pub const ELASTICLUNR_JS: &str = include_str!("elasticlunr.min.js");

lazy_static! {
    static ref AMMONIA: ammonia::Builder<'static> = {
        let mut clean_content = HashSet::new();
        clean_content.insert("script");
        clean_content.insert("style");
        let mut builder = ammonia::Builder::new();
        builder
            .tags(HashSet::new())
            .tag_attributes(HashMap::new())
            .generic_attributes(HashSet::new())
            .link_rel(None)
            .allowed_classes(HashMap::new())
            .clean_content_tags(clean_content);
        builder
    };
}

/// Returns the generated JSON index with all the documents of the site added using
/// the language given
/// Errors if the language given is not available in Elasticlunr
/// TODO: is making `in_search_index` apply to subsections of a `false` section useful?
pub fn build_index(lang: &str, library: &Library) -> Result<Index> {
    let language = match Language::from_code(lang) {
        Some(l) => l,
        None => {
            bail!("Tried to build search index for language {} which is not supported", lang);
        }
    };

    let mut index = Index::with_language(language, &["title", "body"]);

    for section in library.sections_values() {
        if section.lang == lang {
            add_section_to_index(&mut index, section, library);
        }
    }

    Ok(index)
}

fn add_section_to_index(index: &mut Index, section: &Section, library: &Library) {
    if !section.meta.in_search_index {
        return;
    }

    // Don't index redirecting sections
    if section.meta.redirect_to.is_none() {
        index.add_doc(
            &section.permalink,
            &[
                &section.meta.title.clone().unwrap_or_default(),
                &AMMONIA.clean(&section.content).to_string(),
            ],
        );
    }

    for key in &section.pages {
        let page = library.get_page_by_key(*key);
        if !page.meta.in_search_index {
            continue;
        }

        index.add_doc(
            &page.permalink,
            &[
                &page.meta.title.clone().unwrap_or_default(),
                &AMMONIA.clean(&page.content).to_string(),
            ],
        );
    }
}

fn parse_language(lang: &str) -> Option<tantivy::tokenizer::Language> {
    use serde_derive::Deserialize;
    #[derive(Deserialize)]
    struct Lang {
        pub language: tantivy::tokenizer::Language,
    }

    // expecting two-character code, but will try other forms as fallback
    match lang.len() {
        2 => isolang::Language::from_639_1(&lang.to_lowercase())
        .and_then(|parsed| {
            let json = format!("{{\"language\":\"{}\"}}", parsed.to_name());
            serde_json::from_str::<Lang>(&json).ok().map(|Lang { language }| language)
        }),

        3 => isolang::Language::from_639_3(&lang.to_lowercase())
        .and_then(|parsed| {
            serde_json::from_str::<tantivy::tokenizer::Language>(parsed.to_name()).ok()
        }),

        // apparently not a code, so this is best available option
        _ => serde_json::from_str::<tantivy::tokenizer::Language>(lang).ok()
    }
}

fn parse_dt_assume_utc(datetime_string: &Option<String>, naive_datetime: &Option<NaiveDateTime>) -> Option<DateTime<Utc>> {
    // start here because it will potentially have timezone in the string
    if let Some(s) = datetime_string.as_ref() {
        if let Ok(utc) = DateTime::from_str(s.as_str()) {
            return Some(utc)
        }
    }

    // otherwise, if we have the NaiveDateTime, we'll assume it's UTC. would not do this if the
    // stakes were higher!
    if let Some(naive) = naive_datetime {
        return Some(Utc.from_utc_datetime(&naive))
    }

    None
}

fn normalize_taxonomy_name(s: &str) -> String {
    s.replace("-", "_")
}

pub fn build_tantivy_index<P: AsRef<std::path::Path>>(
    lang: &str,
    library: &Library,
    index_dir: P,
) -> Result<usize> {
    use tantivy::{schema::*, tokenizer::*, Index, Document};
    use tantivy::doc;

    let parsed_lang: Language = parse_language(lang)
    .ok_or_else(|| { Error::from(format!("failed to parse language: '{}'", lang)) })?;

    let tokenizer_name: String = match parsed_lang {
        Language::English => "en_stem".to_string(),
        other => format!("{:?}_stem", other).to_lowercase(),
    };

    let text_indexing_options = TextFieldIndexing::default()
    .set_index_option(IndexRecordOption::WithFreqsAndPositions)
    .set_tokenizer(&tokenizer_name);

    let text_options = TextOptions::default()
    .set_indexing_options(text_indexing_options)
    .set_stored();

    struct IndexContent<'a> {
        pub title: &'a str,
        pub description: &'a str,
        pub permalink: &'a str,
        pub body: String,

        pub datetime: Option<DateTime<Utc>>,
        pub taxonomies: &'a HashMap<String, Vec<String>>,
    }

    let mut seen: HashSet<String> = Default::default(); // unique permalinks already indexed
    let mut all_taxonomies: HashSet<String> = Default::default(); // remember any taxonomy used anywhere so we can add to schema 
    let mut index_pages: Vec<IndexContent> = Vec::new();
    let mut n_indexed = 0;

    let empty_taxonomies: HashMap<String, Vec<String>> = Default::default();

    for section in library.sections_values() {

        // reason for macro: Section/Page are different types but have same attributes
        macro_rules! extract_content {
            ($page:ident) => {{
                let already_indexed = seen.contains(&$page.permalink);
                if ! already_indexed  && $page.meta.in_search_index && $page.lang == lang {
                    seen.insert($page.permalink.clone()); // mark ask indexed
                    n_indexed += 1;

                    let cleaned_body: String = AMMONIA.clean(&$page.content).to_string();

                    Some(IndexContent {
                        title: $page.meta.title.as_ref().map(|x| x.as_str()).unwrap_or(""),
                        description: $page.meta.description.as_ref().map(|x| x.as_str()).unwrap_or(""),
                        permalink:  $page.permalink.as_str(),
                        body: cleaned_body,

                        // page-only fields, leave blank
                        datetime: None,
                        taxonomies: &empty_taxonomies,
                    })
                } else {
                    None
                }
            }}
        }

        if section.meta.redirect_to.is_none() {
            if let Some(content) = extract_content!(section) {
                index_pages.push(content);
            }
        }

        for key in &section.pages {
            let page = library.get_page_by_key(*key);
            match extract_content!(page) {
                Some(mut index_content) => {
                    all_taxonomies.extend(page.meta.taxonomies.keys().map(|x| normalize_taxonomy_name(x)));
                    index_content.taxonomies = &page.meta.taxonomies;
                    index_content.datetime = parse_dt_assume_utc(&page.meta.date, &page.meta.datetime);
                    index_pages.push(index_content);
                }
                None => {}
            }
        }
    }

    let mut schema = SchemaBuilder::new();

    let mut fields: HashMap<String, Field> = Default::default();

    for text_field_name in &["title", "body", "description"] {
        fields.insert(text_field_name.to_string(), schema.add_text_field(text_field_name, text_options.clone()));
    }
    fields.insert("permalink".to_string(), schema.add_text_field("permalink", STORED)); 
    fields.insert("datetime".to_string(), schema.add_date_field("datetime", STORED | INDEXED)); 

    let reserved_field_names: HashSet<String> = fields.keys().map(|s| s.to_string()).collect();

    for taxonomy_name in all_taxonomies.difference(&reserved_field_names) {
        fields.insert(taxonomy_name.to_string(), schema.add_text_field(taxonomy_name.as_str(), text_options.clone()));
    }

    let schema = schema.build(); 

    let index = Index::create_in_dir(&index_dir, schema.clone())
        .map_err(|e| { Error::from(format!("creating tantivy index failed: {}", e)) })?;

    // take care of non-English stemmers if needed
    if index.tokenizers().get(&tokenizer_name).is_none() {
        let tokenizer = TextAnalyzer::from(SimpleTokenizer)
            .filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .filter(Stemmer::new(parsed_lang));
        index.tokenizers().register(&tokenizer_name, tokenizer);
    }

    let mut wtr = index.writer(1024 * 1024 * 256)
        .map_err(|e| { Error::from(format!("creating tantivy index writer failed: {}", e)) })?;

    // now, let's index!

    for page in index_pages {
        let mut document: Document = doc!(
            fields["title"] => page.title,
            fields["description"] => page.description,
            fields["permalink"] => page.permalink,
            fields["body"] => page.body,
        );

        if let Some(utc) = page.datetime {
            document.add_date(fields["datetime"], &utc);
        }

        for (taxonomy, terms) in page.taxonomies.iter().filter(|(k, _)| ! reserved_field_names.contains(k.as_str())) {
            let normalized_taxonomy = normalize_taxonomy_name(taxonomy);
            for term in terms.iter() {
                document.add_text(fields[&normalized_taxonomy], term);
            }
        }

        wtr.add_document(document);
    }


    //wtr.prepare_commit().map_err(|e| { Error::from(format!("tantivy IndexWriter::commit failed: {}", e)) })?;
    wtr.commit().map_err(|e| { Error::from(format!("tantivy IndexWriter::commit failed: {}", e)) })?;
    wtr.wait_merging_threads().map_err(|e| { Error::from(format!("tantivy IndexWriter::wait_merging_threads failed: {}", e)) })?;

    drop(index);

    Ok(n_indexed)
}