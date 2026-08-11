---
description: Close a work session — evidence, honest status, handoff
---

Close the current session.

1. Run `scripts/dev.sh impact` and `scripts/dev.sh session end`.
2. Complete `.claude/context/session.md`: every finding names its evidence,
   every gate says pass / fail / not run, and the status line is one of
   `complete`, `partial`, `blocked`, `investigation-only`.
3. Promote durable findings out of the session file into
   `docs/performance/`, `docs/architecture/decisions/` or `CHANGELOG.md`.
4. Run `scripts/dev.sh generate` if anything a generator reads has changed.
5. Delete scratch files and check `git status` for anything unintended.
6. Archive: `mv .claude/context/session.md .claude/context/<date>-<slug>.md`.

Do not write `complete` unless the gates that the change's risk class requires
were actually run in the current state of the tree. A disproved hypothesis
reported with its evidence is a good outcome; a false `complete` is not.

Full workflow: `.claude/workflows/session.md`.

$ARGUMENTS
