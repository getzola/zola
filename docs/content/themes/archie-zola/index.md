
+++
title = "archie-zola"
description = "A zola theme based on Hugo archie."
template = "theme.html"
date = 2025-08-18T14:52:13+08:00

[taxonomies]
theme-tags = []

[extra]
created = 2025-08-18T14:52:13+08:00
updated = 2025-08-18T14:52:13+08:00
repository = "https://github.com/XXXMrG/archie-zola.git"
homepage = "https://github.com/XXXMrG/archie-zola"
minimum_version = "0.14.0"
license = "MIT"
demo = "https://archie-zola.netlify.app/"

[extra.author]
name = "Keith"
homepage = "https://github.com/XXXMrG"
+++        

# archie-zola

A clean, minimal Zola theme forked from [archie](https://github.com/athul/archie). Perfect for personal blogs and portfolios with dark/light mode support.

## Table of Contents

- Demo
- Features
- Installation
- Quick Start
- Configuration
- Content Management
- Customization
- Troubleshooting
- Contributing

## Demo

**Live Demo:** [https://archie-zola.netlify.app](https://archie-zola.netlify.app)

### Screenshots

| Light Mode | Dark Mode |
|------------|-----------|
| ![Light](https://archie-zola.netlify.app/screenshot/screenshot-light.png) | ![Dark](https://archie-zola.netlify.app/screenshot/screenshot-dark.png) |

## Features

- ✅ **Responsive Design** - Works on desktop, tablet, and mobile
- ✅ **Dark/Light Mode** - Auto-detection + manual toggle
- ✅ **Fast & Lightweight** - Minimal CSS and JavaScript
- ✅ **SEO Optimized** - Meta tags, Open Graph, structured data
- ✅ **Syntax Highlighting** - Code blocks with theme support
- ✅ **LaTeX Math** - KaTeX integration for mathematical expressions
- ✅ **Custom CSS/JS** - Easy theme customization
- ✅ **Google Analytics** - Built-in GA4 support
- ✅ **Tags & Pagination** - Organize and navigate content
- ✅ **Social Links** - Footer social media integration

### Coming Soon
- 🔄 **Twitter Cards** - Rich social media previews
- 🔄 **YouTube Embeds** - Video integration

## Installation

### Method 1: Git Submodule (Recommended)

```bash
# Add as submodule
git submodule add https://github.com/XXXMrG/archie-zola.git themes/archie-zola

# Enable in config.toml
echo 'theme = "archie-zola"' >> config.toml
```

### Method 2: Direct Clone

```bash
# Clone to themes directory
git clone https://github.com/XXXMrG/archie-zola.git themes/archie-zola

# Enable in config.toml
echo 'theme = "archie-zola"' >> config.toml
```

### Updating the Theme

```bash
# Initialize submodules (first time)
git submodule update --init

# Update to latest version
git submodule update --remote

# Commit the update
git add themes/archie-zola
git commit -m "Update archie-zola theme"
```

## Quick Start

1. **Install the theme** (see above)
2. **Copy example config** to your `config.toml`:

```toml
base_url = "https://yourdomain.com"
title = "Your Blog"
description = "Your blog description"
theme = "archie-zola"

[extra]
mode = "toggle"  # auto | dark | toggle
subtitle = "Your tagline here"
```

3. **Create your first post**:

```bash
mkdir -p content/posts
echo '+++
title = "Hello World"
date = 2024-01-01
+++

Your first post content here.' > content/posts/hello-world.md
```

4. **Build and serve**:

```bash
zola serve
```

## Configuration

### Basic Settings

```toml
base_url = "https://yourdomain.com"
title = "Your Site Title"
description = "Your site description"
theme = "archie-zola"

[extra]
# Theme mode: "auto" | "dark" | "toggle"
mode = "toggle"

# Subtitle shown under title on homepage
subtitle = "Your tagline or description"

# Use CDN for fonts and icons (default: false)
useCDN = false

# Favicon path
favicon = "/icon/favicon.png"

# Footer copyright text
copyright = "Your Name"

# Google Analytics ID (optional)
ga = "G-XXXXXXXXXX"
```

### Dark Mode Options

- **`auto`** - Automatically follows system preference
- **`dark`** - Always dark mode
- **`toggle`** - Shows a toggle button for user choice

### Navigation Menu

```toml
[[extra.translations.en]]
show_more = "Read more ⟶"
previous_page = "← Previous"
next_page = "Next →"
posted_on = "on "
posted_by = "Published by"
read_time = "minute read"
all_tags = "All tags"
menus = [
    { name = "Home", url = "/", weight = 1 },
    { name = "Posts", url = "/posts", weight = 2 },
    { name = "Tags", url = "/tags", weight = 3 },
    { name = "About", url = "/about", weight = 4 },
]
```

### Social Links

```toml
[[extra.social]]
icon = "github"
name = "GitHub"
url = "https://github.com/yourusername"

[[extra.social]]
icon = "twitter"
name = "Twitter"
url = "https://twitter.com/yourusername"

[[extra.social]]
icon = "linkedin"
name = "LinkedIn"
url = "https://linkedin.com/in/yourusername"
```

### Custom Meta Tags

Add custom meta tags to individual pages:

```toml
title = "Your Post Title"
description = "Post description"
date = "2024-01-01"

[extra]
meta = [
    {property = "og:title", content = "Custom OG Title"},
    {property = "og:description", content = "Custom OG Description"},
    {property = "og:image", content = "https://example.com/image.jpg"},
]
```

If not specified, the theme will automatically generate Open Graph tags from your page title and description.

## Content Management

### Posts and Pagination

Create posts in `content/posts/` directory. Configure pagination in `content/posts/_index.md`:

```toml
+++
paginate_by = 5
sort_by = "date"
template = "posts.html"
+++
```

### Tags

Add tags to any post:

```toml
+++
title = "Your Post"
date = "2024-01-01"

[taxonomies]
tags = ["tech", "programming", "rust"]
+++
```

Enable tags in your main `config.toml`:

```toml
taxonomies = [
    { name = "tags" }
]
```

### Author Information

Add author info to posts:

```toml
[extra]
author = { name = "Your Name", social = "https://github.com/yourusername" }
```

### TL;DR Summary

Add a quick summary to posts:

```toml
[extra]
tldr = "Quick summary of what this post is about"
```

## Customization

### Custom CSS & JavaScript

Add custom styling and functionality:

```toml
[extra]
# Custom CSS files (placed in static/css/)
custom_css = [
    "css/custom.css",
    "css/my-theme.css"
]

# Custom JS files (placed in static/js/)
custom_js = [
    "js/custom.js",
    "js/analytics.js"
]
```

**File structure:**
```
static/
├── css/
│   ├── custom.css
│   └── my-theme.css
└── js/
    ├── custom.js
    └── analytics.js
```

The files will be automatically included in the `<head>` section of all pages.

### LaTeX Math Support

Enable mathematical expressions with KaTeX:

```toml
[extra]
katex_enable = true
```

Then use in your content:

```markdown
Inline math: \\( E = mc^2 \\)

Block math:
$$
\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}
$$
```

### Extending the Theme

Follow Zola's [theme extension guide](https://www.getzola.org/documentation/themes/extending-a-theme/) to override templates and add custom functionality.

## Troubleshooting

### Common Issues

**Theme not loading**
- Ensure `theme = "archie-zola"` is in your `config.toml`
- Check that the theme is in `themes/archie-zola/`

**Submodule not updating**
```bash
git submodule update --init --recursive
```

**Build errors**
- Check your `config.toml` syntax
- Ensure all required fields are present
- Verify custom CSS/JS files exist

**Dark mode not working**
- Set `mode = "toggle"` or `"auto"` in `[extra]` section
- Clear browser cache

### Performance Tips

- Set `useCDN = true` for faster font loading
- Optimize images in `static/` directory
- Use `zola build --drafts` during development

## Contributing

We welcome contributions! Here's how you can help:

1. **Report bugs** - Open an issue with reproduction steps
2. **Suggest features** - Describe your use case
3. **Submit PRs** - Follow the existing code style
4. **Improve docs** - Help make this README better

### Development Setup

```bash
# Fork and clone the repo
git clone https://github.com/yourusername/archie-zola.git
cd archie-zola

# Create a test site
zola init test-site
cd test-site
echo 'theme = "archie-zola"' >> config.toml

# Link your local theme
ln -s ../archie-zola themes/

# Start developing
zola serve
```

## License

This theme is released under the MIT License. See [LICENSE](LICENSE) for details.

## Credits

- Original [Archie theme](https://github.com/athul/archie) by @athul
- Ported to Zola by @XXXMrG
- Icons by [Feather Icons](https://feathericons.com/)

---

⭐ **Star this repo** if you find it useful!
        