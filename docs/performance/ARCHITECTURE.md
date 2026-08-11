# Zola build architecture (performance view)

Milestone **M1** of the large-site performance program. This document describes the
build pipeline as it exists at commit `d225f3fd` (v0.23.3), from the perspective of
"what work is done, how many times, and against which data structures".

It is descriptive, not prescriptive. Hypotheses about cost are marked as such and are
only promoted to findings in `SCALING.md` / `CPU-PROFILE.md` / `HOTSPOTS.md`, where they
are backed by measurements.

Notation used throughout:

```
P = number of pages
S = number of sections
L = number of internal links
T = number of taxonomies
K = number of taxonomy terms
A = number of aliases
D = section tree depth
```

Reference real-world workload (`~/dev/vomaste.cz`): P ≈ 3.7k, S ≈ 1.6k, D = 4,
T = 0, 989 static files, 6.1 MB of markdown, `minify_html = true`,
`build_search_index = false`, `generate_feeds = false`, 57 templates,
62 `load_data` call sites (including one in `base.html`).

---

## 1. Entry points

| Layer | File | Notes |
| ----- | ---- | ----- |
| CLI definition | `src/cli.rs` | `clap` derive; also `include!`d by `build.rs` to generate man pages/completions |
| Process entry | `src/main.rs` | Resolves root dir + config file (`zola.toml`, then `config.toml`), dispatches subcommand |
| `build` | `src/cmd/build.rs` | `Site::new` → `load()` → `build()` |
| `serve` | `src/cmd/serve.rs` | axum server + `notify` watcher; rebuild paths keyed by `ChangeKind` (`src/fs_utils.rs`) |
| `check` | `src/cmd/check.rs` | `load()` only, with external link checking enabled |

`zola build` is the benchmark target of this program. `serve` shares `Site::load` and
`Queue`, so most findings transfer, but its incremental paths are analysed separately
(§10).

---

## 2. Build pipeline

```
main
└── Site::new(path, config_file)                         [components/site/src/lib.rs:81]
    ├── get_config()                                     parse config.toml
    ├── config.merge_with_theme()                        theme.toml [extra]
    ├── load_tera()                                      glob + parse all templates
    └── imageproc::Processor::new()

└── Site::load()                                         [site/src/lib.rs:188]
    ├── WalkDir over content/                            sections serially, pages collected
    │   ├── Section::from_file()  (per dir, serial)      front matter + assets
    │   └── Page::from_file()     (par_iter over paths)  front matter + assets + slug + permalink
    ├── create_default_index_sections()
    ├── add_page() ×P                                    permalinks map + Library::insert_page
    ├── Library::find_path_collisions()
    ├── RenderCache::new(config)                         serialize config per language
    ├── populate_sections()                              ancestors, subsections, section↔page, sorting
    ├── populate_taxonomies()  → Library::find_taxonomies()
    ├── tpls::register_early_global_fns()                clones config/permalinks/tera into Tera fns
    ├── render_markdown()                                par_iter pages, then par_iter sections
    │   ├── (optional) Tera templating of raw markdown   md_render.rs, guarded by memchr scan
    │   └── markdown::render_content()                   pulldown-cmark + highlighting + link fixing
    ├── Library::fill_backlinks()
    ├── RenderCache::build()                             serialize every page/section/taxonomy to tera::Value
    ├── tpls::register_tera_global_fns()
    ├── link_checking::check_internal_links_with_anchors()
    └── (check mode only) link_checking::check_external_links()

└── Site::build()                                        [site/src/lib.rs:714]
    ├── clean()                                          rm -rf public (Disk mode only)
    ├── sass::compile_sass()                             theme + own
    ├── build_search_index()                             elasticlunr or fuse, per language
    ├── render_themes_css()
    ├── Queue::full_build(site).process()                ← the bulk of the work
    │   ├── job construction (serial)                    aliases, sections, pages, taxonomies, feeds
    │   └── par_iter over jobs
    │       ├── execute_job()  → Renderer::render_*      Tera render into String
    │       ├── write_output()  → minify + fs::write     or SITE_CONTENT for serve
    │       └── copy_assets()                            colocated assets per page
    ├── process_images()                                 imageproc, behind a Mutex
    └── copy_static_directories()
```

