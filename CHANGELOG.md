# Changelog

## 0.13.0 (2021-01-09)

- Enable HTML minification
- Support `output_dir` in `config.toml`
- Allow sections to be drafted
- Allow specifying default language in filenames
- Render emoji in Markdown content if the `render_emoji` option is enabled
- Enable YouTube privacy mode for the YouTube shortcode
- Add language as class to the `<code>` block and as `data-lang`
- Add bibtex to `load_data`
- Add a `[markdown]` section to `config.toml` to configure rendering
- Add `highlight_code` and `highlight_theme` to a `[markdown]` section in `config.toml`
- Add `external_links_target_blank`, `external_links_no_follow` and `external_links_no_referrer`
- Add a `smart_punctuation` option in the `[markdown]` section in `config.toml` to turn elements like dots and dashes 
into their typographic forms
- Add iteration count variable `nth` for shortcodes to know how many times a shortcode has been invoked in a given
content
- Update some highlighting syntaxes and the TS syntax will now be used instead of JS due to issues with it
- Remove `zola serve --watch-only`: since we build the HTML in memory and not on disk, it doesn't make sense anymore
- Update clojure syntax
- Prefer extra syntaxes to the default ones if we have a match for language
- Fix `zola serve` having issues with non-ascii paths
- 404 page now gets the site default language as `lang`

## 0.12.2 (2020-09-28)

- Fix `zola serve` being broken on reload

## 0.12.1 (2020-09-27)

- Add line highlighting in code blocks
- Fix the new `zola serve` being broken on Windows
- Fix slugified taxonomies not being rendered at the right path
- Fix issues with shortcodes with newlines and read more

## 0.12.0 (2020-09-04)

### Breaking

- All paths like `current_path`, `page.path`, `section.path` (except colocated assets) now have a leading `/`
- Search index generation for Chinese and Japanese has been disabled by default as it leads to a big increase in 
binary size

### Other

- Add 2 syntax highlighting themes: `green` and `railsbase16-green-screen-dark`
- Enable task lists in Markdown
- Add support for SVG in `get_image_metadata`
- Fix parsing of dates in arrays in `extra`
- Add a `--force` argument to `zola init` to allow creating a Zola site in a non-empty directory
- Make themes more flexible: `include` can now be used
- Make search index generation configurable, see docs for examples
- Fix Sass trying to load folders starting with `_`, causing issues with frameworks
- Update livereload.js version
- Add Markdown-outputting shortcodes
- Taxonomies with the same name but different casing are now merged, eg Author and author

## 0.11.0 (2020-05-25)

### Breaking
- RSS feed support has been altered to allow, *and default to*, Atom feeds, Atom being technically superior and just as widely-supported in normal use cases.
  - New config value `feed_filename`, defaulting to `atom.xml` (change to `rss.xml` to reinstate the old behaviour)
  - Config value `rss_limit` is renamed to `feed_limit`
  - Config value `languages.*.rss` is renamed to `languages.*.feed`
  - Config value `generate_rss` is renamed to `generate_feed`
  - Taxonomy value `rss` is renamed to `feed`

  Users with existing feeds should either set `feed_filename = "rss.xml"` in config.toml to keep things the same, or set up a 3xx redirect from rss.xml to atom.xml so that existing feed consumers aren’t broken.

- The feed template variable `last_build_date` is renamed to `last_updated` to more accurately reflect its semantics
- The sitemap template’s `SitemapEntry` type’s `date` field has been renamed to `updated` to reflect that it will use the `updated` front-matter field if available, rather than `date`
- Code blocks are now wrapped in `<pre><code>` instead of just `<pre>`

### Other
- Add `updated` front-matter field for pages, which sitemap templates will use for the `SitemapEntry.date` field instead of the `date` front-matter field, and which the default Atom feed template will use
- Add `lang` to the feed template context
- Add `taxonomy` and `term` to the feed template context for taxonomy feeds
- Fix link checker not looking for anchor with capital id/name
- Pass missing `lang` template parameter to taxonomy list template
- Fix default index section not having its path set to '/'
- Change cachebust strategy to use SHA256 instead of timestamp

## 0.10.1 (2020-03-12)

- Set user agent for HTTP requests
- Add nyx-bold highlight theme
- Add lyric and subtitles highlighting
- Enable strikethrough in markdown filter

## 0.10.0 (2020-02-17)

### Breaking
- Remove `toc` variable in section/page context and pass it to `page.toc` and `section.toc` instead so they are
accessible everywhere

### Other
- Add zenburn syntax highlighting theme
- Fix `zola init .`
- Add `total_pages` to paginator
- Do not prepend URL prefix to links that start with a scheme
- Allow skipping anchor checking in `zola check` for some URL prefixes
- Allow skipping prefixes in `zola check`
- Check for path collisions when building the site
- Fix bug in template extension with themes
- Use Rustls instead of openssl
- The continue reading HTML element is now a `<span>` instead of a `<p>`
- Update livereload.js
- Add --root global argument

## 0.9.0 (2019-09-28)

### Breaking

- Add `--drafts` flag to `build`, `serve` and `check` to load drafts. Drafts are never loaded by default anymore
- Using `fit` in `resize_image` on an image smaller than the given height/width is now a no-op and will not upscale images anymore

