# Publication methodology

What a paper in this series is allowed to say, and how each claim is backed.
`docs/performance/README.md` governs how measurements are *produced*; this
document governs how they are *published*, which is stricter, because a reader
of a paper cannot re-run the harness before deciding whether to believe it.

## 1. Claim classes

Every claim that matters belongs to exactly one class. The class is recorded in
`evidence.md`, not sprinkled through the prose — a paper that labels every
sentence is unreadable. The prose must nonetheless *sound* like its class:
measured results get numbers, hypotheses get "we think", proposals get "would".

| Class | Means | Prose must |
| ----- | ----- | ---------- |
| `measured` | a benchmark, profiler or test produced this number | give the number, the workload, and the conditions |
| `observed` | a behaviour reproduced by hand and described | say what was done and what happened |
| `code-fact` | true by reading the source | cite the file, and be checkable |
| `interpretation` | a conclusion drawn from the above | be visibly a conclusion, not a measurement |
| `hypothesis` | plausible, untested | be marked as untested |
| `proposal` | future architecture | be in the future tense and not carry predicted numbers as if measured |
| `rejected` | tested and did not pay off | give the numbers that rejected it |

The class boundary that matters most in practice is `measured` against
`interpretation`. "Peak memory fell 88%" is measured. "The memory wall is gone"
is interpretation, and needs the reasoning that gets you there.

## 2. Numbers

* **Never invent one.** If no artifact contains it, it does not go in.
* **Never round in the flattering direction.** −33.3% does not become "over a
  third". 9248 MB does not become "nearly 10 GB".
* **Never quote a figure the harness itself calls unresolved.** If the paired
  rounds disagreed on the sign, the paper says the rounds disagreed on the sign.
* **Never universalise.** Not "Zola is 33% faster" but "on the reference
  workload, paired median wall time fell 33%".
* **Say what did not move.** A change that helped one scenario and did nothing
  on five others is described that way, in the same table.

Every figure printed in a paper is declared in `data/measurements.toml`, and any
figure derived from a committed benchmark artifact carries the path into that
artifact so the validator can re-extract it. Numbers that no artifact holds —
values read off a terminal, profiler symbol totals, memory footprints — are
declared with `source_note` describing exactly how they were obtained, and are
classed `observed` rather than `measured` unless a committed file backs them.

## 3. Baseline discipline

A performance paper must identify, unambiguously:

* the **baseline** — commit, and what it is (upstream? upstream plus
  instrumentation? the previous change?);
* the **candidate** — commit;
* the **workload** — page count, output size, and what makes it representative
  or not;
* the **machine** — cores, memory, OS, and whether it was otherwise busy;
* the **procedure** — how many rounds, interleaved or not, what statistic;
* the **correctness gate** — how output equivalence was established.

Comparisons across sessions are not permitted in a headline table. This series
already published one that understated its own result by about half, because its
"before" and "after" were measured weeks apart on a machine in different states.
One interleaved session, or no table.

## 4. Upstream, fork, experiment, proposal

Four different things, never blurred:

| | means |
| --- | --- |
| **upstream** | behaviour of the unmodified project at a named version or commit |
| **this fork** | a change made here, on a named commit |
| **experiment** | tried and measured, may or may not have landed |
| **proposal** | designed, not built, no measurements exist |

A bug demonstrated against upstream must name the baseline binary it was
demonstrated against. Saying "Zola does X" when the fork does X and upstream
does not is the single worst error this series can make: it misinforms readers
about software that other people maintain.

## 5. Negative results

A rejected optimization is publishable and often more useful than an accepted
one, because the attractive-but-wrong idea is the one the reader is about to
have. A negative result is reported with the same rigour as a positive one:
hypothesis, experiment, numbers, decision. It is never softened into "we chose
not to pursue".

## 6. Measurement mistakes

If the program measured something wrongly and later corrected it, the correction
belongs in the paper. Two are already on record in this repository: a sequential
A/B that produced a fabulous number from thermal drift, and an `ps -o rss`
reading that under-reported a process's memory by a factor of 400. Publishing
those costs nothing and tells the reader what kind of measurement discipline
produced the rest of the numbers.

## 7. Reproduction

Every paper carries commands that exist in this repository and were run. If a
result depends on a workload that cannot be redistributed, the paper describes
the workload's shape — page count, output size, template structure — and gives a
synthetic fixture that approximates it. It does not publish paths into anyone's
private site.

## 8. When evidence changes

Numbers move: a re-run on a quiet machine, a later optimization, a corrected
mistake. The order of operations is fixed:

```
update the artifact in benchmarks/results/
  → update data/measurements.toml
    → regenerate figures
      → audit the prose that cites them
        → regenerate the social derivatives
```

Never patch the four copies by hand. `scripts/dev.sh papers validate` exists
because someone will try.
