use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use slotmap::{DenseSlotMap, Key};

use front_matter::SortBy;

use config::Config;
use content::{Page, Section};
use sorting::{find_siblings, sort_pages_by_date, sort_pages_by_weight};

/// Houses everything about pages and sections
/// Think of it as a database where each page and section has an id (Key here)
/// that can be used to find the actual value
/// Sections and pages can then refer to other elements by those keys, which are very cheap to
/// copy.
/// We can assume the keys are always existing as removing a page/section deletes all references
/// to that key.
#[derive(Debug)]
pub struct Library {
    /// All the pages of the site
    pages: DenseSlotMap<Page>,
    /// All the sections of the site
    sections: DenseSlotMap<Section>,
    /// A mapping path -> key for pages so we can easily get their key
    pub paths_to_pages: HashMap<PathBuf, Key>,
    /// A mapping path -> key for sections so we can easily get their key
    pub paths_to_sections: HashMap<PathBuf, Key>,
    /// Whether we need to look for translations
    is_multilingual: bool,
}

impl Library {
    pub fn new(cap_pages: usize, cap_sections: usize, is_multilingual: bool) -> Self {
        Library {
            pages: DenseSlotMap::with_capacity(cap_pages),
            sections: DenseSlotMap::with_capacity(cap_sections),
            paths_to_pages: HashMap::with_capacity(cap_pages),
            paths_to_sections: HashMap::with_capacity(cap_sections),
            is_multilingual,
        }
    }

    /// Add a section and return its Key
    pub fn insert_section(&mut self, section: Section) -> Key {
        let path = section.file.path.clone();
        let key = self.sections.insert(section);
        self.paths_to_sections.insert(path, key);
        key
    }

    /// Add a page and return its Key
    pub fn insert_page(&mut self, page: Page) -> Key {
        let path = page.file.path.clone();
        let key = self.pages.insert(page);
        self.paths_to_pages.insert(path, key);
        key
    }

    pub fn pages(&self) -> &DenseSlotMap<Page> {
        &self.pages
    }

    pub fn pages_mut(&mut self) -> &mut DenseSlotMap<Page> {
        &mut self.pages
    }

    pub fn pages_values(&self) -> Vec<&Page> {
        self.pages.values().collect::<Vec<_>>()
    }

    pub fn sections(&self) -> &DenseSlotMap<Section> {
        &self.sections
    }

    pub fn sections_mut(&mut self) -> &mut DenseSlotMap<Section> {
        &mut self.sections
    }

    pub fn sections_values(&self) -> Vec<&Section> {
        self.sections.values().collect::<Vec<_>>()
    }

