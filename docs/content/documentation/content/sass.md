+++
title = "Sass"
weight = 110
+++

Sass is a popular CSS preprocessor that adds special features (e.g., variables, nested rules) to facilitate the
maintenance of large sets of CSS rules. If you're curious about what Sass
is and why it might be useful for styling your static site, the following links
may be of interest:

* The [official Sass website](https://sass-lang.com/)
* [Why Sass?](https://alistapart.com/article/why-sass), by Dan Cederholm

It currently uses [grass](https://github.com/connorskees/grass), a Rust implementation of Sass roughly equivalent
with dart-sass.

## Using Sass in Zola

Zola processes any files with the `sass` or `scss` extension in the `sass`
folder, and places the processed output into a `css` file with the same folder
structure and base name into the `public` folder:

```bash
.
└── sass
    ├── style.scss // -> ./public/style.css
    ├── indented_style.sass // -> ./public/indented_style.css
    ├── _include.scss # This file won't get put into the `public` folder, but other files can @import it.
    ├── assets
    │   ├── fancy.scss // -> ./public/assets/fancy.css
    │   ├── same_name.scss // -> ./public/assets/same_name.css
    │   ├── same_name.sass # CONFLICT! This has the same base name as the file above, so Zola will return an error.
    │   └── _common_mixins.scss # This file won't get put into the `public` folder, but other files can @import it.
    └── secret-side-project
        └── style.scss // -> ./public/secret-side-project/style.css
```

Files with a leading underscore in the name are not placed into the `public`
folder, but can still be used as `@import` dependencies. For more information, see the "Partials" section of
[Sass Basics](https://sass-lang.com/guide).

Files with the `scss` extension use "Sassy CSS" syntax,
while files with the `sass` extension use the "indented" syntax: <https://sass-lang.com/documentation/syntax>.
Zola will return an error if `scss` and `sass` files with the same
base name exist in the same folder to avoid confusion -- see the example above.

## Site configuration in Sass

Zola supports referencing a subset of the site's configuration (defined in its `config.toml`) in Sass files.  An example where this might be useful is a theme author allowing the site owner to configure theme colors by defining certain keys in their site `config.toml`.  At build time, Zola generates a Sass file containing the appropriate config expressed as a [Sass map literal](https://sass-lang.com/documentation/values/maps/) and makes that Sass file available to theme-defined and user-defined Sass files when they are compiled.  The config keys that are currently exposed are:

1. `base_url`
2. `theme`
3. Everything in `extra`

### Example

`config.toml`
```
title = "My Test Site"
base_url = "https://replace-this-with-your-url.com"
compile_sass = true

[extra.sass]
background_color = "red"
```

`style.scss`
```
@use 'zola';
@use 'sass:map';

body {
  background: map.get(zola.$config, extra, sass, background_color);
}
```