+++
title = "With assets"
description = "hey there"
date = 2015-03-01
+++

Hello world [here](with.js)

Colocated link: [asset](@/posts/with-assets/zola.png)

Colocated image: ![asset](@/posts/with-assets/zola.png)

get_url asset: {{ get_url(path="@/posts/with-assets/zola.png") }}

current.assets: {{ current.assets | join(sep=" ") }}

get_url from current.assets: {% for asset in current.assets %}{{ get_url(path=asset) }} {% endfor %}
