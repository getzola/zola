#![feature(test)]
extern crate test;
extern crate tera;

extern crate content;
extern crate front_matter;
extern crate config;

use std::collections::HashMap;

use config::Config;
use tera::Tera;
use front_matter::{SortBy, InsertAnchor};
use content::{Page, sort_pages, populate_previous_and_next_pages};


fn create_pages(number: usize, sort_by: SortBy) -> Vec<Page> {
    let mut pages = vec![];
    let config = Config::default();
    let tera = Tera::default();
    let permalinks = HashMap::new();

    for i in 0..number {
        let mut page = Page::default();
        match sort_by {
            SortBy::Weight => { page.meta.weight = Some(i); },
            SortBy::Order => { page.meta.order = Some(i); },
            _ => (),
        };
        page.raw_content = r#"
# Modus cognitius profanam ne duae virtutis mundi

## Ut vita

Lorem markdownum litora, care ponto nomina, et ut aspicit gelidas sui et
purpureo genuit. Tamen colla venientis [delphina](http://nil-sol.com/ecquis)
Tusci et temptata citaeque curam isto ubi vult vulnere reppulit.

- Seque vidit flendoque de quodam
- Dabit minimos deiecto caputque noctis pluma
- Leti coniunx est Helicen
- Illius pulvereumque Icare inpositos
- Vivunt pereo pluvio tot ramos Olenios gelidis
- Quater teretes natura inde

### A subsection

Protinus dicunt, breve per, et vivacis genus Orphei munere. Me terram [dimittere
casside](http://corpus.org/) pervenit saxo primoque frequentat genuum sorori
praeferre causas Libys. Illud in serpit adsuetam utrimque nunc haberent,
**terrae si** veni! Hectoreis potes sumite [Mavortis retusa](http://tua.org/)
granum captantur potuisse Minervae, frugum.

> Clivo sub inprovisoque nostrum minus fama est, discordia patrem petebat precatur
absumitur, poena per sit. Foramina *tamen cupidine* memor supplex tollentes
dictum unam orbem, Anubis caecae. Viderat formosior tegebat satis, Aethiopasque
sit submisso coniuge tristis ubi!

## Praeceps Corinthus totidem quem crus vultum cape

```rs
#[derive(Debug)]
pub struct Site {
    /// The base path of the gutenberg site
    pub base_path: PathBuf,
    /// The parsed config for the site
    pub config: Config,
    pub pages: HashMap<PathBuf, Page>,
    pub sections: HashMap<PathBuf, Section>,
    pub tera: Tera,
    live_reload: bool,
    output_path: PathBuf,
    static_path: PathBuf,
    pub tags: Option<Taxonomy>,
    pub categories: Option<Taxonomy>,
    /// A map of all .md files (section and pages) and their permalink
    /// We need that if there are relative links in the content that need to be resolved
    pub permalinks: HashMap<String, String>,
}
```

## More stuff
And a shortcode:

{{ youtube(id="my_youtube_id") }}

### Another subsection
Gotta make the toc do a little bit of work

# A big title

- hello
- world
- !

```py
if __name__ == "__main__":
    gen_site("basic-blog", [""], 250, paginate=True)
```
"#.to_string();
        page.render_markdown(&permalinks, &tera, &config, InsertAnchor::None).unwrap();
        pages.push(page);
    }

    pages
}

// Most of the time spent in those benches are due to the .clone()...
// but i don't know how to remove them so there are some baseline bench with
// just the cloning and with a bit of math we can figure it out

#[bench]
fn bench_baseline_cloning(b: &mut test::Bencher) {
    let pages = create_pages(250, SortBy::Order);
    b.iter(|| pages.clone());
}

#[bench]
fn bench_sorting_none(b: &mut test::Bencher) {
    let pages = create_pages(250, SortBy::Order);
    b.iter(|| sort_pages(pages.clone(), SortBy::None));
}

#[bench]
fn bench_sorting_order(b: &mut test::Bencher) {
    let pages = create_pages(250, SortBy::Order);
    b.iter(|| sort_pages(pages.clone(), SortBy::Order));
}

#[bench]
fn bench_populate_previous_and_next_pages(b: &mut test::Bencher) {
    let pages = create_pages(250, SortBy::Order);
    let (sorted_pages, _) = sort_pages(pages, SortBy::Order);
    b.iter(|| populate_previous_and_next_pages(&sorted_pages.clone()));
}

#[bench]
fn bench_page_render_html(b: &mut test::Bencher) {
    let pages = create_pages(10, SortBy::Order);
    let (mut sorted_pages, _) = sort_pages(pages, SortBy::Order);
    sorted_pages = populate_previous_and_next_pages(&sorted_pages);

    let config = Config::default();
    let mut tera = Tera::default();
    tera.add_raw_template("page.html", "{{ page.content }}").unwrap();
    let page = &sorted_pages[5];
    b.iter(|| page.render_html(&tera, &config).unwrap());
}
