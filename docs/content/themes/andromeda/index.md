
+++
title = "Andromeda"
description = "Photography journal blog theme"
template = "theme.html"
date = 2024-01-25T10:41:35+02:00

[extra]
created = 2024-01-25T10:41:35+02:00
updated = 2024-01-25T10:41:35+02:00
repository = "https://github.com/Pixadus/andromeda-theme.git"
homepage = "https://github.com/Pixadus/andromeda-theme"
minimum_version = "0.16.0"
license = "MIT"
demo = "https://andromeda-theme.netlify.app/"

[extra.author]
name = "Parker Lamb"
homepage = "https://blog.andromeda.is"
+++        

# Andromeda Theme for Zola

> Andromeda is a lightweight **photojournal & blog** theme designed for Zola.

With built-in support for galleries and some options for customization, Andromeda is designed for photojournalism without complications. 

Index demo:
![Index demo graphic](index_demo.jpg)

Post demo:
![Post demo graphic](post_demo.png)

---

## Installation

Assuming you already have a site set up (see the [Zola guide for setting up a site](https://www.getzola.org/documentation/getting-started/overview/)), 

1. Create a `themes` directory in the root of your site if it does not already exist. 
2. Clone the theme into your themes directory: 
    ```
    git clone https://github.com/Pixadus/andromeda-theme themes/andromeda
    ```
3. Duplicate the structure of the the `config.toml` file found in `themes/andromeda/config.toml` or [this repository](https://github.com/Pixadus/andromeda-theme/blob/main/config.toml) within your own `config.toml`.
4. Set the theme to Andromeda, by including `theme = andromeda` in your `config.toml` file.

## Creating pages

To create a new post, create a `.md` file within `/content`, with the header format:

```markdown 
+++
title = "Post title"
date = 2023-04-25
description = "Post description"
extra = {header_img = "image-url"}
+++
```
**Note**: The +++ are necessary.

The `header_img` field is the image shown on the homepage of the blog and in the heading of each page. It can be a remote URL or local - if local, by default this will be files stored in the `static` folder, or `/images` in the URL. 

### Galleries

Galleries can be set up by using the following template in your Markdown file:

```html
<div class="gallery">
    <a href="original_photo1.jpg" data-ngthumb="thumbnail_photo1.jpg"></a>
    <a href="original_photo2.jpg" data-ngthumb="thumbnail_photo2.jpg"></a>
</div>
```

For more or less photos, use `<a href>` tags. [Flickr](https://www.flickr.com/) provides a good hosting option as it automatically generates thumbnails for you. 

## Configuration

Andromeda supports custom navbar links - see [config.toml](https://github.com/Pixadus/andromeda-theme/blob/main/config.toml) for an example. You may also set a custom `favicon.ico` though `config.toml`. 

If you wish to customize the design of the gallery, basic Javascript knowledge will be necessary. Andromeda uses `nanogallery2` by default - the [documentation can be found here](https://nanogallery2.nanostudio.org/documentation.html). Customizations to the gallery design are done within the `{%/* macro pagefooter() */%}` block within `/templates/macros.html`. 

By default, this script is divided into three sections (indicated by `item==`): single-image, two-image and three+ image gallery setups. 

## Credits

The demo images used included [Antelope Canyon by Anishkumar Sugumaran](https://www.flickr.com/photos/anishkumar_sugumaran/52831738797/in/explore-2023-04-26/) and [Bryce Canyon by Marco Isler](https://www.flickr.com/photos/27263572@N05/52838617702/in/explore-2023-04-26/). 
        