
+++
title = "Lekhz"
description = "A text focussed and minimal personal portfolio theme with dark mode."
template = "theme.html"
date = 2025-06-25T23:58:28-07:00

[taxonomies]
theme-tags = ['minimal', 'clean', 'blog', 'responsive', 'personal', 'simple', 'minimalist', 'portfolio', 'text-focussed', 'dark', 'dark-mode', 'rss']

[extra]
created = 2025-06-25T23:58:28-07:00
updated = 2025-06-25T23:58:28-07:00
repository = "https://github.com/ba11b0y/lekhz.git"
homepage = "https://github.com/ba11b0y/lekhz"
minimum_version = "0.38"
license = "MIT"
demo = ""

[extra.author]
name = "Rahul Tiwari"
homepage = "https://ba11b0y.github.io/rahul/"
+++        

# lekhz

lekhz is a simple, minimalistic, and fast personal website template for Zola.
Ported from the Hugo theme [lekh](https://github.com/ba11b0y/lekh)

<img width="1461" alt="lekhz-dark" src="https://github.com/user-attachments/assets/c4ce7b2b-e55b-4bd9-bb58-c3bb6480700c" />



## Contents

- Features
- Installation
- Customization

## Features
* Social media links
* Markdown supported
* Easy to personalize
* RSS feed
* Dark mode (taken from https://www.gwern.net/ as it is.)
* GoatCounter counts(analytics). Know more about GoatCounter [here](https://goatcounter.com)

## Installation

Create a new Zola site if you haven't already.

```bash
zola init my-site
cd my-site
git init
```

Add the theme as a submodule:

```bash
git submodule add https://github.com/ba11b0y/lekhz.git themes/lekhz
```
**OR**

Clone the theme

```bash
cd themes
git clone https://github.com/ba11b0y/lekhz.git
```

and then enable it in your `config.toml`:

```toml
theme = "lekhz"
```

To start with, copy the contents of the `content` folder to your new site.

```bash
cp -r themes/lekhz/content/* content/
```

## Customization options

Add the following to your `config.toml` file in the `[extra]` section:

```toml
[extra]
lekhz_name = "Rahul Tiwari"
lekhz_about = "About me description here"
lekhz_email = "jprrahultiwari@gmail.com"
lekhz_resume = "resume.pdf"
lekhz_post_limit = 3
lekhz_goatcounter_code = ""
# Example profiles configuration
lekhz_profiles = [
    { name = "GitHub", url = "https://github.com/ba11b0y" },
    { name = "Twitter", url = "https://x.com/ba11b0y" },
    { name = "LinkedIn", url = "https://www.linkedin.com/in/ba11b0y/" },
    { name = "Goodreads", url = "https://www.goodreads.com/user/show/91520565-rahul-tiwari"}
]
```

## Some more screenshots

<img width="1451" alt="light-lekhz" src="https://github.com/user-attachments/assets/d120cdd4-3aa2-4e2d-9889-83e97034d9ba" />

<img width="1461" alt="lekhz-posts" src="https://github.com/user-attachments/assets/688bdfa9-dfdb-4ee6-8b94-1e60e5f93261" />




        