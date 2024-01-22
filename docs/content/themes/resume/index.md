
+++
title = "resume"
description = "A resume theme"
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/AlongWY/zola-resume.git"
homepage = "https://github.com/alongwy/zola-resume"
minimum_version = "0.11.0"
license = "MIT"
demo = "https://resume.alongwy.top"

[extra.author]
name = "Feng Yunlong"
homepage = "https://www.alongwy.top"
+++        

# Zola Resume

[Chinese Version](README.CN.md)

Redesigned form [hugo resume](https://github.com/eddiewebb/hugo-resume).

## Features
+ This is basically a single-page website with auto-scrolling based on left-hand nav.
+ Dedicated project/publications pages allow more detail.
+ Includes a client-side search at '/search'. 
+ Includes an `/admin` endpoint that can allow authorized users to use a WYSIWYG editor and commit files back to markdown, but with a Wordpress/CMS like experience.

## Quick Start

```bash
git clone git@github.com:alongwy/zola-resume.git
cd zola-resume
zola serve
# open http://127.0.0.1:1111/
```

## Installation
Just earlier we showed you how to run the theme directly. Now we start to install the theme in an existing site step by step.

### Step 1: Create a new zola site

```bash
zola init mysite
```

### Step 2: Install zola-resume
Download this theme to your themes directory:

```bash
cd mysite/themes
git clone git@github.com:alongwy/zola-resume.git
```

Or install as a submodule:

```bash
cd mysite
git init  # if your project is a git repository already, ignore this command
git submodule add git@github.com:alongwy/zola-resume.git themes/zola-resume
```

### Step 3: Configuration
Enable the theme in your config.toml in the site derectory:

```toml
theme = "zola-resume"
```

Or copy the config.toml.example from the theme directory to your project's root directory:

```bash
cp themes/zola-resume/config.toml.example config.toml
```

#### For CMS

```bash
cp themes/zola-resume/static/admin/config.yml static/admin/config.yml
```

and change those

```yaml
# static/admin/config.yml

backend:
  name: github
  repo: USERNAME/REPO
  branch: BRANCH
  cms_label_prefix: netlify-cms/
  site_domain: DOMAIN.netlify.com
```

### Step 4: Add new content
You can copy the content from the theme directory to your project:

```
cp -r themes/zola-resume/data .
cp -r themes/zola-resume/content .
```

You can modify or add new posts in the content/blog, content/projects or other content directories as needed.

### Step 5: Run the project
Just run zola serve in the root path of the project:

```
zola serve
```

This will start the Zola development web server accessible by default at http://127.0.0.1:1111. Saved changes will live reload in the browser.

## Examples

![screenshot](https://raw.githubusercontent.com/alongwy/zola-resume/master/screenshot.png)

See [along's site](https://resume.alongwy.top) for a live example.

## Setup & Use

This theme uses a combination of custom sections and some data files to drive content.

### Summary
Edit the main `contents/_index.md with a brief bio/summary`

### Data files
Data files are used for simple content presented on the homepage.

- [data/certifications.json](https://github.com/AlongWY/zola-resume/blob/main/data/certifications.json)
- [data/social.json](https://github.com/AlongWY/zola-resume/blob/main/data/social.json)
- [data/skills.json](https://github.com/AlongWY/zola-resume/blob/main/data/skills.json)
- [data/experience.json](https://github.com/AlongWY/zola-resume/blob/main/data/experience.json)
- [data/education.json](https://github.com/AlongWY/zola-resume/blob/main/data/education.json)

### Projects/Opensource

The difference indicates your role as originator or colaborator.

### Publications
Similar to projects, create them under `publications`. Include any papers, speaking engagements, articles, etc.

### Blog / Posts
Similar to posts, create them under `blog`. Include any thoughts, musiings, etc.
**This template does not support a `posts` folder**

### Template params

Almost All personal information outside the above details is captured by extra in [`config.toml`](https://github.com/AlongWY/zola-resume/blob/main/config.toml), or can be edited in the "Settings" collection if using CMS.

## CMS Editor with Netlify CMS
**Does not require deployment to Netlify!**

[Netlify CMS](https://www.netlifycms.org/) is an open source project that enables CMS like experience for static site generation tools like Hugo. This theme includes a fully working integration and guide in [static/admin](https://github.com/AlongWY/zola-resume/tree/main/static/admin)

## Credits

This project ports the Hugo Resume theme by Feng Yunlong to support zola.


        