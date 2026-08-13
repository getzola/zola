<!-- Derived from ../paper.md (ZPERF-001). Every number here appears there. -->

We profiled a static-site generator at 4,000–16,000 pages instead of guessing at
it: wall time down 14% to 74%, peak memory down 40% to 89%, and on a real
3776-page site −33.3% wall and −35.1% CPU. The interesting part is that total CPU
barely moved on most workloads — the work wasn't reduced, the waiting and the
allocating were. The largest single win wasn't in the codebase at all: 34 s of
138 s of busy CPU was inside the platform allocator. Four optimizations were
rejected on measurement, one of them 10–30% slower than the serial code it
replaced. And the programme's own scope was wrong: it benchmarked `build` for its
entire length and never `serve`, which held 9371 MB for a site that builds in
493 MB.