### Phase boundaries actually present in the code

The code has no explicit phase enum. The de-facto boundaries are:

| # | Phase | Boundary | Parallel? |
| - | ----- | -------- | --------- |
| 1 | CONFIG | `Site::new` | no |
| 2 | TEMPLATE LOAD | `load_tera` inside `Site::new` | no |
| 3 | DISCOVER | `WalkDir` loop in `load()` | no (dir walk), yes (page parse) |
| 4 | PARSE | `Page::from_file` / `Section::from_file` | pages: rayon; sections: serial |
| 5 | INDEX | `insert_page`, `find_path_collisions`, `populate_sections`, `find_taxonomies` | no |
| 6 | RENDER MARKDOWN | `render_markdown()` | rayon (pages, then sections) |
| 7 | RESOLVE | `fill_backlinks`, `RenderCache::build`, internal link check | no |
| 8 | RENDER HTML + WRITE | `Queue::process()` | rayon, render and write interleaved per job |
| 9 | POST | images, static copy | images: rayon behind a Mutex; copy: serial |

Note that **phase 8 fuses render and write**: each rayon job renders, minifies and writes
its own output. There is no separate WRITE phase and no write batching.

---

## 3. Important types

| Type | Crate | Role | Lifetime |
| ---- | ----- | ---- | -------- |
| `Site` | `site` | Owns everything: config, tera, library, cache, paths | whole build |
| `Library` | `content` | All pages/sections + relationship maps | `Arc<Library>` inside `Site`, mutated via `Arc::make_mut` during load |
| `Page` / `Section` | `content` | Parsed content + front matter + computed fields | owned by `Library` |
| `FileInfo` | `content` | Path decomposition (path, relative, canonical, components, parent, grand_parent) | inside `Page`/`Section` |
| `RenderCache` | `render` | Pre-serialized `tera::Value` for every page/section/taxonomy/config | `Arc<RenderCache>` in `Site`, also cloned into Tera functions |
| `Renderer<'a>` | `render` | Borrows tera+config+library+cache; the only caller of `tera.render` | per job |
| `Queue<'a>` | `site` | Job list + paginators | per build |
| `Job<'a>` | `site` | Unit of output: Page, Section, Paginated, TaxonomyList/Term, Feed, Alias, Sitemap, NotFound, Robots | per output |
| `Taxonomy` / `TaxonomyTerm` | `content` | Terms with `Vec<PathBuf>` of member pages | `Site::taxonomies` |
| `Paginator<'a>` | `render` | Borrowed page list + pagers with index ranges | held by `Queue` |
| `MarkdownContext<'a>` | `markdown` | Borrowed tera/config/permalinks/colocated assets for one render | per page |
| `Processor` | `imageproc` | Image op queue | `Arc<Mutex<Processor>>` in `Site` |

### Ownership model

* `Site` owns `Arc<Library>` and `Arc<RenderCache>`. During `load()` these are mutated
  through `Arc::make_mut`, which is cheap only because the refcount is 1 at that point —
  **except** that `RenderCache` is also cloned into Tera functions
  (`GetPage`, `GetSection`, `GetTaxonomy`, …) by `register_tera_global_fns`. Those clones
  are `Arc` clones, so they are cheap, but they mean the cache is effectively frozen once
  registered; `rebuild_cache()` creates a fresh one and re-registers.
* Pages and sections are keyed by their **absolute** `PathBuf` everywhere. There are no
  integer IDs; every cross-reference (`section.pages`, `term.pages`, `page.lower/higher`,
  `translations`, `backlinks`) stores cloned `PathBuf`s.
* Rendering borrows immutably from `Site`, so `Queue::process` can be `par_iter`.

---

## 4. Significant collections

`AHashMap`/`AHashSet` = `ahash`; `HashMap` = std (SipHash).

### 4.1 `Library` (`components/content/src/library.rs:50`)

