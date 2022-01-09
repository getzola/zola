use std::collections::HashMap;

use config::Config;
use front_matter::InsertAnchor;
use templates::ZOLA_TERA;
use rendering::{render_content, RenderContext};

#[test]
fn can_render_shortcode_in_codeblock() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default_for_test();
    let mut context = RenderContext::new(
        &ZOLA_TERA,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let shortcode_def = utils::templates::get_shortcodes(&ZOLA_TERA);
    context.set_shortcode_definitions(&shortcode_def);
    // simple case
    let res = render_content(
        r#"
```
{{ youtube(id="dQw4w9WgXcQ") }}
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<pre><code>&lt;div &gt;\n    &lt;iframe src=&quot;https:&#x2F;&#x2F;www.youtube-nocookie.com&#x2F;embed&#x2F;dQw4w9WgXcQ&quot; webkitallowfullscreen mozallowfullscreen allowfullscreen&gt;&lt;&#x2F;iframe&gt;\n&lt;&#x2F;div&gt;\n\n</code></pre>\n"
    );
    // mixed with other contents
    let res = render_content(
        r#"
```
<div id="custom-attr">
{{ youtube(id="dQw4w9WgXcQ") }}
</div>
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<pre><code>&lt;div id=&quot;custom-attr&quot;&gt;\n&lt;div &gt;\n    &lt;iframe src=&quot;https:&#x2F;&#x2F;www.youtube-nocookie.com&#x2F;embed&#x2F;dQw4w9WgXcQ&quot; webkitallowfullscreen mozallowfullscreen allowfullscreen&gt;&lt;&#x2F;iframe&gt;\n&lt;&#x2F;div&gt;\n\n&lt;&#x2F;div&gt;\n</code></pre>\n"
    );
    // mixed content with syntax and line numbers
    let res = render_content(
        r#"
```html,linenos
<div id="custom-attr">
{{ youtube(id="dQw4w9WgXcQ") }}
</div>
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<pre data-linenos data-lang=\"html\" class=\"language-html \"><code class=\"language-html\" data-lang=\"html\"><table><tbody><tr><td>1</td><td>&lt;div id=&quot;custom-attr&quot;&gt;\n<tr><td>2</td><td>&lt;div &gt;\n<tr><td>3</td><td>    &lt;iframe src=&quot;https:&#x2F;&#x2F;www.youtube-nocookie.com&#x2F;embed&#x2F;dQw4w9WgXcQ&quot; webkitallowfullscreen mozallowfullscreen allowfullscreen&gt;&lt;&#x2F;iframe&gt;\n<tr><td>4</td><td>&lt;&#x2F;div&gt;\n<tr><td>5</td><td>\n<tr><td>6</td><td>&lt;&#x2F;div&gt;\n</tr></tbody></table></code></pre>\n"
    );
}

#[test]
fn can_render_multiple_shortcodes_in_codeblock() {
    let permalinks_ctx = HashMap::new();
    let config = Config::default_for_test();
    let mut context = RenderContext::new(
        &ZOLA_TERA,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let shortcode_def = utils::templates::get_shortcodes(&ZOLA_TERA);
    context.set_shortcode_definitions(&shortcode_def);
    // simple case
    let res = render_content(
        r#"
```
{{ youtube(id="dQw4w9WgXcQ") }}
{{ gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57", class="gist") }}
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<pre><code>&lt;div &gt;\n    &lt;iframe src=&quot;https:&#x2F;&#x2F;www.youtube-nocookie.com&#x2F;embed&#x2F;dQw4w9WgXcQ&quot; webkitallowfullscreen mozallowfullscreen allowfullscreen&gt;&lt;&#x2F;iframe&gt;\n&lt;&#x2F;div&gt;\n\n&lt;div class=&quot;gist&quot;&gt;\n    &lt;script src=&quot;https:&amp;#x2F;&amp;#x2F;gist.github.com&amp;#x2F;Keats&amp;#x2F;e5fb6aad409f28721c0ba14161644c57.js&quot;&gt;&lt;&#x2F;script&gt;\n&lt;&#x2F;div&gt;\n\n</code></pre>\n"
    );
    // mixed with other contents
    let res = render_content(
        r#"
```
text 1
{{ youtube(id="dQw4w9WgXcQ") }}
text 2
{{ gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57", class="gist") }}
text 3
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<pre><code>text 1\n&lt;div &gt;\n    &lt;iframe src=&quot;https:&#x2F;&#x2F;www.youtube-nocookie.com&#x2F;embed&#x2F;dQw4w9WgXcQ&quot; webkitallowfullscreen mozallowfullscreen allowfullscreen&gt;&lt;&#x2F;iframe&gt;\n&lt;&#x2F;div&gt;\n\ntext 2\n&lt;div class=&quot;gist&quot;&gt;\n    &lt;script src=&quot;https:&amp;#x2F;&amp;#x2F;gist.github.com&amp;#x2F;Keats&amp;#x2F;e5fb6aad409f28721c0ba14161644c57.js&quot;&gt;&lt;&#x2F;script&gt;\n&lt;&#x2F;div&gt;\n\ntext 3\n</code></pre>\n"
    );
    // mixed content with syntax and line numbers
    let res = render_content(
        r#"
```html,linenos
<span>text 1</span>
{{ youtube(id="dQw4w9WgXcQ") }}
<span>text 2</span>
{{ gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57", class="gist") }}
<span>text 3</span>
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
r#"<pre data-linenos data-lang="html" class="language-html "><code class="language-html" data-lang="html"><table><tbody><tr><td>1</td><td>&lt;span&gt;text 1&lt;&#x2F;span&gt;
<tr><td>2</td><td>&lt;div &gt;
<tr><td>3</td><td>    &lt;iframe src=&quot;https:&#x2F;&#x2F;www.youtube-nocookie.com&#x2F;embed&#x2F;dQw4w9WgXcQ&quot; webkitallowfullscreen mozallowfullscreen allowfullscreen&gt;&lt;&#x2F;iframe&gt;
<tr><td>4</td><td>&lt;&#x2F;div&gt;
<tr><td>5</td><td>
<tr><td>6</td><td>&lt;span&gt;text 2&lt;&#x2F;span&gt;
<tr><td>7</td><td>&lt;div class=&quot;gist&quot;&gt;
<tr><td>8</td><td>    &lt;script src=&quot;https:&amp;#x2F;&amp;#x2F;gist.github.com&amp;#x2F;Keats&amp;#x2F;e5fb6aad409f28721c0ba14161644c57.js&quot;&gt;&lt;&#x2F;script&gt;
<tr><td>9</td><td>&lt;&#x2F;div&gt;
<tr><td>10</td><td>
<tr><td>11</td><td>&lt;span&gt;text 3&lt;&#x2F;span&gt;
</tr></tbody></table></code></pre>
"#
    );
}

#[test]
fn is_highlighting_linenos_still_working() {
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let mut context = RenderContext::new(
        &ZOLA_TERA,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let shortcode_def = utils::templates::get_shortcodes(&ZOLA_TERA);
    context.set_shortcode_definitions(&shortcode_def);
    // single shortcode mixed with syntax and line numbers
    let res = render_content(
        r#"
```html,linenos
<div id="custom-attr">
{{ youtube(id="dQw4w9WgXcQ") }}
</div>
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
        "<pre data-linenos data-lang=\"html\" style=\"background-color:#2b303b;color:#c0c5ce;\" class=\"language-html \"><code class=\"language-html\" data-lang=\"html\"><table><tbody><tr><td>1</td><td><span>&lt;</span><span style=\"color:#bf616a;\">div </span><span style=\"color:#8fa1b3;\">id</span><span>=&quot;</span><span style=\"color:#a3be8c;\">custom-attr</span><span>&quot;&gt;\n</span><tr><td>2</td><td><span>&lt;</span><span style=\"color:#bf616a;\">div </span><span>&gt;\n</span><tr><td>3</td><td><span>    &lt;</span><span style=\"color:#bf616a;\">iframe </span><span style=\"color:#d08770;\">src</span><span>=&quot;</span><span style=\"color:#a3be8c;\">https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ</span><span>&quot; </span><span style=\"color:#d08770;\">webkitallowfullscreen mozallowfullscreen allowfullscreen</span><span>&gt;&lt;/</span><span style=\"color:#bf616a;\">iframe</span><span>&gt;\n</span><tr><td>4</td><td><span>&lt;/</span><span style=\"color:#bf616a;\">div</span><span>&gt;\n</span><tr><td>5</td><td><span>\n</span><tr><td>6</td><td><span>&lt;/</span><span style=\"color:#bf616a;\">div</span><span>&gt;\n</span></tr></tbody></table></code></pre>\n"
    );
    // multiple shortcode mixed with syntax and line numbers
    let res = render_content(
        r#"
```html,linenos
<span>text 1</span>
{{ youtube(id="dQw4w9WgXcQ") }}
<span>text 2</span>
{{ gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57", class="gist") }}
<span>text 3</span>
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
r#"<pre data-linenos data-lang="html" style="background-color:#2b303b;color:#c0c5ce;" class="language-html "><code class="language-html" data-lang="html"><table><tbody><tr><td>1</td><td><span>&lt;</span><span style="color:#bf616a;">span</span><span>&gt;text 1&lt;/</span><span style="color:#bf616a;">span</span><span>&gt;
</span><tr><td>2</td><td><span>&lt;</span><span style="color:#bf616a;">div </span><span>&gt;
</span><tr><td>3</td><td><span>    &lt;</span><span style="color:#bf616a;">iframe </span><span style="color:#d08770;">src</span><span>=&quot;</span><span style="color:#a3be8c;">https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ</span><span>&quot; </span><span style="color:#d08770;">webkitallowfullscreen mozallowfullscreen allowfullscreen</span><span>&gt;&lt;/</span><span style="color:#bf616a;">iframe</span><span>&gt;
</span><tr><td>4</td><td><span>&lt;/</span><span style="color:#bf616a;">div</span><span>&gt;
</span><tr><td>5</td><td><span>
</span><tr><td>6</td><td><span>&lt;</span><span style="color:#bf616a;">span</span><span>&gt;text 2&lt;/</span><span style="color:#bf616a;">span</span><span>&gt;
</span><tr><td>7</td><td><span>&lt;</span><span style="color:#bf616a;">div </span><span style="color:#d08770;">class</span><span>=&quot;</span><span style="color:#a3be8c;">gist</span><span>&quot;&gt;
</span><tr><td>8</td><td><span>    &lt;</span><span style="color:#bf616a;">script </span><span style="color:#d08770;">src</span><span>=&quot;</span><span style="color:#a3be8c;">https:</span><span style="color:#8fa1b3;">&amp;#x</span><span style="color:#d08770;">2F;</span><span style="color:#8fa1b3;">&amp;#x</span><span style="color:#d08770;">2F;</span><span style="color:#a3be8c;">gist.github.com</span><span style="color:#8fa1b3;">&amp;#x</span><span style="color:#d08770;">2F;</span><span style="color:#a3be8c;">Keats</span><span style="color:#8fa1b3;">&amp;#x</span><span style="color:#d08770;">2F;</span><span style="color:#a3be8c;">e5fb6aad409f28721c0ba14161644c57.js</span><span>&quot;&gt;&lt;/</span><span style="color:#bf616a;">script</span><span>&gt;
</span><tr><td>9</td><td><span>&lt;/</span><span style="color:#bf616a;">div</span><span>&gt;
</span><tr><td>10</td><td><span>
</span><tr><td>11</td><td><span>&lt;</span><span style="color:#bf616a;">span</span><span>&gt;text 3&lt;/</span><span style="color:#bf616a;">span</span><span>&gt;
</span></tr></tbody></table></code></pre>
"#
    );
}

#[test]
fn codeblock_shortcode_mix_all_stars() {
    let permalinks_ctx = HashMap::new();
    let mut config = Config::default_for_test();
    config.markdown.highlight_code = true;
    let mut context = RenderContext::new(
        &ZOLA_TERA,
        &config,
        &config.default_language,
        "",
        &permalinks_ctx,
        InsertAnchor::None,
    );
    let shortcode_def = utils::templates::get_shortcodes(&ZOLA_TERA);
    context.set_shortcode_definitions(&shortcode_def);
    // single shortcode mixed with syntax and line numbers
    let res = render_content(
        r#"
```html,linenos
<a href="javascript:void(0);">{{/* before(texts="1") */}}</a>
Normally people would not write something & like <> this：
<div id="custom-attr">
An inline {{ youtube(id="dQw4w9WgXcQ", autoplay=true, class="youtube") }} shortcode
</div>
Plain text in-between
{%/* quote(author="Vincent") */%}
A quote
{%/* end */%}
{{ gist(url="https://gist.github.com/Keats/e5fb6aad409f28721c0ba14161644c57", class="gist") }}
{# A Tera comment, you should see it #}
<!-- end text goes here -->
```
    "#,
        &context,
    )
    .unwrap();
    assert_eq!(
        res.body,
r#"<pre data-linenos data-lang="html" style="background-color:#2b303b;color:#c0c5ce;" class="language-html "><code class="language-html" data-lang="html"><table><tbody><tr><td>1</td><td><span>&lt;</span><span style="color:#bf616a;">a </span><span style="color:#d08770;">href</span><span>=&quot;</span><span style="color:#a3be8c;">javascript:void(0);</span><span>&quot;&gt;{{ before(texts=&quot;1&quot;) }}&lt;/</span><span style="color:#bf616a;">a</span><span>&gt;
</span><tr><td>2</td><td><span>Normally people would not write something &amp; like </span><span style="background-color:#bf616a;color:#2b303b;">&lt;&gt;</span><span> this：
</span><tr><td>3</td><td><span>&lt;</span><span style="color:#bf616a;">div </span><span style="color:#8fa1b3;">id</span><span>=&quot;</span><span style="color:#a3be8c;">custom-attr</span><span>&quot;&gt;
</span><tr><td>4</td><td><span>An inline &lt;</span><span style="color:#bf616a;">div </span><span style="color:#d08770;">class</span><span>=&quot;</span><span style="color:#a3be8c;">youtube</span><span>&quot;&gt;
</span><tr><td>5</td><td><span>    &lt;</span><span style="color:#bf616a;">iframe </span><span style="color:#d08770;">src</span><span>=&quot;</span><span style="color:#a3be8c;">https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ?autoplay=1</span><span>&quot; </span><span style="color:#d08770;">webkitallowfullscreen mozallowfullscreen allowfullscreen</span><span>&gt;&lt;/</span><span style="color:#bf616a;">iframe</span><span>&gt;
</span><tr><td>6</td><td><span>&lt;/</span><span style="color:#bf616a;">div</span><span>&gt;
</span><tr><td>7</td><td><span> shortcode
</span><tr><td>8</td><td><span>&lt;/</span><span style="color:#bf616a;">div</span><span>&gt;
</span><tr><td>9</td><td><span>Plain text in-between
</span><tr><td>10</td><td><span>{% quote(author=&quot;Vincent&quot;) %}
</span><tr><td>11</td><td><span>A quote
</span><tr><td>12</td><td><span>{% end %}
</span><tr><td>13</td><td><span>&lt;</span><span style="color:#bf616a;">div </span><span style="color:#d08770;">class</span><span>=&quot;</span><span style="color:#a3be8c;">gist</span><span>&quot;&gt;
</span><tr><td>14</td><td><span>    &lt;</span><span style="color:#bf616a;">script </span><span style="color:#d08770;">src</span><span>=&quot;</span><span style="color:#a3be8c;">https:</span><span style="color:#8fa1b3;">&amp;#x</span><span style="color:#d08770;">2F;</span><span style="color:#8fa1b3;">&amp;#x</span><span style="color:#d08770;">2F;</span><span style="color:#a3be8c;">gist.github.com</span><span style="color:#8fa1b3;">&amp;#x</span><span style="color:#d08770;">2F;</span><span style="color:#a3be8c;">Keats</span><span style="color:#8fa1b3;">&amp;#x</span><span style="color:#d08770;">2F;</span><span style="color:#a3be8c;">e5fb6aad409f28721c0ba14161644c57.js</span><span>&quot;&gt;&lt;/</span><span style="color:#bf616a;">script</span><span>&gt;
</span><tr><td>15</td><td><span>&lt;/</span><span style="color:#bf616a;">div</span><span>&gt;
</span><tr><td>16</td><td><span>
</span><tr><td>17</td><td><span>{# A Tera comment, you should see it #}
</span><tr><td>18</td><td><span style="color:#65737e;">&lt;!-- end text goes here --&gt;
</span></tr></tbody></table></code></pre>
"#
    );
}