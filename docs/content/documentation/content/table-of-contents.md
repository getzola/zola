+++
title = "Table of Contents"
weight = 60
+++

Each page/section will automatically generate a table of contents for itself based on the headers generated with markdown.

It is available in the template through the `page.toc` or `section.toc` variable.
You can view the [template variables](@/documentation/templates/pages-sections.md#table-of-contents)
documentation for information on its structure.

Here is an example of using that field to render a two-level table of contents:

```jinja2
{% if page.toc %}
    <ul>
    {% for h1 in page.toc %}
        <li>
            <!-- the "title" property strips the HTML tags from the original title -->
            <a href="{{ h1.permalink | safe }}">{{ h1.title }}</a>
            {% if h1.children %}
                <ul>
                    {% for h2 in h1.children %}
                        <li>
                            <!-- "title_raw" lets you access the title with its original HTML tags -->
                            <a href="{{ h2.permalink | safe }}">{{ h2.title_raw | safe }}</a>
                        </li>
                    {% endfor %}
                </ul>
            {% endif %}
        </li>
    {% endfor %}
    </ul>
{% endif %}
```

While headers are neatly ordered in this example, it will work just as well with disjoint headers.

Note that all existing HTML tags from the title will NOT be present in the table of contents to
avoid various issues.
