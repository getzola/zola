
+++
title = "coffee"
description = "A simple theme for Zola inspired by coffee."
template = "theme.html"
date = 2025-10-31T17:39:00+09:00

[taxonomies]
theme-tags = ['dark', 'simple', 'mermaid', 'katex']

[extra]
created = 2025-10-31T17:39:00+09:00
updated = 2025-10-31T17:39:00+09:00
repository = "https://github.com/Myxogastria0808/coffee.git"
homepage = "https://github.com/Myxogastria0808/coffee/"
minimum_version = "0.19.0"
license = "MIT"
demo = "https://zola-coffee-theme.netlify.app/"

[extra.author]
name = "Myxogastria0808"
homepage = "https://yukiosada.work"
+++        

# coffee theme

**coffee** is a blog theme for zola!

This theme can be used **mermaid** and **katex**.

- demo site

[https://zola-coffee-theme.netlify.app/](https://zola-coffee-theme.netlify.app/)

- [theme logo](https://github.com/Myxogastria0808/coffee/blob/main/logo/README.md)

<div align="center">
  <img src="https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/logo/coffee.svg" width="300px" height="300px" />
</div>

- screenshot

<div align="center">
  <img src="https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/screenshot.png" width="1920px" height="935px" />
</div>

## Setup Environment

1. Install zola

Please install zola by referring to the following.

[https://www.getzola.org/documentation/getting-started/installation/](https://www.getzola.org/documentation/getting-started/installation/)

2. Setup coffee theme

> [!TIP]
> If you want to use the coffee theme repository as a blog, you do not need to do the `2.` step other than making extra settings in the `2-6.` step after cloning the repository.

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

Done! Your site was created in /home/hello/Desktop/coffee-sample/docs

Get started by moving into the directory and using the built-in server: `zola serve`
Visit https://www.getzola.org for the full documentation.
```

2-2. Change directory to your blog project

```sh
cd ./< your blog project >/themes/
```

2-3. Clone coffee theme to theme directory and remove .git directory of coffee theme repository

```sh
git clone https://github.com/Myxogastria0808/coffee.git
rm -rf coffee/.git
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
theme = "coffee"

# The URL the site will be built for
base_url = "/"

# The site title and description; used in feeds by default.
title = "coffee"
description = "A simple theme for Zola inspired by coffee."

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
[extra.coffee] #<- CAUTION: You have to be [extra.coffee], not [extra].
# default value: 'en'
lang = "en"
# default value: 'blog'
keyword = "blog"
# default value: 'https://raw.githubusercontent.com/Myxogastria0808/coffee/heads/main/static/favicon.svg'
# A shortcut icon has to be a SVG image.
icon = "https://raw.githubusercontent.com/Myxogastria0808/coffee/heads/main/static/favicon.svg"
# default value: ''
twitter_site = ""
# default value: '@yuki_osada0808'
twitter_creator = "@yuki_osada0808"
# default value: 'https://raw.githubusercontent.com/Myxogastria0808/coffee/heads/main/static/coffee.webp'
meta_image = "https://raw.githubusercontent.com/Myxogastria0808/coffee/heads/main/static/coffee.webp"
# default value: 'coffee theme'
meta_image_alt = "coffee theme"
# default value: '512'
meta_image_width = "512"
# default value: '512'
meta_image_height = "512"

# default value is below
about = """
Hello, my name is <strong>Myxogastria0808.</strong><br/>
I created a Zola theme named <strong>"coffee"</strong>.
This template can be used <strong>mermaid</strong> and <strong>katex</strong>.
<h4>Have a nice day!</h4>
"""
# default value: 'https://raw.githubusercontent.com/Myxogastria0808/coffee/heads/main/static/coffee.webp'
about_image = "https://raw.githubusercontent.com/Myxogastria0808/coffee/heads/main/static/coffee.webp"
# default value: 'coffee theme'
about_image_alt = "coffee theme"
# default value: '512'
about_image_width = "512"
# default value: '512'
about_image_height = "512"
```

- example (part of `config.toml`)

```toml
[extra.coffee]
keyword = "blog coffee drink"

about = """
Hello, my name is <strong>Myxogastria0808.</strong><br/>
This blog is made by Zola. This is a sample blog of coffee theme.
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
> This repository includes the coffee theme repository as a submodule.

- repository

[https://github.com/Myxogastria0808/coffee-sample.git](https://github.com/Myxogastria0808/coffee-sample.git)

- demo site

[https://zola-coffee-theme-sample.netlify.app/](https://zola-coffee-theme-sample.netlify.app/)

## Post Example

You can see the post example below.

```md
+++
title = "coffee"
date = 2025-08-19
authors = ["Myxogastria0808"]
[taxonomies]
tags = ["coffee"]
+++

The best part of my morning is the quiet moment with a steaming mug of coffee.
It's a simple, potent reminder that a new day has truly begun.

## Greeting

Hello, coffee lovers! ☕

My name is Myxogastria0808, and I like coffee too!

## How I like to drink coffee

My favorite way to drink coffee is to add caramel flavored sugar to black coffee!

## Coffee logo

I create a logo for my blog theme named "coffee".

{{/* image(path="/content/coffee/coffee.webp") */}}

```

Please refer to the following for an actual example.

- markdown example

[https://github.com/Myxogastria0808/coffee/blob/main/content/sample/index.md](https://github.com/Myxogastria0808/coffee/blob/main/content/sample/index.md)

- preview URL

[https://zola-coffee-theme.netlify.app/sample/](https://zola-coffee-theme.netlify.app/sample/)

## coffee Theme Specific Notation

### List of languages in Code Block

[https://www.getzola.org/documentation/content/syntax-highlighting/](https://www.getzola.org/documentation/content/syntax-highlighting/)

#### Example

````
```rs
fn main() {
    println!("Hello, world!");
}
```
````

![codeblock](https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/assets/codeblock.png)

### Image

```
{{/* image(path="/image/path") */}}
```

You can add `width=int`, `height=int`, and `caption=String` as options for image shortcode.

You can see the image shortcode examples below.

- markdown example

[https://github.com/Myxogastria0808/coffee/blob/main/content/sample/index.md](https://github.com/Myxogastria0808/coffee/blob/main/content/sample/index.md)

- preview URL

[https://zola-coffee-theme.netlify.app/sample/](https://zola-coffee-theme.netlify.app/sample/)

#### Example applying all of `width`, `height`, and `caption`

```
{{/* image(path="/content/sample/image.jpg", width=1000, height=200, caption="caption") */}}
```

![image](https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/assets/image.png)

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

![katex](https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/assets/katex.png)

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

![mermaid](https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/assets/mermaid.png)

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

![note](https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/assets/note.png)

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

![tip](https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/assets/tip.png)

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

![important](https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/assets/important.png)

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

![warning](https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/assets/warning.png)

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

![caution](https://raw.githubusercontent.com/Myxogastria0808/coffee/refs/heads/main/assets/caution.png)

## Structure of this template

The following is expressed in pseudo-HTML.

### Top page (Also serves as an article list page)

```
<base.html>
  <index.html></index.html>
</base.html>
```

### Tag list page

```
<base.html>
  <taxonomy_list.html></taxonomy_list.html>
</base.html>
```

### List of specific tags page

```
<base.html>
  <taxonomy_single.html></taxonomy_single.html>
</base.html>
```

### Post page

```
<base.html>
  <blog-template.html></blog-template.html>
</base.html>
```

### 404 page

```
<base.html>
  <404.html></404.html>
</base.html>
```

## References

[https://www.getzola.org/documentation/getting-started/overview/#content](https://www.getzola.org/documentation/getting-started/overview/#content)

[https://swaits.com/adding-mermaid-js-to-zola/](https://swaits.com/adding-mermaid-js-to-zola/)

[https://sippo.work/blog/20231105-deploy-zola-with-cloudflare-pages/](https://sippo.work/blog/20231105-deploy-zola-with-cloudflare-pages/)

[https://zenn.dev/com4dc/scraps/c6c0f5fb87a1f9](https://zenn.dev/com4dc/scraps/c6c0f5fb87a1f9)

        