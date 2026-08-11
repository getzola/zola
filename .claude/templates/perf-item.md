# PERF-NNN — <one line: what is slow, where>

Copy the parts you need into `docs/performance/HOTSPOTS.md`. An item is not
admissible without the **Evidence** section: a hotspot that comes from reading
code is a guess, and guesses have been wrong on this codebase before.

**Location.** `components/<crate>/src/<file>.rs:<line>` — the function or loop.

**Problem.** What the code does that costs more than it should. Describe the
mechanism, not the symptom.

**Evidence.** Where the number comes from. One of:

* `docs/performance/BASELINE.md` or a file under `benchmarks/results/<sha>/`
* a CPU profile (`docs/performance/CPU-PROFILE.md`) with sample counts
* an allocation profile (`docs/performance/ALLOCATIONS.md`)
* a scaling curve (`docs/performance/SCALING.md`)

State the workload it was measured on and what fraction of the build it is.

**Current complexity.** Including the serialization factor — work that is
O(n) but forced onto one thread is a different problem from work that is O(n²).

**Expected complexity.** What it becomes after the change.

**Proposed change.** Concrete enough to implement: which function, which data
structure, what happens to the lock/allocation/syscall being removed.

**Correctness risk.** Low / Medium / High, and what specifically could change
in the output.

**Benchmark.** The exact scenario and size that will show the difference, plus
the command:

```
scripts/perf/run.sh site benchmarks/sites/<scenario>-<n>
```

**Priority.** P0 catastrophic scaling · P1 major build cost ·
P2 significant allocation/I/O cost · P3 micro optimization.
