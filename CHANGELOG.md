# Changelog

## 0.1.2 (unreleased)

- Add `redirect_to` to section front matter to redirect when landing on section
root page
- Make `title` in config optional
- Improved `gutenberg init` UX and users first experience

## 0.1.1 (2017-07-16)

- Fix RSS feed not behaving (https://github.com/Keats/gutenberg/issues/101)

## 0.1.0 (2017-07-14)

- Parallelize all the things
- Add weight sorting
- Remove `section` from the `page` rendering context: this is too expensive. Use
the global function `get_section` if you need to get it
- Put back a 20 page limit on rss feed by default (configurable)
- Remove index page getting all sections: use the `get_section` global fn instead to
only get the ones you need
- Remove pages from pagers in pagination: they were not supposed to be there
- Add built-in Sass compilation support


## 0.0.7 (2017-06-19)

- Sort individual tag/category pages by date
- Add extra builtin shortcode for Streamable videos
- `path` and `permalink` now end with a `/`
- Generate table of contents for each page
- Add `section` to a page Tera context if there is one
- Add `aliases` to pages for when you are changing urls but want to redirect
to the new one
- Name the homepage section `index` (previously empty string)

## 0.0.6 (2017-05-24)

- Fix missing serialized data for sections
- Change the single item template context for categories/tags
- Add a `get_url` and a `get_section` global Tera function
- Add a config option to control how many articles to show in RSS feed
- Move `insert_anchor_links` from config to being a section option and it can
now be insert left or right


## 0.0.5 (2017-05-15)

- Fix XML templates overriding and reloading
- `title` and `description` are now optional in the front matter
- Add GenericConfig, Vim, Jinja2 syntax
- Add `_index.md` for homepage as well and make that into a normal section
- Allow sorting by `none`, `date` and `order` for sections
- Add pagination
- Add a `get_page` global function to tera
- Revamp index page, no more `pages` variables
- Fix livereload stopping randomly
- Smarter re-rendering in `serve` command

## 0.0.4 (2017-04-23)

- Fix RSS feed link and description
- Renamed `Page::url` and `Section::url` to `Page::path` and `Section::path`
- Pass `current_url` and `current_path` to every template
- Add id to headers to allow anchor linking
- Make relative link work with anchors
- Add option to render an anchor link automatically next to headers
- Only copy the static files that changed, not the whole directory in `gutenberg serve`
- Use summary if available in RSS feed
- Add tables and footnotes support in markdown
- Add more language syntaxes
- Only load templates ending by `.html`

## 0.0.3 (2017-04-05)

- Add some colours in console
- Allow using a file other than config.toml for config
- Add sections to the index page context
- Fix page rendering not working when containing `+++`
- Add shortcodes (see README for details)
- Allow relative links to other content in markdown links
- Add `markdown`, `base64_encode` and `base64_decode` filters to the Tera instance of Gutenberg
- Work on Windows!
