+++
title = "Shortcodes in Templates Demo"
template = "shortcodes_demo.html"

[extra]
show_note = true
+++

This is the content of the demo page. It can also use shortcodes:

{{ note(text="This shortcode is in markdown content!", type="info") }}

And here's an emphasized {{ emphasize(text="important word") }} in the markdown.

## More Content

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.

{{ youtube(id="dQw4w9WgXcQ") }}

The demo template above shows shortcodes being used directly in the HTML template, which is the new feature!
