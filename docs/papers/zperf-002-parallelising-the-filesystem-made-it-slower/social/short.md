<!-- derived from ../paper.md; ZPERF-002 -->

Four optimizations we implemented and threw away, in a fork of the static-site
generator Zola. Three were filesystem work: caching the directories a build has
already created, parallelising the delete of the previous output, and
parallelising the copy of the static tree. All three failed the same way — the
parallel copy was 10–30% slower in every round on the workload it should have
won, 5000 files of 1 KB, and renaming the output aside to delete it in the
background took a 929.8 ms phase to 0.2 ms while changing wall time by nothing.
On this platform, filesystem metadata operations do not parallelise; they
anti-parallelise. One machine, one filesystem, and the numbers are transcribed
phase timings rather than committed benchmark artifacts, because the rejected
code was reverted — details and caveats in ZPERF-002.
