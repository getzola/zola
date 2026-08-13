# Final report — large-site performance program

Answers to the twenty questions the program set, with the measurement that
supports each. Everything here is reproducible from `scripts/perf/`; the
underlying data is in `BASELINE.md`, `SCALING.md`, `CPU-PROFILE.md`,
`ALLOCATIONS.md`, `REAL-SITE.md` and `OPTIMIZATIONS.md`.

Machine: Apple M4 Pro, 12 cores, 24 GiB, macOS 26.2, rustc 1.90, release
profile pinned by `scripts/perf/build.sh`.

## 1. Where is build time spent?

`mixed-realistic-4000`, current tree, 1.23 s wall:

| phase | wall | share |
| ----- | ---- | ----- |
| `render + write outputs` | 715 ms | 58.0% |
| `render markdown` | 292 ms | 23.7% |
| `parse pages` | 77 ms | 6.3% |
| `discover + parse sections` | 52 ms | 4.2% |
| `build render cache` | 30 ms | 2.5% |
| `site::new` (config + templates) | 31 ms | 2.5% |
| everything else | < 13 ms each | |

Parallel work, CPU summed across threads: file writing 6.96 s, markdown
rendering 3.47 s, minification 0.82 s, file reading 0.80 s, Tera page render
0.42 s.

## 2. How does every major phase scale with page count?

Linearly. Doubling ratios across 100 → 16 000 pages sit in the 1.85–2.13 band
for every scenario (`SCALING.md`). Per-page cost is flat from 2000 pages up.

## 3–4. Which operations are superlinear, and why?

**None in Zola.** The static audit found no `for page { for page }` pattern, no
linear scan behind a keyed lookup, and the measurements agree. The architecture
was already index-based before this work.

The one quadratic thing found is in the *reference site*, not the generator: it
inlines the whole navigation tree into every page, so total output grows with
pages × dossiers. It emits 9.0 GB of HTML for 6.2 MB of markdown, and 88% of a
typical page is that duplicated navigation (`REAL-SITE.md`).

## 5. Which data structures cause repeated scans?

None. `Library` keys pages and sections by absolute path in `AHashMap`s, and
`RenderCache` adds canonical-path and per-language indexes. The costs found
were constant factors, not lookups.

## 6. Which values were unnecessarily recomputed?

Two, both fixed:

* every page's serialized `Value` was rebuilt through serde once per section
  and once per taxonomy term that contained it (PERF-005a);
* the giallo grammar registry was deep-copied into four Tera functions that
  never highlight anything (PERF-010).

## 7. Which allocations dominate?

Before: `RenderCache::build` — 7.4–10 M allocations, 0.96–1.2 GB, 86% of the
live heap. After PERF-005a and PERF-010 the peak heap on
`mixed-realistic-4000` is 209 MB against 1371 MB at the start of this work.
What remains is the rendered HTML itself plus per-output churn in the write
path, which is released as it goes.

## 8. Which filesystem operations dominate?

Writing outputs: 4804 writes at ~1.4 ms each on `mixed-realistic-4000`. Two
attempts to remove the redundant `create_dir_all` calls around them changed
nothing measurable (PERF-003, rejected twice) — the cost is the file creation
and write themselves. Deleting the previous output is next (up to 36% of wall
on a 9 GB output tree), and parallelising *that* was measured slower on APFS
(PERF-004, rejected).

## 9. How much time is template rendering?

0.42 s of CPU for 4000 pages on `mixed-realistic-4000` — 0.1 ms per page, about
6% of the parallel work. On a data-driven site it is the dominant template cost
only through `load_data`: 73.8% of busy CPU on the reference workload before
PERF-001.

## 10. How much time is Markdown rendering?

292 ms of wall / 3.47 s of CPU on `mixed-realistic-4000`. Almost all of it is
syntax highlighting: with `[markdown.highlighting]` removed, the same phase on
`markdown-heavy-2000` drops from 3.96 s to 28 ms. `pulldown-cmark` itself is
0.4% of busy CPU.

## 11. How much time is internal-link processing?

Under 1%: `check internal links` is 3.5 ms for 4000 pages, `fill backlinks`
6.5 ms. `dense-internal-links` (40 links per page) is 0.47 ms/page against
0.36 ms/page for `simple-pages`.

## 12. How much time is taxonomy construction?

`populate taxonomies` is 4.6 ms at 4000 pages. Taxonomies used to be expensive
*downstream* — every membership cost ~67 KB of heap and ~60 ms per 2000 pages
in the render cache — which PERF-005a removed.

## 13. How much time is search generation?

Not on the critical path for the workloads measured: both the reference site
and the synthetic scenarios keep `build_search_index = false`, and the phase is
skipped entirely. It remains unmeasured and is called out as such.

## 14. How much time is output I/O?

58% of wall on `mixed-realistic-4000` (`render + write outputs`, which fuses
rendering, minification and writing), of which the write syscalls are 6.96 s of
the 8.5 s of CPU in that phase.

## 15. Which architectural changes produced the largest wins?

