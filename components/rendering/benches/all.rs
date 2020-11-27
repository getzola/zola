#![feature(test)]
extern crate test;

use std::collections::HashMap;

use config::Config;
use front_matter::InsertAnchor;
use rendering::{render_content, render_shortcodes, RenderContext};
use tera::Tera;

static CONTENT: &'static str = r#"
# Modus cognitius profanam ne duae virtutis mundi

## Ut vita

Lorem markdownum litora, care ponto nomina, et ut aspicit gelidas sui et
purpureo genuit. Tamen colla venientis [delphina](http://nil-sol.com/ecquis)
Tusci et temptata citaeque curam isto ubi vult vulnere reppulit.

- :one: Seque vidit flendoque de quodam
- :two: Dabit minimos deiecto caputque noctis pluma
- :three: Leti coniunx est Helicen
- :four: Illius pulvereumque Icare inpositos
- :five: Vivunt pereo pluvio tot ramos Olenios gelidis
- :six: Quater teretes natura inde

### A subsection

Protinus dicunt, breve per, et vivacis genus Orphei munere. Me terram [dimittere
casside](http://corpus.org/) pervenit saxo primoque frequentat genuum sorori
praeferre causas Libys. Illud in serpit adsuetam utrimque nunc haberent,
**terrae si** veni! Hectoreis potes sumite [Mavortis retusa](http://tua.org/)
granum captantur potuisse Minervae, frugum.

> Clivo sub inprovisoque nostrum minus fama est, discordia patrem petebat precatur
absumitur, poena per sit. Foramina *tamen cupidine* memor supplex tollentes
dictum unam orbem, Anubis caecae. Viderat formosior tegebat satis, Aethiopasque
sit submisso coniuge tristis ubi! :exclamation:

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

# A big title :fire:

- hello
- world
- !

```py
if __name__ == "__main__":
    gen_site("basic-blog", [""], 250, paginate=True)
```
"#;

#[bench]
fn bench_render_content_with_highlighting(b: &mut test::Bencher) {
    let mut tera = Tera::default();
    tera.add_raw_template("shortcodes/youtube.html", "{{id}}").unwrap();
    let permalinks_ctx = HashMap::new();
    let config = Config::default();
    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);
    b.iter(|| render_content(CONTENT, &context).unwrap());
}

#[bench]
fn bench_render_content_without_highlighting(b: &mut test::Bencher) {
    let mut tera = Tera::default();
    tera.add_raw_template("shortcodes/youtube.html", "{{id}}").unwrap();
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default();
    config.markdown.highlight_code = false;
    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);
    b.iter(|| render_content(CONTENT, &context).unwrap());
}

#[bench]
fn bench_render_content_no_shortcode(b: &mut test::Bencher) {
    let tera = Tera::default();
    let content2 = CONTENT.replace(r#"{{ youtube(id="my_youtube_id") }}"#, "");
    let mut config = Config::default();
    config.markdown.highlight_code = false;
    let permalinks_ctx = HashMap::new();
    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);

    b.iter(|| render_content(&content2, &context).unwrap());
}

#[bench]
fn bench_render_shortcodes_one_present(b: &mut test::Bencher) {
    let mut tera = Tera::default();
    tera.add_raw_template("shortcodes/youtube.html", "{{id}}").unwrap();
    let config = Config::default();
    let permalinks_ctx = HashMap::new();
    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);

    b.iter(|| render_shortcodes(CONTENT, &context));
}

#[bench]
fn bench_render_content_no_shortcode_with_emoji(b: &mut test::Bencher) {
    let tera = Tera::default();
    let content2 = CONTENT.replace(r#"{{ youtube(id="my_youtube_id") }}"#, "");
    let mut config = Config::default();
    config.markdown.highlight_code = false;
    config.markdown.render_emoji = true;
    let permalinks_ctx = HashMap::new();
    let context = RenderContext::new(&tera, &config, "", &permalinks_ctx, InsertAnchor::None);

    b.iter(|| render_content(&content2, &context).unwrap());
}