### Other
- Add `--open` flag to open server URL in default browser
- Fix sitemaps namespace & do not urlencode URLs
- Update livereload
- Add `hard_link_static` config option to hard link things in the static directory instead of copying
- Add warning for old style internal links since they would still function silently
- Print some counts when running `zola check`
- Re-render all pages/sections when `anchor-link.html` is changed
- Taxonomies can now have the same name in multiple languages
- `zola init` can now be create sites inside the current directory
- Fix table of contents generation for deep heading levels
- Add `lang` in all templates context except sitemap, robots
- Add `lang` parameter to `get_taxonomy` and `get_taxonomy_url`
- Rebuild whole site on changes in `themes` changes
- Add one-dark syntax highlighting theme
- Process images on changes in `zola serve` if needed after change

## 0.8.0 (2019-06-22)

### Breaking

- Allow specifying heading IDs. It is a breaking change in the unlikely case you are using `{#..}` in your heading
- Internal links are now starting by `@/` rather than `./` to avoid confusion with relative links
- Latest Tera version now cares about where the `safe` filter is, always put it at the end of an expression.

### Other

- Fix image processing not happening if called from the template
- Add a `zola check` command to that validates the site and checks all external links
- Sections can have `aliases` as well
- Anchors in internal links are now checked for existence

## 0.7.0 (2019-04-28)

### Breaking
- Remove --base-path option, it broke `serve` on Windows and wasn't properly tested

### Other
- Strip wrapping whitespaces from shortcodes
- Sort sitemap elements by `permalink`

## 0.6.0 (2019-03-25)

### Breaking
- `earlier/later` and `lighter/heavier` are not set anymore on pages when rendering
a section
- The table of content for a page/section is now only available as the `toc` variable when
rendering it and not anymore on the `page`/`section` variable
- Default directory for `load_data` is now the root of the site instead of the `content` directory
- Change variable sent to the sitemap template, see documentation for details

### Other
- Add support for content in multiple languages
- Lower latency on serve before rebuilding from 2 to 1 second
- Allow processing PNG and produced images are less blurry
- Add an id (`zola-continue-reading`) to the paragraph generated after a summary
- Add Dracula syntax highlighting theme
- Fix using inline styles in headers
- Fix sections with render=false being shown in sitemap
- Sitemap is now split when there are more than 30 000 links in it
- Add link to sitemap in robots.txt
- Markdown rendering is now fully CommonMark compliant
- `load_data` now defaults to loading file as plain text, unless `format` is passed
or the extension matches csv/toml/json
- Sitemap entries get an additional `extra` field for pages only
- Add a `base-path` command line option to `build` and `serve`


## 0.5.1 (2018-12-14)

- Fix deleting markdown file in `zola serve`
- Fix pagination for taxonomies being broken and add missing documentation for it
- Add missing pager pages from the sitemap
- Allow and parse full RFC339 datetimes in filenames
- Live reload is now enabled for the 404 page on serve


## 0.5.0 (2018-11-17)

### Breaking

- Gutenberg has changed name to `zola`!
- The `pagers` variable of Paginator objects has been removed
- `section.subsections` is now an array of paths to be used with the `get_section`
Tera function
- Table of content now strips HTML from the titles to avoid various issues
- `gutenberg-anchor` CSS class has been renamed `zola-anchor`
- `data` is now a reserved variable name in templates, it is unused right now but
might change soon.

### Others
- Many many times faster (x5-x40) for most sites
- Update dependencies, fixing a few bugs with templates
- Load only .html files in themes from the templates folder
- Background colour is set fewer times when highlighting syntaxes, resulting in smaller HTML filesize
- Link checker will not try to validate email links anymore
- Load table and footnote markdown extensions in `markdown` filter
- `get_url` now defaults to not adding a trailing slash
- Fix `--base-url` not overriding processed images URLs
- Add more Emacs temp file to the ignored patterns in `gutenberg serve`
- Files starting with `.` are not considered pages anymore even if they end with `.md`
- `_processed_images` folder for image processing has been renamed `processed_images` to avoid issues with GitHub Pages
- Syntax highlighting default was mistakenly `true`, it has been set to `false`
- Add NO_COLOR and CLICOLOR support for having colours or not in CLI output
- Fix `robots.txt`template not being used
- RSS feed now takes all available articles by default instead of limiting to 10000
- `templates` directory is now optional
- Add Reason and F# syntax highlighting
- Add `ancestors` to pages and sections pointing to the relative path of all ancestor
sections up to the index to be used with the `get_section` Tera function
- Add a `load_data` Tera function to load local CSV/TOML/JSON files
- Add `relative_path` to pages and sections in templates
- Do not have a trailing slash for the RSS permalinks
- `serve` will now try to find other ports than 1111 rather than panicking
- Ensure content directory exists before rendering aliases
- Do not include drafts in pagination
- Pages filenames starting by a date will now use that date as page date if there isn't one defined in frontmatter
- Accept markdown files starting with BOM
- Add a `watch-only` flag to the `serve` command for when you don't want a webserver
- Add `transparent` sections, for when you want to separate content by sections but want to group them at a higher level (think a `posts` folder with years
that want to use pagination on the index).
- Add `page_template` to section front-matter for when you want to specify the template to use for every page under it
- Improves to `zola serve`: now handles directories renaming

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
