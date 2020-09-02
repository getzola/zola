
+++
title = "simple-dev-blog"
description = "A simple dev blog theme with no javascript, prerendered linked pages and SEO tags."
template = "theme.html"
date = 2020-09-02T11:42:41+05:30

[extra]
created = 2020-09-02T11:42:41+05:30
updated = 2020-09-02T11:42:41+05:30
repository = "https://github.com/bennetthardwick/simple-dev-blog-zola-starter"
homepage = "https://github.com/bennetthardwick/simple-dev-blog-zola-starter"
minimum_version = "0.4.0"
license = "MIT"
demo = "https://simple-dev-blog-zola-starter.netlify.app/"

[extra.author]
name = "Bennett Hardwick"
homepage = "https://bennetthardwick.com/"
+++        


![preview image](https://i.imgur.com/IWoJtkF.png)

# simple-dev-blog-zola-starter

A simple dev-blog theme for Zola. It uses no JavaScript, prerenders links between navigation, blog posts and tags and adds common tags for SEO. 

You can view it live [here](https://simple-dev-blog-zola-starter.netlify.app/).

### How to get started

To create a new Zola site, first download the CLI and install it on your system.
You can find installation instructions [on the Zola website](https://www.getzola.org/documentation/getting-started/installation/).

1. After you've installed the Zola CLI, run the following command to create a new site:

   ```sh
   zola init my_amazing_site
   cd my_amazing_site
   ```

2. After you've created the site, install the "Simple Dev Blog" theme like so:

   ```sh
   git clone --depth=1 \
     https://github.com/bennetthardwick/simple-dev-blog-zola-starter \
     themes/simple-dev-blog
   ```

3. Now in your `config.toml` file, choose the theme by setting `theme = "simple-dev-blog"`.

4. That's it! Now build your site by running the following command, and navigate to `127.0.0.1:111`:

   ```sh
   zola serve
   ```

You should now have a speedy simple dev blog up and running, have fun!


        