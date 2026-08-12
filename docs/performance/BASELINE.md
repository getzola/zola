# Baseline measurement (M4)

Everything below is measured, not estimated. Raw data:
`benchmarks/results/m4-pro-12c-24gb-mac16-8/20260811T201125Z-1d46fbad-dirty/{baseline-matrix,baseline-16k,site-vomaste-proxy}.json`.

## Environment

| | |
| --- | --- |
| CPU | Apple M4 Pro, 12 physical / 12 logical cores |
| Memory | 24 GiB |
| OS | macOS 26.2, arm64 |
| rustc / cargo | 1.90.0 |
| zola | 0.23.3 (`d225f3fd`), branch `perf/large-site-scaling` |
| Build profile | `release`, pinned (below) |
| Runner | hyperfine 1 warmup + 3 measured runs per cell |

### Why the build environment is pinned

The developer machine's global `~/.cargo/config.toml` overrides the workspace
release profile and injects target rustflags. Two of those are not benign:

* `[target.aarch64-apple-darwin] rustflags = ["-C", "lto=thin", …]` makes build
  scripts fail to compile (`-C embed-bitcode=no` and `-C lto` are
  incompatible), so `cargo build --release` does not work at all in this
  checkout without intervention;
* `panic = "abort"` and `lto = "thin"` silently produce a different binary from
  what `Cargo.toml` asks for.

`scripts/perf/build.sh` therefore clears `RUSTFLAGS` and `RUSTC_WRAPPER`
(sccache) and pins `lto=true`, `codegen-units=1`, `panic=unwind`, `strip=true`
— i.e. exactly the workspace's declared release profile. All numbers here come
from a binary built that way.

`profile.profiling` (added to `Cargo.toml`) inherits `release` and only adds
`debug = "full"` and `strip = false`. It is used for CPU profiles; it is never
used for timing numbers.

### Measurement protocol

* Each measured run is `zola build --force -o <tmpdir>`, executed with the site
  as CWD; the output directory always lives outside the site.
* A warmup run precedes the measured runs, so every measured run cleans a
  *populated* output directory — the same work a rebuild into an existing
  `public/` does. Skipping this would hide `clean()` entirely, which turns out
  to be one of the largest costs on the reference workload.
* Peak RSS and user/sys split come from one extra run under `/usr/bin/time -l`.
* Cargo compilation is never part of any measurement.

## Reference workload

The motivating site (`~/dev/vomaste.cz`) targets Zola 0.22 and cannot be built
by 0.23.3: 39 of its 40 templates use Tera 1 `{% import %}` macros, removed by
the 0.23 breaking change. It is therefore benchmarked through a
**content-faithful proxy** (`scripts/perf/make_proxy_site.py`): the real content
tree, front matter, internal-link graph, section shape and static files, with
substitute templates. The proxy reproduces the site's per-page `load_data()`
view-model pattern, which turns out to dominate its template cost.

| Property | Value |
| -------- | ----- |
| markdown files | 5368 (3728 pages + 1640 sections) |
| internal `@/` links | 6625 |
| pages with a `view_model` in front matter | 4843 (4695 distinct JSON files, 65 MB) |
| static files | 989 |
| templates referenced from front matter | 38 |
| input content | 6.2 MB |
| generated output | 6544 files, 73 MB |

### Reference proxy results

| metric | value |
| ------ | ----- |
| wall time (median of 3) | **2.087 s** (min 1.963, max 2.100, σ 0.075) |
| user CPU | 0.83 s |
| system CPU | **8.33 s** |
| CPU utilisation | 4.53 cores |
| peak RSS | 373 MB |

The user/system split is the headline: **the reference-shaped workload spends
ten times more CPU in the kernel than in user space.** Phase timings
(`zola build --timings`, rebuild into a populated directory) explain where:

| phase | time | share |
| ----- | ---- | ----- |
| `clean output dir` | 663 ms | 36.3% |
| `render + write outputs` | 532 ms | 29.2% |
| `discover + parse sections` | 232 ms | 12.7% |
| `copy static` | 170 ms | 9.3% |
| `parse pages` | 113 ms | 6.2% |
| `build render cache` | 77 ms | 4.2% |
| `populate sections` | 16 ms | 0.9% |
| everything else | < 10 ms each | — |

Of those, `clean output dir`, `discover + parse sections` and `copy static` are
**fully sequential**: 1.07 s of the ~1.8 s build runs on one core.

## Synthetic scenario baseline

Median wall time in seconds, 1 warmup + 3 runs:

| pages | simple | template-heavy | deep-sections | dense-links | data-heavy | many-taxonomies | mixed-realistic | markdown-heavy |
| ----- | ------ | -------------- | ------------- | ----------- | ---------- | --------------- | --------------- | -------------- |
| 100 | 0.128 | 0.136 | 0.261 | 0.141 | 0.188 | 0.228 | 0.263 | 0.213 |
| 250 | 0.174 | 0.196 | 0.316 | 0.211 | 0.228 | 0.305 | 0.326 | 0.391 |
| 500 | 0.258 | 0.295 | 0.395 | 0.305 | 0.313 | 0.468 | 0.453 | 0.677 |
| 1000 | 0.419 | 0.498 | 0.568 | 0.549 | 0.483 | 0.688 | 0.705 | 1.253 |
| 2000 | 0.753 | 0.874 | 0.901 | 0.961 | 0.857 | 1.240 | 1.303 | 2.396 |
| 4000 | 1.417 | 1.675 | 1.602 | 2.015 | 1.619 | 2.358 | 2.366 | 4.691 |
| 8000 | 2.882 | 3.394 | 3.163 | 3.727 | 3.070 | 4.764 | 5.050 | 9.418 |
| 16000 | 5.689 | — | — | 8.915 | — | — | 9.368 | — |

Peak RSS in MB:

| pages | simple | template-heavy | deep-sections | dense-links | data-heavy | many-taxonomies | mixed-realistic | markdown-heavy |
| ----- | ------ | -------------- | ------------- | ----------- | ---------- | --------------- | --------------- | -------------- |
| 100 | 174 | 177 | 177 | 188 | 192 | 204 | 190 | 204 |
| 1000 | 219 | 231 | 223 | 349 | 364 | 549 | 431 | 468 |
| 4000 | 383 | 425 | 401 | 901 | 953 | 1696 | 1369 | 1362 |
| 8000 | 615 | 699 | 654 | 1659 | 1754 | **3246** | 2633 | 2564 |
| 16000 | 1064 | — | — | 3141 | — | — | **5149** | — |

## What the baseline already says

1. **Fixed startup cost is ~110–130 ms and ~170 MB RSS**, before a single page
   is processed. An empty-ish 100-page build is 128 ms of which most is
   startup. The RSS floor is the syntax-highlighting registry.
2. **Time is linear in page count** for every scenario (see `SCALING.md`);
   nothing measured here is quadratic.
3. **Memory is linear but with a very large constant**: 77 KB/page for
   `simple-pages`, ~330 KB/page for `mixed-realistic`, ~406 KB/page for
   `many-taxonomies`. A 16k-page mixed site needs 5.1 GB.
4. **Markdown rendering dominates CPU when there are code blocks**
   (`markdown-heavy` is 2–3× everything else per page), and the CPU profile
   shows most of it is *blocked on a lock*, not computing.
5. **On the reference-shaped workload the build is I/O bound**, and the largest
   single item is deleting the previous output tree.
