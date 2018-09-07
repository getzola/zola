+++
title = "Troubleshooting"
weight = 200
+++

If someting is not working the way you expect, you can add this snippet at the bottom of your `templates/index.html` template file:


```jinja2
<pre>
    {{ __tera_context }}
</pre>
```

This snippet displays the context data of a page and can help to find the source of your trouble.
