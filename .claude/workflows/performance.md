# Performance work

The rules that make a performance claim believable. The harness, the scenarios
and the measured baseline are documented in `docs/performance/README.md`; this
file is the discipline around them.

## The doctrine

1. **Measure before you change anything.** A hotspot identified by reading code
   is a hypothesis. `docs/performance/HOTSPOTS.md` records several places that
   *look* expensive and are not.
2. **Release builds only.** A debug build's profile is not Zola's profile.
   Build with `scripts/perf/run.sh build`, never a bare `cargo build --release`
   — a global `~/.cargo/config.toml` can silently change the profile.
3. **Representative workloads.** Synthetic scenarios isolate one cost each; a
   proxy of a real site tells you whether that cost matters. Report both.
4. **Scaling over absolutes.** The primary signal is the 8k/4k and 4k/2k ratio,
   not milliseconds on your laptop. Absolute numbers are only comparable within
   one interleaved measurement session.
5. **Interleave A/B runs.** Sequential "all baseline then all candidate" has
   produced a fake 3.2× on this machine because of thermal state. Use
   `scripts/perf/run.sh ab <a-bin> <b-bin> <site>...`, which alternates the two
   within each round and flips their order between rounds.

   Judge on the **paired per-round delta and whether every round agrees on its
   sign** — not on the difference between two medians. Comparing medians of
   absolute numbers threw away a unanimous −24% on the reference site because
   filesystem stalls made the absolutes noisier than the effect.
5b. **Watch CPU as well as wall.** A build that writes gigabytes stalls on the
   filesystem in ways that move wall time by seconds in both directions and say
   nothing about the change. When the two disagree, say so and quote both.
5c. **Measure at the level where the effect is visible.** If a change is worth
   1% and the build's noise floor is 5%, a whole-build A/B cannot resolve it:
   measure the phase (`--timings`) or the symbol (a profile). Report which one
   was measured, and report an unresolved effect as unresolved — never round it
   up into a win.
6. **Correctness is a gate, not a caveat.** Output must be byte-identical unless
   the change is explicitly about output. `scripts/perf/run.sh equivalence` must
   report IDENTICAL.
7. **One hotspot per change.** Two optimizations in one commit cannot be
   attributed, and cannot be reverted independently.
8. **Memory counts.** Report peak RSS before and after. A wall-clock win that
   doubles memory is not obviously a win.
9. **No unjustified `unsafe`, no speculative parallelism.** Both need a number.
10. **Record the negative results.** A change that did not help is worth more
    written down than deleted — see the "no win on the I/O-bound workload" entry
    in `docs/performance/OPTIMIZATIONS.md`.

## Working an item

```bash
scripts/dev.sh perf build                     # pinned release binary
cp target/release/zola /tmp/zola-baseline-$(git rev-parse --short HEAD)
# ... implement the change ...
scripts/dev.sh perf build
scripts/dev.sh perf site benchmarks/sites/<scenario>-<n>
scripts/dev.sh perf equivalence /tmp/zola-baseline-<sha> target/release/zola <site>
```

Generate a synthetic site first if it does not exist:

```bash
python3 scripts/perf/gen_site.py --scenario mixed-realistic --pages 4000
```

## What a finished item owes

An entry appended to `docs/performance/OPTIMIZATIONS.md` containing:

* the problem and the mechanism,
* the before evidence (profile samples, phase timings, or scaling ratio),
* what changed, in which file,
* the interleaved before/after table, with every round shown,
* peak RSS before and after,
* the correctness gates and their results, including output equivalence,
* an honest reading — including "no measurable win on workload X" when that is
  what happened,
* the commit.

Result JSON goes under `benchmarks/results/<sha>/`. Profiler output does not get
committed; `benchmarks/profiles/` is ignored for a reason.

Then re-run `scripts/dev.sh generate` so `docs/performance/STATUS.md` reflects
the new state, and check the backlog still validates:

```bash
scripts/dev.sh perf-index check
```

## Stop conditions

* The benchmark does not reproduce the hypothesis → stop, write down the number.
* Output equivalence fails and you cannot explain the difference → stop.
* The win is within run-to-run noise → stop; report the noise band.
* The change needs an architectural decision (a cache format, an invalidation
  model) → stop and write a decision record first
  (`docs/architecture/decisions/`).
