
+++
title = "dinkleberg"
description = "The Rust BR theme for Gutenberg"
template = "theme.html"
date = 2018-05-31T23:02:50-03:00

[extra]
created = 2019-04-06T11:27:43+02:00
updated = 2018-05-31T23:02:50-03:00
repository = "https://github.com/rust-br/dinkleberg.git"
homepage = "https://github.com/rust-br/dinkleberg"
minimum_version = "0.4.0"
license = "MIT"
demo = "https://rust-br.github.io/blog/"

[extra.author]
name = "Guilherme Diego"
homepage = "https://github.com/guidiego"
+++        

![753986_1](https://user-images.githubusercontent.com/10289071/40806112-dd79ae78-64f6-11e8-8f24-63f387d5bb8f.jpg) 

Rust BR Blog template for Gutenberg

## Features
- A kind of i18n for base words as: "Next", "Previous", "Pages", "Categories"
- Blog Title and Logo on extra configurations
- Auto-sidebar links by configuration
- Simple design based on Medium
- SEO using structured data and another features

## Configurations
```toml
[extra]
blog_logo="/imgs/common/logo.png" #will appear on top header
blog_title="rust::br::Blog" #will appear on top header after logo

## i18n words
label_tags = "Tags"
label_tag = "Tag"
label_categories = "Categorias"
label_category = "Categoria"
label_relative_posts = "Postagens Relacionadas"
label_next = "Próxima"
label_previous = "Anterior"
label_page = "Página"
label_of = "de"

og_image="" # Image that will appear on social media
og_alt_image="" # Alt for og_image
og_site_name="" # Site Name for Open Graphic
keywords="" # Keywords for SEO

educational_use="knowledge share" # OPTIONAL
copyright_year="2018" # OPTIONAL

fb_app_id="???" # OPTIONAL, Facebook App Id to help in metrics
twitter_username="@???" # OPTIONAL, Twitter User to help with metrics

## Sidebar automatic links
sidebar = [
    {name = "Social", urls=[
        {name="Telegram", url="https://t.me/rustlangbr"},
        {name="Github", url="https://github.com/rust-br"},
    ]},
    {name = "Divida Conhecimento!", urls=[
        {name="Contribuir!", url="https://rust-br.github.io/blog/hello-world"}
    ]}
]

```

This configuration was the same configuration that we use on [RustBR Blog](https://rust-br.github.io/blog)

### Favicons and other stuff
By default Dinkleberg wait that you have all icons on root of your static, for it you can use the site [https://www.favicon-generator.org/](https://www.favicon-generator.org/) to generate that bundle and put it inside you `/static` :D

        