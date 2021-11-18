mod common;

use common::ShortCode;

const COMPLETE_PAGE: &str = r#"
<!-- Adapted from https://markdown-it.github.io/ -->

# h1 Heading

## h2 Heading

### h3 Heading

#### h4 Heading

##### h5 Heading

###### h6 Heading

## Horizontal Rules

___

---

***

## Emphasis

**This is bold text**

__This is bold text__

*This is italic text*

_This is italic text_

~~Strikethrough~~


## Blockquotes


> Blockquotes can also be nested...
>> ...by using additional greater-than signs right next to each other...
> > > ...or with spaces between arrows.


## Lists

Unordered

+ Create a list by starting a line with `+`, `-`, or `*`
+ Sub-lists are made by indenting 2 spaces:
  - Marker character change forces new list start:
    * Ac tristique libero volutpat at
    + Facilisis in pretium nisl aliquet
    - Nulla volutpat aliquam velit
+ Very easy!

Ordered

1. Lorem ipsum dolor sit amet
2. Consectetur adipiscing elit
3. Integer molestie lorem at massa


1. You can use sequential numbers...
1. ...or keep all the numbers as `1.`

Start numbering with offset:

57. foo
1. bar


## Code

Inline `code`

Indented code

    // Some comments
    line 1 of code
    line 2 of code
    line 3 of code


Block code "fences"

```
Sample text here...
```

Syntax highlighting

``` js
var foo = function (bar) {
  return bar++;
};

console.log(foo(5));
```

## Shortcodes

{% quote(author="John Doe") %}
This is a test quote!
1900-01-01
{% end %}

## Tables

| Option | Description |
| ------ | ----------- |
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |

Right aligned columns

| Option | Description |
| ------:| -----------:|
| data   | path to data files to supply the data that will be passed into templates. |
| engine | engine to be used for processing templates. Handlebars is the default. |
| ext    | extension to be used for dest files. |


## Links

