use std::cmp::Ordering;

use rayon::prelude::*;

use page::Page;
use front_matter::SortBy;

/// Sort pages by the given criteria
///
/// Any pages that doesn't have a required field when the sorting method is other than none
/// will be ignored.
pub fn sort_pages(pages: Vec<Page>, sort_by: SortBy) -> (Vec<Page>, Vec<Page>) {
    if sort_by == SortBy::None {
        return (pages,  vec![]);
    }

    let (mut can_be_sorted, cannot_be_sorted): (Vec<_>, Vec<_>) = pages
        .into_par_iter()
        .partition(|page| {
            match sort_by {
                SortBy::Date => page.meta.date.is_some(),
                SortBy::Weight => page.meta.weight.is_some(),
                _ => unreachable!()
            }
        });

    match sort_by {
        SortBy::Date => {
            can_be_sorted.par_sort_unstable_by(|a, b| {
                let ord = b.meta.date().unwrap().cmp(&a.meta.date().unwrap());
                if ord == Ordering::Equal {
                    a.permalink.cmp(&b.permalink)
                } else {
                    ord
                }
            })
        },
        SortBy::Weight => {
            can_be_sorted.par_sort_unstable_by(|a, b| {
                let ord = a.meta.weight().cmp(&b.meta.weight());
                if ord == Ordering::Equal {
                    a.permalink.cmp(&b.permalink)
                } else {
                    ord
                }
            })
        },
        _ => unreachable!()
    };

    (can_be_sorted, cannot_be_sorted)
}

/// Horribly inefficient way to set previous and next on each pages that skips drafts
/// So many clones
pub fn populate_siblings(input: &[Page], sort_by: SortBy) -> Vec<Page> {
    let mut res = Vec::with_capacity(input.len());

    // The input is already sorted
    for (i, _) in input.iter().enumerate() {
        let mut new_page = input[i].clone();

        if new_page.is_draft() {
            res.push(new_page);
            continue;
        }

        if i > 0 {
            let mut j = i;
            loop {
                if j == 0 {
                    break;
                }

                j -= 1;

                if input[j].is_draft() {
                    continue;
                }

                // Remove prev/next otherwise we serialise the whole thing...
                let mut next_page = input[j].clone();

                match sort_by {
                    SortBy::Weight => {
                        next_page.lighter = None;
                        next_page.heavier = None;
                        new_page.lighter = Some(Box::new(next_page));
                    },
                    SortBy::Date => {
                        next_page.earlier = None;
                        next_page.later = None;
                        new_page.later = Some(Box::new(next_page));
                    },
                    SortBy::None => {}
                }
                break;
            }
        }

        if i < input.len() - 1 {
            let mut j = i;
            loop {
                if j == input.len() - 1 {
                    break;
                }

                j += 1;

                if input[j].is_draft() {
                    continue;
                }

                // Remove prev/next otherwise we serialise the whole thing...
                let mut previous_page = input[j].clone();
                match sort_by {
                    SortBy::Weight => {
                        previous_page.lighter = None;
                        previous_page.heavier = None;
                        new_page.heavier = Some(Box::new(previous_page));
                    },
                    SortBy::Date => {
                        previous_page.earlier = None;
                        previous_page.later = None;
                        new_page.earlier = Some(Box::new(previous_page));
                    },
                    SortBy::None => {
                    }
                }
                break;
            }
        }
        res.push(new_page);
    }

    res
}

#[cfg(test)]
mod tests {
    use front_matter::{PageFrontMatter, SortBy};
    use page::Page;
    use super::{sort_pages, populate_siblings};

    fn create_page_with_date(date: &str) -> Page {
        let mut front_matter = PageFrontMatter::default();
        front_matter.date = Some(date.to_string());
        Page::new("content/hello.md", front_matter)
    }

    fn create_page_with_weight(weight: usize) -> Page {
        let mut front_matter = PageFrontMatter::default();
        front_matter.weight = Some(weight);
        Page::new("content/hello.md", front_matter)
    }

    #[test]
    fn can_sort_by_dates() {
        let input = vec![
            create_page_with_date("2018-01-01"),
            create_page_with_date("2017-01-01"),
            create_page_with_date("2019-01-01"),
        ];
        let (pages, _) = sort_pages(input, SortBy::Date);
        // Should be sorted by date
        assert_eq!(pages[0].clone().meta.date.unwrap().to_string(), "2019-01-01");
        assert_eq!(pages[1].clone().meta.date.unwrap().to_string(), "2018-01-01");
        assert_eq!(pages[2].clone().meta.date.unwrap().to_string(), "2017-01-01");
    }

    #[test]
    fn can_sort_by_weight() {
        let input = vec![
            create_page_with_weight(2),
            create_page_with_weight(3),
            create_page_with_weight(1),
        ];
        let (pages, _) = sort_pages(input, SortBy::Weight);
        // Should be sorted by weight
        assert_eq!(pages[0].clone().meta.weight.unwrap(), 1);
        assert_eq!(pages[1].clone().meta.weight.unwrap(), 2);
        assert_eq!(pages[2].clone().meta.weight.unwrap(), 3);
    }

    #[test]
    fn can_sort_by_none() {
        let input = vec![
            create_page_with_weight(2),
            create_page_with_weight(3),
            create_page_with_weight(1),
        ];
        let (pages, _) = sort_pages(input, SortBy::None);
        assert_eq!(pages[0].clone().meta.weight.unwrap(), 2);
        assert_eq!(pages[1].clone().meta.weight.unwrap(), 3);
        assert_eq!(pages[2].clone().meta.weight.unwrap(), 1);
    }

    #[test]
    fn ignore_page_with_missing_field() {
        let input = vec![
            create_page_with_weight(2),
            create_page_with_weight(3),
            create_page_with_date("2019-01-01"),
        ];
        let (pages, unsorted) = sort_pages(input, SortBy::Weight);
        assert_eq!(pages.len(), 2);
        assert_eq!(unsorted.len(), 1);
    }

    #[test]
    fn can_populate_siblings() {
        let input = vec![
            create_page_with_weight(1),
            create_page_with_weight(2),
            create_page_with_weight(3),
        ];
        let pages = populate_siblings(&input, SortBy::Weight);

        assert!(pages[0].clone().lighter.is_none());
        assert!(pages[0].clone().heavier.is_some());
        assert_eq!(pages[0].clone().heavier.unwrap().meta.weight.unwrap(), 2);

        assert!(pages[1].clone().heavier.is_some());
        assert!(pages[1].clone().lighter.is_some());
        assert_eq!(pages[1].clone().lighter.unwrap().meta.weight.unwrap(), 1);
        assert_eq!(pages[1].clone().heavier.unwrap().meta.weight.unwrap(), 3);

        assert!(pages[2].clone().lighter.is_some());
        assert!(pages[2].clone().heavier.is_none());
        assert_eq!(pages[2].clone().lighter.unwrap().meta.weight.unwrap(), 2);
    }
}
