+++
title = "CSS Syntax Highlighting"
weight = 80
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

you get the colors directly encoded in the html file

```html
<pre style="background-color:#2b303b;">
    <code>
        <span style="color:#b48ead;">let</span>
        <span style="color:#c0c5ce;"> highlight = </span>
        <span style="color:#d08770;">true</span>
        <span style="color:#c0c5ce;">;
    </span>
  </code>
</pre>
```

this is nice, because if everything is inside one file, you get fast
page loadings. But if you would like to have the user choose a theme from a
list, or different color schemes for dark/light color schemes, you need a
different solution.

If you use the special color scheme

```toml
highlight_theme = "css"
```

you get CSS class definitions

```html
<pre class="code">
    <code class="language-html">
        <span class="source rust">
            <span class="storage type rust">let</span> highlight
            <span class="keyword operator assignment rust">=</span>
            <span class="constant language rust">true</span>
            <span class="punctuation terminator rust">;</span>
        </span>
    </code>
</pre>
```

now you can generate and use CSS either manually or with Zola

```toml
highlighting_themes_css = [
  { theme = "base16-ocean-dark", filename = "syntax-theme-dark.css" },
  { theme = "base16-ocean-light", filename = "syntax-theme-light.css" },
]
```

```css
@import url("syntax-theme-dark.css") (prefers-color-scheme: dark);
@import url("syntax-theme-light.css") (prefers-color-scheme: light);
```
