use std::collections::{HashMap, HashSet};

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
pub fn build_index(lang: &str, library: &Library) -> Result<String> {
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

    Ok(index.to_json())
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

pub fn build_tantivy_index(
    lang: &str,
    library: &Library,
    output_dir: &str,
    //skip_section_pages: bool,
) -> Result<()> {
    use tantivy::{schema::*, tokenizer::*, Index, Document};
    use tantivy::doc;

    let parsed_lang: Language = parse_language(lang)
    .ok_or_else(|| { Error::from(format!("failed to parse language: '{}'", lang)) })?;

    let tokenizer_name: String = match parsed_lang {
        Language::English => "en_stem".to_string(),
        other => format!("{:?}_stem", parsed_lang).to_lowercase(),
    };

    let mut text_indexing_options = TextFieldIndexing::default()
    .set_index_option(IndexRecordOption::WithFreqsAndPositions)
    .set_tokenizer(&tokenizer_name);

    let text_options = TextOptions::default()
    .set_indexing_options(text_indexing_options)
    .set_stored();

    let mut schema = SchemaBuilder::new();

    let title = schema.add_text_field("title", text_options.clone());
    let body = schema.add_text_field("body", text_options.clone());
    let permalink = schema.add_text_field("permalink", STORED); 
    let schema = schema.build();

    let index_dir = std::path::Path::new(output_dir).join("tantivy-index"); 
    //utils::fs::ensure_directory_exists(&index_dir)?;

    let mut index = Index::create_in_dir(&index_dir, schema.clone())
    .map_err(|e| { Error::from(format!("creating tantivy index failed: {}", e)) })?;

    if index.tokenizers().get(&tokenizer_name).is_none() { // if non-english, we need to register stemmer
        let tokenizer = TextAnalyzer::from(SimpleTokenizer)
        .filter(RemoveLongFilter::limit(40))
        .filter(LowerCaser)
        .filter(Stemmer::new(parsed_lang));
        index.tokenizers().register(&tokenizer_name, tokenizer);
    }

    //let mut wtr = index.writer_with_num_threads(num_cpus::get_physical(), 1024 * 1024 * 256)
    //let mut wtr = index.writer_with_num_threads(4, 1024 * 1024 * 256)
    let mut wtr = index.writer(1024 * 1024 * 256)
    .map_err(|e| { Error::from(format!("creating tantivy index writer failed: {}", e)) })?;

    //index_writer.set_merge_policy(Box::new(NoMergePolicy));

    //let mut sections_it = library.sections_values().iter().filter(|s| s.lang == lang && s.meta.in_search_index);

    let mut seen: HashSet<String> = Default::default();
    let mut n_indexed = 0;
    //let group_size = 100_000;

    for section in library.sections_values() {
        if section.lang != lang { continue }

        if ! section.meta.in_search_index { continue }

        // Don't index redirecting sections
        //if section.meta.redirect_to.is_none() {
        //    index.add_doc(
        //        &section.permalink,
        //        &[
        //            &section.meta.title.clone().unwrap_or_default(),
        //            &AMMONIA.clean(&section.content).to_string(),
        //        ],
        //    );
        //}

        for key in &section.pages {
            let page = library.get_page_by_key(*key);

            if !page.meta.in_search_index { continue; }

            if seen.contains(&page.permalink) { continue }

            seen.insert(page.permalink.clone());


            //let mut doc = Document::default();
            //doc.add(FieldValue::new(title, Value::from(page.meta.title.as_ref().map(|x| x.as_str()).unwrap_or(""))));

            let cleaned_body: String = AMMONIA.clean(&page.content).to_string();
            //doc.add(FieldValue::new(body, Value::from(cleaned_body.as_str())));

            //doc.add(FieldValue::new(permalink, Value::from(page.permalink.as_str())));

            let opstamp = wtr.add_document(doc!(
                title => page.meta.title.as_ref().map(|x| x.as_str()).unwrap_or(""),
                body => cleaned_body.as_str(),
                permalink => page.permalink.as_str(),
            ));

            println!("added {:?} {}", opstamp, page.permalink);

            n_indexed += 1;
             //if n_indexed % group_size == 0 { }
        
    }
}

    wtr.prepare_commit().map_err(|e| { Error::from(format!("tantivy IndexWriter::commit failed: {}", e)) })?;
    let commit_opstamp = wtr.commit().map_err(|e| { Error::from(format!("tantivy IndexWriter::commit failed: {}", e)) })?;
    println!("finished indexing {} pages", n_indexed);
    wtr.wait_merging_threads().map_err(|e| { Error::from(format!("tantivy IndexWriter::wait_merging_threads failed: {}", e)) })?;

    drop(index);

    Ok(())
}
