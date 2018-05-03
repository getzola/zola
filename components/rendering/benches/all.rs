#![feature(test)]
extern crate test;
extern crate tera;

extern crate rendering;
extern crate config;
extern crate front_matter;

use std::collections::HashMap;

use tera::Tera;
use rendering::{Context, markdown_to_html, render_shortcodes};
use front_matter::InsertAnchor;
use config::Config;

static CONTENT: &'static str = r#"
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
"#;

#[bench]
fn bench_markdown_to_html_with_highlighting(b: &mut test::Bencher) {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let context = Context::new(&tera_ctx, true, "base16-ocean-dark".to_string(), "", &permalinks_ctx, InsertAnchor::None);
    b.iter(|| markdown_to_html(CONTENT, &context));
}

#[bench]
fn bench_markdown_to_html_without_highlighting(b: &mut test::Bencher) {
    let tera_ctx = Tera::default();
    let permalinks_ctx = HashMap::new();
    let context = Context::new(&tera_ctx, false, "base16-ocean-dark".to_string(), "", &permalinks_ctx, InsertAnchor::None);
    b.iter(|| markdown_to_html(CONTENT, &context));
}

#[bench]
fn bench_render_shortcodes_one_present(b: &mut test::Bencher) {
    let mut tera = Tera::default();
    tera.add_raw_template("shortcodes/youtube.html", "{{id}}").unwrap();

    b.iter(|| render_shortcodes(CONTENT, &tera, &Config::default()));
}

#[bench]
fn bench_render_shortcodes_none(b: &mut test::Bencher) {
    let mut tera = Tera::default();
    tera.add_raw_template("shortcodes/youtube.html", "{{id}}").unwrap();
    let content2 = CONTENT.replace(r#"{{ youtube(id="my_youtube_id") }}"#, "");

    b.iter(|| render_shortcodes(&content2, &tera, &Config::default()));
}
