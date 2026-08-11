---
description: Prove the branch is healthy and report each gate's actual result
argument-hint: [fast | full]
---

Run the quality gates: $ARGUMENTS

* no argument or `gate` → `scripts/dev.sh quality`
* `fast` → `scripts/dev.sh check`
* `full` → `scripts/dev.sh quality-full`

Report each gate with its real outcome. Do not summarise a failure as a warning,
do not claim a gate passed that you did not run, and do not stop at the first
failure without saying what the remaining gates would have been.

If a gate fails, show the first error verbatim and say what you intend to do
about it before doing it.

Tiers and the risk table: `.claude/workflows/quality.md`.
