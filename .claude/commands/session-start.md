---
description: Open a work session — collect repository state and orient before editing
---

Start a session in this repository.

1. Run `scripts/dev.sh session start`. If it reports an existing session, read
   `.claude/context/session.md` and continue that one instead of starting over.
2. Read `CLAUDE.md`, then `docs/architecture/COMPONENTS.md` for the component
   you are about to work in.
3. Run `git log --oneline -10` and read `docs/performance/STATUS.md`.
4. Fill in **Objective** and **Constraints and assumptions** in
   `.claude/context/session.md`.

Then report, in under ten lines: branch and commit, whether the worktree is
clean, the objective, the open backlog items relevant to it, and the first thing
you intend to read. Do not edit code in this step.

Full workflow: `.claude/workflows/session.md`.

$ARGUMENTS
