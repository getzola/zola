# zola (né Gutenberg) <img src="docs/static/logos/Zola-logo-main-coffee.svg" align="right" alt="zola logo" width="30%"/>

[![Build Status](https://dev.azure.com/getzola/zola/_apis/build/status/getzola.zola?branchName=master)](https://dev.azure.com/getzola/zola/_build/latest?definitionId=1&branchName=master)
![GitHub all releases](https://img.shields.io/github/downloads/getzola/zola/total)

A fast static site generator in a single binary with everything built-in.

To find out more see the [Zola Documentation](https://www.getzola.org/documentation/getting-started/overview/), look
in the [docs/content](docs/content) folder of this repository or visit the [Zola community forum](https://zola.discourse.group).

---

## About this fork

This is a fork of [getzola/zola](https://github.com/getzola/zola) carrying a
**large-site performance program**: an evidence-driven effort to make builds of
sites with thousands to tens of thousands of pages faster and, above all,
smaller in memory. Every change here is backed by a before/after measurement and
by byte-for-byte output equivalence against the version it replaces.

Everything it produces — the harness, the evidence, the rejected experiments —
lives in [`docs/performance/`](docs/performance/README.md). Start with
[`FINAL-REPORT.md`](docs/performance/FINAL-REPORT.md).

### What changed, and what it bought

Measured on an Apple M4 Pro (12 cores, 24 GiB), release builds, in a single
interleaved session: the two binaries alternate within every round and swap
order between rounds, and each figure is the paired per-round delta. "Before" is
upstream 0.23.3 with only the measurement instrumentation added. Every wall
figure below is unanimous across rounds.

| workload | wall time | peak memory |
| -------- | --------- | ----------- |
| `mixed-realistic` 4 000 pages | 1.92 s → **1.08 s (−46%)** | 1307 MB → **217 MB (−83%)** |
| `mixed-realistic` 16 000 pages | 9.57 s → **4.22 s (−56%)** | 4913 MB → **574 MB (−88%)** |
| `many-taxonomies` 4 000 pages | 1.93 s → **0.94 s (−51%)** | 1618 MB → **180 MB (−89%)** |
| `markdown-heavy` 4 000 pages | 7.18 s → **1.91 s (−74%)** | 1297 MB → **505 MB (−61%)** |
| `data-heavy` 4 000 pages | 2.70 s → **1.31 s (−52%)** | 909 MB → **549 MB (−40%)** |
| `dense-internal-links` 4 000 pages | 2.25 s → **1.15 s (−43%)** | 860 MB → **306 MB (−64%)** |
| `template-heavy` 4 000 pages | 1.20 s → **1.08 s (−23%)** | 406 MB → **169 MB (−58%)** |
| `simple-pages` 4 000 pages | 1.07 s → **0.84 s (−22%)** | 366 MB → **144 MB (−60%)** |
| `deep-sections` 4 000 pages | 1.10 s → **0.95 s (−14%)** | 383 MB → **160 MB (−58%)** |
| a real 3 776-page site, 9 GB of output | 44.3 s → **32.2 s (−33%)** | 676 MB → **504 MB (−26%)** |

Memory per page fell from roughly **307 KB to 36 KB**. A 16 000-page site that
needed 4.9 GB now needs 0.57 GB, which removes the wall that would otherwise
have stopped such sites somewhere around 50–70k pages.

(The real site's absolute seconds are inflated — the machine was busy with
unrelated work during that pair of runs, and the same binary builds it in 30.2 s
on an idle machine. Interleaving is what makes the −33% trustworthy anyway.)

Total CPU time barely moves in most of those rows, and that is the honest summary
of what this work was: it did not make Zola execute fewer instructions, it made
Zola **stop waiting and stop allocating** — a mutex that serialised twelve
threads onto one, a serial phase that copied every page into every container it
belonged to, a lock held across file I/O, and an allocator that could not keep up
with megabyte-sized strings.

The individual changes, each with its own measurement:

| | change | effect |
| --- | ------ | ------ |
| PERF-005a | `RenderCache` reuses `Arc`-backed page values instead of re-serializing them into every section and taxonomy term | cache phase −94%, peak RSS −72…−86% |
| PERF-002 | syntax highlighting gets a regset per thread instead of sharing one behind a mutex (in the vendored dependency, see below) | markdown phase −84%, `markdown-heavy` builds −64% |
| PERF-010 | the highlighting registry is shared behind an `Arc` rather than deep-copied into four Tera functions | flat ~100 MB off every build |
| PERF-012 | mimalloc replaces the platform allocator, which was taking a quarter of the CPU on sites whose pages are megabytes rather than kilobytes | real site: build CPU −24%, peak RSS −10%; no effect on small-page sites |
| PERF-001 | `load_data` releases its cache lock before doing I/O and parsing, instead of holding it throughout | page-render CPU 6.5 s → 1.8 s on a data-driven site |
| PERF-006 | the content walk reads each directory once instead of twice | discovery −35% on section-dense trees |
| — | **builds are reproducible**: maps reaching templates iterate in a stable order | −9% wall, −15% RSS as a side effect |

### Behaviour differences from upstream

Two, both deliberate and both in [`CHANGELOG.md`](CHANGELOG.md):

* **Deterministic output.** Upstream produces different bytes on two runs of the
  same binary for any page with more than one taxonomy, because `page.taxonomies`
  and Tera's own maps were hash-ordered. Here they iterate in a stable order and
  `page.taxonomies` is sorted by name. This changes output for templates that
  iterate a map — by reordering it, never by changing its content.
* **`zola build --timings`.** A developer diagnostic that prints a hierarchical
  breakdown of every build phase plus per-item costs inside the parallel ones.
  Disabled state costs one relaxed atomic load per instrumentation point.

Not a behaviour difference but worth knowing when you build it: the binary uses
**mimalloc** as its global allocator, via a default-on `mimalloc` feature.
`cargo build --release --no-default-features` restores the platform allocator.
No new build dependency — oniguruma already required a C toolchain — and it was
verified against musl, which is a release target.

### The vendored dependency

[`vendor/giallo`](vendor/README.md) carries `getzola/giallo@5e19db8` plus one
patch: giallo shared a single `Mutex<RegSet>` across all worker threads, so
syntax highlighting did not parallelise at all — the markdown phase was *slower*
on twelve threads than on one. The patch
([`docs/performance/giallo-thread-local-regset.patch`](docs/performance/giallo-thread-local-regset.patch))
gives each thread its own regset.

It is pulled in with `[patch.crates-io]` and belongs upstream; `vendor/README.md`
records where it came from and the four steps to drop it once a giallo release
contains the fix.

### Reproducing any of this

```bash
scripts/perf/run.sh build          # release binary with a pinned profile
scripts/perf/run.sh quick          # ~1 minute smoke run
scripts/perf/run.sh baseline       # the full scenario × size matrix
scripts/perf/run.sh scaling benchmarks/results/<hardware>/<commit>/baseline-matrix.json --markdown
scripts/perf/run.sh ab <a-bin> <b-bin> <site>...   # interleaved A/B, paired per-round verdict
```

The table above is reproduced by building the binary at commit `9ec4407a` and
running `run.sh ab` against the current one.

The generator produces byte-identical sites from a seed, results are filed under
`benchmarks/results/<hardware>/<commit-utc>-<sha>/` so they group by machine and
sort by commit date, and `scripts/perf/compare_output.py` is the output-equivalence
gate every change had to pass. [`docs/performance/README.md`](docs/performance/README.md)
explains the harness.

### Public documentation still to write

The site under `docs/content/` is user-facing prose and is written by humans;
two pages describe behaviour this fork changed and have **not** been updated:

* `documentation/templates/overview.md` — that iterating a map in a template now
  has a defined order, and that `page.taxonomies` comes out sorted by name.
* `documentation/getting-started/cli-usage.md` — the `zola build --timings`
  flag, if it is to be user-facing rather than a developer diagnostic.

### Working in this fork

`scripts/dev.sh quality` is the one command that answers "is the branch healthy"
(format, the clippy ratchet, the whole test suite). [`AGENTS.md`](AGENTS.md) has
the engineering rules, [`CLAUDE.md`](CLAUDE.md) the code map.

---

This tool and its template engine [tera](https://keats.github.io/tera/) were born from an intense dislike of the (insane) Golang template engine and therefore of
Hugo that I was using before for 6+ sites.

## List of features

- [Single binary](https://www.getzola.org/documentation/getting-started/cli-usage/)
- [Syntax highlighting](https://www.getzola.org/documentation/content/syntax-highlighting/)
- [Sass compilation](https://www.getzola.org/documentation/content/sass/)
- Assets co-location
- [Multilingual site support](https://www.getzola.org/documentation/content/multilingual/) (Basic currently)
- [Image processing](https://www.getzola.org/documentation/content/image-processing/)
- [Themes](https://www.getzola.org/documentation/themes/overview/)
- [Shortcodes](https://www.getzola.org/documentation/content/shortcodes/)
- [Internal links](https://www.getzola.org/documentation/content/linking/)
- [External link checker](https://www.getzola.org/documentation/getting-started/cli-usage/#check)
- [Table of contents automatic generation](https://www.getzola.org/documentation/content/table-of-contents/)
- Automatic header anchors
- [Aliases](https://www.getzola.org/documentation/content/page/#front-matter)
- [Pagination](https://www.getzola.org/documentation/templates/pagination/)
- [Custom taxonomies](https://www.getzola.org/documentation/templates/taxonomies/)
- [Search with no servers or any third parties involved](https://www.getzola.org/documentation/content/search/)
- [Live reload](https://www.getzola.org/documentation/getting-started/cli-usage/#serve)
- Deploy on many platforms easily: [Netlify](https://www.getzola.org/documentation/deployment/netlify/), [Vercel](https://www.getzola.org/documentation/deployment/vercel/), [Cloudflare Pages](https://www.getzola.org/documentation/deployment/cloudflare-pages/), etc

## License

This project contains code under multiple licenses.

Code introduced after version 0.22 is licensed under the EUPL-1.2.
Code that existed prior to commit 3c9131db0d203640b6d5619ca1f75ce1e0d49d8f remains licensed under the MIT License, including in later versions of the project.

See LICENSE and LICENSE-MIT for details.
