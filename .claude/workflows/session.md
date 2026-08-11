# Sessions

Long-running work — a performance program especially — outlives a single
context window. The session record is what makes the next window productive
instead of archaeological.

## Two kinds of memory

**Ephemeral.** `.claude/context/` is gitignored. It holds the working state of
the session in progress: what you are doing, what you have found, what you have
run. It is allowed to be messy and is never reviewed by anyone.

**Durable.** Findings that outlive the session belong in tracked files:

| Finding | Home |
| ------- | ---- |
| measured result of a `PERF-*` item | `docs/performance/OPTIMIZATIONS.md` |
| a new or revised hotspot | `docs/performance/HOTSPOTS.md` |
| how the build actually works | `docs/performance/ARCHITECTURE.md` |
| a decision with lasting consequences | `docs/architecture/decisions/` |
| anything a Zola user would notice | `CHANGELOG.md` |

Never leave a durable finding only in `.claude/context/`. Nobody else will read
it there, and the next session may not either.

## Start

```bash
scripts/dev.sh session start
```

Writes `.claude/context/session.md` from the template and appends the state that
is expensive to reconstruct: branch, commit, worktree cleanliness, toolchain,
CPU count, and the open `PERF-*` items. Fill in **Objective** before editing
code.

Then orient:

* `CLAUDE.md` — pipeline, conventions, rules.
* `docs/architecture/COMPONENTS.md` — who owns what (generated).
* `docs/performance/STATUS.md` — the backlog and its state (generated).
* `git log --oneline -10` — what the previous sessions actually did.

## During

Append to *Findings* as you go, with sources. Record the commands you ran,
including the ones that failed — a failed command that you do not write down
will be run again by the next session.

## End

```bash
scripts/dev.sh session end
```

Prints the handoff checklist. The session record must end with an honest status:

| Status | Means |
| ------ | ----- |
| `complete` | the objective was met and every gate it needed passed |
| `partial` | some of it landed; the record says exactly which part |
| `blocked` | it cannot proceed; the record says what is blocking and what was tried |
| `investigation-only` | no production change; findings only |

`complete` requires evidence. A gate that was not run is recorded as "not run",
never as passing. If the objective was not met, that is a normal outcome — an
investigation that disproves a hypothesis is a result, and mislabelling it
`complete` costs the next session a day.

Finally, promote the durable parts, delete the scratch, and archive the file:

```bash
mv .claude/context/session.md .claude/context/$(date -u +%Y-%m-%d)-<slug>.md
```
