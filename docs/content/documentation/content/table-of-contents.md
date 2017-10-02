+++
title = "Table of Contents"
weight = 60
+++

Each page/section will automatically generate a table of content for itself based on the headers present. 

TODO: add link for template variables
It is available in the template through `section.toc` and `page.toc`. You can view the [template variables]() 
documentation for information on its structure.

Here is an example of using that field to render a 2-level table of content:

```jinja2
<ul>
{% for h1 in page.toc %}
    <li>
        <a href="{{h1.permalink | safe}}">{{ h1.title }}</a>
        {% if h1.children %}
            <ul>
                {% for h2 in h1.children %}
                    <li>
                        <a href="{{h2.permalink | safe}}">{{ h2.title }}</a>
                    </li>
                {% endfor %}
            </ul>
        {% endif %}
    </li>
{% endfor %}
</ul>
```

While headers are neatly ordered in that example, it will work just as well with disjoint headers.
