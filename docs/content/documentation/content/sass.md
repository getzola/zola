+++
title = "Sass"
weight = 110
+++

Sass is a popular CSS extension language that approaches some of the harder
parts of maintaining large sets of CSS rules. If you're curious about what Sass
is and why it might be useful for styling your static site, the following links
may be of interest:

* The [official Sass website](http://sass-lang.com/)
* [Why Sass?](https://alistapart.com/article/why-sass), by Dan Cederholm

## Using Sass in Zola

Zola processes any files with the `sass` or `scss` extensions in the `sass`
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
        └── style.scss // -> ./public/secret-side-project/fancy.css
```

Files with a leading underscore in the name are not placed into the `public`
folder, but can still be used as `@import` dependencies. For more information, see the "Partials" section of
[Sass Basics](https://sass-lang.com/guide#partials).

Files with the `scss` extension use ["Sassy CSS" syntax](http://sass-lang.com/documentation/#Formatting),
while files with the `sass` extension use the ["indented" syntax](http://sass-lang.com/documentation/file.INDENTED_SYNTAX.html).
Zola will return an error if a `scss` and `sass` file exist with the same
base name in the same folder to avoid confusion -- see the example above.