    /// Find out the direct subsections of each subsection if there are some
    /// as well as the pages for each section
    pub fn populate_sections(&mut self, config: &Config) {
        let root_path =
            self.sections.values().find(|s| s.is_index()).map(|s| s.file.parent.clone()).unwrap();
        // We are going to get both the ancestors and grandparents for each section in one go
        let mut ancestors: HashMap<PathBuf, Vec<_>> = HashMap::new();
        let mut subsections: HashMap<PathBuf, Vec<_>> = HashMap::new();

        for section in self.sections.values_mut() {
            // Make sure the pages of a section are empty since we can call that many times on `serve`
            section.pages = vec![];
            section.ignored_pages = vec![];

            if let Some(ref grand_parent) = section.file.grand_parent {
                subsections
                    // Using the original filename to work for multi-lingual sections
                    .entry(grand_parent.join(&section.file.filename))
                    .or_insert_with(|| vec![])
                    .push(section.file.path.clone());
            }

            // Index has no ancestors, no need to go through it
            if section.is_index() {
                ancestors.insert(section.file.path.clone(), vec![]);
                continue;
            }

            let mut path = root_path.clone();
            let root_key = self.paths_to_sections[&root_path.join(&section.file.filename)];
            // Index section is the first ancestor of every single section
            let mut parents = vec![root_key];
            for component in &section.file.components {
                path = path.join(component);
                // Skip itself
                if path == section.file.parent {
                    continue;
                }
                if let Some(section_key) =
                    self.paths_to_sections.get(&path.join(&section.file.filename))
                {
                    parents.push(*section_key);
                }
            }
            ancestors.insert(section.file.path.clone(), parents);
        }

        for (key, page) in &mut self.pages {
            let parent_filename = if page.lang != config.default_language {
                format!("_index.{}.md", page.lang)
            } else {
                "_index.md".to_string()
            };
            let mut parent_section_path = page.file.parent.join(&parent_filename);
            while let Some(section_key) = self.paths_to_sections.get(&parent_section_path) {
                let parent_is_transparent;
                // We need to get a reference to a section later so keep the scope of borrowing small
                {
                    let mut section = self.sections.get_mut(*section_key).unwrap();
                    section.pages.push(key);
                    parent_is_transparent = section.meta.transparent;
                }
                page.ancestors =
                    ancestors.get(&parent_section_path).cloned().unwrap_or_else(|| vec![]);
                // Don't forget to push the actual parent
                page.ancestors.push(*section_key);

                // Find the page template if one of a parent has page_template set
                // Stops after the first one found, keep in mind page.ancestors
                // is [index, ..., parent] so we need to reverse it first
                if page.meta.template.is_none() {
                    for ancestor in page.ancestors.iter().rev() {
                        let s = self.sections.get(*ancestor).unwrap();
                        if s.meta.page_template.is_some() {
                            page.meta.template = s.meta.page_template.clone();
                            break;
                        }
                    }
                }

                if !parent_is_transparent {
                    break;
                }

                // We've added `_index(.{LANG})?.md` so if we are here so we need to go up twice
                match parent_section_path.clone().parent().unwrap().parent() {
                    Some(parent) => parent_section_path = parent.join(&parent_filename),
                    None => break,
                }
            }
        }

        self.populate_translations();
        self.sort_sections_pages();

        let sections = self.paths_to_sections.clone();
        let mut sections_weight = HashMap::new();
        for (key, section) in &self.sections {
            sections_weight.insert(key, section.meta.weight);
        }

        for section in self.sections.values_mut() {
            if let Some(ref children) = subsections.get(&section.file.path) {
                let mut children: Vec<_> = children.iter().map(|p| sections[p]).collect();
                children.sort_by(|a, b| sections_weight[a].cmp(&sections_weight[b]));
                section.subsections = children;
            }
            section.ancestors =
                ancestors.get(&section.file.path).cloned().unwrap_or_else(|| vec![]);
        }
    }

    /// Sort all sections pages according to sorting method given
    /// Pages that cannot be sorted are set to the section.ignored_pages instead
    pub fn sort_sections_pages(&mut self) {
        let mut updates = HashMap::new();
        for (key, section) in &self.sections {
            let (sorted_pages, cannot_be_sorted_pages) = match section.meta.sort_by {
                SortBy::None => continue,
                SortBy::Date => {
                    let data = section
                        .pages
                        .iter()
                        .map(|k| {
                            if let Some(page) = self.pages.get(*k) {
                                (k, page.meta.datetime, page.permalink.as_ref())
                            } else {
                                unreachable!("Sorting got an unknown page")
                            }
                        })
                        .collect();

                    sort_pages_by_date(data)
                }
                SortBy::Weight => {
                    let data = section
                        .pages
                        .iter()
                        .map(|k| {
                            if let Some(page) = self.pages.get(*k) {
                                (k, page.meta.weight, page.permalink.as_ref())
                            } else {
                                unreachable!("Sorting got an unknown page")
                            }
                        })
                        .collect();

                    sort_pages_by_weight(data)
                }
            };
            updates.insert(key, (sorted_pages, cannot_be_sorted_pages, section.meta.sort_by));
        }

        for (key, (sorted, cannot_be_sorted, sort_by)) in updates {
            // Find sibling between sorted pages first
            let with_siblings = find_siblings(
                sorted
                    .iter()
                    .map(|k| {
                        if let Some(page) = self.pages.get(*k) {
                            (k, page.is_draft())
                        } else {
                            unreachable!("Sorting got an unknown page")
                        }
                    })
                    .collect(),
            );

            for (k2, val1, val2) in with_siblings {
                if let Some(page) = self.pages.get_mut(k2) {
                    match sort_by {
                        SortBy::Date => {
                            page.earlier = val2;
                            page.later = val1;
                        }
                        SortBy::Weight => {
                            page.lighter = val1;
                            page.heavier = val2;
                        }
                        SortBy::None => unreachable!("Impossible to find siblings in SortBy::None"),
                    }
                } else {
                    unreachable!("Sorting got an unknown page")
                }
            }

            if let Some(s) = self.sections.get_mut(key) {
                s.pages = sorted;
                s.ignored_pages = cannot_be_sorted;
            }
        }
    }

