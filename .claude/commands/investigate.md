---
description: Understand before changing — map, trace, read tests, measure, report findings
argument-hint: [question or component]
---

Investigate: $ARGUMENTS

Follow `.claude/workflows/investigate.md`. Do not edit production code during
this command.

Produce, in the session record and in your reply:

* the question, stated so that evidence can answer it;
* the call path from the entry point to the code in question;
* the invariants the code must preserve, and where they are asserted;
* the evidence, each item naming its source (`file:line`, a profile, a
  benchmark result file, a test run);
* what you now believe and how confident you are;
* the smallest next step that would confirm or refute it.

If the evidence contradicts the premise of the question, say so and stop. That
is a complete result, not a failure.
