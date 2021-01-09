use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use slotmap::{DefaultKey, DenseSlotMap};

use front_matter::SortBy;

use crate::content::{Page, Section};
use crate::sorting::{find_siblings, sort_pages_by_date, sort_pages_by_weight};
use config::Config;

// Like vec! but for HashSet
macro_rules! set {
    ( $( $x:expr ),* ) => {
        {
            let mut s = HashSet::new();
            $(
                s.insert($x);
            )*
            s
        }
    };
}

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
    pages: DenseSlotMap<DefaultKey, Page>,
    /// All the sections of the site
    sections: DenseSlotMap<DefaultKey, Section>,
    /// A mapping path -> key for pages so we can easily get their key
    pub paths_to_pages: HashMap<PathBuf, DefaultKey>,
    /// A mapping path -> key for sections so we can easily get their key
    pub paths_to_sections: HashMap<PathBuf, DefaultKey>,
    /// Whether we need to look for translations
    is_multilingual: bool,

    // aliases -> files,
    // so we can easily check for conflicts
    pub reverse_aliases: HashMap<String, HashSet<String>>,

    pub translations: HashMap<PathBuf, HashSet<DefaultKey>>,
}

impl Library {
    pub fn new(cap_pages: usize, cap_sections: usize, is_multilingual: bool) -> Self {
        Library {
            pages: DenseSlotMap::with_capacity(cap_pages),
            sections: DenseSlotMap::with_capacity(cap_sections),
            paths_to_pages: HashMap::with_capacity(cap_pages),
            paths_to_sections: HashMap::with_capacity(cap_sections),
            is_multilingual,
            reverse_aliases: HashMap::new(),
            translations: HashMap::new(),
        }
    }

    fn insert_reverse_aliases(&mut self, entries: Vec<String>, file_rel_path: &str) {
        for entry in entries {
            self.reverse_aliases
                .entry(entry)
                .and_modify(|s| {
                    s.insert(file_rel_path.to_owned());
                })
                .or_insert_with(|| {
                    let mut s = HashSet::new();
                    s.insert(file_rel_path.to_owned());
                    s
                });
        }
    }

    /// Add a section and return its Key
    pub fn insert_section(&mut self, section: Section) -> DefaultKey {
        let file_path = section.file.path.clone();
        let rel_path = section.path.clone();

        let mut entries = vec![rel_path.clone()];
        entries.extend(section.meta.aliases.iter().map(|a| a.clone()).collect::<Vec<String>>());
        self.insert_reverse_aliases(entries, &section.file.relative);

        let key = self.sections.insert(section);
        self.paths_to_sections.insert(file_path, key);
        key
    }

    /// Add a page and return its Key
    pub fn insert_page(&mut self, page: Page) -> DefaultKey {
        let file_path = page.file.path.clone();
        let rel_path = page.path.clone();

        let mut entries = vec![rel_path.clone()];
        entries.extend(page.meta.aliases.iter().map(|a| a.clone()).collect::<Vec<String>>());
        self.insert_reverse_aliases(entries, &page.file.relative);

        let key = self.pages.insert(page);

        self.paths_to_pages.insert(file_path, key);
        key
    }

    pub fn pages(&self) -> &DenseSlotMap<DefaultKey, Page> {
        &self.pages
    }

    pub fn pages_mut(&mut self) -> &mut DenseSlotMap<DefaultKey, Page> {
        &mut self.pages
    }

    pub fn pages_values(&self) -> Vec<&Page> {
        self.pages.values().collect::<Vec<_>>()
    }

    pub fn sections(&self) -> &DenseSlotMap<DefaultKey, Section> {
        &self.sections
    }

