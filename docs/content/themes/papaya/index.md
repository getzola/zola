
+++
title = "Papaya"
description = "A clean Zola theme for blogging and projects"
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/justint/papaya.git"
homepage = "https://github.com/justint/papaya"
minimum_version = "0.16.1"
license = "MIT"
demo = "https://justintennant.me/papaya"

[extra.author]
name = "Justin Tennant"
homepage = "https://justintennant.me"
+++        

# Papaya

A clean [Zola](https://getzola.org) theme for blogging and projects, forked from [Anpu](https://github.com/zbrox/anpu-zola-theme).

## Preview

**Demo site**: [https://justintennant.me/papaya/](https://justintennant.me/papaya/)

![index light/dark](https://raw.githubusercontent.com/justint/papaya/main/pics/blendedindex.png)

<p align="center">
  <img alt="Light Projects" src="https://raw.githubusercontent.com/justint/papaya/main/pics/projects.png" width="45%">
&nbsp; &nbsp; &nbsp; &nbsp;
  <img alt="Dark Projects" src="https://raw.githubusercontent.com/justint/papaya/main/pics/projects_dark.png" width="45%">
</p>

<p align="center">
  <img alt="Light Project" src="https://raw.githubusercontent.com/justint/papaya/main/pics/project.png" width="45%">
&nbsp; &nbsp; &nbsp; &nbsp;
  <img alt="Dark Project" src="https://raw.githubusercontent.com/justint/papaya/main/pics/project_dark.png" width="45%">
</p>

## Features

- Blog posts
- Project pages
- Automatic light/dark mode
- Categories and tags
- Optional multilingual support
- Customizable sections and navigation menu links
- Featured images for posts/pages
- Smart image embedding shortcode (`{{/* img() */}}`)
- GitHub repository star/fork counts
- [Open Graph Protocol](https://ogp.me/) tags
- [Utterances](https://utteranc.es/) support
- Social/contact links
- 100% Google Lighthouse score

## Installation

1. Clone this repository to your `themes` folder:
    
    ```bash
    git clone https://github.com/justint/papaya.git themes/papaya
    ```

2. Set your theme setting in `config.toml` to `papaya`:

    ```toml
    theme = "papaya"
    ```

3. Copy the following sections and keys (and their contents/values) from papaya's [`config.toml`](https://github.com/justint/papaya/blob/main/config.toml) and paste them into your site's `config.toml`:

   - `[languages]`
     - `[languages.en]`
     - `[languages.en.translations]`
   - `[extra.cdn]`
     - `font_awesome`

4. In your `content` directory, add new `blog` and `projects` directories. Copy the `_index.md` file from Papaya's `content/blog` into your `content/blog`, and the `_index.md` and `categories.json` files from Papaya's `content/projects` into your `content/projects`.
 
   Your `content` directory structure should look like this:
   ```
   content
   ├── blog
   │  └── _index.md
   └── projects
      └── _index.md
      └── categories.json
   ```
 
5. _(optional)_ To enable GitHub repository stars/fork counts (disabled by default to avoid hitting API rate limits), set the `$ZOLA_ENV` environment variable to `prod` prior to your `zola serve`/`zola build` execution.
   
   For csh/tsch:
   ```shell
   setenv ZOLA_ENV prod
   ```
   
   For bash/ksh/zsh:
   ```shell
   export ZOLA_ENV=prod
   ```

## Customization

Here are the customizable features of Papaya: 

- Project categories
- Light/dark mode
- Multilingual support
- Sections and navigation menu links
- Post/project date formats
- Post/project featured images
- Open Graph Protocol locale/profile information
- Utterances
- Social/contact links

### Project categories

In your `content/projects/categories.json`, you can specify the categories of projects. The formatting of the file is:

```json
{
   "title": "keyword"
}
```

- `"title"`: the title text displayed for each category grouping on your projects page.
- `"keyword"`: the taxonomy term you'll use in your project pages.

A project can have multiple categories, and will be displayed once in each category configured.

Projects without categories will be displayed in the "Other" category listing of your project page. If you don't want the "Other" category displayed, you can copy the `templates/projects.html` to your own `templates` directory and delete/comment out the "Other" category code.

Example `categories.json`:

```json
{
  "Software": "software",
  "Films": "film"
}
```

Example project page front matter:

```toml
title = "Example software project"
date = 2021-08-11

[taxonomies]
categories = ["software"]
```

The example project page above would be grouped into & displayed within the "Software" category of your projects page.

### Light/dark mode

The Papaya theme can be set to `"light"`, `"dark"`, or `"auto"` mode in the `config.toml`.

In `"auto"`, the light and dark modes are implicitly chosen by the `prefers-color-scheme` CSS media feature. The theme will switch automatically based on the viewer's OS or user agent setting.

### Multilingual support

Currently Zola has basic internationalization (`i18n`) support, you can read more in [zola's Multilingual Sites doc](https://www.getzola.org/documentation/content/multilingual/).

To write a multilingual site, follow the steps below (English and Chinese in this example):

1. Add a `default_language` configuration and `[languages.zh]` and `[languages.en]` sections to your `config.toml`:

    ```toml
    default_language = "en"

    [languages]

    [languages.en]

    [languages.zh]
    title = "中文标题"
    description = "中文描述"
    ```

    Under the `[languages.zh]` section you can override default configurations like `title`, `description`, etc.

2. Add translations of all keywords in `[languages.zh.translations]` and `languages.en.translations]` sections (see Papaya's [`config.toml`](config.toml) for a listing of all keywords):

    ```toml
    [languages]

    [languages.en]

    [languages.en.translations]
    projects = "Projects"
    blog = "Blog"
    about = "About"
    recent_projects = "Recent Projects"
    more_projects = "More Projects"
    recent_blog_posts = "Recent Blog Posts"
    more_blog_posts = "More blog posts"
    ...

    [languages.zh]

    [languages.zh.translations]
    projects = "项目"
    blog = "博文"
    about = "关于"
    recent_projects = "近期项目"
    more_projects = "更多项目"
    recent_blog_posts = "近期博文"
    more_blog_posts = "更多博文"
    ...
    ```

3. Add a `_index.zh.md` file into every section. 

   For example: add `content/blog/_index.zh.md` and `content/projects/_index.zh.md`. 

4. Provide a `{page-name}.zh.md` (or `index.zh.md` into the page's directory, if it has one) for every page you'd like to translate.

   For example: add `content/blog/what-is-zola.zh.md` and `content/blog/blog-with-image/index.zh.md`.

6. Add a `content/categories.zh.json` file. For example:

    ```json
    {
        "软件": "software",
        "电影": "film"
    }
    ```

Now you will have a website that supports both English and Chinese! Since `default_language` in `config.toml` is set to "en", by visiting `{base_url}` you will see the English version of this blog. You can visit the Chinese version by visiting `{base_url}/zh`.

A page (post or project) can be available in both languages or only in one language, and it's not necessary that a page is available in the default language.

### Sections and navigation menu links

The navigation menu is constructed from a list of `menu_items` in your `config.toml`. For example:
```toml
[extra]

menu_items = [
   { name = "projects", url = "$LANG_BASE_URL/projects", show_recent = true, recent_items = 3, recent_trans_key = "recent_projects", more_trans_key = "more_projects" },
   { name = "blog", url = "$LANG_BASE_URL/blog", show_recent = true, recent_items = 3, recent_trans_key = "recent_blog_posts", more_trans_key = "more_blog_posts" },
   { name = "tags", url = "$LANG_BASE_URL/tags" },
   { name = "about", url = "$LANG_BASE_URL/about" },
]
```

A `menu_item` can be one of two things:

- **a link to a section.** Section links can be optionally configured to display its most recently authored items on your index page. See Configuring section menu items.

- **a link to a URL.** See Configuring URL menu items

#### Configuring section menu items

A section is created whenever a directory (or subdirectory) in the content section contains an `_index.md` file; see the [Zola docs on sections](https://www.getzola.org/documentation/content/section/). 

Papaya has two sections by default: `projects` and `blog`. You can add additional sections or change section names.  For example, you can add a section called _Diary_. In order to add this section, you need to:

1. Create a directory called `diary` in `content/`.

2. Create an `_index.md` inside `content/diary/`, for example:

    ```toml
    +++
    title = "Diary"
    render = true
    # diary will use blog.html for its template
    template = "blog.html"
    +++
    ```

Sections can be added to the navigation menu, and optionally configured to display its most recently authored items on your index page. To add your section to the navigation menu:

1. In your `config.toml` under the `[extra]` section, add your section to the `menu_items`:

    ```toml
    [extra]
    menu_items = [
        ...
        { name = "diary", url = "$LANG_BASE_URL/diary" }
    ]
    ```
   
2. In your `config.toml` under the `[languages.<code>.translations]` section, add your section name translation keys:

   ```toml
   [languages]
   
   [languages.en]
   
   [languages.en.translations]
   diary = "Diary"
   
   [languages.zh]

   [languages.zh.translations]
   diary = "日记"
   ```

   This will add a simple hyperlink to your new _Diary_ section in the navigation menu.

To also display recently authored items from your _Diary_ section on your index page:

1. Add the following attributes to your menu item:

   - `show_recent`: Adds the section's recent items listing to your index page.
   - `recent_items`: Number of recent items to display.
   - `recent_trans_key`: Translation key for the recent items listing title text.
   - `more_trans_key`: Translation key for the hyperlink text to the section.

   For example:

   ```toml
   [extra]
   menu_items = [
       ...
       { name = "diary", url = "$LANG_BASE_URL/diary", show_recent = true, recent_items = 3, recent_trans_key = "recent_diary", more_trans_key = "more_diary" }
   ]
   ```

2. In your `config.toml` under the `[languages.<code>.translations]` section, add your section name, `recent_trans_key`, and `more_trans_key` translation keys:

    ```toml
    [languages]

    [languages.en]

    [languages.en.translations]
    diary = "Diary"
    recent_diary = "Recent Diaries"
    more_diary = "More Diaries"

    [languages.zh]

    [languages.zh.translations]
    diary = "日记"
    recent_diary = "近期日记"
    more_diary = "更多日记"
    ```
   
   This will add both a hyperlink to your new _Diary_ section in the navigation menu, and a listing of the three most recent items from your _Diary_ section on your index page.

#### Configuring URL menu items

If you want to add a simple link to the navigation menu, add an item with a `name` and `url`. For example:

```toml
[extra]
sections = [
    ...
    { name = "tag", url = "$LANG_BASE_URL/tags" }
]
```

A translation key for your link's `name` must be added into your `config.toml`:

```toml
[languages]

[languages.en]

[languages.en.translations]
tag = "Tag"

[languages.zh]

[langauges.zh.translations]
tag = "标签"
```

If you include `$BASE_URL` in the URL of a link it will be replaced with the base URL of your site, and `$LANG_BASE_URL` will be replaced with the language-specific base URL of your site.

### Post/project date formats

You can have different date formats in different languages. You need to set the `date_format` value in every langauge's translation section.

Example:

```toml
[languages]

[languages.en]

[languages.en.translations]
date_format = "%e %B %Y"

[languages.zh]

[languages.zh.translations]
date_format = "%Y 年 %m 月 %d 日"
```

The formatting uses the standard `date` filter in Tera. The date format options you can use are listed in the [chrono crate documentation](https://tera.netlify.app/docs/#date).

### Post/project featured images

Posts and projects can have featured images which display at the top of their page before the page contents.

```toml
[extra]
featured_image = "image.jpg"
featured_image_alt = "A lodge overlooks a forested mountain range."
```

![Featured image](pics/featured_image.png)

Featured images can also be extended to the full width of the viewport:

```toml
[extra]
featured_image = "image.jpg"
featured_image_alt = "A lodge overlooks a forested mountain range."
featured_image_extended = true
```

![Featured image, extended](pics/featured_image_extended.png)

### Open Graph Protocol locale/profile information

In your `config.toml` you can add a `[extra.ogp]` section to specify your Open Graph Protocol locale and profile information.

Open Graph Protocol provides you control over how your website's content should be displayed on social media sites. 

For the more information on Open Graph Protocol and valid property values, visit the official [website](https://ogp.me/). 

Example:

```toml
[extra.ogp]
locale = "en_US"
first_name = "Papaya"
last_name = "Tiliqua"
gender = "female"
username = "tiliquasp"
```

### Utterances

[Utterances](https://utteranc.es/) is a comments widget built on GitHub issues. When enabled, Papaya can display GitHub issues as comments on your blog posts.

To enable:

1. Follow instructions on the [utterances](https://utteranc.es/) website.

2. Once you're at the "Enable Utterances" step, enter the following keys into your `config.toml`:

   ```toml
   [extra.utterances]
   enabled = true
   repo = "yourname/yourrepository" # put your repository's short path here
   post_map = "pathname"
   label = "utterances"
   theme = "preferred-color-scheme"

### Social/contact links

In your `config.toml` you can add a `[extra.social]` section to specify your social network/contact accounts. Changing these will update what links appear on your website's footer.

Example:

```toml
[extra.social]
email = "papaya@tiliqua.sp"
github = "papaya"
linkedin = "papayatiliqua"
twitter = "papayathehisser"
```

If you want to include other custom social websites, you can add them to `other`:

Example:

```toml
[extra.social]
other = [
    { name = "BTC", font_awesome = "fa-brands fa-btc", url = "https://www.bitcoin.com/" }
]
```

The `font_awesome` attribute specifies the Font Awesome classes; you can find them in [Font Awesome](https://fontawesome.com/). Be aware that different versions of Font Awesome may include different sets of icons; you can change your version of Font Awesome by updating the CDN path in the `[extra.cdn]` section:

```toml
[extra]

[extra.cdn]
font_awesome = "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.0.0-beta2/css/all.min.css"
```

## Image embedding shortcode

Included with Papaya is a shortcode for embedding images into your posts:

```
img(path, alt, caption, class, extended_width_pct, quality)
```

You can use `./<image-path>` to specify the relative path of image which is relative to current markdown file.

### Arguments

- `path`: The path to the image. It can be either:
  - a full path (eg: `https://somesite.com/my-image.jpg`), 
  - relative to the `content` directory in the [directory structure](https://www.getzola.org/documentation/getting-started/directory-structure/) (eg: `@/projects/project-1/my-image.jpg`), or
  - relative to the current markdown file (eg: `./my-image.jpg`).
- `alt`: _(optional)_ The alternate text for the image.
- `caption`: _(optional)_ A caption for the image. Text/HTML/Tera templates supported.
- `class`: _(optional)_ Any CSS classes to assign to the image. Multiple classes should be separated with a space (`" "`).
- `quality`: _(optional)_ JPEG or WebP quality of the image, in percent.  Only used when encoding JPEGs or WebPs; default value is `90`.
- `extended_width_pct`: _(optional)_ The percentage by which the image's width should be expanded past it's default figure width, up to maximum configured pixel width. 

   Range is `0.0-1.0`, or `-1` for document width. 

   Max pixel width can be defined in your `config.toml`  with the `extra.images.max_width` property (2500px default).

   See Extended width images section for more details and examples.

The benefits of using this shortcode over regular Markdown/HTML image embedding are:

- Images are automatically resized for best performance, using Zola's [image processing functions](https://www.getzola.org/documentation/content/image-processing/)
- Images & captions are ✨pre-styled✨ for you
- Images can have their width extended past the document's width (see: Extended width images
- Less HTML/CSS boilerplate to write


### Extended width images

Images embedded into pages using the `img` shortcode can be configured to extend past their document width. This is especially nice for displaying wide/landscape images at higher resolutions.

By default, images embedded with the `img` shortcode will be inserted as a `figure` with default margins:

```js
{{/* img(path="image.jpg", 
       alt="A very cute leopard gecko.", 
       caption="A very cute leopard gecko. Default sizing.") */}}
```

![Default sized image](pics/img_default.png)

With the `extended_width_pct` argument, we can specify a percentage of how much the image should expand outside its default figure width, up to your maximum configured image width (`config.extra.images.max_width`, 2500px default).

Here's an example with `extended_width_pct=0.1`:

```js
{{/* img(path="image.jpg", 
       alt="A very cute leopard gecko.", 
       caption="A very cute leopard gecko. extended_width_pct=0.1",
       extended_width_pct=0.1) */}}
```

![Image extended by 0.1](pics/img_0.1.png)

The image is now displayed with a 10% larger width, while maintaining its original aspect ratio.

Here's an even wider example:

```js
{{/* img(path="image.jpg", 
       alt="A very cute leopard gecko.", 
       caption="A very cute leopard gecko. extended_width_pct=0.2",
       extended_width_pct=0.2) */}}
```

![Image extended by 0.2](pics/img_0.2.png)

The images will resize in resolution up to your maximum configured image width, and will display on the webpage up to the maximum width of the viewport.

You can also force the image width to match the document's width by setting `extended_width_pct` to `-1`:

```js
{{/* img(path="image.jpg", 
       alt="A very cute leopard gecko.", 
       caption="A very cute leopard gecko. extended_width_pct=-1",
       extended_width_pct=-1) */}}
```

![Image fixed to document width](pics/img_-1.png)

## Why "Papaya"?

🦎

        