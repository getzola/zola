# Acceptance — translate envelope validation

All greps against `src/cmd/translate.rs` on branch `fix/translate-envelope-validation`.

## Must exist

```
rg -n 'flags\.len\(\)\s*!=\s*sent\.len\(\)|sent\.len\(\)\s*!=\s*flags\.len\(\)|flags\.len\(\)\s*==\s*sent\.len\(\)' src/cmd/translate.rs
rg -n 'as_i64' src/cmd/translate.rs
rg -n 'ok_array_shorter_than_sent|flags\.len' src/cmd/translate.rs
rg -n 'numeric_negative_error_code|"code":\s*-1|code":-1' src/cmd/translate.rs
```

`as_i64` must sit in the `code`/`status`/`error_code` parse (near `as_u64` /
`as_str`), not only in unrelated code.

## Must still exist (do not regress)

```
rg -n 'impl Default for OpenRouterClient' src/cmd/translate.rs
rg -n 'fn parse_translate_response' src/cmd/translate.rs
rg -n 'mirrored_error_code_1003_is_a_hard_failure' src/cmd/translate.rs
rg -n 'ok_all_true_or_absent_still_passes' src/cmd/translate.rs
```

## Must not exist (findings 1/2/5 left untouched)

No new `starts_with("http")` / scheme check on `TRANSLATE_URL`.
No signature change on `parse_translate_response`.
No deletion of `impl Default for OpenRouterClient`.

## Tests

```
cargo test --lib -- ok_array_shorter_than_sent numeric_negative_error_code ok_false ok_array_with_a_false mirrored_error_code_1003 ok_all_true
```

All pass.

## Ship

- Branch `fix/translate-envelope-validation` off master, not a direct master push
- PR body names findings 3+4; 1/2/5 deferred as taste
- PR URL returned
