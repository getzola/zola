# Scaling curves (M5)

Method: `scripts/perf/scaling.py` over the baseline result files. For every
scenario it computes T(n) medians, consecutive doubling ratios, a log-log
least-squares exponent k in T ∝ n^k, and picks the best one-parameter fit among
O(1), O(log n), O(n), O(n log n), O(n²) by relative RMSE.

A single doubling ratio never decides a classification here; the exponent is
fitted over all measured sizes and cross-checked against the asymptotic
(largest-two-sizes) ratio, because the ~120 ms fixed startup cost depresses the
whole-range exponent on small sites.

## Time

| scenario | 8k/4k ratio | 16k/8k ratio | ms/page at 8k | whole-range k | best fit | verdict |
| -------- | ----------- | ------------ | ------------- | ------------- | -------- | ------- |
| simple-pages | 2.03 | 1.97 | 0.360 | 0.77 | O(n) | linear |
| template-heavy | 2.03 | — | 0.424 | 0.74 | O(n) | linear |
| deep-sections | 1.98 | — | 0.395 | 0.57 | O(n) | linear |
| dense-internal-links | 1.85 | 2.39 | 0.466 | 0.83 | O(n) | linear (see note) |
| data-heavy | 1.90 | — | 0.384 | 0.66 | O(n) | linear |
| many-taxonomies | 2.02 | — | 0.596 | 0.70 | O(n) | linear |
| mixed-realistic | 2.13 | 1.86 | 0.631 | 0.74 | O(n) | linear |
| markdown-heavy | 2.01 | — | 1.177 | 0.87 | O(n) | linear |

**No scenario is superlinear in time.** Every doubling ratio sits in the
1.85–2.13 band, and per-page cost is flat from 2000 pages upwards. The
whole-range exponents of 0.57–0.87 are an artefact of the fixed startup cost,
not sublinear work: at n=100 the build is mostly startup, at n≥2000 it is
mostly per-page work.

The one ratio worth naming is `dense-internal-links` 16k/8k = 2.39. Its 8k point
(1.85) and 16k point (2.39) average out to 2.1 across the two doublings, and its
RSS at 16k is 3.1 GB, so the excess is most plausibly memory pressure rather
than an algorithmic term. It is not currently reproducible as a growth-rate
change; flagged for re-measurement rather than treated as a finding.

This result is important in its own right: **the current architecture is already
index-based**. The static audit found no `for page in pages { for other in pages }`
pattern, and the measurements agree. Optimization effort therefore belongs on
constant factors, serialization and memory — not on asymptotics.

## Memory

This is where scaling actually hurts.

| scenario | KB/page at 8k | RSS at 8k | RSS at 16k | RSS ratio 8k→16k |
| -------- | ------------- | --------- | ---------- | ---------------- |
| simple-pages | 77 | 615 MB | 1064 MB | 1.73 |
| deep-sections | 82 | 654 MB | — | — |
| template-heavy | 87 | 699 MB | — | — |
| dense-internal-links | 207 | 1659 MB | 3141 MB | 1.89 |
| data-heavy | 219 | 1754 MB | — | — |
| markdown-heavy | 321 | 2564 MB | — | — |
| mixed-realistic | 329 | 2633 MB | 5149 MB | 1.96 |
| many-taxonomies | **406** | **3246 MB** | — | — |

Memory is linear in page count, with a per-page constant that varies 5× by
scenario. The ordering is informative:

* `simple-pages` (77 KB/page) is the floor: one `Page`, one serialized
  `tera::Value`, one output string.
* `many-taxonomies` costs 5.3× that. Its only structural difference is 6
  taxonomy terms per page. In `RenderCache::build`, every taxonomy term's
  cached `Value` embeds the **fully re-serialized** value of each member page
  (`cache.rs:200`), and `tera::Value`'s `Serialize` impl walks and rebuilds maps
  and strings rather than sharing the underlying `Arc`s (`tera-2.1.1
  value/mod.rs:1062`). So a page that belongs to 6 terms is materialised 7+
  times.
* `mixed-realistic` and `dense-internal-links` sit in between, consistent with
  section embedding (`cache.rs:125`) plus per-page backlink/sibling values.

At 24 GB of RAM, `many-taxonomies` extrapolates to an OOM somewhere around
50–60k pages, and `mixed-realistic` around 70k. That is the real scaling wall,
and it is a *constant-factor* wall, not an asymptotic one.

## Where the time goes as size grows

From `zola build --timings` on `mixed-realistic-4000` (wall 1.6 s):

| phase | wall | share | parallel? |
| ----- | ---- | ----- | --------- |
| `load` | 939 ms | 65.9% | partly |
| ├ `render markdown` | 346 ms | 24.3% | rayon |
| ├ `build render cache` | **346 ms** | 24.3% | **sequential** |
| ├ `discover + parse sections` | 81 ms | 5.7% | **sequential** |
| ├ `parse pages` | 81 ms | 5.7% | rayon |
| ├ `register tera fns (early)` | 44 ms | 3.1% | sequential |
| └ rest | < 15 ms each | | |
| `build` | 465 ms | 32.6% | |
| └ `render + write outputs` | 464 ms | 32.6% | rayon |

Parallel work, CPU time summed across threads (same run):

| accumulator | CPU | calls | per call |
| ----------- | --- | ----- | -------- |
| markdown render (pages) | 4.10 s | 4000 | 1.0 ms |
| write file | 3.63 s | 4804 | 0.8 ms |
| minify html | 1.00 s | 4802 | 0.2 ms |
| read file | 0.76 s | 4000 | 0.2 ms |
| tera page render | 0.62 s | 4000 | 0.2 ms |
| front matter parse | 0.16 s | 4000 | 0.04 ms |

Two structural observations follow, both confirmed by the CPU profile
(`CPU-PROFILE.md`):

1. `build render cache` is 24% of wall time and runs on **one core**. It scales
   linearly with pages but never uses more than a single thread.
2. `markdown render` reports 4.1 s of CPU inside 346 ms of wall — apparent 12×
   parallelism — but most of that CPU is threads *blocked on a lock*, not doing
   work.
