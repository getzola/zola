+++
title = "Archive"
weight = 90
+++

Zola doesn't have a built-in way to display an archive page (a page showing
all post titles ordered by year). However, this can be accomplished directly in the templates:

```jinja2
{% for year, posts in section.pages | group_by(attribute="year") %}
    <h2>{{ year }}</h2>

    <ul>
    {% for post in posts %}
        <li><a href="{{ post.permalink }}">{{ post.title }}</a></li>
    {% endfor %}
    </ul>
{% endfor %}
```

This snippet assumes that posts are sorted by date and that you want to display the archive
in descending order. If you want to show articles in ascending order, you need to further
process the list of pages:
```jinja2
{% set posts_by_year = section.pages | group_by(attribute="year") %}
{% set_global years = [] %}
{% for year, ignored in posts_by_year %}
    {% set_global years = years | concat(with=year) %}
{% endfor %}
{% for year in years | reverse %}
    {% set posts = posts_by_year[year] %}
    {# (same as the previous snippet) #}
{% endfor %}
```
