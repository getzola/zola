<!-- Derived from ../paper.md (ZPERF-001). Every number here appears there.
     Each post stands alone; numbers may not be introduced here. -->

**1/**
Zola is a static-site generator with a reputation for being fast. That's what
made it worth profiling: no obvious algorithmic disaster to find. We measured it
from 100 to 16,000 pages, plus one real site with 3776 pages and 9.03 GB of
output. Results and the mistakes, below.

**2/**
Against the same binary plus instrumentation, one interleaved session:
wall time −14% to −74% depending on workload, peak memory −40% to −89%.
A 16,000-page build went from 4913 MB to 574 MB.
The real site: −33.3% wall, −35.1% CPU.

**3/**
The most informative number is the one that didn't move. On most workloads total
CPU stayed roughly flat while wall time halved. We didn't make the program
compute less. We stopped it waiting and stopped it allocating.

**4/**
The single largest win wasn't in the codebase. A profile put 34 s of 138 s of
busy CPU inside the platform allocator. Replacing it with mimalloc: −23.7% build
CPU on the real site, and nothing measurable on the small-page fixtures. A
large-page win, reported as one.

**5/**
Four optimizations were rejected on measurement. Parallelising the static file
copy was the instructive failure: on 5000 small files it was 10–30% slower, every
round. Three attempts to parallelise filesystem work, three failures with the
same shape. Metadata operations here anti-parallelise.

**6/**
The most expensive mistake wasn't in any patch. The programme benchmarked
`zola build` throughout and never `zola serve` — the command you actually sit in
front of. `serve` held 9371 MB for a site that builds in 493 MB.

**7/**
Worse, `serve --fast` detected an edit, re-parsed the file, printed "Done in 0ms"
and served the page as it was before the edit. Pre-existing upstream, and
invisible to a benchmark that only times full builds.

**8/**
We nearly missed the memory finding: `ps -o rss` reported 8–20 MB for that
process. Wrong metric on macOS — it compresses an idle process's pages out of
resident memory. `footprint -p` said 9.2 GB.

**9/**
Full paper, evidence manifest and reproduction commands are in the repository.
Work done in a fork; not affiliated with the upstream project.
