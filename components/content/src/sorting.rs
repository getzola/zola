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
            SortBy::Title | SortBy::TitleBytes => page.meta.title.is_some(),
            SortBy::Weight => page.meta.weight.is_some(),
            SortBy::Slug => true,
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
            SortBy::TitleBytes => {
                a.meta.title.as_ref().unwrap().cmp(b.meta.title.as_ref().unwrap())
            }
            SortBy::Weight => a.meta.weight.unwrap().cmp(&b.meta.weight.unwrap()),
            SortBy::Slug => natural_lexical_cmp(&a.slug, &b.slug),
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

    fn create_page_with_slug(slug: &str) -> Page {
        let front_matter = PageFrontMatter { slug: Some(slug.to_owned()), ..Default::default() };
        let mut page =
            Page::new(format!("content/hello-{}.md", slug), front_matter, &PathBuf::new());
        // Normally, the slug field is populated when a page is parsed, but
        // since we're creating one manually, we have to set it explicitly
        page.slug = slug.to_owned();
        page
    }

    #[test]
    fn can_sort_by_dates() {
        let page1 = create_page_with_date("2018-01-01", None);
        let page2 = create_page_with_date("2017-01-01", None);
        let page3 = create_page_with_date("2019-01-01", None);
        let (pages, ignored_pages) = sort_pages(&[&page1, &page2, &page3], SortBy::Date);
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
        let (pages, ignored_pages) = sort_pages(&[&page1, &page2, &page3], SortBy::UpdateDate);
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
        let (pages, ignored_pages) = sort_pages(&[&page1, &page2, &page3], SortBy::Weight);
        // Should be sorted by weight
        assert_eq!(pages[0], page3.file.path);
        assert_eq!(pages[1], page1.file.path);
        assert_eq!(pages[2], page2.file.path);
        assert_eq!(ignored_pages.len(), 0);
    }

    #[test]
    fn can_sort_by_title() {
        let titles = vec![
            "åland",
            "bagel",
            "track_3",
            "microkernel",
            "Österrike",
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
            sort_pages(&pages.iter().collect::<Vec<_>>(), SortBy::Title);
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
                "åland",
                "bagel",
                "BART",
                "μ-kernel",
                "meter",
                "métro",
                "microkernel",
                "Österrike",
                "track_1",
                "track_3",
                "track_13",
                "Underground"
            ]
        );

        let (sorted_pages, ignored_pages) =
            sort_pages(&pages.iter().collect::<Vec<_>>(), SortBy::TitleBytes);
        // Should be sorted by title in bytes order
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
                "BART",
                "Underground",
                "bagel",
                "meter",
                "microkernel",
                "métro",
                "track_1",
                "track_13",
                "track_3",
                // Non ASCII letters are not merged with the ASCII equivalent (o/a/m here)
                "Österrike",
                "åland",
                "μ-kernel"
            ]
        );
    }

    #[test]
    fn can_sort_by_slug() {
        let page1 = create_page_with_slug("2");
        let page2 = create_page_with_slug("3");
        let page3 = create_page_with_slug("1");
        let (pages, ignored_pages) = sort_pages(&[&page1, &page2, &page3], SortBy::Slug);
        assert_eq!(pages[0], page3.file.path);
        assert_eq!(pages[1], page1.file.path);
        assert_eq!(pages[2], page2.file.path);
        assert_eq!(ignored_pages.len(), 0);

        // 10 should come after 2
        let page1 = create_page_with_slug("1");
        let page2 = create_page_with_slug("10");
        let page3 = create_page_with_slug("2");
        let (pages, ignored_pages) = sort_pages(&[&page1, &page2, &page3], SortBy::Slug);
        assert_eq!(pages[0], page1.file.path);
        assert_eq!(pages[1], page3.file.path);
        assert_eq!(pages[2], page2.file.path);
        assert_eq!(ignored_pages.len(), 0);
    }

    #[test]
    fn can_find_ignored_pages() {
        let page1 = create_page_with_date("2018-01-01", None);
        let page2 = create_page_with_weight(1);
        let (pages, ignored_pages) = sort_pages(&[&page1, &page2], SortBy::Date);
        assert_eq!(pages[0], page1.file.path);
        assert_eq!(ignored_pages.len(), 1);
        assert_eq!(ignored_pages[0], page2.file.path);
    }
}