| change | effect |
| ------ | ------ |
| PERF-005a — stop re-serializing page values | cache phase −94%, wall −19…−25%, peak RSS −72…−86% |
| PERF-010 — share the highlighting registry | flat ~100 MB off every build, register phase 41 ms → 0.6 ms |
| PERF-001 — release the `load_data` lock during I/O | page-render CPU 6.5 s → 1.8–2.7 s on the reference workload; −23% wall on `data-heavy` |
| determinism fix | −9% wall, −15% RSS as a side effect of replacing hash maps with order-preserving ones |
| PERF-006 — one directory read per directory | discovery −35% on section-dense trees, ~1% of the build |
| PERF-012 — mimalloc as the global allocator | −24% build CPU on a site whose pages are megabytes; nothing on small-page sites |

Measured end to end — the binary this program started from against the binary it
ended with, in one interleaved session — the result is **−14% to −74% wall and
−40% to −89% peak memory** across the nine scenarios. The full table is in
`OPTIMIZATIONS.md` under "The whole program, measured in one session"; it
supersedes the earlier cumulative table, whose before/after columns came from
different sessions and understated the result by about half.

## 16. Full-build speedup on the representative ~4k-page site

The reference site could not be built by the version under test at all — it
targets Zola 0.22 — so it was migrated first (`REAL-SITE.md`). With that:

* Zola 0.22, original templates: **252 s**
* Zola 0.23 + this work, migrated templates: **~25 s**

Within this program alone — its own starting binary against the current one, in
one interleaved session — the same site is **44.3 s → 32.2 s (−33% wall), 383 s
→ 247 s CPU (−35%), 676 MB → 504 MB (−26%)**, unanimous across rounds. Those
seconds were measured on a machine carrying unrelated load; idle, the current
binary builds the site in **30.2 s**. The rest of the gap to 252 s is the
0.22 → 0.23 engine work and the template migration.

## 17. Behaviour at 8k and 16k pages

`mixed-realistic`: 16 000 pages **9.57 s → 4.22 s** and **4913 MB → 574 MB**,
still linear in page count. The memory wall that would have hit around 50–70k
pages is gone: at the old rate a 100 000-page site needed ~30 GB, at the new one
it needs ~3.6 GB.

## 18. Has peak memory improved or regressed?

Improved on every scenario measured, by 40–89%, and the improvement grows with
the site: 1307 MB → 217 MB at 4000 pages (`mixed-realistic`), 4913 MB → 574 MB
at 16 000. Per-page cost fell from ~307 KB to ~36 KB.

Memory, not CPU, is where this program moved the needle furthest — and it is
the axis that decides whether a site can be built at all.

**With one large caveat, found late and not addressed**: all of the above is
`zola build`. `zola serve` keeps every rendered page in memory and needs
**9242 MB** for the same site that builds in 493 MB — 19× more, and the largest
memory figure anywhere in this program. See PERF-016 in `HOTSPOTS.md`. Nothing
here improved it, because nothing here measured it.

## 19. Are generated outputs equivalent?

Yes, and the gate is now meaningful. Every optimization commit was verified
byte-for-byte with `scripts/perf/compare_output.py` across several scenarios.
The one deliberate exception is the determinism fix, which is *about* output:
it changes 487 of 1547 files on `mixed-realistic-1000`, and all 487 contain
exactly the same characters — pure reordering. Before that fix, two runs of the
same binary already differed, which is why the gate had to be repaired first.

## 20. What bottleneck is dominant now?

1. ~~**Syntax highlighting (PERF-002)**~~ — **fixed**, in giallo rather than in
   Zola: one `RegSet` per thread instead of one `Mutex<RegSet>` shared by all.
   The markdown phase drops 83–84% and `markdown-heavy` builds 61%; the phase
   now scales 5.9× on 12 threads where it previously got slower with more
   cores. Patch and numbers in `OPTIMIZATIONS.md`; Zola picks it up by bumping
   the dependency. This makes **output writing** the largest remaining item.
2. **Output writing** — 1.4 ms per file, filesystem-bound, two fixes rejected.
3. **Deleting the previous output** — up to 36% of wall on a large output tree,
   and *not* recoverable: parallel deletion is slower on APFS, and renaming the
   tree aside to delete it in the background removes the phase from the
   timeline (930 ms → 0.2 ms) without changing wall time at all, because the
   build already saturates the machine. Both measured, both rejected.
4. **For the reference site specifically**: none of the above. Its build is
   dominated by producing 9 GB of HTML, 88% of which is the same navigation
   tree repeated on every page. The largest available win for that site is in
   the site, not in Zola.

## What was rejected, and why that matters

* **PERF-003** (cache created directories) — shared-lock and thread-local
  variants both measured; neither moved the write path.
* **PERF-004** (output cleaning) — two approaches. Parallel deletion is
  measurably slower on APFS. Renaming the tree aside and deleting it in the
  background does exactly what it promises to the phase timeline and nothing at
  all to wall time: the work is moved, not removed, and the build already uses
  every core and the same disk. 8 interleaved rounds on the case that should
  have shown it best: +1.8% median.
* An early cumulative table was thrown away after a full disk made builds fail
  fast and the numbers look excellent.
* A first version of the determinism test passed by luck with three keys; it
  was rewritten with twenty so that it actually fails without the fix.

These are recorded in full because the program asked for the negative results,
and because each of them would otherwise look like an obvious thing to try.
