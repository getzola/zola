+++
title = "Archive"
weight = 90
+++

Zola doesn't have a built-in way to display an archive page (a page showing
all post titles ordered by year). However, this can be accomplished directly in the templates:

```jinja
{% raw -%}
{% for year, posts in section.pages | group_by(attribute="year") %}
    <h2>{{ year }}</h2>

    <ul>
    {% for post in posts %}
        <li><a href="{{ post.permalink }}">{{ post.title }}</a></li>
    {% endfor %}
    </ul>
{% endfor %}
{%- endraw %}
```

This snippet assumes that posts are sorted by date and that you want to display the archive
in descending order. If those conditions are not true, you need to further
process the list of pages:

```jinja
{% raw -%}
{% for year, posts in section.pages | sort(attribute="year") | group_by(attribute="year") %}
    <h2>{{ year }}</h2>

    <ul>
    {% for post in posts | sort(attribute="date") %}
        <li><a href="{{ post.permalink }}">{{ post.title }}</a></li>
    {% endfor %}
    </ul>
{% endfor %}
{%- endraw %}
```

You may then invert the ordering of `year` or `date` with the `reverse` filter,
like `{% raw -%}{% for post in posts | sort(attribute="date") | reverse %}{%- endraw %}`.

Note the need for all posts to have `date` defined.
