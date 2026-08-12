# Incremental build design

A design, not an implementation. Written because the performance program asks
for it before anyone starts, and because the measurements now say clearly which
parts of a rebuild are worth skipping and which are not.

**Position: do not build this yet.** `zola serve --fast` already covers the
inner loop the feature would serve, and the phases a dependency graph could skip
are no longer where the time goes. The numbers are below; the design is here so
that when the trade changes, the work starts from evidence rather than from a
blank page.

## What a rebuild costs today

`mixed-realistic-4000`, current tree, 1.7 s wall
(`FINAL-REPORT.md` §1 for the full table):

| phase | wall | could incrementality skip it? |
| ----- | ---- | ----------------------------- |
| `render + write outputs` | 58% | only for pages that did not change |
| `render markdown` | 24% | yes, per page |
| `parse pages` | 6% | yes, per page |
| `discover + parse sections` | 4% | partly — the walk still has to happen |
| `build render cache` | 2.5% | partly |
| everything else | < 6% | no |

So on a 4000-page site, a perfect single-page incremental rebuild has about
**88% of the work available to skip** — but the phases it must still run
(discovery, config, template load, and the writes for whatever did change) put a
floor of roughly 150–200 ms on any rebuild.

`zola serve --fast` already reaches most of that floor for the common case: a
content edit re-parses and re-renders one page and runs `Queue::single_page`.
What it does *not* do is decide which *other* outputs a change invalidates — it
re-runs `populate_sections()` (which re-sorts every section in the site) and
leaves anything that embeds the page — its section's page list, taxonomy terms,
feeds, the sitemap — stale until a full rebuild.

That gap is the honest scope of this feature: **not "make rebuilds fast" but
"make partial rebuilds correct"**.

## Dependency nodes

A node is anything that can be invalidated independently. Keyed the way the
`Library` already keys things, so a graph can be built without re-keying the
world.

| node | identity | produced by |
| ---- | -------- | ----------- |
| `SourceFile(path)` | absolute path | the filesystem |
| `Page(path)` | absolute `.md` path | `Page::from_file` |
| `Section(path)` | absolute `_index.md` path | `Section::from_file` |
| `Asset(path)` | absolute path | colocated files, `static/` |
| `Template(name)` | Tera name (`page.html`, `macros/ui.html`) | `templates::load_tera` |
| `DataFile(path)` | path passed to `load_data` | `LoadData` |
| `Config` | — | `config::get_config` |
| `Taxonomy(lang, slug)` | as in `Library::taxonomies_def` | `find_taxonomies` |
| `TaxonomyTerm(lang, slug, term)` | — | `Taxonomy::new` |
| `Output(path)` | path under `public/` | a `Job` in `site::queue` |

## Dependency edges

Most of these already exist in the `Library`; the graph is largely a matter of
reading them in the other direction.

| edge | where it is known today |
| ---- | ----------------------- |
| `SourceFile → Page/Section` | 1:1 by path |
| `Page → Section` (membership) | `Section.pages`, built in `populate_sections` |
| `Section → Section` (parent/child) | `Section.subsections`, `Section.ancestors` |
| `Page → Page` (siblings) | `Page.lower` / `Page.higher`, a product of section sorting |
| `Page → Page` (links) | `Library.backlinks`, built from `internal_links` |
| `Page → Taxonomy term` | `Library.taxonomies_def` |
| `Page/Section → Template` | `meta.template`, `page_template`, plus Tera's own inheritance and component graph |
| `Page/Section → DataFile` | **not tracked**: `load_data` calls happen inside template rendering |
| `Page → Asset` | `Page.assets`, `Library.colocated_assets` |
| `* → Config` | everything |
| `Page/Section/Taxonomy → Output` | `Queue::full_build`, which is where a job list already exists |

Two edges are missing and both matter:

* **template → template**: Tera knows `extends`, `include` and component calls
  internally; the graph needs that relation exported, otherwise editing
  `macros/ui.html` can only be handled by invalidating everything.
* **page → data file**: a `load_data(path=…)` inside a template is invisible to
  the caller. Recording it needs `LoadData` to report what it read for the
  render in progress — a per-render accumulator, not a global one, since
  renders run in parallel.

## Invalidation rules

