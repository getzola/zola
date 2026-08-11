# Implementation

Follows an investigation. If there was no investigation, go back.

## Plan first

A plan is acceptable when every step names a file, an action, and how you will
know it worked. Reject your own plan if a step reads like a goal rather than an
edit:

| Rejected | Accepted |
| -------- | -------- |
| optimise rendering | in `components/render/src/cache.rs`, build the section map from existing `Value`s instead of `Value::from_serializable`; verify with the output-equivalence gate on `mixed-realistic-1000` |
| improve the cache | scope the `result_cache` lock in `load_data.rs` to the lookup and the insert; verify with `cargo test -p templates` and a thread sweep |
| clean up `site` | move `write_output`'s directory creation into a shared set built from the job list; verify with `cargo test -p site` |

The plan must also state the risk class (`scripts/dev.sh impact` prints it) and
therefore which gates apply.

## Rules

* **One hotspot, one behaviour, one commit.** Do not fold an unrelated
  refactor into a change that has to be reviewed for output equivalence.
* **Read the file before editing it.** Including the parts you are not changing.
* **Do not change observable behaviour** unless that is the point of the change.
  Output bytes, URLs, error messages and their order, and template-visible
  structure are all observable.
* **Do not add `unsafe`** without a comment proving why it is sound and a
  measurement proving it is worth it.
* **Do not add parallelism** to a phase you have not measured. Rayon over a tiny
  slice is slower than a loop.
* **Do not edit generated files.** `docs/architecture/COMPONENTS.md`,
  `docs/performance/STATUS.md`, man pages and shell completions are outputs.
  Change the source, then `scripts/dev.sh generate`.
* **Leave nothing behind.** Scratch scripts, generated sites, profiler output
  and debug prints do not get committed.

## Validation

Run what the risk class asks for. `scripts/dev.sh impact` lists it; the tiers are
defined in `quality.md`.

Minimum for any production change:

```
scripts/dev.sh quality
```

For HIGH and CRITICAL changes, additionally a test that fails without the change.
For CRITICAL changes, byte-for-byte output equivalence against the previous
binary — see `performance.md`.

## Reporting

Report what you ran and what it said. "Tests pass" is only true if you ran them
in this state of the tree. If a gate was skipped, say which and why.