[link text](http://duckduckgo.com)

[link with title](http://duckduckgo.com/)

## Images

![Minion](https://octodex.github.com/images/minion.png)
![Stormtroopocat](https://octodex.github.com/images/stormtroopocat.jpg "The Stormtroopocat")

Like links, Images also have a footnote style syntax

![Alt text][id]

With a reference later in the document defining the URL location:

[id]: https://octodex.github.com/images/dojocat.jpg  "The Dojocat"

### Footnotes

Footnote 1 link[^first].

Footnote 2 link[^second].

Duplicated footnote reference[^second].

[^first]: Footnote **can have markup**
and multiple paragraphs.

[^second]: Footnote text."#;

#[test]
fn complete_page() {
    let config = config::Config::default_for_test();

    let mut tera = tera::Tera::default();

    let shortcodes: Vec<ShortCode> = vec![ShortCode::new(
        "quote",
        r"<blockquote>
{{ body }} <br>
-- {{ author}}
</blockquote>",
        false,
    )];

    let mut permalinks = std::collections::HashMap::new();

    permalinks.insert("".to_string(), "".to_string());

    // Add all shortcodes
    for ShortCode { name, is_md, output } in shortcodes.into_iter() {
        tera.add_raw_template(
            &format!("shortcodes/{}.{}", name, if is_md { "md" } else { "html" }),
            &output,
        )
        .unwrap();
    }

    let mut context = rendering::RenderContext::new(
        &tera,
        &config,
        &config.default_language,
        "",
        &permalinks,
        front_matter::InsertAnchor::None,
    );
    let shortcode_def = utils::templates::get_shortcodes(&tera);
    context.set_shortcode_definitions(&shortcode_def);

    let rendered = rendering::render_content(COMPLETE_PAGE, &context);
    assert!(rendered.is_ok(), "Rendering failed");

    let rendered = rendered.unwrap();

    let asserted_internal_links: Vec<(String, Option<String>)> = vec![];
    let asserted_external_links: Vec<String> =
        vec!["http://duckduckgo.com".to_string(), "http://duckduckgo.com/".to_string()];

    assert_eq!(rendered.internal_links, asserted_internal_links, "Internal links unequal");
    assert_eq!(rendered.external_links, asserted_external_links, "External links unequal");

    assert_eq!(
        rendered.body,
        r##"<!-- Adapted from https://markdown-it.github.io/ -->
<h1 id="h1-heading">h1 Heading</h1>
<h2 id="h2-heading">h2 Heading</h2>
<h3 id="h3-heading">h3 Heading</h3>
<h4 id="h4-heading">h4 Heading</h4>
<h5 id="h5-heading">h5 Heading</h5>
<h6 id="h6-heading">h6 Heading</h6>
<h2 id="horizontal-rules">Horizontal Rules</h2>
<hr />
<hr />
<hr />
<h2 id="emphasis">Emphasis</h2>
<p><strong>This is bold text</strong></p>
<p><strong>This is bold text</strong></p>
<p><em>This is italic text</em></p>
<p><em>This is italic text</em></p>
<p><del>Strikethrough</del></p>
<h2 id="blockquotes">Blockquotes</h2>
<blockquote>
<p>Blockquotes can also be nested...</p>
<blockquote>
<p>...by using additional greater-than signs right next to each other...</p>
<blockquote>
<p>...or with spaces between arrows.</p>
</blockquote>
</blockquote>
</blockquote>
<h2 id="lists">Lists</h2>
<p>Unordered</p>
<ul>
<li>Create a list by starting a line with <code>+</code>, <code>-</code>, or <code>*</code></li>
<li>Sub-lists are made by indenting 2 spaces:
<ul>
<li>Marker character change forces new list start:
<ul>
<li>Ac tristique libero volutpat at</li>
</ul>
<ul>
<li>Facilisis in pretium nisl aliquet</li>
</ul>
<ul>
<li>Nulla volutpat aliquam velit</li>
</ul>
</li>
</ul>
</li>
<li>Very easy!</li>
</ul>
<p>Ordered</p>
<ol>
<li>
<p>Lorem ipsum dolor sit amet</p>
</li>
<li>
<p>Consectetur adipiscing elit</p>
</li>
<li>
<p>Integer molestie lorem at massa</p>
</li>
<li>
<p>You can use sequential numbers...</p>
</li>
<li>
<p>...or keep all the numbers as <code>1.</code></p>
</li>
</ol>
<p>Start numbering with offset:</p>
<ol start="57">
<li>foo</li>
<li>bar</li>
</ol>
<h2 id="code">Code</h2>
<p>Inline <code>code</code></p>
<p>Indented code</p>
<pre><code>&#x2F;&#x2F; Some comments
line 1 of code
line 2 of code
line 3 of code
</code></pre>
<p>Block code &quot;fences&quot;</p>
<pre><code>Sample text here...
</code></pre>
<p>Syntax highlighting</p>
<pre data-lang="js" class="language-js "><code class="language-js" data-lang="js">var foo = function (bar) {
  return bar++;
};

console.log(foo(5));
</code></pre>
<h2 id="shortcodes">Shortcodes</h2>
<blockquote>
This is a test quote!
1900-01-01 <br>
-- John Doe
</blockquote><h2 id="tables">Tables</h2>
<table><thead><tr><th>Option</th><th>Description</th></tr></thead><tbody>
<tr><td>data</td><td>path to data files to supply the data that will be passed into templates.</td></tr>
<tr><td>engine</td><td>engine to be used for processing templates. Handlebars is the default.</td></tr>
<tr><td>ext</td><td>extension to be used for dest files.</td></tr>
</tbody></table>
<p>Right aligned columns</p>
<table><thead><tr><th align="right">Option</th><th align="right">Description</th></tr></thead><tbody>
<tr><td align="right">data</td><td align="right">path to data files to supply the data that will be passed into templates.</td></tr>
<tr><td align="right">engine</td><td align="right">engine to be used for processing templates. Handlebars is the default.</td></tr>
<tr><td align="right">ext</td><td align="right">extension to be used for dest files.</td></tr>
</tbody></table>
<h2 id="links">Links</h2>
<p><a href="http://duckduckgo.com">link text</a></p>
<p><a href="http://duckduckgo.com/">link with title</a></p>
<h2 id="images">Images</h2>
<p><img src="https://octodex.github.com/images/minion.png" alt="Minion" />
<img src="https://octodex.github.com/images/stormtroopocat.jpg" alt="Stormtroopocat" title="The Stormtroopocat" /></p>
<p>Like links, Images also have a footnote style syntax</p>
<p><img src="https://octodex.github.com/images/dojocat.jpg" alt="Alt text" title="The Dojocat" /></p>
<p>With a reference later in the document defining the URL location:</p>
<h3 id="footnotes">Footnotes</h3>
<p>Footnote 1 link<sup class="footnote-reference"><a href="#first">1</a></sup>.</p>
<p>Footnote 2 link<sup class="footnote-reference"><a href="#second">2</a></sup>.</p>
<p>Duplicated footnote reference<sup class="footnote-reference"><a href="#second">2</a></sup>.</p>
<div class="footnote-definition" id="first"><sup class="footnote-definition-label">1</sup>
<p>Footnote <strong>can have markup</strong>
and multiple paragraphs.</p>
</div>
<div class="footnote-definition" id="second"><sup class="footnote-definition-label">2</sup>
<p>Footnote text.</p>
</div>
"##
    );
}
