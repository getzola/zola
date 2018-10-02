use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use slotmap::{DenseSlotMap, Key};
use tera::{Value, to_value};

use front_matter::SortBy;

use sorting::{find_siblings, sort_pages_by_weight, sort_pages_by_date};
use content::{Page, Section};

#[derive(Debug)]
struct Values {
    pages: HashMap<Key, Value>,
    sections: HashMap<Key, Value>,
}

impl Values {
    pub fn new(cap_pages: usize, cap_sections: usize) -> Values {
        Values {
            pages: HashMap::with_capacity(cap_pages),
            sections: HashMap::with_capacity(cap_sections),
        }
    }

    pub fn get_page(&self, key: &Key) -> &Value {
        return self.pages.get(key).unwrap()
    }

    pub fn insert_page(&mut self, key: Key, value: Value) {
        self.pages.insert(key, value);
    }

    pub fn remove_page(&mut self, key: &Key) {
        self.pages.remove(key);
    }

    pub fn get_section(&self, key: &Key) -> &Value {
        return self.sections.get(key).unwrap()
    }

    pub fn insert_section(&mut self, key: Key, value: Value) {
        self.sections.insert(key, value);
    }

    pub fn remove_section(&mut self, key: &Key) {
        self.sections.remove(key);
    }
}

// Houses everything about pages/sections/taxonomies
#[derive(Debug)]
pub struct Library {
    pages: DenseSlotMap<Page>,
    sections: DenseSlotMap<Section>,
    paths_to_pages: HashMap<PathBuf, Key>,
    paths_to_sections: HashMap<PathBuf, Key>,

    values: Values,
}

impl Library {
    pub fn new(cap_pages: usize, cap_sections: usize) -> Self {
        Library {
            pages: DenseSlotMap::with_capacity(cap_pages),
            sections: DenseSlotMap::with_capacity(cap_sections),
            paths_to_pages: HashMap::with_capacity(cap_pages),
            paths_to_sections: HashMap::with_capacity(cap_sections),
            values: Values::new(cap_pages, cap_sections),
        }
    }

    pub fn insert_section(&mut self, section: Section) -> Key {
        let path = section.file.path.clone();
        let key = self.sections.insert(section);
        self.paths_to_sections.insert(path, key);
        key
    }

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

