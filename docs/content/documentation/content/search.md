+++
title = "Search"
weight = 100
+++

Zola can build a search index from the sections and pages content to
be used by a JavaScript library: [elasticlunr](http://elasticlunr.com/).

To enable it, you only need to set `build_search_index = true` in your `config.toml` and Zola will
generate an index for the `default_language` set for all pages not excluded from the search index.

It is very important to set the `default_language` in your `config.toml` if you are writing a site not in
English: the index building pipelines are very different depending on the language.

After `zola build` or `zola serve`, you should see two files in your static directory:

- `search_index.${default_language}.js`: so `search_index.en.js` for a default setup
- `elasticlunr.min.js`

As each site will be different, Zola makes no assumptions about how your search and doesn't provide
the JavaScript/CSS code to do an actual search and display results. You can however look at how this very site
is implementing it to have an idea: [search.js](https://github.com/getzola/zola/tree/master/docs/static/search.js).
