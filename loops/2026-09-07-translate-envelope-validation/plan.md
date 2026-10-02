# Plan — translate envelope validation

Repo: `/home/dev/dev_ws/c/zola` (github.com/curriculo-tech/zola)
File: `src/cmd/translate.rs` ONLY.

## Qwen (hermes) — implement both findings + tests

### Finding #3 (~line 237, `Value::Array(flags)` arm)

Today: walks flags for a non-true bool; a short array with all-true entries
passes even when `flags.len() < sent.len()`.

Fix: before (or as part of) that walk, if `flags.len() != sent.len()`, `bail!`
a malformed-envelope error. Existing all-true same-length case
(`ok_all_true_or_absent_still_passes`, body-only `ok:[true]` with `sent.len()==1`)
must keep passing.

### Finding #4 (~line 254, `code`/`status`/`error_code`)

Today:

```
v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
```

`as_u64()` is None on negatives; `as_str()` is None on numbers. `"code": -1`
is skipped.

Fix: also try `as_i64`. Negative codes MUST be treated as error envelopes
(bail), not dropped. Do **not** convert i64→u64 via `try_from` (that drops
negatives again). Simplest: parse as `i64` (`as_i64`, then `as_u64` via
`i64::try_from`, then string parse). Keep 0 and 200 as success. Existing
`mirrored_error_code_1003_is_a_hard_failure` must keep passing.

### Tests (same file, `mod tests`)

Add next to the existing ok/code tests (~line 870):

1. `ok_array_shorter_than_sent_is_malformed` — three fields sent,
   `{"ok":[true],"translations":["T","D","B"]}` → Err (malformed / length).
2. `numeric_negative_error_code_is_a_hard_failure` —
   `{"code":-1,"translations":["T"]}` → Err mentioning the code / error.

Run: `cargo test --lib -- parse_translate ok_array mirrored_error numeric_negative ok_false`

### Do-nots

- Do not change `impl Default for OpenRouterClient` (finding #1).
- Do not change `parse_translate_response` signature / `fields` param (finding #2).
- Do not add URL-scheme validation on `TRANSLATE_URL` (finding #5).
- Do not touch other files. No new deps. No rustfmt-of-the-world.
- Do not push. Do not open a PR. Do not commit (parent commits).

## GLM — same scope if Qwen misses

Same file, same two findings, same tests, same do-nots.
Do not open a PR. Do not push to master.