    /// Finds all the translations for each section/page and set the `translations`
    /// field of each as needed
    /// A no-op for sites without multiple languages
    fn populate_translations(&mut self) {
        if !self.is_multilingual {
            return;
        }

        // Sections first
        let mut sections_translations = HashMap::new();
        for (key, section) in &self.sections {
            sections_translations
                .entry(section.file.canonical.clone()) // TODO: avoid this clone
                .or_insert_with(Vec::new)
                .push(key);
        }

        for (key, section) in self.sections.iter_mut() {
            let translations = &sections_translations[&section.file.canonical];
            if translations.len() == 1 {
                section.translations = vec![];
                continue;
            }
            section.translations = translations.iter().filter(|k| **k != key).cloned().collect();
        }

        // Same thing for pages
        let mut pages_translations = HashMap::new();
        for (key, page) in &self.pages {
            pages_translations
                .entry(page.file.canonical.clone()) // TODO: avoid this clone
                .or_insert_with(Vec::new)
                .push(key);
        }

        for (key, page) in self.pages.iter_mut() {
            let translations = &pages_translations[&page.file.canonical];
            if translations.len() == 1 {
                page.translations = vec![];
                continue;
            }
            page.translations = translations.iter().filter(|k| **k != key).cloned().collect();
        }
    }

    /// Find all the orphan pages: pages that are in a folder without an `_index.md`
    pub fn get_all_orphan_pages(&self) -> Vec<&Page> {
        let pages_in_sections =
            self.sections.values().flat_map(|s| &s.pages).collect::<HashSet<_>>();

        self.pages
            .iter()
            .filter(|(key, _)| !pages_in_sections.contains(&key))
            .map(|(_, page)| page)
            .collect()
    }

    /// Find the parent section & all grandparents section that have transparent=true
    /// Only used in rebuild.
    pub fn find_parent_sections<P: AsRef<Path>>(&self, path: P) -> Vec<&Section> {
        let mut parents = vec![];
        let page = self.get_page(path.as_ref()).unwrap();
        for ancestor in page.ancestors.iter().rev() {
            let section = self.get_section_by_key(*ancestor);
            if parents.is_empty() || section.meta.transparent {
                parents.push(section);
            }
        }

        parents
    }

    /// Only used in tests
    pub fn get_section_key<P: AsRef<Path>>(&self, path: P) -> Option<&Key> {
        self.paths_to_sections.get(path.as_ref())
    }

    pub fn get_section<P: AsRef<Path>>(&self, path: P) -> Option<&Section> {
        self.sections.get(self.paths_to_sections.get(path.as_ref()).cloned().unwrap_or_default())
    }

    pub fn get_section_mut<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut Section> {
        self.sections
            .get_mut(self.paths_to_sections.get(path.as_ref()).cloned().unwrap_or_default())
    }

    pub fn get_section_by_key(&self, key: Key) -> &Section {
        self.sections.get(key).unwrap()
    }

    pub fn get_section_mut_by_key(&mut self, key: Key) -> &mut Section {
        self.sections.get_mut(key).unwrap()
    }

    pub fn get_section_path_by_key(&self, key: Key) -> &str {
        &self.get_section_by_key(key).file.relative
    }

    pub fn get_page<P: AsRef<Path>>(&self, path: P) -> Option<&Page> {
        self.pages.get(self.paths_to_pages.get(path.as_ref()).cloned().unwrap_or_default())
    }

    pub fn get_page_by_key(&self, key: Key) -> &Page {
        self.pages.get(key).unwrap()
    }

    pub fn get_page_mut_by_key(&mut self, key: Key) -> &mut Page {
        self.pages.get_mut(key).unwrap()
    }

    pub fn remove_section<P: AsRef<Path>>(&mut self, path: P) -> Option<Section> {
        if let Some(k) = self.paths_to_sections.remove(path.as_ref()) {
            self.sections.remove(k)
        } else {
            None
        }
    }

    pub fn remove_page<P: AsRef<Path>>(&mut self, path: P) -> Option<Page> {
        if let Some(k) = self.paths_to_pages.remove(path.as_ref()) {
            self.pages.remove(k)
        } else {
            None
        }
    }

    /// Used in rebuild, to check if we know it already
    pub fn contains_section<P: AsRef<Path>>(&self, path: P) -> bool {
        self.paths_to_sections.contains_key(path.as_ref())
    }

    /// Used in rebuild, to check if we know it already
    pub fn contains_page<P: AsRef<Path>>(&self, path: P) -> bool {
        self.paths_to_pages.contains_key(path.as_ref())
    }
}
