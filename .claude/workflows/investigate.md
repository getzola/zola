# Investigation

Use before any non-trivial change. The output is understanding and evidence, not
a diff. An investigation that ends with "I think X is the problem" and no
measurement has not finished.

## When to use

* A bug whose cause is not obvious from the stack trace.
* Any performance question.
* Before touching a component you have not read in this session.
* When a plan would otherwise start with "probably".

## Steps

1. **State the question.** One sentence, answerable with evidence. "Why is
   `zola build` slow on this site" is not a question; "which build phase
   accounts for most of the wall time on a 4000-page site" is.

2. **Map the territory.** `docs/architecture/COMPONENTS.md` says which crate
   owns what. `CLAUDE.md` describes the build pipeline in order. Find the entry
   point before reading anything else — for a build that is
   `Site::build` in `components/site/src/lib.rs`.

3. **Trace the path.** Follow the call chain from the entry point to the code in
   question and write it down. Note every place the data is copied, locked,
   serialized or written.

4. **Read the tests first.** `components/*/tests/` and the unit tests next to
   the code define the behaviour you must not change.
   `components/site/tests/site.rs` asserts exact counts against `test_site`.

5. **Find the invariants.** What must remain true: URL shape, output bytes,
   error ordering, front-matter semantics, template-visible structure. Write
   them down; they become the acceptance criteria.

6. **Measure, if the question is about cost.** Never optimise from a reading of
   the code. See `performance.md` for the harness. Attach numbers to the claim.

   **Both sides of a comparison must be measured against the same state.** Two
   correct measurements taken on different states support no conclusion at all,
   and the artifact never says which state it came from. Record the commit — and
   whether the tree was dirty — next to every number, and re-measure rather than
   reuse a figure taken before an edit you have since made.

   This is the most common way a comparison goes wrong here. It has produced a
   "the bug does not reproduce" that was a build made after the fix, a page-count
   disagreement between two trees a content edit apart, and two correct word
   counts of the same file one revision apart. In each case both measurements
   were right and the conclusion drawn across them was wrong.

7. **Write the findings.** Into the session record. Every finding names its
   source: `file:line`, a profile, a result file, or a test run.

## Stop conditions

Stop and report instead of continuing:

* The measurement contradicts the hypothesis. Report the measurement.
* The behaviour you would have to change is asserted by a test and you cannot
  tell whether the assertion is deliberate.
* The change would grow past what was asked. Report the boundary you hit.
* You cannot find evidence for the claim that motivated the work.

None of these is a failure. Reporting a disproved hypothesis with the number
that disproved it is a complete, useful result.

## Output

A findings section in `.claude/context/session.md` containing:

* the question,
* the call path,
* the invariants,
* the evidence, with sources,
* what you now believe, and how confident you are,
* the smallest next step that would confirm or refute it.
