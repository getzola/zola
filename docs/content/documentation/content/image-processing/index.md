+++
title = "Image processing"
weight = 120
+++

Zola provides support for automatic image resizing through the built-in function `resize_image`,
which is available in template code as well as in shortcodes.

The function usage is as follows:

```jinja2
resize_image(path, width, height, op, format, quality)
```

### Arguments

- `path`: The path to the source image. The following directories will be searched, in this order:
    - `/` (the root of the project; that is, the directory with your `config.toml`)
    - `/static`
    - `/content`
    - `/public`
    - `/themes/current-theme/static`
- `width` and `height`: The dimensions in pixels of the resized image. Usage depends on the `op` argument.
- `op` (_optional_): Resize operation. This can be one of:
    - `"scale"`
    - `"fit_width"`
    - `"fit_height"`
    - `"fit"`
    - `"fill"`

  What each of these does is explained below. The default is `"fill"`.
- `format` (_optional_): Encoding format of the resized image. May be one of:
    - `"auto"`
    - `"jpg"`
    - `"png"`
    - `"webp"`

  The default is `"auto"`, this means that the format is chosen based on input image format.
  JPEG is chosen for JPEGs and other lossy formats, and PNG is chosen for PNGs and other lossless formats.
- `quality` (_optional_): JPEG or WebP quality of the resized image, in percent. Only used when encoding JPEGs or WebPs; for JPEG default value is `75`, for WebP default is lossless.

### Image processing and return value

Zola performs image processing during the build process and places the resized images in a subdirectory in the static files directory:

```
static/processed_images/
```

The filename of each resized image is a hash of the function arguments,
which means that once an image is resized in a certain way, it will be stored in the above directory and will not
need to be resized again during subsequent builds (unless the image itself, the dimensions, or other arguments have changed).

The function returns an object with the following schema:

```
/// The final URL for that asset
url: String,
/// The path to the static asset generated
static_path: String,
/// New image width
width: u32,
/// New image height
height: u32,
/// Original image width
orig_width: u32,
/// Original image height
orig_height: u32,
```

## Resize operations

The source for all examples is this 300 pixel × 380 pixel image:

![zola](01-zola.png)

### **`"scale"`**
  Simply scales the image to the specified dimensions (`width` & `height`) irrespective of the aspect ratio.

  `resize_image(..., width=150, height=150, op="scale")`

  {{ resize_image(path="documentation/content/image-processing/01-zola.png", width=150, height=150, op="scale") }}

### **`"fit_width"`**
  Resizes the image such that the resulting width is `width` and height is whatever will preserve the aspect ratio.
  The `height` argument is not needed.

  `resize_image(..., width=100, op="fit_width")`

  {{ resize_image(path="documentation/content/image-processing/01-zola.png", width=100, height=0, op="fit_width") }}

### **`"fit_height"`**
  Resizes the image such that the resulting height is `height` and width is whatever will preserve the aspect ratio.
  The `width` argument is not needed.

  `resize_image(..., height=150, op="fit_height")`

  {{ resize_image(path="documentation/content/image-processing/01-zola.png", width=0, height=150, op="fit_height") }}

### **`"fit"`**
  Like `"fit_width"` and `"fit_height"` combined, but only resize if the image is bigger than any of the specified dimensions.
  This mode is handy, if for example images are automatically shrunk to certain sizes in a shortcode for
  mobile optimization.
  Resizes the image such that the result fits within `width` and `height` while preserving the aspect ratio. This
  means that both width or height will be at max `width` and `height`, respectively, but possibly one of them
  smaller so as to preserve the aspect ratio.


  `resize_image(..., width=5000, height=5000, op="fit")`

  {{ resize_image(path="documentation/content/image-processing/01-zola.png", width=5000, height=5000, op="fit") }}

  `resize_image(..., width=150, height=150, op="fit")`

  {{ resize_image(path="documentation/content/image-processing/01-zola.png", width=150, height=150, op="fit") }}

