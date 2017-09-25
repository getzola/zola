use rayon::prelude::*;

use page::Page;
use front_matter::SortBy;

/// Sort pages by the given criteria
///
/// Any pages that doesn't have a the required field when the sorting method is other than none
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
                SortBy::Order => page.meta.order.is_some(),
                SortBy::Weight => page.meta.weight.is_some(),
                _ => unreachable!()
            }
        });

    match sort_by {
        SortBy::Date => can_be_sorted.par_sort_unstable_by(|a, b| b.meta.date().unwrap().cmp(&a.meta.date().unwrap())),
        SortBy::Order => can_be_sorted.par_sort_unstable_by(|a, b| b.meta.order().cmp(&a.meta.order())),
        SortBy::Weight => can_be_sorted.par_sort_unstable_by(|a, b| a.meta.weight().cmp(&b.meta.weight())),
        _ => unreachable!()
    };

    (can_be_sorted, cannot_be_sorted)
}

/// Horribly inefficient way to set previous and next on each pages
/// So many clones
pub fn populate_previous_and_next_pages(input: &[Page]) -> Vec<Page> {
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
                next_page.previous = None;
                next_page.next = None;
                new_page.next = Some(Box::new(next_page));
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
                previous_page.previous = None;
                previous_page.next = None;
                new_page.previous = Some(Box::new(previous_page));
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
    use super::{sort_pages, populate_previous_and_next_pages};

    fn create_page_with_date(date: &str) -> Page {
        let mut front_matter = PageFrontMatter::default();
        front_matter.date = Some(date.to_string());
        Page::new("content/hello.md", front_matter)
    }

    fn create_page_with_order(order: usize) -> Page {
        let mut front_matter = PageFrontMatter::default();
        front_matter.order = Some(order);
        Page::new("content/hello.md", front_matter)
    }

    fn create_draft_page_with_order(order: usize) -> Page {
        let mut front_matter = PageFrontMatter::default();
        front_matter.order = Some(order);
        front_matter.draft = Some(true);
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
        assert_eq!(pages[0].clone().meta.date.unwrap(), "2019-01-01");
        assert_eq!(pages[1].clone().meta.date.unwrap(), "2018-01-01");
        assert_eq!(pages[2].clone().meta.date.unwrap(), "2017-01-01");
    }

    #[test]
    fn can_sort_by_order() {
        let input = vec![
            create_page_with_order(2),
            create_page_with_order(3),
            create_page_with_order(1),
        ];
        let (pages, _) = sort_pages(input, SortBy::Order);
        // Should be sorted by date
        assert_eq!(pages[0].clone().meta.order.unwrap(), 3);
        assert_eq!(pages[1].clone().meta.order.unwrap(), 2);
        assert_eq!(pages[2].clone().meta.order.unwrap(), 1);
    }

    #[test]
    fn can_sort_by_weight() {
        let input = vec![
            create_page_with_weight(2),
            create_page_with_weight(3),
            create_page_with_weight(1),
        ];
        let (pages, _) = sort_pages(input, SortBy::Weight);
        // Should be sorted by date
        assert_eq!(pages[0].clone().meta.weight.unwrap(), 1);
        assert_eq!(pages[1].clone().meta.weight.unwrap(), 2);
        assert_eq!(pages[2].clone().meta.weight.unwrap(), 3);
    }

    #[test]
    fn can_sort_by_none() {
        let input = vec![
            create_page_with_order(2),
            create_page_with_order(3),
            create_page_with_order(1),
        ];
        let (pages, _) = sort_pages(input, SortBy::None);
        // Should be sorted by date
        assert_eq!(pages[0].clone().meta.order.unwrap(), 2);
        assert_eq!(pages[1].clone().meta.order.unwrap(), 3);
        assert_eq!(pages[2].clone().meta.order.unwrap(), 1);
    }

    #[test]
    fn ignore_page_with_missing_field() {
        let input = vec![
            create_page_with_order(2),
            create_page_with_order(3),
            create_page_with_date("2019-01-01"),
        ];
        let (pages, unsorted) = sort_pages(input, SortBy::Order);
        assert_eq!(pages.len(), 2);
        assert_eq!(unsorted.len(), 1);
    }

    #[test]
    fn can_populate_previous_and_next_pages() {
        let input = vec![
            create_page_with_order(1),
            create_page_with_order(2),
            create_page_with_order(3),
        ];
        let pages = populate_previous_and_next_pages(&input);

        assert!(pages[0].clone().next.is_none());
        assert!(pages[0].clone().previous.is_some());
        assert_eq!(pages[0].clone().previous.unwrap().meta.order.unwrap(), 2);

        assert!(pages[1].clone().next.is_some());
        assert!(pages[1].clone().previous.is_some());
        assert_eq!(pages[1].clone().previous.unwrap().meta.order.unwrap(), 3);
        assert_eq!(pages[1].clone().next.unwrap().meta.order.unwrap(), 1);

        assert!(pages[2].clone().next.is_some());
        assert!(pages[2].clone().previous.is_none());
        assert_eq!(pages[2].clone().next.unwrap().meta.order.unwrap(), 2);
    }

    #[test]
    fn can_populate_previous_and_next_pages_skip_drafts() {
        let input = vec![
            create_draft_page_with_order(0),
            create_page_with_order(1),
            create_page_with_order(2),
            create_page_with_order(3),
            create_draft_page_with_order(4),
        ];
        let pages = populate_previous_and_next_pages(&input);

        assert!(pages[0].clone().next.is_none());
        assert!(pages[0].clone().previous.is_none());

        assert!(pages[1].clone().next.is_none());
        assert!(pages[1].clone().previous.is_some());
        assert_eq!(pages[1].clone().previous.unwrap().meta.order.unwrap(), 2);

        assert!(pages[2].clone().next.is_some());
        assert!(pages[2].clone().previous.is_some());
        assert_eq!(pages[2].clone().previous.unwrap().meta.order.unwrap(), 3);
        assert_eq!(pages[2].clone().next.unwrap().meta.order.unwrap(), 1);

        assert!(pages[3].clone().next.is_some());
        assert!(pages[3].clone().previous.is_none());
        assert_eq!(pages[3].clone().next.unwrap().meta.order.unwrap(), 2);

        assert!(pages[4].clone().next.is_none());
        assert!(pages[4].clone().previous.is_none());
    }
}
