+++
title = "Taxonomies"
weight = 90
+++

Zola has built-in support for taxonomies, a way to group content according to user-defined categories.

## Definitions

- Taxonomy: A category that can be used to group content
- Term: A specific group within a taxonomy
- Value: A piece of content that can be associated with a term

## Example: a movie website

Imagine that you want to make a website to display information about various movies. In that case, you could use the following taxonomies:

- Director
- Genres
- Awards
- Release year

Then, at build time, Zola can create pages for each taxonomy, listing all of the known terms and linking it to all associated pieces of content.

Imagine again we have the following movies:
```
- Shape of water                             Value
  - Director ............................ Taxonomy
    - Guillermo Del Toro                      Term
  - Genres .............................. Taxonomy
    - Thriller                                Term
    - Drama                                   Term
  - Awards .............................. Taxonomy
    - Golden globe                            Term
    - Academy award                           Term
    - BAFTA                                   Term
  - Release year ........................ Taxonomy
    - 2017                                    Term

- The Room                                   Value
  - Director ............................ Taxonomy
    - Tommy Wiseau                            Term
  - Genres .............................. Taxonomy
    - Romance                                 Term
    - Drama                                   Term
  - Release Year ........................ Taxonomy
    - 2003                                    Term

- Bright                                     Value
  - Director ............................ Taxonomy
    - David Ayer                              Term
  - Genres .............................. Taxonomy
    - Fantasy                                 Term
    - Action                                  Term
  - Awards .............................. Taxonomy
    - California on Location Awards           Term
  - Release Year ........................ Taxonomy
    - 2017                                    Term
```

In this example, the page for `Release year` would include links to pages for both 2003 and 2017, whereas the page for 2017 would list both *Shape of Water* and *Bright*.

## Configuration

A taxonomy has six variables:

- `name`: a required string that will be used in the URLs, usually the plural version (i.e. tags, categories, etc.)
- `paginate_by`: if this is set to a number, each term page will be paginated by this much.
- `paginate_path`: if set, this path will be used by the paginated page and the page number will be appended after it.
For example the default would be `page/1`.
- `feed`: if set to `true`, a feed (atom by default) will be generated for each term.
- `lang`: only set this if you are making a multilingual site and want to indicate which language this taxonomy is for
- `render`: if set to `false`, pages will not be rendered for the taxonomy or for individual terms.

Insert into the configuration file (`config.toml`):

⚠️ Place the taxonomies key in the main section and not in the `[extra]` section

**Example 1:** (one language)

```toml
taxonomies = [
    { name = "director", feed = true},
    { name = "genres", feed = true},
    { name = "awards", feed = true},
    { name = "release-year", feed = true},
]
```

**Example 2:** (multilingual site)

```toml
# These taxonomies go in the main section
taxonomies = [
    {name = "director", feed = true},
    {name = "genres", feed = true},
    {name = "awards", feed = true},
    {name = "release-year", feed = true},
]

[languages.fr]
taxonomies = [
    {name = "director", feed = true},
    {name = "genres", feed = true},
    {name = "awards", feed = true},
    {name = "release-year", feed = true},
]
```

## Using taxonomies

Once the configuration is done, you can then set taxonomies in your content and Zola will pick them up:

**Example:**

```toml
+++
title = "Shape of water"
date = 2019-08-15 # date of the post, not the movie
[taxonomies]
director=["Guillermo Del Toro"]
genres=["Thriller","Drama"]
awards=["Golden Globe", "Academy award", "BAFTA"]
release-year = ["2017"]
+++
```

## Output paths

In a similar manner to how section and pages calculate their output path:
- the taxonomy name is never slugified
- the taxonomy term (e.g. as specific tag) is slugified when `slugify.taxonomies` is enabled (`"on"`, the default) in the configuration

The taxonomy pages are then available at the following paths:

```txt
$BASE_URL/$NAME/ (taxonomy)
$BASE_URL/$NAME/$SLUG (taxonomy entry)
```
Note that taxonomies are case-insensitive, so terms that have the same slug will get merged: sections and pages containing the tag "example" will be shown in the same taxonomy page as those containing "Example".
