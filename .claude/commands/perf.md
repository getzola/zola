---
description: Work a PERF-* item under the measurement doctrine
argument-hint: [PERF-NNN | question]
---

Performance work: $ARGUMENTS

Read `.claude/workflows/performance.md` first — the rules there are what make a
result believable, and several of them exist because the obvious approach
already produced a wrong answer on this machine.

1. Read the item in `docs/performance/HOTSPOTS.md` and its evidence in
   `BASELINE.md` / `SCALING.md` / `CPU-PROFILE.md` / `ALLOCATIONS.md`.
2. Reproduce the *before* measurement yourself. If you cannot reproduce it, stop
   and report that.
3. Keep the baseline binary before changing anything.
4. Implement one hotspot.
5. Measure interleaved (alternate baseline and candidate), report every round.
6. Run output equivalence. It must say IDENTICAL.
7. Record peak RSS before and after.
8. Append the entry to `docs/performance/OPTIMIZATIONS.md`, then
   `scripts/dev.sh generate`.

If the number does not move, say so and keep the finding. "No measurable win on
workload X" is a result worth committing to the record.
