
+++
title = "austere"
description = "A minimal theme for Zola with a focus on writing."
template = "theme.html"
date = 2025-12-21T15:33:47Z

[taxonomies]
theme-tags = ['dark', 'blog', 'minimal', 'personal', 'responsive', 'seo', 'writing']

[extra]
created = 2025-12-21T15:33:47Z
updated = 2025-12-21T15:33:47Z
repository = "https://github.com/tomwrw/austere-theme-zola"
homepage = "https://github.com/tomwrw/austere-theme-zola"
minimum_version = "0.17.0"
license = "MIT"
demo = ""

[extra.author]
name = "tomwrw"
homepage = "https://www.tomwrw.com"
+++        

<p align="center">
  <img src="logo.svg" alt="austere" width="200">
</p>

<h1 align="center">austere</h1>

<p align="center">A minimal theme for <a href="https://getzola.org">Zola</a> with a focus on writing.</p>

---

## Features

- **Lightweight** - ~2KB inline CSS, no external stylesheets
- **Dark/Light mode** - Toggle with system preference detection
- **Customizable colours** - Configure all theme colours in `config.toml`
- **Responsive images** - Automatic resizing and srcset generation
- **Search** - Client-side search powered by Fuse.js
- **Feeds** - Atom and RSS feed generation
- **SEO** - OpenGraph and Twitter Card meta tags
- **Accessibility** - Skip links, semantic HTML, focus states
- **Projects page** - Showcase your work with a dedicated projects section
- **Analytics** - Optional Umami or Google Analytics support

## Installation

1. Download this theme to your `themes` directory:
   ```bash
   cd your-zola-site
   git clone https://github.com/tomwrw/austere-theme-zola themes/austere
   ```

2. Set the theme in your `config.toml`:
   ```toml
   theme = "austere"
   ```

3. Copy the example content to get started:
   ```bash
   cp -r themes/austere/content/* content/
   ```

## Configuration

### Required Settings

```toml
base_url = "https://yoursite.com"

# Required for search
build_search_index = true

[search]
index_format = "fuse_javascript"

# Required for syntax highlighting
[markdown]
highlight_code = true
highlight_theme = "css"
```

### Theme Options

All theme options go under `[extra]` in your `config.toml`:

#### Site Identity

| Option | Description | Default |
|--------|-------------|---------|
| `strapline` | Tagline shown in header | *(none)* |
| `favicon` | Path to favicon | *(none)* |
| `profile_picture` | Profile image on home page | *(none)* |
| `keywords` | SEO meta keywords | *(none)* |
| `footer_text` | Footer HTML content | *(none)* |

```toml
[extra]
strapline = "Welcome to my website"
favicon = "/favicon.ico"
profile_picture = "/images/me.png"
keywords = "blog, writing, zola"
footer_text = "Made with <a href='https://getzola.org'>Zola</a>"
```

#### Navigation

```toml
[extra]
menu_links = [
  { url = "$BASE_URL/", name = "Home" },
  { url = "$BASE_URL/posts/", name = "Posts" },
  { url = "$BASE_URL/about/", name = "About" },
  { url = "$BASE_URL/projects/", name = "Projects" },
  { url = "$BASE_URL/tags/", name = "Tags" },
  { url = "$BASE_URL/search/", name = "Search" },
]
```

#### Colours

Customize the color scheme for light and dark modes:

```toml
[extra.colours.light]
background = "#FAF7F2"
text = "#1a1a1a"
text_muted = "#3a3a3a"
accent = "#B85450"
code_bg = "#f0ebe3"
border = "#e0d9ce"

[extra.colours.dark]
background = "#141413"
text = "#e8e8e8"
text_muted = "#a0a0a0"
accent = "#E07A5F"
code_bg = "#1e1e1d"
border = "#2a2a29"
```

#### Responsive Images

```toml
[extra]
image_format = "auto"      # auto, webp, jpg, png
image_quality = 80         # 1-100
images_default_size = 1024
images_sizes = [512, 1024, 2048]
```

#### Analytics (Optional)

```toml
[extra]
# Umami Analytics
umami_website_id = "your-website-id"
umami_src = "https://cloud.umami.is/script.js"  # optional, custom domain
umami_domains = "yoursite.com"                  # optional, limit tracking

# OR Google Analytics
google_analytics_tag_id = "G-XXXXXXXXXX"
```

## Content

### Posts

Create posts in `content/posts/`:

```markdown
+++
title = "My Post Title"
date = 2024-01-15
description = "A brief description for SEO"
[taxonomies]
tags = ["zola", "blogging"]
+++

Your content here...
```

For posts with images, use a folder structure:

```
content/posts/my-post/
├── index.md
└── image.jpg
```

### Pages

Create standalone pages in `content/`:

```markdown
+++
title = "About"
template = "page.html"
+++

Page content...
```

### Projects

Create `content/projects/_index.md`:

```markdown
+++
title = "Projects"
template = "projects.html"
+++
```

Then create `content/projects/projects.toml`:

```toml
[[projects]]
title = "Project Name"
emoji = "🚀"
description = "What this project does"
url = "https://github.com/you/project"
image = "screenshot.png"  # optional, relative to projects folder
date = "2024"
status = "Active"
tags = ["rust", "web"]
```

### Search

Create `content/search.md`:

```markdown
+++
title = "Search"
template = "search.html"
+++
```

## Shortcodes

### Responsive Image

```markdown
{{/* image(src="photo.jpg", alt="Description") */}}
```

### YouTube Embed

```markdown
{{/* youtube(id="dQw4w9WgXcQ") */}}
{{/* youtube(id="dQw4w9WgXcQ", autoplay=true) */}}
```

### Spotify Embed

```markdown
{{/* spotify(id="album-id") */}}
```

## Customization

### Template Hooks

Override these macros in your own `templates/macros/hooks.html`:

```html
{%/* macro post_above_content(page) */%}
<!-- Content before post body -->
{%/* endmacro */%}

{%/* macro post_below_content(page) */%}
<!-- Content after post body -->
{%/* endmacro */%}

{%/* macro post_below_tags(page) */%}
<!-- Content after post tags -->
{%/* endmacro */%}

{%/* macro posts_below_title(page) */%}
<!-- Content after post title in list view -->
{%/* endmacro */%}
```

### OpenGraph Images

Add a preview image to any page:

```markdown
+++
[extra]
og_preview_img = "preview.jpg"
+++
```

## Requirements

- Zola 0.17.0 or later

## License

MIT

## Credits

- [Zap](https://github.com/jimmyff/zap) by [jimmyff](https://github.com/jimmyff) - Original theme inspiration
- [Zola](https://getzola.org) - Static site generator
- [Fuse.js](https://fusejs.io) - Client-side search
- [Source Serif 4](https://fonts.google.com/specimen/Source+Serif+4) - Typography
- Favicon by [IconsMind](https://iconarchive.com/artist/iconsmind.html)

        