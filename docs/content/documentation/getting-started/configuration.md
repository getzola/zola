+++
title = "Configuration"
weight = 4
+++

The default configuration will be enough to get Gutenberg running locally but not more than that. 
It follows the philosophy of only paying for what you need: almost everything is turned off by default.

To change the config, edit the `config.toml` file. 
If you are not familiar with TOML, have a look at [the TOML Spec](https://github.com/toml-lang/toml)
to learn about it.

Only one variable - `base_url` - is mandatory, everything else is optional. You can find all variables
used by Gutenberg config as well as their default values below:


```toml
# Base URL of the site, the only required config argument
base_url = "mywebsite.com"

# Used in RSS by default
title = ""
description = ""
language_code = "en"

# Theme name to use
theme = ""

# Highlight all code blocks found
highlight_code = false

# Which theme to use for the code highlighting. 
# See below for list of accepted values
highlight_theme = "base16-ocean-dark"

# Whether to generate a RSS feed automatically
generate_rss = false

# The number of articles to include in the RSS feed
rss_limit = 20

# Whether to generate a tags page and individual 
# tag pages for pages with tags
generate_tags_pages = false

# Whether to generate a categories page and individual 
# category pages for pages with a category
generate_categories_pages = false

# Whether to compile the Sass files found in the `sass` directory
compile_sass = false

# You can put any kind of data in there and it 
# will be accessible in all templates
[extra]
```

## Syntax highlighting

Gutenberg currently has the following highlight themes available:

- base16-ocean-dark
- base16-ocean-light
- gruvbox-dark
- gruvbox-light
- inspired-github
- kronuz
- material-dark
- material-light
- monokai
- solarized-dark
- solarized-light
- 1337

Gutenberg uses the Sublime Text themes, making it very easy to add more. 
If you want a theme not on that list, please open an issue or a pull request on the [Gutenberg repo](https://github.com/Keats/gutenberg).