| Collection | Type | Key | Value | Size | Access pattern | Cost |
| ---------- | ---- | --- | ----- | ---- | -------------- | ---- |
| `pages` | `AHashMap` | absolute `PathBuf` | `Page` | P (3.7k) | keyed get in queue/cache/pagination; full iteration in load, sitemap, feeds, backlinks | O(1) get, O(P) scans |
| `sections` | `AHashMap` | absolute `PathBuf` | `Section` | S (1.6k) | keyed get; full iteration in `populate_sections`, queue | O(1) get |
| `reverse_aliases` | `AHashMap` | url path `String` | `AHashSet<PathBuf>` | P+S+A | built on insert; scanned once in `find_path_collisions` | O(1) insert |
| `translations` | `AHashMap` | canonical `PathBuf` | `AHashSet<PathBuf>` | P+S (multilingual only) | `find_translations` per serialized page | O(1) |
| `backlinks` | `AHashMap` | relative md path `String` | `AHashSet<PathBuf>` | ≤ L | built once; read per serialized page | O(1) get + O(b log b) sort per page |
| `colocated_assets` | `AHashMap` | `String` | `(String, String)` | assets | read during markdown link resolution | O(1) |
| `taxonomies_def` | `AHashMap<String, AHashMap<String, AHashMap<String, Vec<PathBuf>>>>` | lang → slug → term | page paths | T·K | built on insert; consumed by `find_taxonomies` | O(1) insert |
| `taxo_name_to_slug` | `AHashMap` | name | slug | T | per page-taxonomy entry | O(1) |

Per-page/section vectors: `ancestors: Vec<String>` (D entries), `components: Vec<String>`,
`assets: Vec<PathBuf>`, `serialized_assets: Vec<String>`, `toc: Vec<Heading>`,
`internal_links: Vec<(String, Option<String>)>`, `external_links: Vec<String>`.
Per-section: `pages`, `hidden_pages`, `ignored_pages`, `subsections`,
`ignored_subsections` — all `Vec<PathBuf>`.

### 4.2 `RenderCache` (`components/render/src/cache.rs:34`)

| Collection | Type | Key | Value | Size | Access | Cost |
| ---------- | ---- | --- | ----- | ---- | ------ | ---- |
| `pages` | `AHashMap` | `PathBuf` | `CachedContent { value, canonical }` | P | O(1) get + `Value::clone` per render | O(1) + clone |
| `sections` | `AHashMap` | `PathBuf` | `CachedContent` | S | same | O(1) + clone |
| `pages_by_canonical` | `AHashMap` | canonical `PathBuf` | `AHashMap<lang, PathBuf>` | P | `get_page(lang=…)` | O(1) |
| `sections_by_canonical` | `AHashMap` | canonical `PathBuf` | `AHashMap<lang, PathBuf>` | S | `get_section` | O(1) |
| `configs` | `AHashMap` | lang | `Value` | languages | cloned into **every** render context | O(1) + clone |
| `taxonomies` | `AHashMap<lang, AHashMap<slug, CachedTaxonomy>>` | | value + per-term values + template names | T·K | O(1) | O(1) + clone |

**Structural note (candidate hotspot).** A cached section `Value` embeds a *clone of the
full serialized `Value` of every page in that section* (`SerializingSection.pages:
Vec<Value>`, built at `cache.rs:125`). A cached taxonomy term embeds the same for its
member pages (`cache.rs:200`). A page `Value` contains the page's entire rendered HTML
(`SerializingPage.content`). So each page's rendered HTML is materialised into the cache
once per page, once per containing section, and once per taxonomy term it belongs to.
Whether this is expensive depends entirely on `tera::Value`'s clone semantics
(shared/refcounted vs deep) — **to be measured, not assumed** (M7/M8).

### 4.3 `Site`

| Collection | Type | Key | Value | Size | Access |
| ---------- | ---- | --- | ----- | ---- | ------ |
| `permalinks` | `std::HashMap` | relative md path | permalink | P+S | built during load; **cloned** into `GetUrl` and `MarkdownFilter`; read per internal link |
| `taxonomies` | `Vec<Taxonomy>` | — | — | T | iterated in queue/sitemap |
| `SITE_CONTENT` | `LazyLock<Arc<RwLock<HashMap<RelativePathBuf, String>>>>` | output path | rendered body | outputs | `serve` only; write lock per output |

