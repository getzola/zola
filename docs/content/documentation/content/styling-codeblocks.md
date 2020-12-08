+++
title = "Styling Codeblocks"
weight = 81
+++
# Highlighted lines
The syntax highlighting uses `<mark>` tags to highlight lines.  For highlights across contiguous lines, a single `<mark>` tag is used.  While not currently well supported, the `<mark>` tag has the aria role of mark which will help convey the same meaning to screen readers as the color does to sighted users.

## Full width
For inline highlighting, the theme's line highlight background color is applied as an inline style. Since `<mark>` is an inline tag the default result looks like this:

<style>
.example-1 mark {
	display: inline !important;
	color: black;
}
.example-2 mark {
	display: block !important;
	color: black;
}
.example-3 mark {
	color: currentcolor;
}
.example-4 pre table td:nth-of-type(1) {
	color: #6b6b6b;
	font-style: italic;
}
.example-5 mark {
	display: block;
	color: currentcolor;
}
</style>

<div class="example-1">

```rust, hl_lines=3-5
fn main() {
	println!("Hello World");
	if youre_happy() && you_know_it() {
		println!("Clap your hands");
	}
}
```

</div>

If you'd rather that the highlight extend to the end of the line, then make `<mark>` tags within `<pre>` tags display block like so:

```css
pre mark {
	display: block;
}
```

This results in the following:

<div class="example-2">

```rust, hl_lines=3-5
fn main() {
	println!("Hello World");
	if youre_happy() && you_know_it() {
		println!("Clap your hands");
	}
}
```

</div>

## Foreground Color
There's currently no line_highlight_foreground color, unfortunately, and you may have noticed that by default the user-agent stylesheet probably made some of the code block black.  This is because the default style for a `<mark>` tag is a <mark>yellow background with black text</mark>.  This is easy to fix using CSS like this:

```css
pre mark {
	color: currentcolor;
}
```

Which now gives us:

<div class="example-3">

```rust, hl_lines=3-5
fn main() {
	println!("Hello World");
	if youre_happy() && you_know_it() {
		println!("Clap your hands");
	}
}
```

</div>

# Line Numbers
When line numbers are active, the code block is turned into a table with one row and two cells.  The first cell contains the numbers and the second cell contains the code.

```rust, linenos
fn main() {
	println!("Hello World");
	if youre_happy() && you_know_it() {
		println!("Clap your hands");
	}
}
```

## Styling Line numbers
By default, the line numbers will inherit the initial color and background from the pre tag.  Since the line numbers are in the first cell of the table, we could style them using `:nth-of-type` like this:

```css
pre table td:nth-of-type(1) {
	color: #6b6b6b;
	font-style: italic;
}
```

Which produces this:

<div class="example-4">

```rust, linenos
fn main() {
	println!("Hello World");
	if youre_happy() && you_know_it() {
		println!("Clap your hands");
	}
}
```

</div>

## Line numbers with highlighted lines
When a line is highlighted two `<mark>` tags are created: one around the line number(s) and one around the code.  Example:

<div class="example-5">

```rust, linenos, hl_lines=3-5
fn main() {
	println!("Hello World");
	if youre_happy() && you_know_it() {
		println!("Clap your hands");
	}
}
```

</div>

# Classed Highlighting
When classed highlighting is enabled (by setting `[markdown]` `highlight_theme = "css"`) you can style highlighted lines and line numbers the same way as above.  Zola can generate a CSS file for a theme using the `highlighting_themes_css` config option which you can learn from / modify.  [CSS Syntax Highlighting](@/documentation/content/css-syntax-highlighting.md) has information on why you might choose classed highlighting, and how it works.