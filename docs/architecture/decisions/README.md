# Architecture decisions

Short records of choices whose consequences outlive the commit that made them.
They exist so a later session can find out *why* something is the way it is
without reconstructing it from a diff.

## When to write one

Write a record when a choice constrains future work:

* a persistent or incremental build cache — its format, and what invalidates it;
* how the dependency graph between content, templates and output is represented;
* artifact hashing, template dependency tracking, storage strategy;
* the parallel execution model of a build phase;
* a dependency direction between crates that has to hold.

Do not write one for a bug fix, a refactor with no external consequence, or a
choice you would happily reverse next week. A record that documents nothing
durable is noise.

## Format

Copy `TEMPLATE.md` to `NNNN-short-slug.md`, next number, no gaps. Sections:
context, decision, alternatives, consequences, evidence, status.

`status` is one of `proposed`, `accepted`, `superseded by NNNN`. Records are not
edited once accepted — a change of mind is a new record that supersedes the old
one, and the old one gets its status updated.

## Index

| # | Decision | Status |
| - | -------- | ------ |
| [0001](0001-crate-layering.md) | The workspace is a layered DAG, and it is enforced | accepted |