### **`"fill"`**
  This is the default operation. It takes the image's center part with the same aspect ratio as the `width` and
  `height` given and resizes that to `width` and `height`. This means that parts of the image that are outside
  of the resized aspect ratio are cropped away.

  `resize_image(..., width=150, height=150, op="fill")`

  {{ resize_image(path="documentation/content/image-processing/01-zola.png", width=150, height=150, op="fill") }}


## Using `resize_image` in markdown via shortcodes

`resize_image` is a Zola built-in Tera function (see the [templates](@/documentation/templates/_index.md) chapter),
but it can be used in Markdown using [shortcodes](@/documentation/content/shortcodes.md).

The examples above were generated using a shortcode file named `resize_image.html` with this content:

```jinja2
{% set image = resize_image(path=path, width=width, height=height, op=op) %}
<img src="{{ image.url }}" />
```

## Creating picture galleries

The `resize_image()` can be used multiple times and/or in loops. It is designed to handle this efficiently.

This can be used along with `assets` [page metadata](@/documentation/templates/pages-sections.md) to create picture galleries.
The `assets` variable holds paths to all assets in the directory of a page with resources
(see [asset colocation](@/documentation/content/overview.md#asset-colocation)); if you have files other than images you
will need to filter them out in the loop first like in the example below.

This can be used in shortcodes. For example, we can create a very simple html-only clickable
picture gallery with the following shortcode named `gallery.html`:

```jinja2
<div>
{% for asset in page.assets -%}
  {%- if asset is matching("[.](jpg|png)$") -%}
    {% set image = resize_image(path=asset, width=240, height=180) %}
    <a href="{{ get_url(path=asset) }}" target="_blank">
      <img src="{{ image.url }}" />
    </a>
  {%- endif %}
{%- endfor %}
</div>
```

As you can notice, we didn't specify an `op` argument, which means that it'll default to `"fill"`. Similarly,
the format will default to `"auto"` (choosing PNG or JPEG as appropriate) and the JPEG quality will default to `75`.

To call it from a Markdown file, simply do:

```jinja2
{{/* gallery() */}}
```

Here is the result:

{{ gallery() }}

<small>
  Image attribution: Public domain, except: _06-example.jpg_: Willi Heidelbach, _07-example.jpg_: Daniel Ullrich.
</small>


## Get image size and relative resizing

Sometimes when building a gallery it is useful to know the dimensions of each asset.  You can get this information with
[get_image_metadata](@/documentation/templates/overview.md#get-image-metadata).

This can also be useful in combination with `resize_image()` to do a relative resizing. So we can create a relative image resizing function with the following shortcode named `resize_image_relative.html`:

```jinja2
{% set mdata = get_image_metadata(path=path) %}
{% set image = resize_image(path=path, width=(mdata.width * scale)|int, op="fit_width") %}
<img src="{{ image.url }}" />
```

It can be invoked from Markdown like this:

`resize_image_relative(..., scale=0.5)`

{{ resize_image_relative(path="documentation/content/image-processing/01-zola.png", scale=0.5) }}

## Creating scaled-down versions of high-resolution images

With the above, we can also create a shortcode that creates a 50% scaled-down version of a high-resolution image (e.g. screenshots taken on Retina Macs), along with the proper HTML5 `srcset` for the original image to be displayed on high-resolution / retina displays.

Consider the following shortcode named `high_res_image.html`:

```jinja2
{% set mdata = get_image_metadata(path=path) %}
{% set w = (mdata.width / 2) | int %}
{% set h = (mdata.height / 2) | int %}
{% set image = resize_image(path=path, width=w, height=h, op="fit_width") %}
<img src="{{ image.url }}" srcset="/{{path}} 2x"/>
```

{{ high_res_image(path="documentation/content/image-processing/08-example.jpg") }}
