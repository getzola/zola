"""
Generates test sites for use in benchmark.
Tested with python3 and probably does not work on Windows.
"""
import datetime
import os
import shutil


PAGE = """
+++
title = "Hello"
date = "REPLACE_ME"
+++

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
"""


def gen_skeleton(name):
    if os.path.exists(name):
        shutil.rmtree(name)

    os.makedirs(os.path.join(name, "content"))
    os.makedirs(os.path.join(name, "static"))

    with open(os.path.join(name, "config.toml"), "w") as f:
        f.write("""
title = "My site"
base_url = "https://replace-this-with-your-url.com"

[extra.author]
name = "Vincent Prouillet"
""")

    # Re-use the test templates
    shutil.copytree("../test_site/templates", os.path.join(name, "templates"))


def gen_section(path, num_pages, paginate):
    with open(os.path.join(path, "_index.md"), "w") as f:
        if paginate:
            f.write("""
+++
paginate_by = 5
sort_by = "date"
template = "section_paginated.html"
+++
""")
        else:
            f.write("+++\n+++\n")

    day = datetime.date.today()
    for (i, page) in enumerate(range(0, num_pages)):
        with open(os.path.join(path, "page-{}.md".format(i)), "w") as f:
            f.write(PAGE.replace("REPLACE_ME", str(day + datetime.timedelta(days=1))))


def gen_site(name, sections, num_pages_per_section, paginate=False):
    gen_skeleton(name)

    for section in sections:
        path = os.path.join(name, "content", section) if section else os.path.join(name, "content")
        if section:
            os.makedirs(path)
        gen_section(path, num_pages_per_section, paginate)


if __name__ == "__main__":
    gen_site("small-blog", [""], 30, paginate=True)
    gen_site("medium-blog", [""], 250, paginate=True)
    gen_site("big-blog", [""], 1000, paginate=True)
    gen_site("huge-blog", [""], 10000, paginate=True)

    gen_site("small-kb", ["help", "help1", "help2", "help3", "help4", "help5", "help6", "help7", "help8", "help9"], 10)
    gen_site("medium-kb", ["help", "help1", "help2", "help3", "help4", "help5", "help6", "help7", "help8", "help9"], 100)
    gen_site("huge-kb", ["help", "help1", "help2", "help3", "help4", "help5", "help6", "help7", "help8", "help9"], 1000)
