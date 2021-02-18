
+++
title = "zola-paper"
description = "A clean theme inspired from hugo-paper."
template = "theme.html"
date = 2021-02-18T22:27:50+01:00

[extra]
created = 2021-02-18T22:27:50+01:00
updated = 2021-02-18T22:27:50+01:00
repository = "https://github.com/schoenenberg/zola-paper.git"
homepage = "https://github.com/schoenenberg/zola-paper"
minimum_version = "0.11.0"
license = "MIT"
demo = "https://schoenenberg.github.io/zola-paper"

[extra.author]
name = "Maximilian Schoenenberg"
homepage = "https://schoenenberg.dev"
+++        

# Zola-Paper
A clean theme inspired from hugo-paper.

[Zola](https://getzola.org) port of [Hugo-Paper](https://github.com/nanxiaobei/hugo-paper/) (with a few tweaks).

**Demo:** [https://schoenenberg.github.com/zola-paper](https://schoenenberg.github.com/zola-paper)

![Screenshot](screenshot.png)
![Dark Mode Screenshot](screenshot_dark.png)

## Installation

The easiest way to install this theme is to either clone it ...

```bash
git clone https://github.com/schoenenberg/zola-paper.git themes/zola-paper
```

... or to use it as a submodule.

```bash
git submodule add https://github.com/schoenenberg/zola-paper.git themes/zola-paper
```

Either way, you will have to enable the theme in your `config.toml`.

```toml
theme = "zola-paper"
```

## Open Graph Integration

This theme has an integration of Open Graph *meta* tags. These are set based on context and available information. See the following example:

```markdown
+++
title = "Lorem ipsum!"

[extra]
author = "Max Mustermann"
author_url = "https://www.facebook.com/example.profile.3"
banner_path = "default-banner"

[taxonomies]
tags = ["rust", "zola", "blog"]
+++

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nunc eu feugiat sapien. Aenean ligula nunc, laoreet id sem in, interdum bibendum felis. Donec vel dui neque.
<!-- more -->
Ut luctus dolor ut tortor hendrerit, sed hendrerit augue scelerisque. Suspendisse quis sodales dui, at tempus ante. Nulla at tempor metus. Aliquam vitae rutrum diam. Curabitur iaculis massa dui, quis varius nulla finibus a. Praesent eu blandit justo. Suspendisse pharetra, arcu in rhoncus rutrum, magna magna viverra erat, ...

```

Required attributes of the `extra` section is `author`. All other attributes are optional. The path for the `banner_path` attribute has to be relative to the content directory.

        