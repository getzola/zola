+++
title = "Image Resizing"
weight = 120
+++

Gutengerb provides support for automatic image resizing through the built-in function `resize_image`,
which is available in template code as well as in shortcodes.

The function usage is as follows:

```jinja2
    resize_image(path, width, height, op, quality)
```

### Arguments

- `path`: The path to the source image relative to the `content` directory in the [directory structure](./documentation/getting-started/directory-structure.md).

- `width` and `height`: The dimensions in pixels of the resized image. Usage depends on the `op` argument.
- `op`: Resize operation. This can be one of five choices: `"scale"`, `"fitwidth"`, `"fitheight"`, `"fit"`, or `"fill"`.
  What each of these does is explained below.
  This argument is optional, default value is `"fill"`.
- `quality`: JPEG quality of the resized image, in percents. Optional argument, default value is `75`.

### Image processing and return value

Gutenberg performs image processing during the build process and places the resized images in a subdirectory in the static files directory:

    static/_resized_images/

Resized images are JPEGs. Filename of each resized image is a hash of the function arguments,
which means that once an image is resized in a certain way, it will be stored in the above directory and will not
need to be resized again during subsequent builds (unless the image itself, the dimensions, or other arguments are changed).
Therefore, if you have a large number of images, they will only need to be resized once.

The function returns a full URL to the resized image.

## Resize operations

The source for all examples is this 300 × 380 pixels image:

![gutenberg](gutenberg.jpg)

### **`"scale"`**
  Simply scales the image to the specified dimensions (`width` & `height`) irrespective of the aspect ratio.

  `resize_image(..., width=150, height=150, op="scale")`

  {{ resize_image(path="documentation/content/image-resizing/gutenberg.jpg", width=150, height=150, op="scale") }}

### **`"fitwidth"`**
  Resizes the image such that the resulting width is `width` and height is whatever will preserve the aspect ratio.
  The `height` argument is not needed.

  `resize_image(..., width=100, op="fitwidth")`

  {{ resize_image(path="documentation/content/image-resizing/gutenberg.jpg", width=100, height=0, op="fitwidth") }}

### **`"fitheight"`**
  Resizes the image such that the resulting height is `height` and width is whatever will preserve the aspect ratio.
  The `width` argument is not needed.

  `resize_image(..., height=150, op="fitheight")`

  {{ resize_image(path="documentation/content/image-resizing/gutenberg.jpg", width=0, height=150, op="fitheight") }}

### **`"fit"`**
  Like `"fitwidth"` and `"fitheight"` combined.
  Resizes the image such that the result fits within `width` and `height` preserving aspect ratio. This means that both width or height
  will be at max `width` and `height`, respectively, but possibly one of them smaller so as to preserve the aspect ratio.

  `resize_image(..., width=150, height=150, op="fit")`

  {{ resize_image(path="documentation/content/image-resizing/gutenberg.jpg", width=150, height=150, op="fit") }}

### **`"fill"`**
  This is the default operation. It takes the image's center part with the same aspect ratio as the `width` & `height` given and resizes that
  to `width` & `height`. This means that parts of the image that are outsize of the resized aspect ratio are cropped away.

  `resize_image(..., width=150, height=150, op="fill")`

  {{ resize_image(path="documentation/content/image-resizing/gutenberg.jpg", width=150, height=150, op="fill") }}


## Using `resize_image` in markdown via shortcodes

`resize_image` is a built-in Tera global function (see the [Templates](./documentation/templates/_index.md) chapter),
but it can be used in markdown, too, using [Shortcodes](./documentation/content/shortcodes.md).

The examples above were generated using a shortcode file named `resize_image.html` with this content:

```jinja2
  <img src="{{ resize_image(path=path, width=width, height=height, op=op) }}" />
```

## Creating picuture galleries

The `resize_image()` can be used multiple times and/or in loops (it is designed to handle this efficiently).

This can be used along with `assets_imgs` [page metadata](./documentation/templates/pages-sections.md) to create picture galleries.
The `assets_imgs` variable holds paths to all images in the directory of a page with resources
(see [Assets colocation](./documentation/content/overview.md#assets-colocation)).

This can be used in shortcodes. For example, we can create a very simple html-only clickable
picture gallery with the following shortcode named `gallery.html`:

```jinja2
{% for img in page.assets_imgs %}
  <a href="{{ config.base_url }}/{{ img }}">
    <img src="{{ resize_image(path=img, width=240, height=180) }}" />
  </a>
  &ensp;
{% endfor %}
```

As you can notice, we didn't specify an `op` argument, which means it'll default to `"fill"`. Similarly, the JPEG quality will default to `75`.

To call it from a markdown file, simply do:

```jinja2
{{/* gallery() */}}
```

Here is the result:

{{ gallery() }}

<small>
  Image attribution: example-01: Willi Heidelbach, example-02: Daniel Ullrich, others: public domain.
</small>
