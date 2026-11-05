- Feature Name: math-render
- Start Date: 2026-11-04
- RFC PR: [zola/rfcs#003](https://github.com/getzola/zola/pulls/3324)

## Summary

[summary]: #summary

Add (opt-in) math support to Zola so that posters that use math
in their Markdown (`$ … $`) are able to do so natively.

## Motivation

[motivation]: #motivation

Currently, Zola does not support math rendering natively.
The workaround currently is to have rendering done by MathJax or KaTeX
on the client side via JavaScript libraries.

This can work alright for light uses of math.
For an example of my own blog,
[this post of mine](https://github.com/vEnhance/wall.evanchen.cc/blob/main/content/000077.md?plain=1)
renders [fine via KaTeX](https://wall.evanchen.cc/77/).

However, there are a couple of disadvantages.

(1) Certain expressions can get mangled due to `pulldown-cmark`,
due to its interpretation of the escaping of backslashes.
Here are two examples.

(1a) The notations `$\{1,2,\dots,n\}$`.
The `pulldown-cmark` would kill the backslashes in front of the curly braces,
resulting in `${1,2,\dots,n}$` instead;
this causes MathJax/KaTeX to not render the curly set braces.

(1b) The $2 \times 2$ matrix

```latex
$$
\begin{matrix}
1 & 0 \\
0 & 1
\end{matrix}
$$
```

would not render correctly,
because the double-backslash gets mangled into a single backslash.
Working around this would require typing a quadruple-backslash right now,
which is an abomination.

(2) This pushes the burden of rendering the math onto the client;
for example, they would need to have JavaScript enabled.
Since Zola is for mostly static sites,
it makes sense we might want Zola to handle the compilation at compile time (say, as an SVG).
Then the author can be assured the math will render correctly and uniformly
for viewers of the website.

This feature is opt-in because the dollar sign can be used in non-math contexts
too and we wouldn't want to mis-render those.
By enabling this feature, the user is sort of committing to
escaping dollar signs (`\$`) when they are used in non-math contexts.
I don't consider this is a downside for a voluntary feature;
people who write math in Markdown are often used to this anyway.

## Guide-level explanation

This is mostly copied from [cestef's proposal](https://github.com/cestef/zola/blob/fe7d68329bc2300bdcd9b91609deec5064dbe2de/docs/content/documentation/content/math.md?plain=1)
with two changes:

- I added a fourth rendering option, `"browser"`, for browser-only rendering.
  This is [#2708](https://github.com/getzola/zola/pull/2708):
  all we do is turn on `Options::ENABLE_MATH`
  and otherwise leave everything else exactly the same.
  Responsibility for rendering is then at the template level.
- I took out the features that Keats suggested not including in the review,
  namely cache directory customization and svgo optimization.

Following [#2791](https://github.com/getzola/zola/pull/2791),
we introduce a new section called `markdown.math` into `zola.toml`.
Zola supports math expression rendering in markdown content using either KaTeX or Typst.
Math rendering can be enabled and configured in your `config.toml`:

```toml
[markdown.math]
# Enable math rendering - options: "none" (default), "browser", "katex", "typst"
engine = "typst"

# Optional: Custom CSS file for styling math output (Typst only)
css = "path/to/math.css"

# Optional: Additional code included for each snippet (Typst only)
addon = "path/to/addon.typ"
```

### Writing Math Expressions

For inline math expressions, surround your Typst/LaTeX formula with single dollar signs:

```md
Einstein's famous equation: $E = m c^2$
```

For block/display math expressions, use double dollar signs:

```md
$$
lim_(x->oo) 1/x = 0
$$
```

### Rendering Options

#### [Typst](https://typst.app) crate

Typst is the default and recommended rendering engine that offers:

- Fast rendering performance
- High-quality vector output as SVGs
- Modern typographic features

Math expressions are automatically cached to improve rendering performance.

#### [KaTeX](https://katex.org) crate

KaTeX is an alternative rendering engine providing:

- Traditional LaTeX syntax support
- Wide symbol coverage

Rendered expressions are cached.

This still requires adding the `katex.css` file
into the HTML template (but no Javascript), e.g.

```html
<link rel="stylesheet"
      href="https://cdn.jsdelivr.net/npm/katex/dist/katex.min.css"
      crossorigin="anonymous">
```

#### Browser-only

Do not do any rendering at compilation time.
All we do is pass in `Options::ENABLE_MATH` to `pulldown-cmark`
so that any math is preserved in
`<span class="math math-inline"></span>`
`<span class="math math-display"></span>`.

Example code for template to do this rendering
(from [#2708](https://github.com/getzola/zola/pull/2708)):

```html
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css"
        integrity="sha384-nB0miv6/jRmo5UMMR1wu3Gz6NLsoTkbqJghGIsx//Rlm+ZU03BU6SQNC66uf4l5+"
        crossorigin="anonymous">
    <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js"
        integrity="sha384-7zkQWkzuo3B5mTepMUcHkMB5jZaolc2xDwL6VFqjFALcbeS9Ggm/Yr2r3Dy4lfFg"
        crossorigin="anonymous"></script>
    <script defer>
        document.addEventListener("DOMContentLoaded", function () {
            const macros = {};
            for (let el of document.getElementsByClassName('math')) {
              katex.render(el.textContent, el, {
                output: 'html',
                throwOnError: false,
                displayMode: el.classList.contains('math-display'),
                macros,
              });
            }
        });
    </script>
```

### Typst-only features

#### Styling

By default, math expressions are rendered with a minimal style.
You can customize the appearance of math expressions by providing a custom CSS file,
which will be injected directly into the rendered SVG:

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

#### Addons

Added addons for math which allow users to add code snippets to all compiled expressions.
This is useful when you need to provide helper functions,
import default packages or override defaults.

## Reference-level explanation

[reference-level-explanation]: #reference-level-explanation

[#2708](https://github.com/getzola/zola/pull/2708)
and [#2791](https://github.com/getzola/zola/pull/2791)
would together be a reasonably complete reference implementation.

In every rendering case (other than `none`),
the first step is to add `Options::ENABLE_MATH`
to make sure the Markdown rendering preserves the math elements as-is.

After that, the flow depends on which rendering system is used.

- Browser-only: nothing left to do
- KaTeX crate: renders LaTeX expressions
- Typst crate: most complicated, but most powerful
  (see [cappuccinocosmico's screenshots](https://github.com/getzola/zola/pull/2791#issuecomment-3014814405))

## Drawbacks

[drawbacks]: #drawbacks

- Typst would increase the size of the binary significantly.
- Also, maintaining multiple parallel pipelines seems like a lot of
  maintenance burden.
- LaTeX and Typst syntax are not mutually compatible,
  although they look superficially similar.
  This could easily lead to some user confusion.
  There is already an example above where we are using `$$a^2+b^2+c^2$$`
  for Typst display math, where in fact in real Typst documents,
  Typst display math is done by `$ a^2+b^2=c^2 $` instead.

## Rationale and alternatives

[rationale-and-alternatives]: #rationale-and-alternatives

I expect most of the discussion of the RFC to
be around exactly which subset of `{browser, katex, typst}`
we want to support as pipelines, either at first or over time.
There are good arguments for only doing some of them.

- AFAICT almost everyone agrees the `browser` option is a fairly cheap path
  ([#2708](https://github.com/getzola/zola/pull/2708) is only 100-ish lines of code)
  that would already make a lot of users quite happy.
  Basically, a lot of math folks would just want to be able to pass
  `Options::ENABLE_MATH` to `pulldown-cmark` as a starting point,
  and we can't do that right now.
  Giving us a switch to do that would already be a huge relief,
  even if the server-side compilation has to wait til someday/never.
  (Though as Keats said, we'd want this to be an
  [enum and not a boolean](https://github.com/getzola/zola/pull/2708#discussion_r1847350890),
  to keep that door open.)

- `katex` support would be cool for the people who
  either have existing work in LaTeX (Typst is very new)
  or don't want to learn Typst.
  It would be more effort than `browser`, but not as much as `typst`.
  However, the benefit is proportionally smaller,
  because some users wouldn't care about the difference
  between Zola rendering LaTeX vs browsers rendering LaTeX,
  especially for occasional usage.

- `typst` is really powerful and already in Rust
  (again, see [cappuccinocosmico's screenshots](https://github.com/getzola/zola/pull/2791#issuecomment-3014814405)).
  It's probably the most work to implement, correspondingly.

## Prior art

[prior-art]: #prior-art

- [#2708](https://github.com/getzola/zola/pull/2708) is implementing what I call `browser` above.
- The [Discourse thread](https://zola.discourse.group/t/add-opt-in-tex-math-parsing-for-markdown/2964/4)
  provides an RFC but only for the scope of 2708.
- [#2791](https://github.com/getzola/zola/pull/2791)
  includes `katex` and `typst`.

## Unresolved questions

[unresolved-questions]: #unresolved-questions

- Would it be prudent to get the `browser` option out the door quickly first,
  since the other two renderers are a lot more scope?
  - Also is there a better name than `browser`?
    I thought about `client` too, don't love either.
- Are there enough people who want `katex` crate
  to be worth the maintenance cost?
  It's possible that LaTeX users think the `browser` option would be good enough.
- Are there enough people who want `typst` crate
  to be worth the maintenance cost?
  There was already lively discussion in
  [#2791](https://github.com/getzola/zola/pull/2791)
  on whether Typst is actually worth supporting this way or not.

## Future possibilities

[future-possibilities]: #future-possibilities

Some Typst fans might actually want to replace Markdown with Typst altogether.
That strikes me as way out of scope, but who knows.
Anyway, it won't happen anyway until
[Typst's HTML conversion stabilizes](https://typst.app/docs/reference/html/).

Also, one thing that it occurs to me as I was reading through the Discourse
was that there are other cases where some `pulldown-cmark` option
would have been nice but wasn't exposed through Zola,
such as [rendering hard breaks](https://zola.discourse.group/t/from-quartz-to-zola-bypassing-hugo/2969/2).
So I wonder whether maybe there's some way we can just more systematically
allow passing options directly to `pulldown-cmark` in `zola.toml`,
rather than handling these cases one at a time.
