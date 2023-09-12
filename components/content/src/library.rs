use std::path::{Path, PathBuf};

use config::Config;
use libs::ahash::{AHashMap, AHashSet};

use crate::ser::TranslatedContent;
use crate::sorting::sort_pages;
use crate::taxonomies::{Taxonomy, TaxonomyFound};
use crate::{Page, Section, SortBy};

macro_rules! set {
    ($($key:expr,)+) => (set!($($key),+));

    ( $($key:expr),* ) => {
        {
            let mut _set = AHashSet::new();
            $(
                _set.insert($key);
            )*
            _set
        }
    };
}

#[derive(Debug, Default)]
pub struct Library {
    pub pages: AHashMap<PathBuf, Page>,
    pub sections: AHashMap<PathBuf, Section>,
    // aliases -> files, so we can easily check for conflicts
    pub reverse_aliases: AHashMap<String, AHashSet<PathBuf>>,
    pub translations: AHashMap<PathBuf, AHashSet<PathBuf>>,
    pub backlinks: AHashMap<String, AHashSet<PathBuf>>,
    // A mapping of {lang -> <slug, {term -> vec<paths>}>>}
    taxonomies_def: AHashMap<String, AHashMap<String, AHashMap<String, Vec<PathBuf>>>>,
    // All the taxonomies from config.toml in their slugifiedv ersion
    // So we don't need to pass the Config when adding a page to know how to slugify and we only
    // slugify once
    taxo_name_to_slug: AHashMap<String, String>,
}

impl Library {
    pub fn new(config: &Config) -> Self {
        let mut lib = Self::default();

        for (lang, options) in &config.languages {
            let mut taxas = AHashMap::new();
            for tax_def in &options.taxonomies {
                taxas.insert(tax_def.slug.clone(), AHashMap::new());
                lib.taxo_name_to_slug.insert(tax_def.name.clone(), tax_def.slug.clone());
            }
            lib.taxonomies_def.insert(lang.to_string(), taxas);
        }
        lib
    }

    fn insert_reverse_aliases(&mut self, file_path: &Path, entries: Vec<String>) {
        for entry in entries {
            self.reverse_aliases
                .entry(entry)
                .and_modify(|s| {
                    s.insert(file_path.to_path_buf());
                })
                .or_insert_with(|| set! {file_path.to_path_buf()});
        }
    }

