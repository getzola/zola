# Real-site measurement: vomaste.cz

The motivating workload, measured for real rather than through a proxy.

> **Update (2026-08-12).** The site has since adopted the migration below into
> its own repository: it now has no Tera 1 macros, no `{% import %}`, no
> shortcodes directory, and no `filter()`/`concat()` calls. The benchmark copy
> is therefore no longer a migrated derivative — it is a straight `rsync` of the
> site's working tree, and it builds unmodified. Everything from here down
> records how it got there, and stays because the byte-equivalence verification
> is the evidence that the migration did not change the site.

## Getting it to build at all

The site targets Zola 0.22: 39 of its 40 templates use Tera 1 `{% import %}`
macros, and it has 5 shortcodes with 198 invocations across 84 markdown files.
Zola 0.23 removed both. It was therefore migrated mechanically (see
`benchmarks/proxies/vomaste-0.23-migration-deliverable/README.md`, which is
gitignored because it contains the site's own content):

* 42 macros → components, 77 `{% import %}` removed, 310 call sites rewritten;
* 4 shortcodes → components with body slots, 198 content invocations rewritten;
* 48 `filter(attribute=…, value=…)` → explicit loops;
* 22 `concat(with=…)` → array spread;
* 255 optional-chaining fixes, 71 typed defaults removed, 387 arguments added
  at call sites.

**Verification**: the 0.22 build and the migrated 0.23 build produce **6500 of
6592 output files byte-identical** (98.6%), with no missing and no extra files.
The 92 differences have documented causes (JSON-LD quote escaping through a
component boundary, CommonMark ending an HTML block at a blank line inside a
component body, and one unexplained link rewrite in `docs-viewer.html`).

## Build time: 0.22 vs 0.23

Interleaved runs, same machine, same content, output to a temp directory,
`--force` so each run cleans a populated tree:

| round | Zola 0.22 (original templates) | Zola 0.23 (migrated, this branch) |
| ----- | ------------------------------ | --------------------------------- |
| 1 | 218.63 s | 30.85 s |
| 2 | 266.92 s | 37.53 s |
| 3 | 252.20 s | 34.87 s |
| **median** | **252.20 s** | **34.87 s** |

**7.2× faster.** The comparison is not a controlled A/B of a single change — it
bundles the 0.22→0.23 engine work (the `RenderCache`, Tera 2), this branch's
PERF-001 fix, and any cost difference introduced by the migration itself. It is
the number that matters in practice: the site's build goes from four minutes to
half a minute.

## Where the remaining 35 seconds go

`zola build --timings` on the migrated site (33.6 s wall):

| phase | wall | share |
| ----- | ---- | ----- |
| `build` | 33.10 s | 98.7% |
| ├ `render + write outputs` | **31.64 s** | **94.4%** |
| ├ `clean output dir` | 1.31 s | 3.9% |
| └ `copy static` | 0.15 s | 0.4% |
| `load` (everything: discovery, parse, markdown, cache) | 0.41 s | 1.2% |

Parallel work, CPU summed across threads:

| accumulator | CPU | calls | per call |
| ----------- | --- | ----- | -------- |
| `out: minify html` | **191.6 s** | 5601 | **34.2 ms** |
| `render: page` | 121.7 s | 3776 | 32.2 ms |
| `render: section` | 52.0 s | 1640 | 31.7 ms |
| `out: write file` | 14.0 s | 5603 | 2.5 ms |
| `parse: read file` | 0.58 s | 3776 | 0.2 ms |
| markdown render (pages + sections) | 0.03 s | 5416 | ~0 ms |

Zola's content pipeline — discovery, front matter, markdown, the link graph,
the render cache — costs **0.41 s out of 35 s** on a 3776-page site. Everything
else is producing and writing the HTML.

## Why: the output is 9 GB

| metric | value |
| ------ | ----- |
| HTML files | 5601 |
| total HTML | **9.0 GB** |
| average page | **1563 KB** |
| largest page | 3.4 MB |
| input markdown | 6.2 MB |

Composition of a representative page (`dossiers/alena-schillerova/claims/clm-06`,
1600 KB after minification):

| block | size | share |
| ----- | ---- | ----- |
| `<nav>` blocks (sidebar navigation tree) | **1407 KB** | **88%** |
| `<main>` (the page's actual content) | 3 KB | 0.2% |
| everything else (head, scripts, footer) | ~190 KB | 12% |

The full navigation tree — every dossier, every registry, every entity — is
inlined into every page. The tree grows with the site, and it is emitted once
per page, so **the site's total output grows quadratically with the number of
dossiers**. At 3776 pages that is 9 GB of HTML of which ~8.9 GB is the same
navigation repeated.

That is what the build spends its time on: rendering 1.5 MB of markup per page
(174 s CPU) and minifying it (192 s CPU).

### What this implies

* **For the site**: the largest available win is not in Zola. Rendering the
  navigation subtree relevant to the current section, or moving the tree to a
  fetched JSON payload, would cut build time by roughly an order of magnitude
  *and* cut page weight by ~500×. A 1.6 MB HTML page where 3 KB is content is
  also a user-facing performance problem.
* **For Zola**: this workload is dominated by `minify_html` and Tera rendering
  of very large outputs, not by the page graph. It confirms PERF-002 (giallo)
  and PERF-005 (render cache) are irrelevant *here* while PERF-004 (clean,
  1.3 s) and the minifier cost are what a site like this feels.
* **For the benchmark suite**: no synthetic scenario currently produces
  megabyte-scale pages. A `huge-pages` scenario would cover this shape.

## Reproducing

```bash
# 1. copy the site (never touches the original); no migration step any more
rsync -a --delete --exclude node_modules/ --exclude public/ --exclude test-results/ \
      --exclude .git/ ~/dev/vomaste.cz/ benchmarks/proxies/vomaste-live/

# 2. benchmark
python3 scripts/perf/bench.py site --path <migrated-copy> --label vomaste --runs 3

# 3. phase breakdown
cd <migrated-copy> && zola build --force --timings -o /tmp/out
```