The rule for each change kind, written as "what must be re-rendered", not "what
must be re-parsed".

### A page's markdown changes

1. re-parse the page; if its front matter is unchanged apart from body text,
   nothing structural moved;
2. re-render its markdown, re-render its output;
3. re-render outputs that embed it: its section's page list, every taxonomy term
   it belongs to, the feeds those belong to, and the pages listing its
   backlinks;
4. if `title`, `date`, `weight` or `slug` changed, the section's *order* changed:
   re-sort that section, re-render its siblings' `lower`/`higher`, and re-render
   the section and its pagers;
5. if `taxonomies` changed, add/remove the memberships and re-render the affected
   terms plus the taxonomy list pages;
6. if `path`/`slug`/`aliases` changed, the URL changed: delete the old outputs,
   re-check path collisions.

Steps 3–6 are the ones `--fast` skips today.

### A page is added or deleted

As above, plus: the parent section's page list changes, the sitemap changes, and
for a deletion every output under the old URL must be removed. Deletion is the
case most likely to leave stale files behind, so it is also the case that needs
an explicit test.

### A section's `_index.md` changes

`sort_by`, `paginate_by`, `transparent`, `render`, `page_template` and `hidden`
each change what the section's *pages* render as, so the section subtree is the
invalidation unit — not the section alone. `transparent` reaches further: it
moves pages into the ancestor's list, so its parent chain is affected too.

### A template changes

Invalidate every page and section whose rendering reaches that template,
following the template → template edge transitively. Without that edge the only
correct answer is "everything", which is what happens today. Note that changing a
template does **not** require re-parsing markdown — except for content that is
itself templated (`md_render.rs`), which is why the current `serve` path
re-renders markdown on template changes.

### A data file changes

Invalidate every page whose render read it. Requires the missing page → data
file edge; without it, a data change is a full rebuild. `LoadData`'s cache key
already includes the file's mtime, so staleness is detected — the gap is only
in knowing *who* to re-render.

### The config changes

Rebuild everything. `base_url`, `taxonomies`, `languages`, `slugify` and the
markdown options all reach every output, and the current behaviour (recreate the
`Site`) is the right one. Not worth refining.

### An asset changes

Copy it. No render depends on the *content* of a colocated asset — only on its
existence, which is captured at parse time. Adding or removing one invalidates
the owning page (its `assets` list is template-visible).

### A taxonomy term's membership changes

Re-render the term page, its pagers and feed, and the taxonomy list page.
Terms are cheap to re-render individually; the expensive part is that
`RenderCache` currently rebuilds wholesale.

## What the implementation would need that does not exist

1. **A persisted graph.** Between runs of `zola build`, invalidation needs the
   previous graph plus content hashes. In `zola serve` it is already in memory.
   Persisting it means a versioned on-disk format and an answer for "the format
   changed / the cache is corrupt" that falls back to a full build.
2. **Incremental `RenderCache`.** Today `RenderCache::build` is all-or-nothing.
   It is now 2.5% of a build (PERF-005a), so per-node invalidation there is a
   refinement, not a prerequisite.
3. **Incremental `populate_sections`.** It re-sorts every section. At 12 ms per
   4000 pages this is not worth making incremental on its own; it matters only
   because `serve --fast` calls it per keystroke.
4. **Output reconciliation.** An incremental build must delete outputs that
   should no longer exist. That is the same problem the output clean solves by
   brute force today, and it is where correctness bugs would live.
5. **A correctness gate.** For every change kind: apply it, rebuild
   incrementally, rebuild fully, and compare byte-for-byte.
   `scripts/perf/compare_output.py` already does the comparison; the harness
   would need to script the edits.

## Where the value actually is

* **`zola serve`**: high. The floor is ~150–200 ms and the current gap is
  correctness, not speed. Closing the section/taxonomy/feed invalidation gap in
  `--fast` is worth more than a persisted graph.
* **CI builds**: low. They start from a clean checkout with no previous graph,
  so nothing is skippable.
* **Local `zola build` loops**: medium, and entirely dependent on item 1 above.

The measured ranking says the same thing: on the reference workload, 94% of the
build is producing and writing output, and no dependency graph makes a page that
*did* change cheaper to write.
