use rayon::prelude::*;

use content::Page;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Date,
    Order,
    Weight,
    None,
}

/// Sort pages using the method for the given section
///
/// Any pages that doesn't have a date when the sorting method is date or order
/// when the sorting method is order will be ignored.
pub fn sort_pages(pages: Vec<Page>, sort_by: SortBy) -> (Vec<Page>, Vec<Page>) {
    match sort_by {
        SortBy::Date => {
            let mut can_be_sorted = vec![];
            let mut cannot_be_sorted = vec![];
            for page in pages {
                if page.meta.date.is_some() {
                    can_be_sorted.push(page);
                } else {
                    cannot_be_sorted.push(page);
                }
            }
            can_be_sorted.par_sort_unstable_by(|a, b| b.meta.date().unwrap().cmp(&a.meta.date().unwrap()));

            (can_be_sorted, cannot_be_sorted)
        },
        SortBy::Order | SortBy::Weight => {
            let mut can_be_sorted = vec![];
            let mut cannot_be_sorted = vec![];
            for page in pages {
                if page.meta.order.is_some() {
                    can_be_sorted.push(page);
                } else {
                    cannot_be_sorted.push(page);
                }
            }
            if sort_by == SortBy::Order {
                can_be_sorted.par_sort_unstable_by(|a, b| b.meta.order().cmp(&a.meta.order()));
            } else {
                can_be_sorted.par_sort_unstable_by(|a, b| a.meta.order().cmp(&b.meta.order()));
            }

            (can_be_sorted, cannot_be_sorted)
        },
        SortBy::None => (pages,  vec![])
    }
}

/// Horribly inefficient way to set previous and next on each pages
/// So many clones
pub fn populate_previous_and_next_pages(input: &[Page]) -> Vec<Page> {
    let pages = input.to_vec();
    let mut res = Vec::new();

    // the input is already sorted
    // We might put prev/next randomly if a page is missing date/order, probably fine
    for (i, page) in input.iter().enumerate() {
        let mut new_page = page.clone();

        if i > 0 {
            let next = &pages[i - 1];
            let mut next_page = next.clone();
            // Remove prev/next otherwise we serialise the whole thing...
            next_page.previous = None;
            next_page.next = None;
            new_page.next = Some(Box::new(next_page));
        }

        if i < input.len() - 1 {
            let previous = &pages[i + 1];
            // Remove prev/next otherwise we serialise the whole thing...
            let mut previous_page = previous.clone();
            previous_page.previous = None;
            previous_page.next = None;
            new_page.previous = Some(Box::new(previous_page));
        }
        res.push(new_page);
    }

    res
}

#[cfg(test)]
mod tests {
    use front_matter::{PageFrontMatter};
    use content::Page;
    use super::{SortBy, sort_pages, populate_previous_and_next_pages};

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
            create_page_with_order(2),
            create_page_with_order(3),
            create_page_with_order(1),
        ];
        let (pages, _) = sort_pages(input, SortBy::Weight);
        // Should be sorted by date
        assert_eq!(pages[0].clone().meta.order.unwrap(), 1);
        assert_eq!(pages[1].clone().meta.order.unwrap(), 2);
        assert_eq!(pages[2].clone().meta.order.unwrap(), 3);
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
            create_page_with_order(3),
            create_page_with_order(2),
            create_page_with_order(1),
        ];
        let pages = populate_previous_and_next_pages(input.as_slice());

        assert!(pages[0].clone().next.is_none());
        assert!(pages[0].clone().previous.is_some());
        assert_eq!(pages[0].clone().previous.unwrap().meta.order.unwrap(), 2);

        assert!(pages[1].clone().next.is_some());
        assert!(pages[1].clone().previous.is_some());
        assert_eq!(pages[1].clone().next.unwrap().meta.order.unwrap(), 3);
        assert_eq!(pages[1].clone().previous.unwrap().meta.order.unwrap(), 1);

        assert!(pages[2].clone().next.is_some());
        assert!(pages[2].clone().previous.is_none());
        assert_eq!(pages[2].clone().next.unwrap().meta.order.unwrap(), 2);
    }
}
