# Session {{DATE}} — <objective in one line>

Status: `active` | `complete` | `partial` | `blocked` | `investigation-only`

## Objective

What this session is trying to achieve, and why now. If it is a backlog item,
name it (`PERF-004`, an issue number, a decision record).

## Constraints and assumptions

Anything the next session would otherwise have to rediscover: an invariant that
must hold, a workload the change must not regress, a decision already made.

## Findings

Evidence only. Each finding names where it came from — a file and line, a
profile, a benchmark result file, a test run. A statement with no source is a
hypothesis, and belongs under *Open questions*.

## Changes

| File | What changed | Why |
| ---- | ------------ | --- |

## Validation

| Gate | Result |
| ---- | ------ |
| `scripts/dev.sh quality` | not run |
| output equivalence | not run / N/A |
| benchmark before/after | not run / N/A |

"not run" is a legitimate answer. Claiming a gate passed without having run it
is not.

## Blocked / not done

What was in scope and did not happen, and what is blocking it.

## Next action

The single most useful thing the next session should do first.

## Handoff checklist

* [ ] Status line above is one of the five values, and honest.
* [ ] Every claim under *Findings* names its evidence.
* [ ] Every gate under *Validation* says pass, fail, or not run.
* [ ] Durable findings promoted out of this file (`docs/performance/`,
      `docs/architecture/decisions/`, `CHANGELOG.md`).
* [ ] Temporary files, scratch scripts and generated sites removed.
* [ ] `git status` reviewed — nothing unintended staged or left behind.
