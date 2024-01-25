
+++
title = "zolastrap"
description = "A bootstrap theme for zola"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/marcodpt/zolastrap.git"
homepage = "https://github.com/marcodpt/zolastrap"
minimum_version = "0.14.1"
license = "MIT"
demo = "https://marcodpt.github.io/zolastrap/"

[extra.author]
name = "Marco Tomic"
homepage = "https://marcodpt.github.io"
+++        

# zolastrap
A bootstrap theme for zola

[Live Demo](https://marcodpt.github.io/zolastrap/)

## `config.toml` [extra] variables
### banner
 - type: string
 - default: ""

Path of a banner image, use empty string for no banner

### date_format
 - type: string
 - default: "%d/%m/%Y"

date format expression

### theme
 - type: string
 - default: "default"

one of the [Bootswatch](https://bootswatch.com) themes

### bg
 - type: string
 - default: "dark"

one of the available backgrounds in
[Bootstrap5](https://getbootstrap.com/docs/5.1/utilities/background/)
for `navbar` and `footer`

### inverted
 - type: boolean
 - default: false

Invert font for `navbar` and `footer` in case default choice is bad

### themes
 - type: string
 - default: "Choose a Theme"

Navbar label for themes dropdown.

This dropdown will allow user to change
[Bootswatch](https://bootswatch.com) theme.

Use empty string in case you do not want the user choose a theme.

### schemes
 - type: string
 - default: "Choose a Color Scheme"

Navbar label for schemes dropdown.

This dropdown will allow user to change footer and navbar
[background](https://getbootstrap.com/docs/5.1/utilities/background/)
color.

Use empty string in case you do not want the user choose a theme.

### search
 - type: string
 - default: "Search"

Placeholder for navbar search input.

Remember that to enable and disable search you should set variable
[build_search_index](https://www.getzola.org/documentation/getting-started/configuration/).

### tag
 - type: string
 - default: "Posts by Topic"

Taxonomy `tag` single label. Useful for translations.

### tags
 - type: string
 - default: "Posts by Topics"

Taxonomy `tag` list label. Useful for translations.
You can have a nice tag list at the bottom of a page using `extra.tags` = true
in the `_index.md`

### links
 - type: array
 - default: []

Navbar links. Use an empty array to ignore this.

Items (object):
 - title (String): label of the navbar link
 - url (String): href of associate link

### email
 - type: string
 - default: ""

Footer email. Use an empty string to ignore this.

### icons
 - type: array
 - default: []

Footer social icons. Use an empty array to ignore this.

Items (object):
 - title (string): Optional title string for icon
 - icon (string): One of 
 - url (string): href of the icon

### utterances
 - type: string
 - default: "" 

[utterances](https://github.com/utterance/utterances) repo url.

Use an empty string to ignore utterances widget.

### utterances_label
 - type: string
 - default: "Comments" 

[utterances](https://github.com/utterance/utterances) widget label.

### utterances_theme
 - type: string
 - default: "github-light" 

[utterances](https://github.com/utterance/utterances) widget theme.

### utterances_issue_term
 - type: string
 - default: "pathname" 

[utterances](https://github.com/utterance/utterances) widget pathname.

## Contributing
Any help is greatly appreciated!

 - [Tera template engine](https://tera.netlify.app/docs)
 - [Zola SSG templates](https://www.getzola.org/documentation/templates/overview/)
 - [Bootstrap5 docs](https://getbootstrap.com/docs/5.1/getting-started/introduction/)
 - [Bootswatch](https://bootswatch.com)

        