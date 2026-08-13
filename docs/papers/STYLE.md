# Style

## Voice

Technical, direct, and confident exactly as far as the evidence goes. The reader
is an engineer who has profiled something before and will notice if a claim is
doing more work than its data.

Write:

> On the reference workload the change reduced paired CPU time by 23.7%,
> unanimously across five rounds. The small-page synthetic fixtures showed no
> measurable effect at all.

Not:

> This groundbreaking optimization dramatically accelerated builds.

Concretely:

* No adjective that a number could replace.
* No "we're excited to share". No "deep dive". No "game-changing".
* No implied endorsement by, or affiliation with, the upstream project.
* Uncertainty is stated, not hedged around. "The rounds disagreed on the sign,
  so this is unresolved" beats "results were somewhat mixed".
* First person plural is fine. So is admitting error — it is usually the most
  interesting sentence on the page.

## Structure

Papers follow a standard skeleton so a reader can find the numbers without
reading the narrative. Sections that would be empty are omitted rather than
filled:

```
Title
Abstract              150–250 words, self-contained
Context               what the software is, what this fork is, what the workload is
Problem               what prompted the work
Methodology           how anything here was measured
Baseline              where we started, with numbers
Investigation         what was looked at, in what order, and why
Findings              what was true
Changes               what was done about it
Results               what that bought, as a table
Negative results      what was tried and rejected, with the numbers
Correctness           how output equivalence was established
Limitations           where these numbers do not apply
What surprised us     the honest part
Architectural implications
Future work           explicitly labelled as not built
Reproduction          commands that exist
Evidence index        pointer to evidence.md
```

A `bug-analysis` or `postmortem` paper varies this: symptom, reproduction, root
cause, fix, why it was not caught, what changed in the process.

## Tables

Tables carry the numbers; prose carries the reasoning. A results table shows
before, after, delta, and whether the rounds agreed. It does not show a delta
without the absolute values it came from.

## Figures

Generated, never drawn. Every figure is produced by `scripts/papers/figures.py`
from a committed artifact, and carries its provenance inside the SVG.

## Length

Whatever the evidence supports. A negative-result paper can be 800 words. A
study of a completed program is a few thousand. Padding to a target is visible
from a distance.

## Titles

Descriptive, specific, no theatre. The strongest number in the paper is usually
enough of a hook on its own; it does not need an exclamation mark or the word
"shocking" in front of it.

Good: *Faster Without Computing Less*, *Why `zola serve` Held 9 GB*.
Bad: *The Ultimate Guide to Blazing Fast Static Sites*.

## Derivatives

`social/linkedin.md` is a long post: the finding, one or two numbers, the honest
caveat, and what is next. `social/short.md` is one paragraph. `social/thread.md`
is a sequence of posts, each of which stands alone.

All three obey one rule the validator enforces: **every number in a derivative
must appear in `paper.md`**. Derivatives select; they never restate.