    /// This will check every section/page paths + the aliases and ensure none of them
    /// are colliding.
    /// Returns Vec<(path colliding, [list of files causing that collision])>
    pub fn find_path_collisions(&self) -> Vec<(String, Vec<PathBuf>)> {
        self.reverse_aliases
            .iter()
            .filter_map(|(alias, files)| {
                if files.len() > 1 {
                    Some((alias.clone(), files.clone().into_iter().collect::<Vec<_>>()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn insert_page(&mut self, page: Page) {
        let file_path = page.file.path.clone();
        let mut entries = vec![page.path.clone()];
        entries.extend(page.meta.aliases.to_vec());
        self.insert_reverse_aliases(&file_path, entries);

        for (taxa_name, terms) in &page.meta.taxonomies {
            for term in terms {
                // Safe unwraps as we create all lang/taxa and we validated that they are correct
                // before getting there
                let taxa_def = self
                    .taxonomies_def
                    .get_mut(&page.lang)
                    .expect("lang not found")
                    .get_mut(&self.taxo_name_to_slug[taxa_name])
                    .expect("taxa not found");

                if !taxa_def.contains_key(term) {
                    taxa_def.insert(term.to_string(), Vec::new());
                }
                taxa_def.get_mut(term).unwrap().push(page.file.path.clone());
            }
        }

        self.pages.insert(file_path, page);
    }

    pub fn insert_section(&mut self, section: Section) {
        let file_path = section.file.path.clone();
        if section.meta.render {
            let mut entries = vec![section.path.clone()];
            entries.extend(section.meta.aliases.to_vec());
            self.insert_reverse_aliases(&file_path, entries);
        }
        self.sections.insert(file_path, section);
    }

    /// Fills a map of target -> {content mentioning it}
    /// This can only be called _after_ rendering markdown as we need to have accumulated all
    /// the links first
    pub fn fill_backlinks(&mut self) {
        self.backlinks.clear();

        let mut add_backlink = |target: &str, source: &Path| {
            self.backlinks
                .entry(target.to_owned())
                .and_modify(|s| {
                    s.insert(source.to_path_buf());
                })
                .or_insert(set! {source.to_path_buf()});
        };

        for (_, page) in &self.pages {
            for (internal_link, _) in &page.internal_links {
                add_backlink(internal_link, &page.file.path);
            }
        }
        for (_, section) in &self.sections {
            for (internal_link, _) in &section.internal_links {
                add_backlink(internal_link, &section.file.path);
            }
        }
    }

    /// This is called _before_ rendering the markdown the pages/sections
    pub fn find_taxonomies(&self, config: &Config) -> Vec<Taxonomy> {
        let mut taxonomies = Vec::new();

        for (lang, taxonomies_data) in &self.taxonomies_def {
            for (taxa_slug, terms_pages) in taxonomies_data {
                let taxo_config = &config.languages[lang]
                    .taxonomies
                    .iter()
                    .find(|t| &t.slug == taxa_slug)
                    .expect("taxo should exist");
                let mut taxo_found = TaxonomyFound::new(taxa_slug.to_string(), lang, taxo_config);
                for (term, page_path) in terms_pages {
                    taxo_found
                        .terms
                        .insert(term, page_path.iter().map(|p| &self.pages[p]).collect());
                }

                taxonomies.push(Taxonomy::new(taxo_found, config));
            }
        }

        taxonomies
    }

    /// Sort all sections pages according to sorting method given
    /// Pages that cannot be sorted are set to the section.ignored_pages instead
    pub fn sort_section_pages(&mut self) {
        let mut updates = AHashMap::new();
        for (path, section) in &self.sections {
            let pages: Vec<_> = section.pages.iter().map(|p| &self.pages[p]).collect();
            let (sorted_pages, cannot_be_sorted_pages) = match section.meta.sort_by {
                SortBy::None => continue,
                _ => sort_pages(&pages, section.meta.sort_by),
            };

            updates
                .insert(path.clone(), (sorted_pages, cannot_be_sorted_pages, section.meta.sort_by));
        }

        for (path, (sorted, unsortable, _)) in updates {
            if !self.sections[&path].meta.transparent {
                // Fill siblings
                for (i, page_path) in sorted.iter().enumerate() {
                    let p = self.pages.get_mut(page_path).unwrap();
                    if i > 0 {
                        // lighter / later / title_prev
                        p.lower = Some(sorted[i - 1].clone());
                    }

                    if i < sorted.len() - 1 {
                        // heavier / earlier / title_next
                        p.higher = Some(sorted[i + 1].clone());
                    }
                }
            }

            if let Some(s) = self.sections.get_mut(&path) {
                s.pages = sorted;
                s.ignored_pages = unsortable;
            }
        }
    }

    /// Find out the direct subsections of each subsection if there are some
    /// as well as the pages for each section
    pub fn populate_sections(&mut self, config: &Config, content_path: &Path) {
        let mut add_translation = |entry: &Path, path: &Path| {
            if config.is_multilingual() {
                self.translations
                    .entry(entry.to_path_buf())
                    .and_modify(|trans| {
                        trans.insert(path.to_path_buf());
                    })
                    .or_insert(set! {path.to_path_buf()});
            }
        };

        let mut ancestors = AHashMap::new();
        let mut subsections = AHashMap::new();
        let mut sections_weight = AHashMap::new();

        // We iterate over the sections twice
        // The first time to build up the list of ancestors for each section
        for (path, section) in &self.sections {
            sections_weight.insert(path.clone(), section.meta.weight);
            if let Some(ref grand_parent) = section.file.grand_parent {
                subsections
                    // Using the original filename to work for multi-lingual sections
                    .entry(grand_parent.join(&section.file.filename))
                    .or_insert_with(Vec::new)
                    .push(section.file.path.clone());
            }

            add_translation(&section.file.canonical, path);

            // Root sections have no ancestors
            if section.is_index() {
                ancestors.insert(section.file.path.clone(), vec![]);
                continue;
            }

            // Index section is the first ancestor of every single section
            let mut cur_path = content_path.to_path_buf();
            let mut parents = vec![section.file.filename.clone()];
            for component in &section.file.components {
                cur_path = cur_path.join(component);
                // Skip itself
                if cur_path == section.file.parent {
                    continue;
                }

                let index_path = cur_path.join(&section.file.filename);
                if let Some(s) = self.sections.get(&index_path) {
                    parents.push(s.file.relative.clone());
                }
            }
            ancestors.insert(section.file.path.clone(), parents);
        }

        // The second time we actually assign ancestors and order subsections based on their weights
        for (path, section) in self.sections.iter_mut() {
            section.subsections.clear();
            section.pages.clear();
            section.ignored_pages.clear();
            section.ancestors.clear();

            if let Some(children) = subsections.get(path) {
                let mut children: Vec<_> = children.clone();
                children.sort_by(|a, b| sections_weight[a].cmp(&sections_weight[b]));
                section.subsections = children;
            }
            if let Some(parents) = ancestors.get(path) {
                section.ancestors = parents.clone();
            }
        }

        // We pre-build the index filename for each language
        let mut index_filename_by_lang = AHashMap::with_capacity(config.languages.len());
        for code in config.languages.keys() {
            if code == &config.default_language {
                index_filename_by_lang.insert(code, "_index.md".to_owned());
            } else {
                index_filename_by_lang.insert(code, format!("_index.{}.md", code));
            }
        }

        // Then once we took care of the sections, we find the pages of each section
        for (path, page) in self.pages.iter_mut() {
            let parent_filename = &index_filename_by_lang[&page.lang];
            add_translation(&page.file.canonical, path);
            let mut parent_section_path = page.file.parent.join(parent_filename);

            while let Some(parent_section) = self.sections.get_mut(&parent_section_path) {
                let is_transparent = parent_section.meta.transparent;
                parent_section.pages.push(path.clone());
                page.ancestors = ancestors.get(&parent_section_path).cloned().unwrap_or_default();
                // Don't forget to push the actual parent
                page.ancestors.push(parent_section.file.relative.clone());

                // Find the page template if one of a parent has page_template set
                // Stops after the first one found, keep in mind page.ancestors
                // is [index, ..., parent] so we need to reverse it first
                if page.meta.template.is_none() {
                    for ancestor in page.ancestors.iter().rev() {
                        let s = self.sections.get(&content_path.join(ancestor)).unwrap();
                        if let Some(ref tpl) = s.meta.page_template {
                            page.meta.template = Some(tpl.clone());
                            break;
                        }
                    }
                }

                if !is_transparent {
                    break;
                }

                // We've added `_index(.{LANG})?.md` so if we are here so we need to go up twice
                match parent_section_path.clone().parent().unwrap().parent() {
                    Some(parent) => parent_section_path = parent.join(parent_filename),
                    None => break,
                }
            }
        }

        // And once we have all the pages assigned to their section, we sort them
        self.sort_section_pages();
    }

    /// Find all the orphan pages: pages that are in a folder without an `_index.md`
    pub fn get_all_orphan_pages(&self) -> Vec<&Page> {
        self.pages.iter().filter(|(_, p)| p.ancestors.is_empty()).map(|(_, p)| p).collect()
    }

    /// Find all the translated content for a given canonical path.
    /// The translated content can be either for a section or a page
    pub fn find_translations(&self, canonical_path: &Path) -> Vec<TranslatedContent<'_>> {
        let mut translations = vec![];

        if let Some(paths) = self.translations.get(canonical_path) {
            for path in paths {
                let (lang, permalink, title, path) = {
                    if self.sections.contains_key(path) {
                        let s = &self.sections[path];
                        (&s.lang, &s.permalink, &s.meta.title, &s.file.path)
                    } else {
                        let s = &self.pages[path];
                        (&s.lang, &s.permalink, &s.meta.title, &s.file.path)
                    }
                };
                translations.push(TranslatedContent { lang, permalink, title, path });
            }
        }

        translations
    }

    pub fn find_pages_by_path(&self, paths: &[PathBuf]) -> Vec<&Page> {
        paths.iter().map(|p| &self.pages[p]).collect()
    }

    pub fn find_sections_by_path(&self, paths: &[PathBuf]) -> Vec<&Section> {
        paths.iter().map(|p| &self.sections[p]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileInfo;
    use config::{LanguageOptions, TaxonomyConfig};
    use std::collections::HashMap;
    use utils::slugs::SlugifyStrategy;

    #[test]
    fn can_find_collisions_with_paths() {
        let mut library = Library::default();
        let mut section = Section { path: "hello".to_owned(), ..Default::default() };
        section.file.path = PathBuf::from("hello.md");
        library.insert_section(section.clone());
        let mut section2 = Section { path: "hello".to_owned(), ..Default::default() };
        section2.file.path = PathBuf::from("bonjour.md");
        library.insert_section(section2.clone());

        let collisions = library.find_path_collisions();
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].0, "hello");
        assert!(collisions[0].1.contains(&section.file.path));
        assert!(collisions[0].1.contains(&section2.file.path));
    }

    #[test]
    fn can_find_collisions_with_aliases() {
        let mut library = Library::default();
        let mut section = Section { path: "hello".to_owned(), ..Default::default() };
        section.file.path = PathBuf::from("hello.md");
        library.insert_section(section.clone());
        let mut section2 = Section { path: "world".to_owned(), ..Default::default() };
        section2.file.path = PathBuf::from("bonjour.md");
        section2.meta.aliases = vec!["hello".to_owned(), "hola".to_owned()];
        library.insert_section(section2.clone());
        // Sections with render=false do not collide with anything
        // https://github.com/getzola/zola/issues/1656
        let mut section3 = Section { path: "world2".to_owned(), ..Default::default() };
        section3.meta.render = false;
        section3.file.path = PathBuf::from("bonjour2.md");
        section3.meta.aliases = vec!["hola".to_owned()];
        library.insert_section(section3);

        let collisions = library.find_path_collisions();
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].0, "hello");
        assert!(collisions[0].1.contains(&section.file.path));
        assert!(collisions[0].1.contains(&section2.file.path));
    }

    #[derive(Debug, Clone)]
    enum PageSort {
        None,
        Date(&'static str),
        Title(&'static str),
        Weight(usize),
    }

    fn create_page(file_path: &str, lang: &str, page_sort: PageSort) -> Page {
        let mut page = Page::default();
        page.lang = lang.to_owned();
        page.file = FileInfo::new_page(Path::new(file_path), &PathBuf::new());
        match page_sort {
            PageSort::None => (),
            PageSort::Date(date) => {
                page.meta.date = Some(date.to_owned());
                page.meta.date_to_datetime();
            }
            PageSort::Title(title) => {
                page.meta.title = Some(title.to_owned());
            }
            PageSort::Weight(w) => {
                page.meta.weight = Some(w);
            }
        }
        page.file.find_language("en", &["fr"]).unwrap();
        page
    }

    fn create_section(
        file_path: &str,
        lang: &str,
        weight: usize,
        transparent: bool,
        sort_by: SortBy,
    ) -> Section {
        let mut section = Section::default();
        section.lang = lang.to_owned();
        section.file = FileInfo::new_section(Path::new(file_path), &PathBuf::new());
        section.meta.weight = weight;
        section.meta.transparent = transparent;
        section.meta.sort_by = sort_by;
        section.meta.page_template = Some("new_page.html".to_owned());
        section.file.find_language("en", &["fr"]).unwrap();
        section
    }

    #[test]
    fn can_populate_sections() {
        let mut config = Config::default_for_test();
        config.languages.insert("fr".to_owned(), LanguageOptions::default());
        let mut library = Library::default();
        let sections = vec![
            ("content/_index.md", "en", 0, false, SortBy::None),
            ("content/_index.fr.md", "fr", 0, false, SortBy::None),
            ("content/blog/_index.md", "en", 0, false, SortBy::Date),
            ("content/wiki/_index.md", "en", 0, false, SortBy::Weight),
            ("content/wiki/_index.fr.md", "fr", 0, false, SortBy::Weight),
            ("content/wiki/recipes/_index.md", "en", 1, true, SortBy::Weight),
            ("content/wiki/recipes/_index.fr.md", "fr", 1, true, SortBy::Weight),
            ("content/wiki/programming/_index.md", "en", 10, true, SortBy::Weight),
            ("content/wiki/programming/_index.fr.md", "fr", 10, true, SortBy::Weight),
            ("content/novels/_index.md", "en", 10, false, SortBy::Title),
            ("content/novels/_index.fr.md", "fr", 10, false, SortBy::Title),
        ];
        for (p, l, w, t, s) in sections.clone() {
            library.insert_section(create_section(p, l, w, t, s));
        }

        let pages = vec![
            ("content/about.md", "en", PageSort::None),
            ("content/about.fr.md", "en", PageSort::None),
            ("content/blog/rust.md", "en", PageSort::Date("2022-01-01")),
            ("content/blog/python.md", "en", PageSort::Date("2022-03-03")),
            ("content/blog/docker.md", "en", PageSort::Date("2022-02-02")),
            ("content/wiki/recipes/chocolate-cake.md", "en", PageSort::Weight(100)),
            ("content/wiki/recipes/chocolate-cake.fr.md", "fr", PageSort::Weight(100)),
            ("content/wiki/recipes/rendang.md", "en", PageSort::Weight(5)),
            ("content/wiki/recipes/rendang.fr.md", "fr", PageSort::Weight(5)),
            ("content/wiki/programming/rust.md", "en", PageSort::Weight(1)),
            ("content/wiki/programming/rust.fr.md", "fr", PageSort::Weight(1)),
            ("content/wiki/programming/zola.md", "en", PageSort::Weight(10)),
            ("content/wiki/programming/python.md", "en", PageSort::None),
            ("content/novels/the-colour-of-magic.md", "en", PageSort::Title("The Colour of Magic")),
            (
                "content/novels/the-colour-of-magic.fr.md",
                "en",
                PageSort::Title("La Huitième Couleur"),
            ),
            ("content/novels/reaper.md", "en", PageSort::Title("Reaper")),
            ("content/novels/reaper.fr.md", "fr", PageSort::Title("Reaper (fr)")),
            ("content/random/hello.md", "en", PageSort::None),
        ];
        for (p, l, s) in pages.clone() {
            library.insert_page(create_page(p, l, s));
        }
        library.populate_sections(&config, Path::new("content"));
        assert_eq!(library.sections.len(), sections.len());
        assert_eq!(library.pages.len(), pages.len());
        let blog_section = &library.sections[&PathBuf::from("content/blog/_index.md")];
        assert_eq!(blog_section.pages.len(), 3);
        // sorted by date in desc order
        assert_eq!(
            blog_section.pages,
            vec![
                PathBuf::from("content/blog/python.md"),
                PathBuf::from("content/blog/docker.md"),
                PathBuf::from("content/blog/rust.md")
            ]
        );
        assert_eq!(blog_section.ignored_pages.len(), 0);
        assert!(&library.pages[&PathBuf::from("content/blog/python.md")].lower.is_none());
        assert_eq!(
            &library.pages[&PathBuf::from("content/blog/python.md")].higher,
            &Some(PathBuf::from("content/blog/docker.md"))
        );
        assert_eq!(
            library.pages[&PathBuf::from("content/blog/python.md")].meta.template,
            Some("new_page.html".to_owned())
        );

        let wiki = &library.sections[&PathBuf::from("content/wiki/_index.md")];
        assert_eq!(wiki.pages.len(), 4);
        // sorted by weight, in asc order
        assert_eq!(
            wiki.pages,
            vec![
                PathBuf::from("content/wiki/programming/rust.md"),
                PathBuf::from("content/wiki/recipes/rendang.md"),
                PathBuf::from("content/wiki/programming/zola.md"),
                PathBuf::from("content/wiki/recipes/chocolate-cake.md"),
            ]
        );
        assert_eq!(wiki.ignored_pages.len(), 1);
        assert_eq!(wiki.ignored_pages, vec![PathBuf::from("content/wiki/programming/python.md")]);
        assert_eq!(
            &library.pages[&PathBuf::from("content/wiki/recipes/rendang.md")].lower,
            &Some(PathBuf::from("content/wiki/programming/rust.md"))
        );
        assert_eq!(
            &library.pages[&PathBuf::from("content/wiki/recipes/rendang.md")].higher,
            &Some(PathBuf::from("content/wiki/programming/zola.md"))
        );
        assert_eq!(
            wiki.subsections,
            vec![
                PathBuf::from("content/wiki/recipes/_index.md"),
                PathBuf::from("content/wiki/programming/_index.md")
            ]
        );
        assert_eq!(wiki.ancestors, vec!["_index.md".to_owned()]);
        assert_eq!(
            library.sections[&PathBuf::from("content/wiki/recipes/_index.md")].ancestors,
            vec!["_index.md".to_owned(), "wiki/_index.md".to_owned()]
        );

        // also works for other languages
        let french_wiki = &library.sections[&PathBuf::from("content/wiki/_index.fr.md")];
        assert_eq!(french_wiki.pages.len(), 3);
        // sorted by weight, in asc order
        assert_eq!(
            french_wiki.pages,
            vec![
                PathBuf::from("content/wiki/programming/rust.fr.md"),
                PathBuf::from("content/wiki/recipes/rendang.fr.md"),
                PathBuf::from("content/wiki/recipes/chocolate-cake.fr.md"),
            ]
        );
        assert_eq!(french_wiki.ignored_pages.len(), 0);
        assert!(&library.pages[&PathBuf::from("content/wiki/recipes/chocolate-cake.fr.md")]
            .higher
            .is_none());
        assert_eq!(
            &library.pages[&PathBuf::from("content/wiki/recipes/chocolate-cake.fr.md")].lower,
            &Some(PathBuf::from("content/wiki/recipes/rendang.fr.md"))
        );

        let orphans = library.get_all_orphan_pages();
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].file.path, PathBuf::from("content/random/hello.md"));

        // And translations should be filled in
        let translations = library.find_translations(&PathBuf::from("content/novels/reaper"));
        assert_eq!(translations.len(), 2);
        assert!(translations[0].title.is_some());
        assert!(translations[1].title.is_some());
    }

    macro_rules! taxonomies {
        ($config:expr, [$($page:expr),+]) => {{
            let mut library = Library::new(&$config);
            $(
                library.insert_page($page);
            )+
            library.find_taxonomies(&$config)
        }};
    }

    fn create_page_w_taxa(path: &str, lang: &str, taxo: Vec<(&str, Vec<&str>)>) -> Page {
        let mut page = Page::default();
        page.file.path = PathBuf::from(path);
        page.lang = lang.to_owned();
        let mut taxonomies = HashMap::new();
        for (name, terms) in taxo {
            taxonomies.insert(name.to_owned(), terms.iter().map(|t| t.to_string()).collect());
        }
        page.meta.taxonomies = taxonomies;
        page
    }

    #[test]
    fn can_make_taxonomies() {
        let mut config = Config::default_for_test();
        config.languages.get_mut("en").unwrap().taxonomies = vec![
            TaxonomyConfig { name: "categories".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "authors".to_string(), ..TaxonomyConfig::default() },
        ];
        config.slugify_taxonomies();

        let page1 = create_page_w_taxa(
            "a.md",
            "en",
            vec![("tags", vec!["rust", "db"]), ("categories", vec!["tutorials"])],
        );
        let page2 = create_page_w_taxa(
            "b.md",
            "en",
            vec![("tags", vec!["rust", "js"]), ("categories", vec!["others"])],
        );
        let page3 = create_page_w_taxa(
            "c.md",
            "en",
            vec![("tags", vec!["js"]), ("authors", vec!["Vincent Prouillet"])],
        );
        let taxonomies = taxonomies!(config, [page1, page2, page3]);

        let tags = taxonomies.iter().find(|t| t.kind.name == "tags").unwrap();
        assert_eq!(tags.len(), 3);
        assert_eq!(tags.items[0].name, "db");
        assert_eq!(tags.items[0].permalink, "http://a-website.com/tags/db/");
        assert_eq!(tags.items[0].pages.len(), 1);
        assert_eq!(tags.items[1].name, "js");
        assert_eq!(tags.items[1].permalink, "http://a-website.com/tags/js/");
        assert_eq!(tags.items[1].pages.len(), 2);
        assert_eq!(tags.items[2].name, "rust");
        assert_eq!(tags.items[2].permalink, "http://a-website.com/tags/rust/");
        assert_eq!(tags.items[2].pages.len(), 2);

        let categories = taxonomies.iter().find(|t| t.kind.name == "categories").unwrap();
        assert_eq!(categories.items.len(), 2);
        assert_eq!(categories.items[0].name, "others");
        assert_eq!(categories.items[0].permalink, "http://a-website.com/categories/others/");
        assert_eq!(categories.items[0].pages.len(), 1);

        let authors = taxonomies.iter().find(|t| t.kind.name == "authors").unwrap();
        assert_eq!(authors.items.len(), 1);
        assert_eq!(authors.items[0].permalink, "http://a-website.com/authors/vincent-prouillet/");
    }

    #[test]
    fn can_make_multiple_language_taxonomies() {
        let mut config = Config::default_for_test();
        config.slugify.taxonomies = SlugifyStrategy::Safe;
        config.languages.insert("fr".to_owned(), LanguageOptions::default());
        config.languages.get_mut("en").unwrap().taxonomies = vec![
            TaxonomyConfig { name: "categories".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() },
        ];
        config.languages.get_mut("fr").unwrap().taxonomies = vec![
            TaxonomyConfig { name: "catégories".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "tags".to_string(), ..TaxonomyConfig::default() },
        ];
        config.slugify_taxonomies();

        let page1 = create_page_w_taxa("a.md", "en", vec![("categories", vec!["rust"])]);
        let page2 = create_page_w_taxa("b.md", "en", vec![("tags", vec!["rust"])]);
        let page3 = create_page_w_taxa("c.md", "fr", vec![("catégories", vec!["rust"])]);
        let taxonomies = taxonomies!(config, [page1, page2, page3]);

        let categories = taxonomies.iter().find(|t| t.kind.name == "categories").unwrap();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories.items[0].permalink, "http://a-website.com/categories/rust/");
        let tags = taxonomies.iter().find(|t| t.kind.name == "tags" && t.lang == "en").unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags.items[0].permalink, "http://a-website.com/tags/rust/");
        let fr_categories = taxonomies.iter().find(|t| t.kind.name == "catégories").unwrap();
        assert_eq!(fr_categories.len(), 1);
        assert_eq!(fr_categories.items[0].permalink, "http://a-website.com/fr/catégories/rust/");
    }

    #[test]
    fn taxonomies_with_unic_are_grouped_with_default_slugify_strategy() {
        let mut config = Config::default_for_test();
        config.languages.get_mut("en").unwrap().taxonomies = vec![
            TaxonomyConfig { name: "test-taxonomy".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "test taxonomy".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "test-taxonomy ".to_string(), ..TaxonomyConfig::default() },
            TaxonomyConfig { name: "Test-Taxonomy ".to_string(), ..TaxonomyConfig::default() },
        ];
        config.slugify_taxonomies();
        let page1 = create_page_w_taxa("a.md", "en", vec![("test-taxonomy", vec!["Ecole"])]);
        let page2 = create_page_w_taxa("b.md", "en", vec![("test taxonomy", vec!["École"])]);
        let page3 = create_page_w_taxa("c.md", "en", vec![("test-taxonomy ", vec!["ecole"])]);
        let page4 = create_page_w_taxa("d.md", "en", vec![("Test-Taxonomy ", vec!["école"])]);
        let taxonomies = taxonomies!(config, [page1, page2, page3, page4]);
        assert_eq!(taxonomies.len(), 1);

        let tax = &taxonomies[0];
        // under the default slugify strategy all of the provided terms should be the same
        assert_eq!(tax.items.len(), 1);
        let term1 = &tax.items[0];
        assert_eq!(term1.name, "Ecole");
        assert_eq!(term1.slug, "ecole");
        assert_eq!(term1.permalink, "http://a-website.com/test-taxonomy/ecole/");
        assert_eq!(term1.pages.len(), 4);
    }

    #[test]
    fn taxonomies_with_unic_are_not_grouped_with_safe_slugify_strategy() {
        let mut config = Config::default_for_test();
        config.slugify.taxonomies = SlugifyStrategy::Safe;
        config.languages.get_mut("en").unwrap().taxonomies =
            vec![TaxonomyConfig { name: "test".to_string(), ..TaxonomyConfig::default() }];
        config.slugify_taxonomies();
        let page1 = create_page_w_taxa("a.md", "en", vec![("test", vec!["Ecole"])]);
        let page2 = create_page_w_taxa("b.md", "en", vec![("test", vec!["École"])]);
        let page3 = create_page_w_taxa("c.md", "en", vec![("test", vec!["ecole"])]);
        let page4 = create_page_w_taxa("d.md", "en", vec![("test", vec!["école"])]);
        let taxonomies = taxonomies!(config, [page1, page2, page3, page4]);
        assert_eq!(taxonomies.len(), 1);
        let tax = &taxonomies[0];
        // under the safe slugify strategy all terms should be distinct
        assert_eq!(tax.items.len(), 4);
    }

    #[test]
    fn can_fill_backlinks() {
        let mut page1 = create_page("page1.md", "en", PageSort::None);
        page1.internal_links.push(("page2.md".to_owned(), None));
        let mut page2 = create_page("page2.md", "en", PageSort::None);
        page2.internal_links.push(("_index.md".to_owned(), None));
        let mut section1 = create_section("_index.md", "en", 10, false, SortBy::None);
        section1.internal_links.push(("page1.md".to_owned(), None));
        section1.internal_links.push(("page2.md".to_owned(), None));
        let mut library = Library::default();
        library.insert_page(page1);
        library.insert_page(page2);
        library.insert_section(section1);
        library.fill_backlinks();

        assert_eq!(library.backlinks.len(), 3);
        assert_eq!(library.backlinks["page1.md"], set! {PathBuf::from("_index.md")});
        assert_eq!(
            library.backlinks["page2.md"],
            set! {PathBuf::from("page1.md"), PathBuf::from("_index.md")}
        );
        assert_eq!(library.backlinks["_index.md"], set! {PathBuf::from("page2.md")});
    }
}
