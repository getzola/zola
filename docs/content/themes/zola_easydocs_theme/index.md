
+++
title = "EasyDocs"
description = "An easy way to create docs for your project"
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/codeandmedia/zola_easydocs_theme.git"
homepage = "https://github.com/codeandmedia/zola_easydocs_theme"
minimum_version = "0.13.0"
license = "MIT"
demo = "https://easydocs.codeandmedia.com"

[extra.author]
name = "Roman Soldatenkov"
homepage = "https://codeandmedia.com"
+++        

## An easy way to create a document library for your project

Demo: [https://easydocs.codeandmedia.com/](https://easydocs.codeandmedia.com/)

This theme for [Zola](https://getzola.org) (static site engine) helps you build and publish your project docs easily and fast. Zola is just one binary that outputs html-pages and additional static assets after building your docs written in Markdown. Thus, you can take the theme, your md-files, Zola and gain flexible and simple website for documentation. 

### Step-by-step

As you may have heard Zola is quite flexible :) So, the scenario below is one of hundreds possible ways to make things done, feel free to find your best. Also, Zola provides their own mechanism to install and use themes, see [the docs](https://www.getzola.org/documentation/themes/installing-and-using-themes/). 

1. Fork the repo and replace demo-content inside content folder with yours. But take a look to _index.md files. It contains `title` and when you want to have anchor right of your headers add `insert_anchor_links = "right"` to each index. `theme.toml`, screenshot and readme may be deleted too. 
2. Inside `config.toml` change URL and title on your own. In extra section you can specify path to your GitHub API for version below the logo on nav, favicon and logo itself. Or just remove the lines if you don't need it. Also, you can configure or turn on some additional settings related to Zola. [Specification is here](https://www.getzola.org/documentation/getting-started/configuration/).
3. In sass/_variables.scss you may change font, color or background if you want.
4. Almost done. Now, you should decide how you want to build and where will be hosted your website. You can build it locally and upload to somewhere. Or build in GitHub Actions and host on GitHub Pages / Netlify / CloudFlare Pages / AnyS3CloudStorage. [Howto for GitHub Pages](https://www.getzola.org/documentation/deployment/github-pages/). [My example](https://github.com/o365hq/o365hq.com/blob/main/.github/workflows/main.yml) of GitHub workflow with 2-steps build (the first checks for links and spelling errors, the second uploads to Azure). [Dockerfile](https://github.com/codeandmedia/zola_docsascode_theme/blob/master/Dockerfile) to make Docker image.

## Provided configurations options

These options can be configured in the `extra` section of the [config.toml](config.toml).
If any are not present it has the same behaviour as the default which is shown in the starter [config.toml](config.toml).

- **easydocs_logo_always_clickable** controls if the logo is always clickable. By default the logo is only clickable if you are not on the home page. If this is enabled it will make the logo clickable when you are on the home page as well. Thus on the home page it will basically just refresh the page as it will take you to the same page.
- **easydocs_uglyurls** provides support for offline sites that do not use a webserver. If set to true links in the nav are generated with the full path indcluding `index.html`. This functionality was  insired by [Abridge theme](https://www.getzola.org/themes/abridge/). Note that for this to work it also requries the base URL to be set to the local folder where the site will be stored eg. `base_url = file:///home/user/mysite/public/`. Therefore this is not portable and only works with a specific local folder, but does not require a webserver to navigate the site.
- **easydocs_heading_threshold** controls minimum number of headings needed on a page before the headings show in the navigation on the left. Defaults to 5. Can be used for example to always show headings on each page by setting it to 1.

Enjoy your docs!

* _Icons: [Office UI Fabric Icons](https://uifabricicons.azurewebsites.net/)_
* _Copy-code-button: [Aaron Luna](https://aaronluna.dev/blog/add-copy-button-to-code-blocks-hugo-chroma/)_
        