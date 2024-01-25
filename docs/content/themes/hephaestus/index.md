
+++
title = "hephaestus"
description = "A portfolio theme"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/BConquest/hephaestus.git"
homepage = "https://github.com/BConquest/hephaestus"
minimum_version = "0.4.0"
license = "AGPL"
demo = "https://bryantconquest.com"

[extra.author]
name = "Bryant Conquest"
homepage = "https://bryantconquest.com"
+++        

# hephaestus
Hephaestus is a portfolio theme for zola. It uses bulma css and supports using icons from ion-icon.

![hephaestus screenshot](screenshot.png?raw=true)

## Contents
- Installation
- Options
	- Navigation Bar
	- Education
	- Projects
	- Skills
	- Social Links

## Installation

First, you will download the theme into your `themes` directory:

```bash
$ cd themes
$ git clone https://github.com/BConquest/hephaestus
```

Second, you will enable the theme in your `config.toml` directory:

```toml
theme = "hephaestus"
```

## Options
### Navigation Bar
To edit the navigation bar you will need to edit your `config.toml` to include:

```toml
menu = [
{ text = "foo", link = "/foo"},
{ text = "bar", link = "/bar"},
]
```
You can have as many items as you want to have and the links can be to anything.

### Education
To edit the education that is displayed you will need to create a directory in `content`.
In the `_index.md` the frontmatter needs to include:

```TOML
title = "foo"
template = "education.html"

[extra]
author = "Name"
```

For every educational level you want to add you will need to create a new markdown file that includes the frontmatter:

```
title = "place of education"

[extra]
image = "image-location"
link = "link to school"
+++
```

Any content that is typed will be rendered underneath these two items.

### Projects
To edit the projects that are displayed you will need to create a directory in `content`.
In the `_index.md` the frontmatter needs to include:

```TOML
title = "foo"
template = "projects.html"

[extra]
author = "bar"
```

Then for every project you want to add you will need to format the `*.md` as:

```md
+++
title = "foo"

[extra]
image = "/image_location"
link = "link to project"
technologies = ["bar", "baz"]
+++

Description of project named foo.
```

### Skills

To edit the skills that you want to display it is important to note that there are two types of skills that can be
displayed (lan, and tools). To format the look you will need to create a directory in `content` that includes the
frontmatter of:

```TOML
title = "foo"
template = "skills.html"
page_template = "skills.html"

[extra]
author = "author-name"
image = "image-location"

lan = [
{ lang = "language", expr = "num between 1-5", image = "image-location", comfort = "word to describe comfort"},
]

tools = [
{ tool = "tool-name", expr = "num between 1-5", image = "tool-image"},
]
```

### Social Links
To edit the social links that appear in the footer of the page, you need to edit your `config.toml` to include:

```
social = [
{ user = "username", link = "link", icon = "icon-name from ion-icon"},
]
```

        