    pub fn sections_mut(&mut self) -> &mut DenseSlotMap<DefaultKey, Section> {
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

        for (key, section) in self.sections.iter_mut() {
            // Make sure the pages of a section are empty since we can call that many times on `serve`
            section.pages = vec![];
            section.ignored_pages = vec![];

            if let Some(ref grand_parent) = section.file.grand_parent {
                subsections
                    // Using the original filename to work for multi-lingual sections
                    .entry(grand_parent.join(&section.file.filename))
                    .or_insert_with(Vec::new)
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

            // populate translations if necessary
            if self.is_multilingual {
                self.translations
                    .entry(section.file.canonical.clone())
                    .and_modify(|trans| {
                        trans.insert(key);
                    })
                    .or_insert(set![key]);
            };
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
                    let section = self.sections.get_mut(*section_key).unwrap();
                    section.pages.push(key);
                    parent_is_transparent = section.meta.transparent;
                }
                page.ancestors =
                    ancestors.get(&parent_section_path).cloned().unwrap_or_else(Vec::new);
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

            // populate translations if necessary
            if self.is_multilingual {
                self.translations
                    .entry(page.file.canonical.clone())
                    .and_modify(|trans| {
                        trans.insert(key);
                    })
                    .or_insert(set![key]);
            };
        }

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
            section.ancestors = ancestors.get(&section.file.path).cloned().unwrap_or_else(Vec::new);
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
            let with_siblings = find_siblings(&sorted);

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
    pub fn get_section_key<P: AsRef<Path>>(&self, path: P) -> Option<&DefaultKey> {
        self.paths_to_sections.get(path.as_ref())
    }

    pub fn get_section<P: AsRef<Path>>(&self, path: P) -> Option<&Section> {
        self.sections.get(self.paths_to_sections.get(path.as_ref()).cloned().unwrap_or_default())
    }

    pub fn get_section_mut<P: AsRef<Path>>(&mut self, path: P) -> Option<&mut Section> {
        self.sections
            .get_mut(self.paths_to_sections.get(path.as_ref()).cloned().unwrap_or_default())
    }

    pub fn get_section_by_key(&self, key: DefaultKey) -> &Section {
        self.sections.get(key).unwrap()
    }

    pub fn get_section_mut_by_key(&mut self, key: DefaultKey) -> &mut Section {
        self.sections.get_mut(key).unwrap()
    }

    pub fn get_section_path_by_key(&self, key: DefaultKey) -> &str {
        &self.get_section_by_key(key).file.relative
    }

    pub fn get_page<P: AsRef<Path>>(&self, path: P) -> Option<&Page> {
        self.pages.get(self.paths_to_pages.get(path.as_ref()).cloned().unwrap_or_default())
    }

    pub fn get_page_by_key(&self, key: DefaultKey) -> &Page {
        self.pages.get(key).unwrap()
    }

    pub fn get_page_mut_by_key(&mut self, key: DefaultKey) -> &mut Page {
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

    /// This will check every section/page paths + the aliases and ensure none of them
    /// are colliding.
    /// Returns (path colliding, [list of files causing that collision])
    pub fn check_for_path_collisions(&self) -> Vec<(String, Vec<String>)> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_find_no_collisions() {
        let mut library = Library::new(10, 10, false);
        let mut page = Page::default();
        page.path = "hello".to_string();
        let mut page2 = Page::default();
        page2.path = "hello-world".to_string();
        let mut section = Section::default();
        section.path = "blog".to_string();
        library.insert_page(page);
        library.insert_page(page2);
        library.insert_section(section);

        let collisions = library.check_for_path_collisions();
        assert_eq!(collisions.len(), 0);
    }

    #[test]
    fn can_find_collisions_between_pages() {
        let mut library = Library::new(10, 10, false);
        let mut page = Page::default();
        page.path = "hello".to_string();
        page.file.relative = "hello".to_string();
        let mut page2 = Page::default();
        page2.path = "hello".to_string();
        page2.file.relative = "hello-world".to_string();
        let mut section = Section::default();
        section.path = "blog".to_string();
        section.file.relative = "hello-world".to_string();
        library.insert_page(page.clone());
        library.insert_page(page2.clone());
        library.insert_section(section);

        let collisions = library.check_for_path_collisions();
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].0, page.path);
        assert!(collisions[0].1.contains(&page.file.relative));
        assert!(collisions[0].1.contains(&page2.file.relative));
    }

    #[test]
    fn can_find_collisions_with_an_alias() {
        let mut library = Library::new(10, 10, false);
        let mut page = Page::default();
        page.path = "hello".to_string();
        page.file.relative = "hello".to_string();
        let mut page2 = Page::default();
        page2.path = "hello-world".to_string();
        page2.file.relative = "hello-world".to_string();
        page2.meta.aliases = vec!["hello".to_string()];
        let mut section = Section::default();
        section.path = "blog".to_string();
        section.file.relative = "hello-world".to_string();
        library.insert_page(page.clone());
        library.insert_page(page2.clone());
        library.insert_section(section);

        let collisions = library.check_for_path_collisions();
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].0, page.path);
        assert!(collisions[0].1.contains(&page.file.relative));
        assert!(collisions[0].1.contains(&page2.file.relative));
    }
}
