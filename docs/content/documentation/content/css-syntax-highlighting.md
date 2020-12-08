+++
title = "CSS Syntax Highlighting"
weight = 82
+++

If you use a highlighting scheme like

```toml
highlight_theme = "base16-ocean-dark"
```

for a code block like

````md
```rs
let highlight = true;
```
````

you get the colors directly encoded in the html file.

```html
<pre style="background-color:#2b303b;">
    <code class="langauge-rs">
        <span style="color:#b48ead;">let</span>
        <span style="color:#c0c5ce;"> highlight = </span>
        <span style="color:#d08770;">true</span>
        <span style="color:#c0c5ce;">;
    </span>
  </code>
</pre>
```

This is nice, because your page will load faster if everything is in one file.
But if you would like to have the user choose a theme from a
list, or use different color schemes for dark/light color schemes, you need a
different solution.

If you use the special `css` color scheme

```toml
highlight_theme = "css"
```

you get CSS class definitions, instead.

```html
<pre class="code">
    <code class="language-rs">
        <span class="source rust">
            <span class="storage type rust">let</span> highlight
            <span class="keyword operator assignment rust">=</span>
            <span class="constant language rust">true</span>
            <span class="punctuation terminator rust">;</span>
        </span>
    </code>
</pre>
```

Zola can output a css file for a theme using the `highlighting_themes_css` option.

```toml
highlighting_themes_css = [
  { theme = "base16-ocean-dark", filename = "syntax-theme-dark.css" },
  { theme = "base16-ocean-light", filename = "syntax-theme-light.css" },
]
```

You can then support light and dark mode like so:

```css
@import url("syntax-theme-dark.css") (prefers-color-scheme: dark);
@import url("syntax-theme-light.css") (prefers-color-scheme: light);
```
