# Changelog

## 0.4.3 (unreleased)

- Gutenberg has changed name to REPLACE_ME!
- Update dependencies, fixing a few bugs with templates
- Load only .html files in themes from the templates folder
- Background colour is set fewer times when highlighting syntaxes
- Link checker will not try to validate email links anymore
- Load table and footnote markdown extensions in `markdown` filter
- `get_url` now defaults to not adding a trailing slash
- Fix `--base-url` not overriding processed images URLs
- Many many times faster (x10-x20) for sites with thousands of pages
- Add more Emacs temp file to the ignored patterns in `gutenberg serve`

## 0.4.2 (2018-09-03)

- Add assets to section indexes
- Allow users to add custom highlighting syntaxes
- Add Swift, MiniZinc syntaxes and update others
- Handle post summaries better: no more cutting references

## 0.4.1 (2018-08-06)

- Fix live reload of a section content change getting no pages data
- Fix critical bug in `serve` in some OSes
- Update deps, should now build and work correctly on BSDs

## 0.4.0 (2018-08-04)

### Breaking

- Taxonomies have been rewritten from scratch to allow custom ones with RSS and pagination
- `order` sorting has been removed in favour of only having `weight`
- `page.next/page.previous` have been renamed to `page.later/page.earlier` and `page.heavier/page.lighter` depending on the sort method

### Others
- Fix `serve` not working with the config flag
- Websocket port on `live` will not get the first available port instead of a fixed one
- Rewrite markdown rendering to fix all known issues with shortcodes
- Add array arguments to shortcodes and allow single-quote/backtick strings
- Co-located assets are now permalinks
- Words are now counted using unicode rather than whitespaces
- Aliases can now be pointing directly to specific HTML files
- Add `year`, `month` and `day` variables to pages with a date
- Fix panic when live reloading a change on a file without extensions
- Add image resizing support
- Add a 404 template
- Enable preserve-order feature of Tera
- Add an external link checker
- Add `get_taxonomy` global function to return the full taxonomy

## 0.3.4 (2018-06-22)

- `cargo update` as some dependencies didn't compile with current Rust version
- Add CMake syntax highlighting and update other syntax highlighting

## 0.3.3 (2018-03-29)

- Fixed config flag in CLI
- Sitemap entries are now sorted by permalinks to avoid random ordering
- Preserve directory structure from sass folder when copying compiled css files
to the public directory
- Do not require themes to have a static folder
- Now supports indented Sass syntax
- Add search index building
- Update Tera: now has `break` and `continue` in loops
- Gutenberg now creates an anchor link at the position of the `<!-- more -->` tag if you
want to link directly to it
- Fix many shortcode parsing issues
- Correctly copy themes shortcodes so they are useable in content
- Fix internal links not working for markdown files directly in `content` directory

## 0.3.2 (2018-03-05)

- Fix `serve` command trying to read all files as markdown
- Add many syntax highlighting themes
- Fix date being serialised incorrectly in page `extra` section of front-matter

## 0.3.1 (2018-02-15)

- Update Tera and other dependencies
- Add option for inline (ie no `<p>...</p>` wrapping) in markdown filter
- Allow to specify both interface and base_url in `gutenberg serve` for usage in Docker

## 0.3.0 (2018-01-25)

### Breaking
- Change names of individual taxonomies to be plural (ie `tags/my-tag` instead of `tag/my-tag`)
- Front matter now uses TOML dates rather strings: remove quotes from your date value to fix it.
For example: `date = "2001-10-10"` becomes `date = 2001-10-10`
- `language_code` has been renamed `default_language` in preparations of i18n support

### Others
- Add `get_taxonomy_url` to retrieve the permalink of a tag/category
- Fix bug when generating permalinks for taxonomies
- Update to Tera 0.11
- Better UX on first `serve` thanks to some default templates.
- Add `output-dir` to `build` and `serve` to generate the site in a folder other than `public`
- Add Prolog syntax highlighting and update all current syntaxes
- Live reloading now works on shortcode template changes
- `gutenberg serve` now reloads site on `config.toml` changes: you will need to F5 to see any changes though
- Add a `trans` global function that will get return the translation of the given key for the given lang, defaulting
to `config.default_language` if not given
- `gutenberg serve` cleans after itself and deletes the output directory on CTRL+C

## 0.2.2 (2017-11-01)

- Fix shortcodes without arguments being ignored
- Fix shortcodes with markdown chars (_, *, etc) in name and args being ignored
- Fix subsections of index not being filled without a `_index.md`
- Fix generated index section not found in `get_section` global function
- Fix permalink generation for index page
- Add Nim syntax highlighting
- Allow static folder to be missing
- Fix shortcodes args being only passed as strings
- Add `page.components` and `section.components` that are equivalent to `path.split('/')`
- Expose `page.draft` in the template

## 0.2.1 (2017-10-17)

- Fix `base-url` argument to `gutenberg build` being called `base`
- Add syntaxes: Crystal, Elixir, Kotlin

## 0.2.0 (2017-10-05)

- Fix `section.subsections` not being filled correctly
- `section.subsections` can now be sorted by a `weight` attribute on a section front-matter
- Do nothing on directory adding/removal in livereload
- Add back `draft` on pages that was wrongly removed
- Page and Section `path` field is not starting with a `/` anymore
- All Tera global fns are now rebuilt on changes
- Use flags for port/interface in `gutenberg serve`
- Fix various issues with headers markdown rendering
- Rename `insert_anchor` in section front-matter to `insert_anchor_links`
- Remove `insert_anchor_links` from the config: it wasn't used
- Add `class` variable to `gist` shortcode
- Add reading analytics to sections content
- Add config to sitemap template
- Add `permalink` to all taxonomy items (tags & categories)
- Tags in the tags page are now sorting alphabetically instead of by number of pages in them
- Remove deprecated `link` param of `get_url`
- Add 1337 color scheme
- Defaults to compressed Sass output
- Fix regression wrt co-located assets slug detecting
- Rename `url` from page front-matter to `path` to be consistent
- Add a `base-url` flag to the `build` command to override the URL from config.toml

## 0.1.3 (2017-08-31)

- Add themes support


## 0.1.2 (2017-08-10)

- Add `redirect_to` to section front matter to redirect when landing on section
root page
- Make `title` in config optional
- Improved `gutenberg init` UX and users first experience
- Make `get_url` work for any path with optional cachebusting.
- Deprecates `link` param of `get_url` in favour of `path` to be consistent

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