### 4.4 Tera-side

| Collection | Owner | Notes |
| ---------- | ----- | ----- |
| `LoadData.result_cache` | `Arc<Mutex<HashMap<u64, Value>>>` | one global mutex; hit path clones the cached `Value` **while holding the lock**; key includes `get_file_time()` → a `stat` per call |
| `Tera` templates | `Site.tera` | immutable during render; `MarkdownFilter` holds a **clone of the whole Tera instance** |
| Tera fn state | `GetUrl`, `GetPage`, … | hold `Arc<RenderCache>`, cloned `Config`, cloned `permalinks` |

---

## 5. Major loops (and their nominal complexity)

| Loop | Location | Iterations | Notes |
| ---- | -------- | ---------- | ----- |
| content walk | `site/lib.rs:196` | files | serial; per directory an extra `WalkDir(max_depth=1)` to find `_index.*` |
| page parse | `site/lib.rs:293` | P | rayon `par_iter` |
| `add_page` | `site/lib.rs:303` | P | serial; validates taxonomies, inserts permalink, `Arc::make_mut` per call |
| ancestor build | `library.rs:342` | S·D | builds a `PathBuf` per component |
| section fixup | `library.rs:378` | S | clones `subsections`/`ancestors` vectors |
| page→section attach | `library.rs:417` | P·(transparent chain) | inner loop walks up transparent parents |
| page_template inheritance | `library.rs:447` | P·D (only if `template` unset) | `content_path.join(ancestor)` allocation per ancestor per page |
| `sort_section_pages` | `library.rs:241` | S sorts of ‖section‖ | each uses rayon `partition` + `par_sort_unstable_by` |
| `sort_section_subsections` | `library.rs:280` | S sorts | same |
| `find_taxonomies` | `library.rs:207` | T·K terms, P memberships | per term: `sort_pages` (rayon) |
| markdown render | `site/lib.rs:480` | P then S | rayon; per page: optional Tera pass + pulldown-cmark + highlighting |
| `fill_backlinks` | `library.rs:182` | L | serial |
| `RenderCache::build` | `cache.rs:59` | P + S + T·K | serial; see §4.2 note |
| internal anchor check | `link_checking.rs:22` | links with anchors | serial; builds a `PathBuf` per link and clones the source path per link |
| job construction | `queue.rs:141` | P + S + A + T·K | serial |
| job execution | `queue.rs:227` | jobs | rayon; render + minify + write per job |
| static copy | `utils/fs.rs:107` | static files | serial `copy_directory` |

---

## 6. Cross-page operations (the interesting ones)

These are the places where producing output for one page touches data belonging to other
pages. They are the candidates for superlinear behaviour.

1. **Section cache values embed all their pages' values** (`cache.rs:125`). Cost scales
   with Σ‖section‖ = O(P) *values*, but each value may be large (contains rendered HTML).
2. **Taxonomy term values embed all member pages' values** (`cache.rs:200`). Cost scales
   with Σ‖term‖ = O(P·taxonomies-per-page).
3. **Backlinks** (`ser.rs:26`): for every serialized page, look up its relative path in
   `backlinks`, resolve each source path, then `sort_by_key` on permalink. O(b log b) per
   page, O(L log L) total.
