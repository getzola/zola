use std::path::{Path, PathBuf};

use config::Config;
use errors::Result;
use libs::ahash::{AHashMap, AHashSet};

use crate::ser::TranslatedContent;
use crate::sorting::sort_pages;
use crate::taxonomies::{find_taxonomies, Taxonomy};
use crate::{Page, Section, SortBy};

#[derive(Debug)]
pub struct Library {
    pub pages: AHashMap<PathBuf, Page>,
    pub sections: AHashMap<PathBuf, Section>,
    pub taxonomies: Vec<Taxonomy>,
    // aliases -> files, so we can easily check for conflicts
    pub reverse_aliases: AHashMap<String, AHashSet<PathBuf>>,
    pub translations: AHashMap<PathBuf, AHashSet<PathBuf>>,
}

impl Library {
    pub fn new() -> Self {
        Self {
            pages: AHashMap::new(),
            sections: AHashMap::new(),
            taxonomies: Vec::new(),
            reverse_aliases: AHashMap::new(),
            translations: AHashMap::new(),
        }
    }

    fn insert_reverse_aliases(&mut self, file_path: &Path, entries: Vec<String>) {
        for entry in entries {
            self.reverse_aliases
                .entry(entry)
                .and_modify(|s| {
                    s.insert(file_path.to_path_buf());
                })
                .or_insert_with(|| {
                    let mut s = AHashSet::new();
                    s.insert(file_path.to_path_buf());
                    s
                });
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
        self.pages.insert(file_path, page);
    }

    pub fn insert_section(&mut self, section: Section) {
        let file_path = section.file.path.clone();
        let mut entries = vec![section.path.clone()];
        entries.extend(section.meta.aliases.to_vec());
        self.insert_reverse_aliases(&file_path, entries);
        self.sections.insert(file_path, section);
    }

    /// Separate from `populate_sections` as it's called _before_ markdown the pages/sections
    pub fn populate_taxonomies(&mut self, config: &Config) -> Result<()> {
        self.taxonomies = find_taxonomies(config, &self.pages)?;
        Ok(())
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
                    let mut p = self.pages.get_mut(page_path).unwrap();
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
    pub fn populate_sections(&mut self, config: &Config) {
        let mut add_translation = |entry: &Path, path: &Path| {
            if config.is_multilingual() {
                self.translations
                    .entry(entry.to_path_buf())
                    .and_modify(|trans| {
                        trans.insert(path.to_path_buf());
                    })
                    .or_insert({
                        let mut s = AHashSet::new();
                        s.insert(path.to_path_buf());
                        s
                    });
            }
        };

        let root_path =
            self.sections.values().find(|s| s.is_index()).map(|s| s.file.parent.clone()).unwrap();
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
            let mut cur_path = root_path.clone();
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

            if let Some(children) = subsections.get(&*path) {
                let mut children: Vec<_> = children.clone();
                children.sort_by(|a, b| sections_weight[a].cmp(&sections_weight[b]));
                section.subsections = children;
            }
            if let Some(parents) = ancestors.get(&*path) {
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
            let mut parent_section_path = page.file.parent.join(&parent_filename);

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
                        let s = self.sections.get(&root_path.join(ancestor)).unwrap();
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
                    Some(parent) => parent_section_path = parent.join(&parent_filename),
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

    pub fn find_taxonomies(&self, config: &Config) -> Result<Vec<Taxonomy>> {
        find_taxonomies(config, &self.pages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileInfo;
    use config::LanguageOptions;

    #[test]
    fn can_find_collisions_with_paths() {
        let mut library = Library::new();
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
        let mut library = Library::new();
        let mut section = Section { path: "hello".to_owned(), ..Default::default() };
        section.file.path = PathBuf::from("hello.md");
        library.insert_section(section.clone());
        let mut section2 = Section { path: "world".to_owned(), ..Default::default() };
        section2.file.path = PathBuf::from("bonjour.md");
        section2.meta.aliases = vec!["hello".to_owned()];
        library.insert_section(section2.clone());

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
        let mut library = Library::new();
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
        library.populate_sections(&config);
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
}