    pub fn pages_values_mut(&mut self) -> Vec<&mut Page> {
        self.pages.values_mut().collect::<Vec<_>>()
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

    pub fn sections_values_mut(&mut self) -> Vec<&mut Section> {
        self.sections.values_mut().collect::<Vec<_>>()
    }

    /// Find out the direct subsections of each subsection if there are some
    /// as well as the pages for each section
    pub fn populate_sections(&mut self) {
        let mut grandparent_paths: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();

        for section in self.sections.values_mut() {
            if let Some(ref grand_parent) = section.file.grand_parent {
                grandparent_paths
                    .entry(grand_parent.to_path_buf())
                    .or_insert_with(|| vec![])
                    .push(section.file.path.clone());
            }
            // Make sure the pages of a section are empty since we can call that many times on `serve`
            section.pages = vec![];
            section.ignored_pages = vec![];
        }

        for (key, page) in &mut self.pages {
            let parent_section_path = page.file.parent.join("_index.md");
            if let Some(section_key) = self.paths_to_sections.get(&parent_section_path) {
                self.sections.get_mut(*section_key).unwrap().pages.push(key);
            }
        }
        self.sort_sections_pages(None);

        let sections = self.paths_to_sections.clone();
        let mut sections_weight = HashMap::new();
        for (key, section) in &self.sections {
            sections_weight.insert(key, section.meta.weight);
        }
        for section in self.sections.values_mut() {
            if let Some(paths) = grandparent_paths.get(&section.file.parent) {
                section.subsections = paths
                    .iter()
                    .map(|p| sections[p])
                    .collect::<Vec<_>>();
                section.subsections
                    .sort_by(|a, b| sections_weight[a].cmp(&sections_weight[b]));
            }
        }
    }

    pub fn sort_sections_pages(&mut self, only: Option<&Path>) {
        let mut updates = HashMap::new();
        for (key, section) in &self.sections {
            if let Some(p) = only {
                if p != section.file.path {
                    continue;
                }
            }

            // TODO: use an enum to avoid duplication there and in sorting.rs?
            let (sorted_pages, cannot_be_sorted_pages) = match section.meta.sort_by {
                SortBy::None => continue,
                SortBy::Date => {
                    let data = section.pages
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
                },
                SortBy::Weight => {
                    let data = section.pages
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
            let with_siblings = find_siblings(sorted.iter().map(|k| {
                if let Some(page) = self.pages.get(*k) {
                    (k, page.is_draft())
                } else {
                    unreachable!("Sorting got an unknown page")
                }
            }).collect());

            for (k2, val1, val2) in with_siblings {
                if let Some(page) = self.pages.get_mut(k2) {
                    match sort_by {
                        SortBy::Date => {
                            page.earlier = val2;
                            page.later = val1;
                        },
                        SortBy::Weight => {
                            page.lighter = val1;
                            page.heavier = val2;
                        },
                        SortBy::None => unreachable!("Impossible to find siblings in SortBy::None")
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

    pub fn cache_all_pages(&mut self) {
        let mut cache = HashMap::with_capacity(self.pages.capacity());
        for (key, page) in &self.pages {
            cache.insert(key, to_value(page.to_serialized(self.pages())).unwrap());
        }

        for (key, value) in cache {
            self.values.insert_page(key, value);
        }
    }

    // We need to do it from the bottom up to ensure all subsections of a section have been
    // cached before doing it
    pub fn cache_all_sections(&mut self) {
        // we get the Key in order we want to process them first
        let mut deps = HashMap::new();
        for (key, section) in &self.sections {
            deps.insert(key, section.subsections.clone());
        }

        loop {
            if deps.is_empty() {
                break;
            }

            let mut processed_keys = vec![];
            for (key, _) in deps.iter().filter(|(_, v)| v.is_empty()) {
                let section = self.sections.get(*key).unwrap();
                let value = to_value(section.to_serialized(self)).unwrap();
                self.values.insert_section(*key, value);
                processed_keys.push(*key);
            }

            // remove the processed keys from the action
            for key in processed_keys {
                deps.remove(&key);
                for (_, subs) in &mut deps {
                    subs.retain(|k| k != &key);
                }
            }
        }
    }

    pub fn get_all_orphan_pages(&self) -> Vec<&Page> {
        let pages_in_sections = self.sections
            .values()
            .flat_map(|s| s.all_pages_path())
            .collect::<HashSet<_>>();

        self.pages
            .values()
            .filter(|page| !pages_in_sections.contains(&page.file.path))
            .collect()
    }

    pub fn get_section(&self, path: &PathBuf) -> Option<&Section> {
        self.sections.get(self.paths_to_sections.get(path).cloned().unwrap_or_default())
    }

    pub fn get_section_mut(&mut self, path: &PathBuf) -> Option<&mut Section> {
        self.sections.get_mut(self.paths_to_sections.get(path).cloned().unwrap_or_default())
    }

    pub fn get_section_by_key(&self, key: Key) -> &Section {
        self.sections.get(key).unwrap()
    }

    pub fn remove_section_by_path(&mut self, path: &PathBuf) -> Option<Section> {
        if let Some(k) = self.paths_to_sections.remove(path) {
            self.values.remove_section(&k);
            self.sections.remove(k)
        } else {
            None
        }
    }

    pub fn contains_section(&self, path: &PathBuf) -> bool {
        self.paths_to_sections.contains_key(path)
    }

    pub fn get_cached_section_value(&self, path: &PathBuf) -> &Value {
        self.values.get_section(self.paths_to_sections.get(path).unwrap())
    }

    pub fn get_cached_section_value_by_key(&self, key: &Key) -> &Value {
        self.values.get_section(key)
    }

    pub fn get_page(&self, path: &PathBuf) -> Option<&Page> {
        self.pages.get(self.paths_to_pages.get(path).cloned().unwrap_or_default())
    }

    pub fn get_cached_page_value(&self, path: &PathBuf) -> &Value {
        self.values.get_page(self.paths_to_pages.get(path).unwrap())
    }

    pub fn get_cached_page_value_by_key(&self, key: &Key) -> &Value {
        self.values.get_page(key)
    }

    pub fn get_page_by_key(&self, key: Key) -> &Page {
        self.pages.get(key).unwrap()
    }

    pub fn remove_page_by_path(&mut self, path: &PathBuf) -> Option<Page> {
        if let Some(k) = self.paths_to_pages.remove(path) {
            self.values.remove_page(&k);
            self.pages.remove(k)
        } else {
            None
        }
    }

    pub fn contains_page(&self, path: &PathBuf) -> bool {
        self.paths_to_pages.contains_key(path)
    }
}
