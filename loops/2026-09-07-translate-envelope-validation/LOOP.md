# translate envelope validation (findings 3+4)

**Status:** in progress
**Created:** 2026-09-07
**Owner:** grok-dev (pi-hand-grok-dev-2026-09-06T22-25-45-056Z)

## Goal

On `curriculo-tech/zola` `src/cmd/translate.rs`, reject two malformed
translate-endpoint envelopes that currently pass:

1. per-string `ok` array shorter than `sent` (finding #3)
2. numeric negative error codes such as `"code": -1` (finding #4, `as_i64`)

Then PR from `fix/translate-envelope-validation` off `master`.

## Decisions (locked)

- Apply ONLY review findings 3 and 4. Skip #1 (Default impl), #2 (fields param), #5 (URL scheme) — taste/cosmetic.
- Base: `origin/master` (PR #23 already merged). Branch: `fix/translate-envelope-validation`.
- Touch only `src/cmd/translate.rs` (plus this loop dir).
- Repo-local git identity: Dev Rishi Khare `<devrishik@gmail.com>` (matches recent authors). Do not switch to Nada/Roshni.
- Assumption: Nada authored the envelope parser in merged #23; this is a follow-up fix with Dev identity, not an identity switch and not an amend of her commits.
- No direct push to master. No self-merge.

## Tasks

- [ ] #3 `Value::Array(flags)`: require `flags.len() == sent.len()`; mismatch is a hard error
- [ ] #4 error-code parse: also try `as_i64` so `"code": -1` is an error envelope
- [ ] unit tests for both
- [ ] `cargo test --lib parse_translate` (or translate tests) pass
- [ ] PR opened, findings 3+4 in body, 1/2/5 deferred

## Log

- 2026-09-07: loop opened off origin/master e5c6a528 (merge #23)
