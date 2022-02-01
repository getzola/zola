use std::cmp::Ordering;

use libs::chrono::NaiveDateTime;
use libs::lexical_sort::natural_lexical_cmp;
use libs::rayon::prelude::*;
use libs::slotmap::DefaultKey;

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
            a.2.cmp(b.2)
        } else {
            ord
        }
    });

    (can_be_sorted.iter().map(|p| *p.0).collect(), cannot_be_sorted.iter().map(|p| *p.0).collect())
}

/// Takes a list of (page key, title, permalink) and sort them by title if possible.
/// Uses the a natural lexical comparison as defined by the lexical_sort crate.
/// Pages without title will be put in the unsortable bucket.
/// The permalink is used to break ties.
pub fn sort_pages_by_title(
    pages: Vec<(&DefaultKey, Option<&str>, &str)>,
) -> (Vec<DefaultKey>, Vec<DefaultKey>) {
    let (mut can_be_sorted, cannot_be_sorted): (Vec<_>, Vec<_>) =
        pages.into_par_iter().partition(|page| page.1.is_some());

    can_be_sorted.par_sort_unstable_by(|a, b| {
        let ord = natural_lexical_cmp(a.1.unwrap(), b.1.unwrap());
        if ord == Ordering::Equal {
            a.2.cmp(b.2)
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
            a.2.cmp(b.2)
        } else {
            ord
        }
    });

    (can_be_sorted.iter().map(|p| *p.0).collect(), cannot_be_sorted.iter().map(|p| *p.0).collect())
}

/// Find the lighter/heavier, earlier/later, and title_prev/title_next
/// pages for all pages having a date/weight/title
pub fn find_siblings(
    sorted: &[DefaultKey],
) -> Vec<(DefaultKey, Option<DefaultKey>, Option<DefaultKey>)> {
    let mut res = Vec::with_capacity(sorted.len());
    let length = sorted.len();

    for (i, key) in sorted.iter().enumerate() {
        let mut with_siblings = (*key, None, None);

        if i > 0 {
            // lighter / later / title_prev
            with_siblings.1 = Some(sorted[i - 1]);
        }

        if i < length - 1 {
            // heavier / earlier / title_next
            with_siblings.2 = Some(sorted[i + 1]);
        }
        res.push(with_siblings);
    }

    res
}

#[cfg(test)]
mod tests {
    use libs::slotmap::DenseSlotMap;
    use std::path::PathBuf;

    use super::{find_siblings, sort_pages_by_date, sort_pages_by_title, sort_pages_by_weight};
    use crate::content::Page;
    use front_matter::PageFrontMatter;

    fn create_page_with_date(date: &str) -> Page {
        let mut front_matter =
            PageFrontMatter { date: Some(date.to_string()), ..Default::default() };
        front_matter.date_to_datetime();
        Page::new("content/hello.md", front_matter, &PathBuf::new())
    }

    fn create_page_with_title(title: &str) -> Page {
        let front_matter = PageFrontMatter { title: Some(title.to_string()), ..Default::default() };
        Page::new("content/hello.md", front_matter, &PathBuf::new())
    }

    fn create_page_with_weight(weight: usize) -> Page {
        let front_matter = PageFrontMatter { weight: Some(weight), ..Default::default() };
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
    fn can_sort_by_titles() {
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
        let mut dense = DenseSlotMap::new();
        let keys: Vec<_> = pages.iter().map(|p| dense.insert(p)).collect();
        let input: Vec<_> = pages
            .iter()
            .enumerate()
            .map(|(i, page)| (&keys[i], page.meta.title.as_deref(), page.permalink.as_ref()))
            .collect();
        let (sorted, _) = sort_pages_by_title(input);
        // Should be sorted by title
        let sorted_titles: Vec<_> = sorted
            .iter()
            .map(|key| dense.get(*key).unwrap().meta.title.as_ref().unwrap())
            .collect();
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
        let key1 = dense.insert(page1);
        let page2 = create_page_with_weight(2);
        let key2 = dense.insert(page2);
        let page3 = create_page_with_weight(3);
        let key3 = dense.insert(page3);

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
