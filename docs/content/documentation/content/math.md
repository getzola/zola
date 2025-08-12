+++
title = "Math Rendering"
weight = 140
+++

Zola supports math expression rendering in markdown content using either KaTeX or Typst. Math rendering can be enabled and configured in your `config.toml`:

```toml
[markdown.math]
# Enable math rendering - options: "none", "katex", "typst" 
engine = "typst" # or "katex"

# Optional: Enable SVGO optimization for generated math SVGs
svgo = false # true, false, or "config_path"

# Optional: Custom CSS file for styling math output
css = "path/to/math.css"

# Optional: Additional code included for each snippet
addon = "path/to/addon.typ"
```

## Writing Math Expressions

### Inline Math

For inline math expressions, surround your Typst/LaTeX formula with single dollar signs:

```md
Einstein's famous equation: $E = m c^2$
```

Einstein's famous equation: $E = m c^2$

### Block Math

For block/display math expressions, use double dollar signs:

```md
$$
lim_(x->oo) 1/x = 0
$$
```

$$
lim_(x->oo) 1/x = 0
$$

## Rendering Options

### [Typst](https://typst.app)

Typst is the default and recommended rendering engine that offers:

- Fast rendering performance
- High-quality vector output as SVGs
- Modern typographic features

### [KaTeX](https://katex.org)

KaTeX is an alternative rendering engine providing:

- Traditional LaTeX syntax support
- Wide symbol coverage

## Performance Optimization

Math expressions are automatically cached to improve rendering performance. By default, rendered expressions are stored in:

Windows: `%LOCALAPPDATA%/zola/`

Linux: `$XDG_CACHE_HOME/zola/` or `~/.cache/zola/`

macOS: `~/Library/Caches/zola/`

This cache directory can be customized using the `cache` option:


```toml
[markdown]
cache = "path/to/cache" # true, false, or "path/to/cache"
```

## SVG Optimization

Enable `svgo = true` to optimize the generated SVG files using [SVGO](https://svgo.dev). This can significantly reduce the file size of complex mathematical expressions.

You can customize the optimization by providing your own SVGO configuration file:

```toml
[markdown.math]
svgo = "svgo.config.mjs" # equivalent to true
```

```js
// svgo.config.mjs
export default {
    multipass: true,
    plugins: [
        "removeDoctype",
        "removeXMLProcInst",
        "removeComments",
        "removeMetadata",
        "removeEditorsNSData",
        "cleanupAttrs",
    ],
};
```

## Styling
By default, math expressions are rendered with a minimal style. You can customize the appearance of math expressions by providing a custom CSS file, which will be injected into the rendered SVG:

```toml
[markdown.math]
css = "static/math.css"
```

Example CSS for styling math expressions:

```css
:root {
    --t: #3c3836;
    --d: #efefeb;
}
@media (prefers-color-scheme: dark) {
    :root {
        --t: #ebdbb2;
        --d: #292724;
    }
}
.typst-text[fill="#000000"],
.typst-shape[fill="#000000"],
.typst-group[fill="#000000"],
.typst-text > [fill="#000000"],
.typst-shape > [fill="#000000"],
.typst-group > [fill="#000000"] {
    fill: var(--t);
}
.typst-text[fill="#ffffff"],
.typst-shape[fill="#ffffff"],
.typst-group[fill="#ffffff"],
.typst-text > [fill="#ffffff"],
.typst-shape > [fill="#ffffff"],
.typst-group > [fill="#ffffff"] {
    fill: var(--d);
}
```