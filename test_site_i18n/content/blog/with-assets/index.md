+++
date = 2018-11-10

[taxonomies]
tags = ["Evidence"]
authors = ["Alex"]
+++

With assets

en asset: {{ get_url(path="@/blog/with-assets/some.js") }}

![js](@/blog/with-assets/some.js)

en wikilink: [[some.js|JS]]

qualified term: [[tags/evidence|Evidence]]

bare term: [[evidence|Evidence]]

term fragment: [[tags/evidence#details|Details]]

fallback term: [[tags/hello|Hello]]

default author: [[authors/alex|Alex]]

filtered term: {{ "[[tags/evidence|Evidence]]" | markdown(inline=true) | safe }}
