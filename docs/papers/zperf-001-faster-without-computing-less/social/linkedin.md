<!-- Derived from ../paper.md (ZPERF-001). Every number here appears there.
     If a figure changes, regenerate this file from the paper — never edit both. -->

**We made a static-site generator 14–74% faster without making it compute less.**

Zola already has a reputation for speed, which is what made it interesting: there
was no obvious algorithmic disaster to find. So we instrumented it, profiled it,
and measured a matrix of workloads from 100 to 16,000 pages plus one real site
with 3776 pages and 9.03 GB of output — then optimized only what the
measurements pointed at.

Against the same binary plus instrumentation, in one interleaved session:

• wall time down 14% to 74%, depending on workload
• peak memory down 40% to 89% — a 16,000-page build went from 4913 MB to 574 MB
• the real site: −33.3% wall, −35.1% CPU

But the number that taught us the most was the one that barely moved. On most
workloads **total CPU stayed roughly flat while wall time halved**. We hadn't
made the program do less work. We'd stopped it waiting and stopped it
allocating: a mutex that serialised twelve threads onto one, a phase that
materialised a copy of every page for every container it belonged to, a lock
held across file I/O.

Three things worth passing on:

**The biggest single win wasn't in the codebase.** A profile showed 34 s of 138 s
of busy CPU inside the platform allocator itself. Swapping it for mimalloc took
23.7% off build CPU on the real site — and did nothing at all on the small-page
fixtures. Large-page win, not a general one. Reading the source would never have
found it.

**Four optimizations were rejected on measurement.** Parallelising the static
file copy was the instructive one: on 5000 small files it was 10–30% *slower*,
every round. Three separate attempts to parallelise filesystem work all failed
the same way. Metadata operations on this platform don't parallelise — they
anti-parallelise.

**The most expensive mistake was a scope decision.** The whole programme
benchmarked `zola build` and never once `zola serve` — the command developers
actually sit in front of. When we finally looked: `serve` held 9371 MB for the
site that builds in 493 MB, and `serve --fast` had a rebuild path that printed
"Done in 0ms" and then served the page as it was *before* the edit. Both
pre-existing upstream, both invisible to a benchmark that only times full builds.

We nearly missed the memory finding entirely, because `ps -o rss` reported 8–20
MB for that process. It's the wrong metric on macOS. `footprint -p` said 9.2 GB.

Full write-up, evidence manifest and reproduction commands are in the repository.
Work done in a fork; not affiliated with the upstream Zola project.
