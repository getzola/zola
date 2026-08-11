---
description: Implement a change with a file-level plan and the gates its risk class requires
argument-hint: [what to implement]
---

Implement: $ARGUMENTS

1. If no investigation has happened for this, run `/investigate` first.
2. Run `scripts/dev.sh impact` to get the risk class and required gates.
3. Write the plan before editing. Every step names a file, an edit, and how you
   will know it worked. Reject steps like "optimise X" — see the table in
   `.claude/workflows/implement.md`.
4. Implement one behaviour. Do not fold in unrelated refactors.
5. Run the gates the risk class requires. At minimum `scripts/dev.sh quality`.
6. Report what you ran and what it said, gate by gate.

Stop and report instead of pushing through if the tests disagree with the change
in a way you cannot explain, if the change outgrows its scope, or if it needs an
architectural decision that is not written down.