4. **Translations** (`library.rs:480`): per serialized page, O(#translations).
5. **Siblings** (`cache.rs:84`): second pass over pages, cloning the neighbour's whole
   `Value` into each page's value. Each page therefore embeds up to two other complete
   page values (which themselves embed their own `lower`/`higher`? — no: siblings are
   injected after the first pass, so the embedded neighbour has no siblings of its own.
   Depth is bounded at 1).
6. **Feeds** (`queue.rs:390`): the site feed filters all pages by language — O(P) per
   language; section feeds use the section's own page list.
7. **Sitemap** (`sitemap.rs:59`): O(P+S+T·K) with a `HashSet` dedupe and a final sort,
   plus a clone of each page's `extra` value.
8. **Orphan pages** (`library.rs:474`): O(P) scan per build.
9. **Internal link check** (`link_checking.rs:22`): O(L) with a `PathBuf` build per link.
10. **Path collisions** (`library.rs:96`): O(‖reverse_aliases‖ + T·K).

Nothing in the above list is *nominally* O(P²). The static reading suggests the
architecture is already index-based (`AHashMap` by path) rather than scan-based. The open
questions are therefore about **constant factors and cloning volume**, not about missing
indexes — which is exactly what M4–M8 must quantify.

---

## 7. Shared and global state

| State | Kind | Contention risk |
| ----- | ---- | --------------- |
| `SITE_CONTENT` | `RwLock<HashMap<…>>` global | write lock per rendered output; `serve` only |
| `Site.imageproc` | `Arc<Mutex<Processor>>` | locked per `resize_image`/`get_image_metadata` template call, and for the whole processing phase |
| `LoadData.result_cache` | `Arc<Mutex<HashMap<u64, Value>>>` | locked per `load_data()` call, including the clone of the cached value |
| `LoadData.client` | `Arc<Mutex<Client>>` | remote data only |
| rayon global pool | implicit | nested parallelism: `Queue::process` (par_iter) → sorting helpers (`par_sort`) are not nested, but `sort_pages` inside `find_taxonomies` runs during a serial phase |
| Tera | `&Tera` shared immutably | fine; but `MarkdownFilter` owns a full clone |

---

## 8. Cache-like structures and their invalidation

| Cache | Populated | Invalidated |
| ----- | --------- | ----------- |
| `RenderCache` | after markdown render (`load()`), and by `rebuild_cache()` | wholesale rebuild; `serve` calls it after content changes |
| `Site.permalinks` | during `add_page`/`add_section` | never cleared during a build; `serve` re-creates the `Site` on config change |
| `LoadData.result_cache` | first `load_data` call per key | key embeds file mtime, so touching the file changes the key; entries are never evicted |
| `imageproc` op dedupe | during template render | `prune()` before processing |
| `copy_file_if_needed` | mtime/size comparison per file | per file |

---

## 9. Where output determinism comes from

Rendering happens in a rayon `par_iter` over jobs, so *execution* order is
nondeterministic, but each job writes to its own path, so the output tree is
deterministic. Ordering-sensitive artefacts are made deterministic explicitly:

* sitemap entries are `sort()`ed (`sitemap.rs:128`);
* backlinks are `sort_by_key(permalink)` (`ser.rs:37`);
* section pages/subsections are sorted with a permalink tie-break (`sorting.rs:42`);
* taxonomy terms are sorted by slug then name (`taxonomies.rs:138`);
* page parse errors are sorted by path before being reported (`site/lib.rs:401`).

Any optimization must preserve these tie-breaks exactly.

---

## 10. `serve` rebuild paths (for completeness)

`src/fs_utils.rs` maps filesystem events to `ChangeKind`; `src/cmd/serve.rs` then chooses:

| ChangeKind | Action |
| ---------- | ------ |
| `Content` | `--fast`: `add_and_render_page/section` + `Queue::single_page/single_section`; otherwise full `load()` + `build()` |
| `Templates` | `reload_templates()` + re-render markdown + full rebuild |
| `Sass`, `StaticFiles` | targeted recompile/copy |
| `Config` | recreate the whole `Site` |

`Queue::single_page` renders exactly one output; `single_section` re-renders the section
and (optionally) its pages. Note that even `--fast` re-runs `populate_sections()` for a
section change, which re-sorts every section in the site.

---

## 11. Open questions carried into M2–M10

1. What is the real cost split between load, markdown, cache build, and Tera render?
2. What are `tera::Value` clone semantics, and how much of the build is `Value` cloning?
3. Does `LoadData`'s single mutex serialize parallel rendering on data-heavy sites?
4. How expensive is `minify_html` at scale (it runs inside every write job)?
5. Does the rayon `par_sort` inside per-section sorting cost more than it saves for
   small sections (S = 1.6k sections of ~2 pages each on the reference site)?
6. How does `RenderCache::build` scale — is it linear in P, or in Σ‖section‖ × value size?
7. What fraction of time is `fs::write` + `create_dir_all` per output?
8. Is `find_related_assets` (a `WalkDir` per colocated page) a significant I/O cost?
