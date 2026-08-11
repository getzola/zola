# Allocation profile (M8)

## Method

macOS has no `heaptrack`/`massif`, and Instruments' allocation template cannot
attribute to Zola's own build phases. Instead the binary can be built with a
counting global allocator:

```bash
cargo build --release --features alloc-stats --target-dir target/allocstats
target/allocstats/release/zola build --force --timings -o /tmp/out
```

`src/alloc_stats.rs` wraps `System` and counts allocations, bytes allocated,
live bytes and peak live bytes with relaxed atomics. `utils::timings` picks the
counters up through a registered probe, so **every build phase reports its own
allocation count, bytes allocated and the live heap when it finished**.

The feature is off by default: a production build contains no wrapper at all.
With it on, wall times inflate (two atomic RMWs per allocation, and
`mixed-realistic-4000` goes from 1.6 s to 3.7 s) — allocation numbers from this
binary are comparable to each other, never to the timing baseline.

## `mixed-realistic-4000`

Peak live heap **1.1 GB**; ~20 M allocations; ~2.5 GB allocated in total.

| phase | allocations | bytes allocated | live heap after |
| ----- | ----------- | --------------- | --------------- |
| `config` (highlighting registry) | 185.3k | 32.9 MB | 23.0 MB |
| `templates::load` | 10.3k | 1.5 MB | 23.4 MB |
| `discover + parse sections` | 58.2k | 6.6 MB | 24.9 MB |
| `parse pages` | 276.1k | 53.0 MB | 41.5 MB |
| `index (insert pages + permalinks)` | 47.6k | 23.4 MB | 49.8 MB |
| `populate sections (graph + sorting)` | 92.9k | 6.2 MB | 51.9 MB |
| `populate taxonomies` | 23.1k | 1.6 MB | 52.8 MB |
| `register tera fns (early)` | **1.1M** | **132.0 MB** | 144.6 MB |
| `render markdown` | 1.6M | 310.4 MB | 163.1 MB |
| `fill backlinks` | 96.5k | 9.8 MB | 167.1 MB |
| **`build render cache`** | **7.4M** | **962.7 MB** | **1.1 GB** |
| `check internal links` | 47.8k | 3.7 MB | 1.1 GB |
| `render + write outputs` | 9.4M | 1.0 GB | 1.1 GB (transient) |

Reading the `live heap after` column as a cumulative profile, the peak heap
decomposes roughly as:

| owner | live bytes | share |
| ----- | ---------- | ----- |
| `RenderCache` | ~950 MB | 86% |
| Tera function state (cloned config, permalinks, colocated assets, Tera) | ~92 MB | 8% |
| `Library` (pages, sections, rendered HTML) | ~110 MB | 10% |
| config + highlighting registry | 23 MB | 2% |

`many-taxonomies-4000` shows the same shape, more extreme: `build render cache`
allocates **10.0 M times / 1.2 GB** and takes the live heap from 152.6 MB to
**1.4 GB**.

`render + write outputs` allocates a lot (9.4 M allocations, 1.0 GB) but
releases nearly all of it: the live heap does not move. That is per-output
churn — render into a `String`, minify into another, write, drop.

## Controlled experiment: what exactly does the render cache duplicate?

Same generator, same 2000 pages, same everything — only the number of taxonomy
memberships per page varies:

| memberships / page | `build render cache` | allocations | bytes allocated | peak live heap |
| ------------------ | -------------------- | ----------- | --------------- | -------------- |
| 0 | 35.5 ms | 553.9k | 95.8 MB | 217.7 MB |
| 2 | 151.5 ms | 2.6M | 347.1 MB | 470.5 MB |
| 4 | 281.5 ms | 5.0M | 619.0 MB | 743.8 MB |
| 8 | 518.1 ms | 9.3M | 1.1 GB | 1.3 GB |

Every additional membership costs ≈ 60 ms, ≈ 1.1 M allocations and ≈ 135 MB of
*live* heap per 2000 pages — i.e. **≈ 67 KB of retained heap per
(page, taxonomy-term) pair**, which is the size of one fully materialised page
value.

That is the direct signature of `cache.rs:200`: `SerializedTaxonomyTerm` holds
`pages: Vec<Value>`, and `Value::from_serializable` re-serializes each page
value through serde rather than sharing it. `tera::Value` is internally
`Arc<Map>` / `Arc<Vec>` / inline-or-`Arc` strings, so *cloning* a value is O(1) —
but `<Value as Serialize>::serialize` (`tera-2.1.1 value/mod.rs:1062`) walks the
whole structure and the deserializing side rebuilds every map, key and string.
The same mechanism applies to `SerializingSection.pages` (`cache.rs:125`), which
is why even taxonomy-free sites pay it once per page.

## Dominant allocation sources, ranked

1. **`RenderCache::build` deep re-serialization** — 7.4–10 M allocations, 86% of
   the live heap. One page value per (page + section membership + each taxonomy
   term membership). → PERF-005.
2. **`register_early_global_fns`** — 1.1 M allocations, 132 MB, ~92 MB retained.
   It clones `Config` (which embeds the highlighting registry), the whole
   `permalinks` map, `colocated_assets` and the entire `Tera` instance, into
   several Tera function objects. It also runs twice, because
   `register_early_global_fns` calls `register_tera_global_fns` itself and
   `Site::load` then calls it again (`site/src/lib.rs`). → PERF-010.
3. **`render markdown`** — 1.6 M allocations / 310 MB, of which ~163 MB is
   retained as the rendered HTML on each `Page`. Largely inherent: the HTML has
   to exist. The transient part is pulldown-cmark event buffering
   (`State::render` collects all events into a `Vec` before processing).
4. **`render + write outputs`** — 9.4 M transient allocations. Per output:
   Tera render → `String`, live-reload injection, minify → another `String`.
   `minify::html` takes the string by value and returns a new one.
5. **`parse pages`** — 276 k allocations / 53 MB for 4000 pages ≈ 69
   allocations per page: front matter parse, slug, permalink, components,
   `PathBuf`s in `FileInfo`. Proportionate.
6. **`index`** — 47.6 k allocations but 23.4 MB, i.e. large ones: this is
   `permalinks` (a `String→String` map over all pages) plus `reverse_aliases`.

## Cross-checks against the "usual suspects" list

| suspect | verdict |
| ------- | ------- |
| repeated `String` cloning | Real, but concentrated in the render cache, not spread out. |
| repeated `PathBuf` cloning | Present everywhere (`Library` keys everything by `PathBuf`, `section.pages: Vec<PathBuf>`), but only 47.6 k allocations at index time — not a top cost at 4k pages. Would matter more if the cache problem were fixed. |
| repeated `HashMap` construction | Not observed outside the cache. |
| serialization/deserialization loops | **Confirmed as the #1 source** (render cache). |
| template context cloning | Cheap: contexts insert `Arc`-backed `Value` clones (`renderer.rs`), O(1) each. |
| unnecessary `Arc`/`Rc` churn | Not observed. |
| repeated path canonicalization | Not observed; `canonicalize` is called once per build in `main.rs`. |
| repeated regex compilation | None — all regexes are `LazyLock` statics. |
| repeated markdown parser setup | `MarkdownContext::options()` rebuilds a small bitflag per page; negligible. |
| large temporary buffers | `State::render` buffers all pulldown-cmark events per page; bounded by page size. |
