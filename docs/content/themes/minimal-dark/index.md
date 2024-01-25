
+++
title = "minimal-dark"
description = "Clean and minimalistic dark theme"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/kuznetsov17/minimal-dark.git"
homepage = "https://github.com/getzola/minimal-dark"
minimum_version = "0.18.0"
license = "MIT"
demo = "https://kuznetsov17.github.io/minimal-dark/"

[extra.author]
name = "Vitaliy Kuznetsov"
homepage = "https://viku.me"
+++        


# General

I am not the best webmaster, but should be somewhat responsive.
I intentionally using the bigger fonts to make, feel free to change it in main.css

# Important
Please make sure to set up your base_url with trailing slash:
```toml
base_url = "https://kuznetsov17.github.io/minimal-dark/"
```
# Comments
Theme supports [Giscuss](https://giscuss.app) for comments. The configuration is done via config.toml. Here you can see the example section used for this page deployment:
```toml
[extra.giscus]
data_repo="kuznetsov17/minimal-dark"
data_repo_id="R_kgDOLIfXYA"
data_category="General"
data_category_id="DIC_kwDOLIfXYM4Ccn56"
data_mapping="title"
data_strict="0"
data_reactions_enabled="0"
data_emit_metadata="0"
data_input_position="top"
data_theme="//kuznetsov17.github.io/minimal-dark/css/gs_dark.css"
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
The section template supports pagination, tags, sorts the pages by publication date. You may see the working example [here](/notes/)

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

# Screenshot
![Screenshot](https://github.com/kuznetsov17/minimal-dark/blob/main/screenshot.png?raw=true)

        