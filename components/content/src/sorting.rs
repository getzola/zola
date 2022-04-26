use std::cmp::Ordering;
use std::path::PathBuf;

use crate::{Page, SortBy};
use libs::lexical_sort::natural_lexical_cmp;
use libs::rayon::prelude::*;

/// Sort by the field picked by the function.
/// The pages permalinks are used to break the ties
pub fn sort_pages(pages: &[&Page], sort_by: SortBy) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let (mut can_be_sorted, cannot_be_sorted): (Vec<&Page>, Vec<_>) =
        pages.par_iter().partition(|page| match sort_by {
            SortBy::Date => page.meta.datetime.is_some(),
            SortBy::UpdateDate => {
                page.meta.datetime.is_some() || page.meta.updated_datetime.is_some()
            }
            SortBy::Title => page.meta.title.is_some(),
            SortBy::Weight => page.meta.weight.is_some(),
            SortBy::None => unreachable!(),
        });

    can_be_sorted.par_sort_unstable_by(|a, b| {
        let ord = match sort_by {
            SortBy::Date => b.meta.datetime.unwrap().cmp(&a.meta.datetime.unwrap()),
            SortBy::UpdateDate => std::cmp::max(b.meta.datetime, b.meta.updated_datetime)
                .unwrap()
                .cmp(&std::cmp::max(a.meta.datetime, a.meta.updated_datetime).unwrap()),
            SortBy::Title => {
                natural_lexical_cmp(a.meta.title.as_ref().unwrap(), b.meta.title.as_ref().unwrap())
            }
            SortBy::Weight => a.meta.weight.unwrap().cmp(&b.meta.weight.unwrap()),
            SortBy::None => unreachable!(),
        };

        if ord == Ordering::Equal {
            a.permalink.cmp(&b.permalink)
        } else {
            ord
        }
    });

    (
        can_be_sorted.iter().map(|p| p.file.path.clone()).collect(),
        cannot_be_sorted.iter().map(|p: &&Page| p.file.path.clone()).collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PageFrontMatter;

    fn create_page_with_date(date: &str, updated_date: Option<&str>) -> Page {
        let mut front_matter = PageFrontMatter {
            date: Some(date.to_string()),
            updated: updated_date.map(|c| c.to_string()),
            ..Default::default()
        };
        front_matter.date_to_datetime();
        Page::new(format!("content/hello-{}.md", date), front_matter, &PathBuf::new())
    }

    fn create_page_with_title(title: &str) -> Page {
        let front_matter = PageFrontMatter { title: Some(title.to_string()), ..Default::default() };
        Page::new(format!("content/hello-{}.md", title), front_matter, &PathBuf::new())
    }

    fn create_page_with_weight(weight: usize) -> Page {
        let front_matter = PageFrontMatter { weight: Some(weight), ..Default::default() };
        Page::new(format!("content/hello-{}.md", weight), front_matter, &PathBuf::new())
    }

    #[test]
    fn can_sort_by_dates() {
        let page1 = create_page_with_date("2018-01-01", None);
        let page2 = create_page_with_date("2017-01-01", None);
        let page3 = create_page_with_date("2019-01-01", None);
        let (pages, ignored_pages) = sort_pages(&vec![&page1, &page2, &page3], SortBy::Date);
        assert_eq!(pages[0], page3.file.path);
        assert_eq!(pages[1], page1.file.path);
        assert_eq!(pages[2], page2.file.path);
        assert_eq!(ignored_pages.len(), 0);
    }

    #[test]
    fn can_sort_by_updated_dates() {
        let page1 = create_page_with_date("2018-01-01", None);
        let page2 = create_page_with_date("2017-01-01", Some("2022-02-01"));
        let page3 = create_page_with_date("2019-01-01", None);
        let (pages, ignored_pages) = sort_pages(&vec![&page1, &page2, &page3], SortBy::UpdateDate);
        assert_eq!(pages[0], page2.file.path);
        assert_eq!(pages[1], page3.file.path);
        assert_eq!(pages[2], page1.file.path);
        assert_eq!(ignored_pages.len(), 0);
    }

    #[test]
    fn can_sort_by_weight() {
        let page1 = create_page_with_weight(2);
        let page2 = create_page_with_weight(3);
        let page3 = create_page_with_weight(1);
        let (pages, ignored_pages) = sort_pages(&vec![&page1, &page2, &page3], SortBy::Weight);
        // Should be sorted by weight
        assert_eq!(pages[0], page3.file.path);
        assert_eq!(pages[1], page1.file.path);
        assert_eq!(pages[2], page2.file.path);
        assert_eq!(ignored_pages.len(), 0);
    }

    #[test]
    fn can_sort_by_title() {
        let titles = vec![
            "bagel",
            "track_3",
            "microkernel",
            "métro",
            "BART",
            "Underground",
            "track_13",
            "μ-kernel",
            "meter",
            "track_1",
        ];
        let pages: Vec<Page> = titles.iter().map(|title| create_page_with_title(title)).collect();
        let (sorted_pages, ignored_pages) =
            sort_pages(&pages.iter().map(|p| p).collect::<Vec<_>>(), SortBy::Title);
        // Should be sorted by title in lexical order
        let sorted_titles: Vec<_> = sorted_pages
            .iter()
            .map(|key| {
                pages.iter().find(|p| &p.file.path == key).unwrap().meta.title.as_ref().unwrap()
            })
            .collect();
        assert_eq!(ignored_pages.len(), 0);
        assert_eq!(
            sorted_titles,
            vec![
                "bagel",
                "BART",
                "μ-kernel",
                "meter",
                "métro",
                "microkernel",
                "track_1",
                "track_3",
                "track_13",
                "Underground",
            ]
        );
    }

    #[test]
    fn can_find_ignored_pages() {
        let page1 = create_page_with_date("2018-01-01", None);
        let page2 = create_page_with_weight(1);
        let (pages, ignored_pages) = sort_pages(&vec![&page1, &page2], SortBy::Date);
        assert_eq!(pages[0], page1.file.path);
        assert_eq!(ignored_pages.len(), 1);
        assert_eq!(ignored_pages[0], page2.file.path);
    }
}
