
+++
title = "zerm"
description = "A minimalistic and dark theme based on Radek Kozieł's theme for Hugo"
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://github.com/ejmg/zerm.git"
homepage = "https://github.com/ejmg/zerm"
minimum_version = "0.8.0"
license = "MIT"
demo = "https://zerm.ejmg.now.sh/"

[extra.author]
name = "elias julian marko garcia"
homepage = "https://github.com/ejmg"
+++        

# zerm

a minimalist and dark theme for [Zola](https://getzola.org).

![Screenshot](../master/zerm-preview.png?raw=true)

[**Live Preview!**](https://zerm.ejmg.now.sh/)

Largely a port of Radek Kozieł's [Terminal
Theme](https://github.com/panr/hugo-theme-terminal) for Hugo. 4/5ths of my way
through porting this theme, I discovered Paweł Romanowski own independent fork
for Zola, [Terminimal](https://github.com/pawroman/zola-theme-terminimal),
which helped me get the PostCSS to Sass styling conversion done more
quickly. My sincerest thanks to both of you!

## differences

This theme is largely true to the original by Radek, but there are some mild
differences. They are almost all stylistic in nature and are intended to
emphasize minimalism even more. Some of them are as follows:
- tags are now included in a post's meta data.
- no post image previews.
- categories are included in the taxonomy.
- bullet points have slightly more margin and different symbols for nesting.
- no social media or comment support.

Some of these might be added later and [PR's are always
welcomed](https://github.com/ejmg/zerm/pulls).

## configuration

Please follow the Zola documentation for [how to use a
theme](https://www.getzola.org/documentation/themes/installing-and-using-themes/#installing-a-theme).

In `config.toml`, you will find all values for customization that are supported
thus far have documentation explaining how they are used. If there is any confusion or something is not working as intended, [please open an issue](https://github.com/ejmg/zerm/issues)!

## license

MIT. See `LICENSE.md` for more details.

        