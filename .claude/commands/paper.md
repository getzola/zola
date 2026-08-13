---
description: Write or update an engineering paper in docs/papers/
argument-hint: [topic | ZPERF-NNN]
---

Paper work: $ARGUMENTS

Read `.claude/workflows/publication.md` first, and
`docs/papers/METHODOLOGY.md` for what may be claimed.

1. Gather the evidence before writing anything: `docs/performance/STATUS.md`,
   `HOTSPOTS.md`, `OPTIMIZATIONS.md`, the artifacts in `benchmarks/results/`,
   and `git log`. Decide what is actually supported.
2. If a finding is not recorded in `docs/performance/`, record it there first.
   A paper must not be the only place a fact exists.
3. Scaffold with `scripts/dev.sh papers new --title "..." --type <type>`.
4. Fill `data/measurements.toml` **before** the prose. Every printed figure gets
   an entry; artifact-backed ones carry `source` and `json_path`.
5. Write `paper.md`. Costs next to benefits. Unresolved results stay unresolved.
   Upstream behaviour, this fork's changes, and future proposals never blur.
6. Generate figures, then derive the social posts from the paper.
7. `scripts/dev.sh papers validate`, then walk `docs/papers/CHECKLIST.md`.
8. Set `status = "review"`. **Never** set `published` — that is the human's call.

Report honestly: which claims are measured, which are observed, which are
interpretation, and what evidence you wanted and could not find.
