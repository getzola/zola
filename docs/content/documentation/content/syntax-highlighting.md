+++
title = "Syntax Highlighting"
weight = 80
+++

Zola comes with built-in syntax highlighting but you first
need to enable it in the [configuration](@/documentation/getting-started/configuration.md).

Once this is done, Zola will automatically highlight all code blocks
in your content. A code block in Markdown looks like the following:

````
```rust
let highlight = true;
```
````

You can replace `rust` with another language or not put anything to get the text
interpreted as plain text.

Zola uses [Giallo](https://github.com/getzola/giallo), a library that uses VSCode syntaxes and themes.

You can see a full list of supported languages and themes in the README: <https://github.com/getzola/giallo?tab=readme-ov-file#built-in>

If a theme or a language you want to highlight is not supported, you can find the JSON grammar files and JSON theme files, copy them somewhere in your site
and load them from the configurations with `extra_grammars` and `extra_themes` in the `[markdown.highlighting]` section.
A good source for themes is [textmate-grammars-themes](https://textmate-grammars-themes.netlify.app/).

In any cases, you will need to add the following CSS to your site CSS for things to display correctly:

```css
.giallo-l {
    display: inline-block;
    min-height: 1lh;
    width: 100%;
}
.giallo-ln {
    display: inline-block;
    user-select: none;
    margin-right: 0.4em;
    padding: 0.4em;
    min-width: 3ch;
    text-align: right;
    opacity: 0.8;
}
```

## Theme selection

You can choose to use a single theme or light/dark themes.

If you want a single theme, use the `theme` key in the `[markdown.highlighting]` section of the configuration.

If you want dual themes, use the `light_theme` and `dark_theme` keys:

```toml
light_theme = "github-light"                                                                                     
dark_theme = "github-dark"
```

## Rendering style

The default rendering style is `inline`, meaning the colours are set directly on the `<span>` elements with the hexadecimal values like
`<span style="color: #83A598;">base_url</span>` for a single theme and `<span style="color: light-dark(#076678, #83A598);">base_url</span>`
for dual themes. The [light-dark()](https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Values/color_value/light-dark) CSS functions
will automatically use the preferred color scheme of the user, with no way of overriding it from JS.

If you want a light/dark theme switch button or have rules against inline styles, you can set `markdown.highlighting.style` to `class` in order
to have the renderer use CSS classes instead and generate one CSS file for each theme in static folder following:

- single theme -> `giallo.css`
- dual themes: light -> `giallo-light.css`, dark -> `giallo-dark.css`

The HTML will look like `<span class="z-support z-type z-property-name z-css">  user-select</span>`.

You can then support light and dark mode like so:

```css
@import url("giallo-dark.css") (prefers-color-scheme: dark);
@import url("giallo-light.css") (prefers-color-scheme: light);
```

Alternately, you can reference the stylesheets in your base template to reduce request chains:

```html
<head>
  <!-- Other content -->
  <link id="giallo-dark" rel="stylesheet" type="text/css" href="/giallo-dark.css" media="(prefers-color-scheme: dark)" />
  <link id="giallo-light" rel="stylesheet" type="text/css" href="/giallo-light.css" media="(prefers-color-scheme: light)" />
</head>
```

## Annotations

You can use additional annotations to customize how code blocks are displayed:

- `linenos` to enable line numbering.

````
```rust,linenos
use highlighter::highlight;
let code = "...";
highlight(code);
```
````

- `linenostart` to specify the number for the first line (defaults to 1)
  
````
```rust,linenos,linenostart=20
use highlighter::highlight;
let code = "...";
highlight(code);
```
````

- `hl_lines` to highlight lines. You must specify a list of inclusive ranges of lines to highlight,
separated by ` ` (whitespace). Ranges are 1-indexed and `linenostart` doesn't influence the values, it always refers to the codeblock line number.
  
````
```rust,hl_lines=1 3-5 9
use highlighter::highlight;
let code = "...";
highlight(code);
```
````

- `hide_lines` to hide lines. You must specify a list of inclusive ranges of lines to hide,
separated by ` ` (whitespace). Ranges are 1-indexed.

````
```rust,hide_lines=1-2
use highlighter::highlight;
let code = "...";
highlight(code);
```
````

- `name` to specify a name the code block is associated with.
  
````
```rust,name=mod.rs
use highlighter::highlight;
let code = "...";
highlight(code);
```
````

Here's an example with all the options used: `scss, linenos, linenostart=10, hl_lines=3-4 8-9, hide_lines=2 7`.

```scss, linenos, linenostart=10, hl_lines=3-4 8-9, hide_lines=2 7
pre mark {
  // If you want your highlights to take the full width
  display: block;
  color: currentcolor;
}
pre table td:nth-of-type(1) {
  // Select a colour matching your theme
  color: #6b6b6b;
  font-style: italic;
}
```

Line 2 and 7 are comments that are not shown in the final output.
