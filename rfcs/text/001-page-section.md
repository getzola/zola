- Feature Name: page-section
- Start Date: 2026-08-20
- RFC PR: [zola/rfcs#0001](https://github.com/getzola/zola/pull/3258)

## Summary
[summary]: #summary

Merge the concept of page and section in Zola.
Pages and sections are currently different concepts in Zola, which causes a lot of confusion

## Motivation
[motivation]: #motivation

The difference between page and section has been a source of confusion for a long time:

- https://zola.discourse.group/t/proposal-deprecate-sections/1968
- https://zola.discourse.group/t/section-vs-page/522
- https://zola.discourse.group/t/having-a-common-property-that-can-be-used-in-pages-and-sections/1258
- https://zola.discourse.group/t/section-vs-pages/2279
- https://zola.discourse.group/t/current-variable-for-fields-in-both-pages-and-sections/935

In the codebase itself, a lot of code is duplicated between pages and sections and those 2 are _almost_ the same struct:
there are just a few more fields for the sections.
A lot of reported bugs are for something that works in page but doesn't work in sections or vice versa.

In the templates, people sometimes do something like `{% set item = page or section %}` or `{% if page %}..{% else %}{% endif %}` so there is a single variable
that can be used on all pages.

## Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

All content files produce a `Page`. Pages with a `_index.md` filename have a `kind == Section` and represent a collection. They do
have a couple additional front-matter fields that only make sense for collections like `sort_by` for example.

```
about.md <-- a normal page
posts/
  _index.md  <-- a page representing a directory
  something.md  <-- a page in the posts directory
  colocated-assets/  <-- we don't know if this a directory or a page with colocated assets
    index.md  <-- until we see index.md which means it's a page of the posts directory
    hey.jpg
```

In the templates, there is just the `self` variable that has `self.{pages,subsections}` filled if it's the directory page
and empty otherwise.
For existing users, we can create a `section` and `page` alias that match what existed before this merge in each case (eg
a directory page gets a `section` variable but a normal page does not).
We do expose a new field, `kind`, which indicates whether it's a section or a page since we need to differentiate between those
still.

On the content side, there are no changes to be made for users. Any directory type attribute (eg a `sort_by`) set on a normal page is an error.
Files with `_index.md` filename will still default to having a `section.html` template.

The `get_section` function works just like `get_page` but also checks that `kind==Section` and errors if it's not the case.

Overall, it shouldn't be a breaking change, except for some narrow cases like someone checking if an attribute exists
on a variable like so:

```j2
{% set item = page or section %}
{% if item.subsections is defined %}
...
{% else %}
...
{% endif %}
```

That will not continue to work as before, without any warnings. You can use the new `kind` field to work reliably instead.

There might be other breakages but only for edge cases
that I don't think many people would do (like printing all the keys of the variable...?).
Still it would come in a new release, like 0.24.

The main benefit for users is the lower cognitive load for writing templates as well as things working more as expected.


## Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

The whole duplication in the `content` component goes away and everything is merged into the `Section` struct
which is renamed to `Page`. For the front-matter, since some fields do not make sense on a basic page (like `sort_by`) and
we want to keep validation, we can use something like:

```rust
enum ParsedFrontMatter {
    /// Regular content files
    Page(PageFrontMatter),
    /// Content files named _index.md
    Section(SectionFrontMatter),
}
```
and factor out the common fields. This way we can still use serde `deny_unknown_fields` and get validation for free.

We keep the current `ser.rs` file to ensure the `page`/`section` variables stay the same.
We keep an enum of `Kind` to differentiate between regular pages and section/collection pages.
If we allow slugs on sections, we do need to ensure all the descendant permalinks are generated correctly.

The `Library` will merge page and sections and we need to ensure `transparent` and the new equivalent of `populate_sections` still work.
There should be no regression for parallelization of rendering or for `zola serve`.

We do not want a single `item.children` property instead of having both `.{pages,subsections}` because sorting pages
and subsection together does not really make sense currently, `sort_by` would not apply to sections.
The `$NAME.{pages,subsections}` fields become a `Vec<Page>`, not a `Vec<&'a str>`: only for the new variables, not for the 
legacy `page` and `section` variables.

All the other changes are mechanical, with a special care of still grouping pages through sections (or directories/any name)
for rendering.

The aliases for the `get_section`, `page`, `section` can stay for a few major versions. If what's currently available is not
enough, we can add more introspection to Tera to be able to show deprecation warnings.

## Drawbacks
[drawbacks]: #drawbacks

- Potentially breaking some templates doing attribute checking
- Some users like the distinction


## Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

This simplifies the internals of Zola since a lot of things need to be added twice currently while also simplifying
the usage.

Alternatives:
- ship just an alias in the templates for `page` and `section` and have them matching fields: solve the user issue, not the internals
- solve the duplication internally but don't expose it: doesn't solve the user issue


## Prior art
[prior-art]: #prior-art

This is based from their docs and I could be wrong.

I had a quick look at Eleventy/Astro but couldn't see exactly how they handle sections, I'll add it if someone can comment and
it's not just: you write some JS/JSON.

### Hugo

Hugo seems to be exactly the same idea as this RFC.
They use the same name for colocated assets and sections: `page bundle`. 
A colocated page bundle is called `leaf bundle` and a section `branch bundle`.
A page can have one kind out of home, page, section, taxonomy, or term (https://gohugo.io/quick-reference/glossary/#page-kind)
so the templates can still differentiate between them if necessary.
It still refers to sections as the listing of pages: https://gohugo.io/methods/page/ and https://gohugo.io/content-management/sections/



https://gohugo.io/content-management/page-bundles/
https://gohugo.io/content-management/page-resources/



## Unresolved questions
[unresolved-questions]: #unresolved-questions

- Best name for the context object for a page? `self`, `this`, `item`, `current`? It can't be `page` since it would break
some `{% if page %}` check
- What does it mean to add a taxonomy to a section since it becomes possible?

## Future possibilities
[future-possibilities]: #future-possibilities

- Allow colocated asset `index.md` to be named after the folder, eg `colocated-assets/colocated-assets.md` to avoid
having a bunch of `index.md` open

