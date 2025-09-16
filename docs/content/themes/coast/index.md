
+++
title = "coast"
description = "A simple theme for Zola that evokes the feel the sea breeze."
template = "theme.html"
date = 2025-09-16T14:29:07+09:00

[taxonomies]
theme-tags = ['light', 'simple', 'mermaid', 'katex']

[extra]
created = 2025-09-16T14:29:07+09:00
updated = 2025-09-16T14:29:07+09:00
repository = "https://github.com/Myxogastria0808/coast.git"
homepage = "https://github.com/Myxogastria0808/coast/"
minimum_version = "0.19.0"
license = "MIT"
demo = "https://zola-coast-theme.netlify.app/"

[extra.author]
name = "Myxogastria0808"
homepage = "https://yukiosada.work"
+++        

# coast theme

**coast** is a blog theme for zola!

This theme can be used **mermaid** and **katex**.

- demo site

https://zola-coast-theme.netlify.app/

- [theme logo](https://github.com/Myxogastria0808/coast/blob/main/logo/README.md)

<div align="center">
  <img src="https://github.com/Myxogastria0808/coast/blob/main/logo/coast.svg" width="300px" height="300px" />
</div>

- screenshot

<div align="center">
  <img src="https://github.com/Myxogastria0808/coast/blob/main/screenshot.png" width="1920px" height="935px" />
</div>

## Setup Environment

1. Install zola

Please install zola by referring to the following.

https://www.getzola.org/documentation/getting-started/installation/

2. Setup coast theme

> [!TIP]
> If you want to use the coast theme repository as a blog, you do not need to do the `2.` step other than making extra settings in the `2-6.` step after cloning the repository.

2-1. Create your blog project

```sh
zola init < your blog project >
```
Please select as follows

```sh
Welcome to Zola!
Please answer a few questions to get started quickly.
Any choices made can be changed by modifying the `config.toml` file later.
> What is the URL of your site? (https://example.com): < "Don't enter anything." >
> Do you want to enable Sass compilation? [Y/n]: y
> Do you want to enable syntax highlighting? [y/N]: y
> Do you want to build a search index of the content? [y/N]: y

Done! Your site was created in /home/hello/Desktop/coast-sample/docs

Get started by moving into the directory and using the built-in server: `zola serve`
Visit https://www.getzola.org for the full documentation.
```

2-2. Change directory to your blog project

```sh
cd ./< your blog project >/themes/
```

2-3. Clone coast theme to theme directory and remove .git directory of coast theme repository

```sh
git clone https://github.com/Myxogastria0808/coast.git
rm -rf coast/.git
```

2-4. Change directory to the root of your blog project

```sh
cd ..
```

2-5. Replace settings to `config.toml` of your blog project

The following is the content of the replacement config.

Please change `base_url` to your blog's URL when deploying your blog.
During development, I recommend leaving base_url as is.

```toml
theme = "coast"

# The URL the site will be built for
base_url = "/"

# The site title and description; used in feeds by default.
title = "coast"
description = "A simple theme for Zola that evokes the feel the sea breeze."

# Whether to automatically compile all Sass files in the sass directory
compile_sass = true

# When set to "true", the generated HTML files are minified.
minify_html = true

# Whether to build a search index to be used later on by a JavaScript library
build_search_index = true

# RSS/Atom feeds
generate_feeds = true
# The filenames to use for the feeds. Used as the template filenames, too.
# Defaults to ["atom.xml"], which has a built-in template that renders an Atom 1.0 feed.
# There is also a built-in template "rss.xml" that renders an RSS 2.0 feed.
feed_filenames = ["rss.xml"]

# The taxonomies to be rendered for the site and their configuration of the default languages
# Example:
#     taxonomies = [
#       {name = "tags", feed = true}, # each tag will have its own feed
#       {name = "tags"}, # you can have taxonomies with the same name in multiple languages
#       {name = "categories", paginate_by = 5},  # 5 items per page for a term
#       {name = "authors"}, # Basic definition: no feed or pagination
#     ]
#
taxonomies = [
    {name = "tags", feed = true},
]

[markdown]
# Whether to do syntax highlighting
# Theme can be customized by setting the `highlight_theme` variable to a theme supported by Zola
highlight_code = true

# When set to "true", emoji aliases translated to their corresponding
# Unicode emoji equivalent in the rendered Markdown files. (e.g.: :smile: => 😄)
render_emoji = true

# Whether external links are to be opened in a new tab
# If this is true, a `rel="noopener"` will always automatically be added for security reasons
external_links_target_blank = true

# Whether to set rel="noreferrer" for all external links
external_links_no_referrer = true

# Whether smart punctuation is enabled (changing quotes, dashes, dots in their typographic form)
# For example, `...` into `…`, `"quote"` into `“curly”` etc
smart_punctuation = true

[search]
# Whether to include the title of the page/section in the index
include_title = true
# Whether to include the description of the page/section in the index
include_description = true
# Whether to include the RFC3339 datetime of the page in the search index
include_date = true
# Whether to include the path of the page/section in the index (the permalink is always included)
include_path = true
# Whether to include the rendered content of the page/section in the index
include_content = true

[slugify]
# Various slugification strategies, see below for details
# Defaults to everything being a slug
paths = "off"
taxonomies = "off"
# Whether to remove date prefixes for page path slugs.
# For example, content/posts/2016-10-08_a-post-with-dates.md => posts/a-post-with-dates
# When true, content/posts/2016-10-08_a-post-with-dates.md => posts/2016-10-08-a-post-with-dates
paths_keep_dates = true
```

2-6. Add extra settings to `config.toml` of your blog project

This theme provides the following additional settings.

All settings have default values, so you only need to add the settings you want to change.

- `config.toml`

```toml
[extra.coast]
# default value: 'en'
lang = "en"
# default value: 'blog'
keyword = "blog"
# default value: 'https://raw.githubusercontent.com/Myxogastria0808/coast/heads/main/static/favicon.svg'
# A shortcut icon has to be a SVG image.
icon = "https://raw.githubusercontent.com/Myxogastria0808/coast/heads/main/static/favicon.svg"
# default value: ''
twitter_site = ""
# default value: '@yuki_osada0808'
twitter_creator = "@yuki_osada0808"
# default value: 'https://raw.githubusercontent.com/Myxogastria0808/coast/heads/main/static/first-view.jpg'
meta_image = "https://raw.githubusercontent.com/Myxogastria0808/coast/heads/main/static/first-view.jpg"
# default value: 'coast theme'
meta_image_alt = "coast theme"
# default value: '3024'
meta_image_width = "3024"
# default value: '3024'
meta_image_height = "3024"

# default value: 'https://raw.githubusercontent.com/Myxogastria0808/coast/heads/main/static/first-view.jpg'
first_view_image = "https://raw.githubusercontent.com/Myxogastria0808/coast/heads/main/static/first-view.jpg"
# default value: 'Hello, World!'
about_title = "Hello, World!"
# default value is below
about_description = """
Hello, my name is Myxogastria0808.<br/>
I created a Zola theme named "coast".
This template can be used mermaid and katex.<br/>
Have a nice day!
"""
```

- example (part of `config.toml`)

```toml
[extra.coast]
keyword = "blog coast sea"

about = """
Hello, my name is Myxogastria0808.<br/>
This blog is made by Zola. This is a sample blog of coast theme.
"""
```

2-7. Replace settings to `content/_index.md` of your blog project

The following is the content of `_index.md`, which will be newly added in the content directory.

- `_index.md`

```md
+++
sort_by = "date"
template = "index.html"
page_template = "blog-template.html"
in_search_index = true
+++

```

3. Build your blog

```sh
zola build
```

4. Check your blog

```sh
zola serve
```

### Setup Example

The following sample have been set up.

> [!NOTE]
> This repository includes the coast theme repository as a submodule.

- repository

https://github.com/Myxogastria0808/coast-sample.git

- demo site

https://zola-coast-theme-sample.netlify.app/

## Post Example

You can see the post example below.

```md
+++
title = "coast"
date = 2025-08-19
authors = ["Myxogastria0808"]
[taxonomies]
tags = ["coast"]
+++

I don’t live by the sea, but whenever I travel and find myself on a shoreline, I feel something special—the rhythm of the waves, the endless horizon, and the quiet moments that stay with me long after I’ve left.

## Greeting

Hello, lovers of the sea! 🌊

My name is Myxogastria0808, and I like the sea too!

## coast logo

I create a logo for my blog theme named "coast".

{{/* image(path="/content/coast/coast.png") */}}

```

Please refer to the following for an actual example.

- markdown example

https://github.com/Myxogastria0808/coast/blob/main/content/sample/index.md

- preview URL

https://zola-coast-theme.netlify.app/sample/

## coast Theme Specific Notation

### List of languages in Code Block

https://www.getzola.org/documentation/content/syntax-highlighting/

#### Example

````
```rs
fn main() {
    println!("Hello, world!");
}
```
````

![codeblock](https://github.com/Myxogastria0808/coast/blob/main/assets/codeblock.png)

### Image

```
{{/* image(path="/image/path") */}}
```

You can add `width=int`, `height=int`, and `caption=String` as options for image shortcode.

You can see the image shortcode examples below.

- markdown example

https://github.com/Myxogastria0808/coast/blob/main/content/sample/index.md

- preview URL

https://zola-coast-theme.netlify.app/sample/

#### Example applying all of `width`, `height`, and `caption`

```
{{/* image(path="/content/sample/image.jpg", width=1000, height=200, caption="caption") */}}
```

![image](https://github.com/Myxogastria0808/coast/blob/main/assets/image.png)

> [!NOTE]
> Images are automatically converted to webp, so you don't need to worry about image size.

### katex

```
$$
<katex's syntax expression>
$$
```

#### Example

```
$$
\frac{dy}{dx} + p(x) y^2 + q(x) y + r(x) = 0
$$
```

![katex](https://github.com/Myxogastria0808/coast/blob/main/assets/katex.png)

### mermaid

```
{%/* mermaid() */%}
<mermaid's syntax expression>
{%/* end */%}
```

#### Example

```
{%/* mermaid() */%}
graph TD;
    A-->B;
    A-->C;
    B-->D;
    C-->D;
{%/* end */%}
```

![mermaid](https://github.com/Myxogastria0808/coast/blob/main/assets/mermaid.png)

### note

```
{%/* note() */%}
Contents of the note.
{%/* end */%}
```

#### Example

```
{%/* note() */%}
This is a note.
{%/* end */%}
```

![note](https://github.com/Myxogastria0808/coast/blob/main/assets/note.png)

### tip

```
{%/* tip() */%}
Contents of the tip.
{%/* end */%}
```

#### Example

```
{%/* tip() */%}
This is a tip.
{%/* end */%}
```

![tip](https://github.com/Myxogastria0808/coast/blob/main/assets/tip.png)

### important

```
{%/* important() */%}
Contents of the important.
{%/* end */%}
```

#### Example

```
{%/* important() */%}
This is a important.
{%/* end */%}
```

![important](https://github.com/Myxogastria0808/coast/blob/main/assets/important.png)

### warning

```
{%/* warning() */%}
Contents of the warning.
{%/* end */%}
```

#### Example

```
{%/* warning() */%}
This is a warning.
{%/* end */%}
```

![warning](https://github.com/Myxogastria0808/coast/blob/main/assets/warning.png)

### caution

```
{%/* caution() */%}
Contents of the caution
{%/* end */%}
```

#### Example

```
{%/* caution() */%}
This is a caution.
{%/* end */%}
```

![caution](https://github.com/Myxogastria0808/coast/blob/main/assets/caution.png)

## Structure of this template

The following is expressed in pseudo-HTML.

### Top page (Also serves as an article list page)

```
<base.html>
  <loading.html></loading.html>
  <main-header.html></main-header.html>
  <index.html></index.html>
</base.html>
```

### Tag list page

```
<base.html>
  <sub-header.html></sub-header.html>
  <taxonomy_list.html></taxonomy_list.html>
</base.html>
```

### List of specific tags page

```
<base.html>
  <sub-header.html></sub-header.html>
  <taxonomy_single.html></taxonomy_single.html>
</base.html>
```

### Post page

```
<base.html>
  <sub-header.html></sub-header.html>
  <blog-template.html></blog-template.html>
</base.html>
```

### 404 page

```
<base.html>
  <sub-header.html></sub-header.html>
  <404.html></404.html>
</base.html>
```

## References

https://www.getzola.org/documentation/getting-started/overview/#content

https://swaits.com/adding-mermaid-js-to-zola/

        