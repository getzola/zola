+++
date = 2018-11-10
+++

Avec des fichiers

fr asset: {{ get_url(path="@/blog/with-assets/some.js") }}

![js](@/blog/with-assets/some.js)

<img src="some.js?v=1" alt="raw html with query">

<img srcset=some.js alt="unquoted">

<img srcset="data:image/png;base64,AAAA 1x, some.js 2x" alt="srcset with data url">

<pre>https://example.com/fr/blog/with-assets/some.js</pre>

<code>some.js</code>
