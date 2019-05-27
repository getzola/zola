+++
title = "Directory structure"
weight = 3
+++

After running `zola init`, you should see the following structure in your folder:


```bash
.
├── config.toml
├── content
├── sass
├── static
├── templates
└── themes

5 directories, 1 file
```

Here's a high level overview of each of these folders and `config.toml`.

## `config.toml`
A mandatory configuration file of Zola in TOML format.
It is explained in details in the [Configuration page](@/documentation/getting-started/configuration.md).

## `content`
Where all your markup content lies: this will be mostly comprised of `.md` files.
Each folder in the `content` directory represents a [section](@/documentation/content/section.md)
that contains [pages](@/documentation/content/page.md) : your `.md` files.

To learn more, read [the content overview](@/documentation/content/overview.md).

## `sass`
Contains the [Sass](http://sass-lang.com) files to be compiled. Non-Sass files will be ignored.
The directory structure of the `sass` folder will be preserved when copying over the compiled files: a file at
`sass/something/site.scss` will be compiled to `public/something/site.css`.

## `static`
Contains any kind of files. All the files/folders in the `static` folder will be copied as-is in the output directory.

## `templates`
Contains all the [Tera](https://tera.netlify.com) templates that will be used to render this site.
Have a look at the [Templates](@/documentation/templates/_index.md) to learn more about default templates
and available variables.

## `themes`
Contains themes that can be used for that site. If you are not planning to use themes, leave this folder empty.
If you want to learn about themes, head to the [themes documentation](@/documentation/themes/_index.md).
