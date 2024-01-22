
+++
title = "kangae"
description = "a lightweight microblog theme for zola"
template = "theme.html"
date = 2024-01-09T08:33:22+01:00

[extra]
created = 2024-01-09T08:33:22+01:00
updated = 2024-01-09T08:33:22+01:00
repository = "https://github.com/ayushnix/kangae.git"
homepage = "https://github.com/ayushnix/kangae"
minimum_version = "0.15.0"
license = "NCSA"
demo = "https://kangae.ayushnix.com/"

[extra.author]
name = "Ayush Agarwal"
homepage = "https://microblog.ayushnix.com"
+++        

# kangae (考え, idea or thought)

[kangae][1] is a lightweight microblog theme for [zola][2].

<details>
  <summary>kangae screenshots on desktop and mobile</summary>

  ![kangae screenshot light mode on desktop](static/images/kangae-desktop-light.webp)
  ![kangae screenshot dark mode on desktop](static/images/kangae-desktop-dark.webp)
  ![kangae screenshot light mode on mobile](static/images/kangae-mobile-light.webp)
  ![kangae screenshot dark mode on mobile](static/images/kangae-mobile-dark.webp)
</details>

I've created kangae from scratch and it is not based on any other theme. However, I was inspired to
create kangae after I came across [Wolfgang Müller's microblog][3]. Thanks Wolf!

kangae is licensed under the [NCSA license][5], which is quite similar to the BSD-3-Clause license.
Unlike BSD-3-Clause, NCSA also covers documentation of a project.

# Showcase

Here's a list of websites using the kangae theme

- [ayushnix microblog][4]

If you want to mention your website in this section, please raise a pull request.

# Installation

Before using this theme, [install zola][6]. After you've installed zola,

```
$ zola init microblog
> What is the URL of your site? (https://example.com):
> Do you want to enable Sass compilation? [Y/n]:
> Do you want to enable syntax highlighting? [y/N]:
> Do you want to build a search index of the content? [y/N]:
$ cd microblog/
```

kangae doesn't use Sass or syntax highlighting so if you don't want to use custom Sass code or
enable syntax highlighting, answer the 2nd and 3rd question with a 'no'. kangae also doesn't use any
JavaScript library to search content. If you don't intend to install a JavaScript library to enable
search on your microblog, answer 'no' to the last question as well.

If you intend to publish your microblog on a forge like GitHub, initialize an empty git repository
using

```
$ git init
$ git commit --allow-empty -m 'initial empty root commit'
```

If you don't want to make an empty commit, add and commit a README or a LICENSE file instead.

At this point, you can install kangae using one of the following methods

## using `git subtree`

```
$ git subtree add -P themes/kangae/ --squash https://github.com/ayushnix/kangae.git master
```

## using `git submodule`

```
$ git submodule add https://github.com/ayushnix/kangae.git themes/kangae
```

## download kangae in themes directory

If you want to keep things simple and figure out version control later, you can

```
$ git clone https://github.com/ayushnix/kangae.git themes/kangae
```

# Configuration

To begin using kangae after installing it,

```
$ cp themes/kangae/config.toml ./
$ sed -i 's;# theme =\(.*\);theme =\1;' config.toml
```

The [`config.toml`][7] file of kangae has been documented carefully using TOML comments. If you have
any questions about configuring kangae which haven't been answered in the `config.toml` file itself,
please [raise an issue][8].

## Shortcodes

kangae provides several shortcodes that can be used to add content in an accessible manner

### kaomoji `(・_・)ノ`

If you want to use kaomoji in your posts, you can use insert them in an accessbile manner using

```
I don't know. {{/* kaomoji(label="shrug kaomoji", text="╮( ˘_˘ )╭") */}} I've never thought about it.
```

Providing a value for the `label` is optional but highly recommended. A short text should be
mentioned that explains what the kaomoji means to convey. The value of `text` should be the actual
emoticon itself.

This shortcode can also be used for any other ASCII emoticon that can fit in an inline paragraph.
This includes western emoticons such as `;)` and combination emoticons such as `<(^_^<)`.

### Quotes

You can add quotes in your microblog posts using

```
{%/* quote(author="Nara Shikamaru") */%}
You would think just this once, when it was life or death, I could pull through.
{%/* end */%}
```

This is the most basic form of improvement in writing quotes over simply using `>` in markdown.

If you want to mention the name of the source from where the quote has been taken, such as the name
of the book or a movie, you can use

```
{%/* quote(citation="Mass Effect 3", author="Javik") */%}
Stand in the ashes of a trillion dead souls, and ask the ghosts if honor matters. The silence is your answer.
{%/* end */%}
```

A `citeurl` can also be given as an argument to this shortcode to provide the actual URL from where
the source is borrowed.

```
{%/* quote(author="Edward Snowden", citeurl="https://old.reddit.com/r/IAmA/comments/36ru89/just_days_left_to_kill_mass_surveillance_under/crglgh2/") */%}
Arguing that you don't care about the right to privacy because you have nothing to hide is no different than saying you don't care about free speech because you have nothing to say.
{%/* end */%}
```

A live preview of these how these shortcodes look like can be found on [this blog post][14].

## Optional Features

kangae includes some optional features that aren't enabled by default

- [style external links using a ↗ unicode symbol][11]

# Donate

If you found kangae helpful in creating your own microblog website, please consider supporting me by
buying me a coffee :coffee:

<a href='https://www.buymeacoffee.com/ayushnix' target='_blank' rel="noopener"><img src='https://cdn.buymeacoffee.com/buttons/default-blue.png' alt='buy ayushnix a coffee at buymeacoffee.com' border='0' height='36'></a>
<a href='https://ko-fi.com/O5O64SQ4C' target='_blank' rel="noopener"><img src='https://cdn.ko-fi.com/cdn/kofi1.png?v=2' alt='buy ayusnix a coffee at ko-fi.com' border='0' height='36'></a>

If you're in India, you can also use UPI for donations. My UPI address is `ayushnix@ybl`.

# Notes

Although I'm not a web developer, I am interested in learning HTML and CSS to create lightweight
textual websites. You may be interested in reading [my log about how I learned HTML and CSS][12].
However, that page is just an unorganized dump of my thoughts and isn't a polished blog post.
[Seirdy's blog post on creating textual websites][13] is probably a better reference.

# TODO (maybe?)

- (responsive) image shortcodes
- run prettier on HTML and CSS before deployment
- twitter and mastodon shortcodes
- add optional support for cross posting and commenting on mastodon without using JS
- add optional support for [giscus][9] and [loading mastodon comments][10]
- add shortcode for asciinema
- add shortcode for blockquote and citation
- pagination
- light and dark mode switch
- content tabs
- microdata and microformats2

[1]: https://kangae.ayushnix.com/
[2]: https://www.getzola.org/
[3]: https://zunzuncito.oriole.systems/
[4]: https://microblog.ayushnix.com
[5]: LICENSE
[6]: https://www.getzola.org/documentation/getting-started/installation/
[7]: config.toml
[8]: https://github.com/ayushnix/kangae/issues/new
[9]: https://giscus.app/
[10]: https://carlschwan.eu/2020/12/29/adding-comments-to-your-static-blog-with-mastodon/
[11]: https://github.com/ayushnix/kangae/blob/master/static/css/style-external-links.css
[12]: https://wiki.ayushnix.com/frontend/creating-a-website/
[13]: https://seirdy.one/2020/11/23/website-best-practices.html
[14]: https://kangae.ayushnix.com/being-shikamaru-102/

        