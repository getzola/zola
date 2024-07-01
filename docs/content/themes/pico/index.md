
+++
title = "pico"
description = "Clean and minimalistic dark theme"
template = "theme.html"
date = 2024-07-01T05:58:26Z

[extra]
created = 2024-07-01T05:58:26Z
updated = 2024-07-01T05:58:26Z
repository = "https://github.com/kuznetsov17/pico.git"
homepage = "https://github.com/kuznetsov17/pico"
minimum_version = "0.18.0"
license = "MIT"
demo = "https://kuznetsov17.github.io/pico/"

[extra.author]
name = "Vitaliy Kuznetsov"
homepage = "https://viku.me"
+++        

# Configuration
# General

I am not the best webmaster, but should be somewhat responsive.
I intentionally using the bigger fonts to make, feel free to change it in main.css

# Light mode
Now light mode also supported. 

# Important
Please make sure to set up your base_url with trailing slash:
```toml
base_url = "https://kuznetsov17.github.io/pico/"
```
# Comments
Theme supports [Giscuss](https://giscuss.app) for comments. The configuration is done via config.toml. Here you can see the example section used for this page deployment:
```toml
[extra.giscus]
data_repo="kuznetsov17/pico"
data_repo_id="R_kgDOLIfXYA"
data_category="General"
data_category_id="DIC_kwDOLIfXYM4Ccn56"
data_mapping="title"
data_strict="0"
data_reactions_enabled="0"
data_emit_metadata="0"
data_input_position="top"
data_theme="//kuznetsov17.github.io/pico/css/gs_dark.css"
data_lang="en"
crossorigin="anonymous"
nonce=""
```

# Page configurations
Customize the page blocks by setting configuration in **[extra]** section:
```toml
show_copyright = true / false # enables / disables footer with copyright
show_comments = true / false # enables / disables comments
show_shares = true / false # enables / disables section with social share buttons
show_toc = true / false # enables / disable TOC
show_date = true / false # displays publication date in page
```

# Blog
I am using this theme for my [notes](https://viku.me/notes/), or probably blog. 
The section template supports pagination, tags, sorts the pages by publication date. You may see the working example [here](@/notes/_index.md)


# Search
The theme supports the search using [elasticrunrjs](http://elasticlunr.com). To enable the search, you will need the following configuration in **config.toml**:

```toml
build_search_index = true

[search]
index_format = "elasticlunr_json"
```

# config.toml extras
```toml
author = "John Doe" # author. Will be puth in page metadata
description = "Some description, if you somehow didn't set it in page / section settings"
logo_src = "images/logo.svg" # logo src
avatar_src = "images/avatar.png" # avatar src
index_page="index" # name of the index page. Should be one of top_menu to make things work
top_menu = ["index","features","notes"] # Menu items
copyright_string = "Сreated by John Doe in 2024 – %YEAR% for fun." # footer content. %YEAR% will be replaced with current year
nonce = "${SOME_HASH_VALUE}" # used for JavaScript src nonce
```

# timeline
```
{%/* timeline() */%}
[{
    "title":"Lorem Ipsum Event",
    "body":"Lorem Ipsum is simply dummy text of the printing and typesetting industry.",
    "date":"Jul-2023"
},
{
    "title":"Lorem Ipsum event 2",
    "body":"Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged.",
    "date":"Jun-2022"
}]
{%/* end */%}
```

# Callouts
```
{%/* callout(type = 'warning') */%}
This is an example of **Warning** callout. [Some link](#)
{%/* end */%}
{%/* callout(type = 'alert') */%}
This is an example of **Alert** callout. [Some link](#)
{%/* end */%}
{%/* callout(type = 'info') */%}
This is an example of **Info** callout. [Some link](#)
{%/* end */%}
```
# Mermaid

Read more on how to use mermaid in their [documentation](https://mermaid.js.org/syntax/examples.html)
```
{%/* mermaid() */%}
gitGraph
       commit
       commit
       branch develop
       checkout develop
       commit
       commit
       checkout main
       merge develop
       commit
       commit
{%/* end */%}
```
```
{%/* mermaid() */%}
graph LR
    A[Square Rect] -- Link text --> B((Circle))
    A --> C(Round Rect)
    B --> D{Rhombus}
    C --> D
{%/* end */%}
```
# Thanks to
 - [Giscuss](https://giscuss.app) for excellent comments system
 - [bootstrap icons](https://icons.getbootstrap.com) for great social icons
 - [Urbanist Font](https://fonts.google.com/specimen/Urbanist)
 - [Mulush Font](https://fonts.google.com/specimen/Mulish)

# Screenshot
![Screenshot](https://github.com/kuznetsov17/pico/blob/main/screenshot.png?raw=true)


        