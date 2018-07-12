+++
title = "Index Page"
weight = 15
+++

The home page of your site (that is, the page that appears at your root URL) is rendered using
a special template, `index.html`.  This template is mandatory—if neither your theme nor your 
`templates` folder has a template named `index.html`, then your site will display only a built-in
error page.  

Other than being governed by a special template, the home page of your site is just another 
`section` of your site.  This means that you can employ all the section variables described in the
following part of the documentation.  And you can set metadata or add content to your home page by
creating a file at `content/_index.md`.

Please note that your home page is _always_ a section page, regardless of whether it has
sub-sections.  Thus, you should always set content for your home page with an `_index.md` file,
never with an `index.md` file.  Similarly, the `index.html` template will always access the
contents of your `_index.md` page using the `{{ section.contents }}` variable, never with the
`{{ page.contents }}` variable.
