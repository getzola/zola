use std::cmp::Ordering;

use chrono::NaiveDateTime;
use rayon::prelude::*;
use slotmap::DefaultKey;

use crate::content::Page;

/// Used by the feed
/// There to not have to import sorting stuff in the site crate
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn sort_actual_pages_by_date(a: &&Page, b: &&Page) -> Ordering {
    let ord = b.meta.datetime.unwrap().cmp(&a.meta.datetime.unwrap());
    if ord == Ordering::Equal {
        a.permalink.cmp(&b.permalink)
    } else {
        ord
    }
}

/// Takes a list of (page key, date, permalink) and sort them by dates if possible
/// Pages without date will be put in the unsortable bucket
/// The permalink is used to break ties
pub fn sort_pages_by_date(
    pages: Vec<(&DefaultKey, Option<NaiveDateTime>, &str)>,
) -> (Vec<DefaultKey>, Vec<DefaultKey>) {
    let (mut can_be_sorted, cannot_be_sorted): (Vec<_>, Vec<_>) =
        pages.into_par_iter().partition(|page| page.1.is_some());

    can_be_sorted.par_sort_unstable_by(|a, b| {
        let ord = b.1.unwrap().cmp(&a.1.unwrap());
        if ord == Ordering::Equal {
            a.2.cmp(&b.2)
        } else {
            ord
        }
    });

    (can_be_sorted.iter().map(|p| *p.0).collect(), cannot_be_sorted.iter().map(|p| *p.0).collect())
}

/// Takes a list of (page key, weight, permalink) and sort them by weight if possible
/// Pages without weight will be put in the unsortable bucket
/// The permalink is used to break ties
pub fn sort_pages_by_weight(
    pages: Vec<(&DefaultKey, Option<usize>, &str)>,
) -> (Vec<DefaultKey>, Vec<DefaultKey>) {
    let (mut can_be_sorted, cannot_be_sorted): (Vec<_>, Vec<_>) =
        pages.into_par_iter().partition(|page| page.1.is_some());

    can_be_sorted.par_sort_unstable_by(|a, b| {
        let ord = a.1.unwrap().cmp(&b.1.unwrap());
        if ord == Ordering::Equal {
            a.2.cmp(&b.2)
        } else {
            ord
        }
    });

    (can_be_sorted.iter().map(|p| *p.0).collect(), cannot_be_sorted.iter().map(|p| *p.0).collect())
}

/// Find the lighter/heavier and earlier/later pages for all pages having a date/weight
pub fn find_siblings(
    sorted: &[DefaultKey],
) -> Vec<(DefaultKey, Option<DefaultKey>, Option<DefaultKey>)> {
    let mut res = Vec::with_capacity(sorted.len());
    let length = sorted.len();

    for (i, key) in sorted.iter().enumerate() {
        let mut with_siblings = (*key, None, None);

        if i > 0 {
            // lighter / later
            with_siblings.1 = Some(sorted[i - 1]);
        }

        if i < length - 1 {
            // heavier/earlier
            with_siblings.2 = Some(sorted[i + 1]);
        }
        res.push(with_siblings);
    }

    res
}

#[cfg(test)]
mod tests {
    use slotmap::DenseSlotMap;
    use std::path::PathBuf;

    use super::{find_siblings, sort_pages_by_date, sort_pages_by_weight};
    use crate::content::Page;
    use front_matter::PageFrontMatter;

    fn create_page_with_date(date: &str) -> Page {
        let mut front_matter = PageFrontMatter::default();
        front_matter.date = Some(date.to_string());
        front_matter.date_to_datetime();
        Page::new("content/hello.md", front_matter, &PathBuf::new())
    }

    fn create_page_with_weight(weight: usize) -> Page {
        let mut front_matter = PageFrontMatter::default();
        front_matter.weight = Some(weight);
        Page::new("content/hello.md", front_matter, &PathBuf::new())
    }

    #[test]
    fn can_sort_by_dates() {
        let mut dense = DenseSlotMap::new();
        let page1 = create_page_with_date("2018-01-01");
        let key1 = dense.insert(page1.clone());
        let page2 = create_page_with_date("2017-01-01");
        let key2 = dense.insert(page2.clone());
        let page3 = create_page_with_date("2019-01-01");
        let key3 = dense.insert(page3.clone());

        let input = vec![
            (&key1, page1.meta.datetime, page1.permalink.as_ref()),
            (&key2, page2.meta.datetime, page2.permalink.as_ref()),
            (&key3, page3.meta.datetime, page3.permalink.as_ref()),
        ];
        let (pages, _) = sort_pages_by_date(input);
        // Should be sorted by date
        assert_eq!(pages[0], key3);
        assert_eq!(pages[1], key1);
        assert_eq!(pages[2], key2);
    }

    #[test]
    fn can_sort_by_weight() {
        let mut dense = DenseSlotMap::new();
        let page1 = create_page_with_weight(2);
        let key1 = dense.insert(page1.clone());
        let page2 = create_page_with_weight(3);
        let key2 = dense.insert(page2.clone());
        let page3 = create_page_with_weight(1);
        let key3 = dense.insert(page3.clone());

        let input = vec![
            (&key1, page1.meta.weight, page1.permalink.as_ref()),
            (&key2, page2.meta.weight, page2.permalink.as_ref()),
            (&key3, page3.meta.weight, page3.permalink.as_ref()),
        ];
        let (pages, _) = sort_pages_by_weight(input);
        // Should be sorted by weight
        assert_eq!(pages[0], key3);
        assert_eq!(pages[1], key1);
        assert_eq!(pages[2], key2);
    }

    #[test]
    fn ignore_page_with_missing_field() {
        let mut dense = DenseSlotMap::new();
        let page1 = create_page_with_weight(2);
        let key1 = dense.insert(page1.clone());
        let page2 = create_page_with_weight(3);
        let key2 = dense.insert(page2.clone());
        let page3 = create_page_with_date("2019-01-01");
        let key3 = dense.insert(page3.clone());

        let input = vec![
            (&key1, page1.meta.weight, page1.permalink.as_ref()),
            (&key2, page2.meta.weight, page2.permalink.as_ref()),
            (&key3, page3.meta.weight, page3.permalink.as_ref()),
        ];

        let (pages, unsorted) = sort_pages_by_weight(input);
        assert_eq!(pages.len(), 2);
        assert_eq!(unsorted.len(), 1);
    }

    #[test]
    fn can_find_siblings() {
        let mut dense = DenseSlotMap::new();
        let page1 = create_page_with_weight(1);
        let key1 = dense.insert(page1.clone());
        let page2 = create_page_with_weight(2);
        let key2 = dense.insert(page2.clone());
        let page3 = create_page_with_weight(3);
        let key3 = dense.insert(page3.clone());

        let input = vec![key1, key2, key3];

        let pages = find_siblings(&input);

        assert_eq!(pages[0].1, None);
        assert_eq!(pages[0].2, Some(key2));

        assert_eq!(pages[1].1, Some(key1));
        assert_eq!(pages[1].2, Some(key3));

        assert_eq!(pages[2].1, Some(key2));
        assert_eq!(pages[2].2, None);
    }
}
