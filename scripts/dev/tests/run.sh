#!/usr/bin/env bash
# Tests for the repository tooling. Automation that gates commits is production
# code; this is its test suite.
#
#   scripts/dev.sh test-tooling
#
# Every case builds a throwaway fixture tree, points the tool at it with
# ZOLA_DEV_ROOT, and asserts on the exit code and the message. Nothing here
# touches the real repository.
set -uo pipefail

cd "$(dirname "$0")/../../.."
ROOT="$(pwd)"
DEV="$ROOT/scripts/dev"

PASS=0
FAIL=0
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

ok()   { PASS=$((PASS + 1)); printf 'ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL + 1)); printf 'FAIL %s\n     %s\n' "$1" "$2"; }

# assert_run <name> <expected-exit> <expected-substring-or-empty> -- cmd...
assert_run() {
  local name="$1" want_rc="$2" want_out="$3"
  shift 4  # name, rc, substring, --
  local out rc
  out="$("$@" 2>&1)"
  rc=$?
  if [ "$rc" -ne "$want_rc" ]; then
    bad "$name" "expected exit $want_rc, got $rc; output: ${out:0:400}"
    return
  fi
  if [ -n "$want_out" ] && [[ "$out" != *"$want_out"* ]]; then
    bad "$name" "expected output to contain '$want_out'; got: ${out:0:400}"
    return
  fi
  ok "$name"
}

# ---------------------------------------------------------------- shell syntax

syntax_bad=0
while IFS= read -r f; do
  [ -f "$f" ] || continue
  case "$f" in *.py) continue ;; esac
  head -1 "$f" | grep -q 'sh$\|bash' || continue
  if ! bash -n "$f" 2>/dev/null; then
    bad "shell syntax: $f" "$(bash -n "$f" 2>&1 | head -3)"
    syntax_bad=1
  fi
done < <(printf '%s\n' scripts/dev.sh scripts/dev/doctor.sh scripts/dev/tests/run.sh \
  scripts/perf/run.sh scripts/perf/build.sh .githooks/pre-commit)
[ "$syntax_bad" -eq 0 ] && ok "shell syntax (all scripts parse)"

if command -v shellcheck >/dev/null 2>&1; then
  sc_out="$(shellcheck --severity=error scripts/dev.sh scripts/dev/doctor.sh \
      scripts/dev/tests/run.sh .githooks/pre-commit scripts/perf/run.sh scripts/perf/build.sh 2>&1)"
  if [ -z "$sc_out" ]; then
    ok "shellcheck (no errors)"
  else
    bad "shellcheck" "$(echo "$sc_out" | head -10)"
  fi
else
  ok "shellcheck (skipped, not installed)"
fi

# --------------------------------------------------------------- python syntax

if python3 -c 'import sys, tomllib' >/dev/null 2>&1; then
  py_out="$(python3 -m py_compile "$DEV"/*.py scripts/perf/*.py 2>&1)"
  if [ -z "$py_out" ]; then ok "python syntax"; else bad "python syntax" "$py_out"; fi
else
  echo "python3 is missing or older than 3.11 — skipping the generator tests"
  printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
  [ "$FAIL" -eq 0 ]
  exit $?
fi

# ------------------------------------------------------------------- fixtures

# A minimal workspace: two components, one dependency, matching metadata.
make_workspace() {
  local dir="$1" extra_dep="${2:-}"
  mkdir -p "$dir/components/base/src" "$dir/components/top/src" \
           "$dir/scripts/dev" "$dir/docs/architecture"
  cat >"$dir/components/base/Cargo.toml" <<'EOF'
[package]
name = "base"
EOF
  cat >"$dir/components/top/Cargo.toml" <<EOF
[package]
name = "top"

[dependencies]
base = { workspace = true }
$extra_dep
EOF
  cat >"$dir/scripts/dev/components.toml" <<'EOF'
[invariants]
forbidden = [
  { from = "base", to = "top", reason = "base is a leaf" },
]

[component.base]
responsibility = "Leaf."

[component.top]
responsibility = "Depends on the leaf."
EOF
}

# A minimal performance backlog.
make_perf_docs() {
  local dir="$1" hotspot_rows="$2" optimizations="$3"
  mkdir -p "$dir/docs/performance"
  cat >"$dir/docs/performance/HOTSPOTS.md" <<EOF
# Hotspots

| ID | Location | Operation | Priority |
| -- | -------- | --------- | -------- |
$hotspot_rows
EOF
  printf '%s\n' "$optimizations" >"$dir/docs/performance/OPTIMIZATIONS.md"
}

# ------------------------------------------------------------- repo_map tests

W="$TMP/ws-clean"
make_workspace "$W"
ZOLA_DEV_ROOT="$W" python3 "$DEV/repo_map.py" generate >/dev/null
assert_run "repo_map: clean workspace passes" 0 "architecture OK" -- \
  env ZOLA_DEV_ROOT="$W" python3 "$DEV/repo_map.py" check

assert_run "repo_map: drift is detected" 1 "is out of date" -- \
  env ZOLA_DEV_ROOT="$W" python3 -c "
import pathlib,sys,os
p=pathlib.Path(os.environ['ZOLA_DEV_ROOT'])/'docs/architecture/COMPONENTS.md'
p.write_text('stale\n')
sys.exit(os.system('python3 \"$DEV/repo_map.py\" check')>>8)
"

W2="$TMP/ws-forbidden"
make_workspace "$W2"
# Give base a dependency on top: the declared-forbidden edge, and a cycle.
cat >>"$W2/components/base/Cargo.toml" <<'EOF'

[dependencies]
top = { workspace = true }
EOF
assert_run "repo_map: forbidden edge fails with its reason" 1 "base is a leaf" -- \
  env ZOLA_DEV_ROOT="$W2" python3 "$DEV/repo_map.py" check
assert_run "repo_map: cycle is reported" 1 "dependency cycle" -- \
  env ZOLA_DEV_ROOT="$W2" python3 "$DEV/repo_map.py" check

W3="$TMP/ws-undeclared"
make_workspace "$W3"
mkdir -p "$W3/components/orphan/src"
printf '[package]\nname = "orphan"\n' >"$W3/components/orphan/Cargo.toml"
assert_run "repo_map: new component must be described" 1 "[component.orphan]" -- \
  env ZOLA_DEV_ROOT="$W3" python3 "$DEV/repo_map.py" check

W4="$TMP/ws-stale-meta"
make_workspace "$W4"
printf '\n[component.ghost]\nresponsibility = "Gone."\n' >>"$W4/scripts/dev/components.toml"
assert_run "repo_map: stale metadata entry is reported" 1 "no longer exists" -- \
  env ZOLA_DEV_ROOT="$W4" python3 "$DEV/repo_map.py" check

assert_run "repo_map: generate refuses on a violation" 1 "refusing to generate" -- \
  env ZOLA_DEV_ROOT="$W2" python3 "$DEV/repo_map.py" generate

# ----------------------------------------------------------- perf_index tests

P="$TMP/perf-clean"
make_perf_docs "$P" \
  '| PERF-001 | `components/base/src/lib.rs:1` | slow | P0 |
| PERF-002 | `components/top/src/lib.rs:2` | slower | P1 |' \
  '# Optimizations

## PERF-001 — fixed

**Commit.** `perf(PERF-001): fix it`'
ZOLA_DEV_ROOT="$P" python3 "$DEV/perf_index.py" generate >/dev/null
assert_run "perf_index: consistent backlog passes" 0 "PERF backlog OK (2 items)" -- \
  env ZOLA_DEV_ROOT="$P" python3 "$DEV/perf_index.py" check

P2="$TMP/perf-nocommit"
make_perf_docs "$P2" '| PERF-001 | `x` | slow | P0 |' \
  '# Optimizations

## PERF-001 — done, but no evidence'
assert_run "perf_index: completed item must cite a commit" 1 "no \`**Commit.**\` line" -- \
  env ZOLA_DEV_ROOT="$P2" python3 "$DEV/perf_index.py" check

P3="$TMP/perf-dangling"
make_perf_docs "$P3" '| PERF-001 | `x` | slow | P0 |' '# Optimizations'
mkdir -p "$P3/docs/performance"
printf 'See PERF-042 for details.\n' >"$P3/docs/performance/NOTES.md"
assert_run "perf_index: dangling reference is reported with its file" 1 \
  "PERF-042 is referenced by docs/performance/NOTES.md" -- \
  env ZOLA_DEV_ROOT="$P3" python3 "$DEV/perf_index.py" check

P4="$TMP/perf-undefined-done"
make_perf_docs "$P4" '| PERF-001 | `x` | slow | P0 |' \
  '# Optimizations

## PERF-099 — never defined

**Commit.** `perf: whatever`'
assert_run "perf_index: completed-but-undefined item is reported" 1 \
  "no such item is defined" -- \
  env ZOLA_DEV_ROOT="$P4" python3 "$DEV/perf_index.py" check

P5="$TMP/perf-subitem"
make_perf_docs "$P5" '| PERF-005 | `components/top/src/c.rs:1` | slow | P1 |' \
  '# Optimizations

## PERF-005a — half of it

**Commit.** `perf(PERF-005a): part one`'
ZOLA_DEV_ROOT="$P5" python3 "$DEV/perf_index.py" generate >/dev/null
assert_run "perf_index: sub-items resolve to their parent" 0 "PERF backlog OK" -- \
  env ZOLA_DEV_ROOT="$P5" python3 "$DEV/perf_index.py" check

assert_run "perf_index: open lists only open items" 0 "PERF-002 (P1" -- \
  env ZOLA_DEV_ROOT="$P" python3 "$DEV/perf_index.py" open
assert_run "perf_index: open omits completed items" 0 "" -- \
  env ZOLA_DEV_ROOT="$P" bash -c "python3 \"$DEV/perf_index.py\" open | grep -qv PERF-001"

P6="$TMP/perf-missing"
mkdir -p "$P6/docs"
assert_run "perf_index: a missing backlog is a clear error, not a traceback" 1 \
  "has no source of truth" -- \
  env ZOLA_DEV_ROOT="$P6" python3 "$DEV/perf_index.py" check

# --------------------------------------------------------------- impact tests

assert_run "impact: queue.rs is CRITICAL" 0 "Risk: **CRITICAL**" -- \
  python3 "$DEV/impact.py" --files components/site/src/queue.rs
assert_run "impact: config is HIGH" 0 "Risk: **HIGH**" -- \
  python3 "$DEV/impact.py" --files components/config/src/lib.rs
assert_run "impact: an ordinary component file is MEDIUM" 0 "Risk: **MEDIUM**" -- \
  python3 "$DEV/impact.py" --files components/utils/src/slugs.rs
assert_run "impact: documentation alone is LOW" 0 "Risk: **LOW**" -- \
  python3 "$DEV/impact.py" --files README.md
assert_run "impact: cli.rs points at the CLI documentation" 0 "cli-usage.md" -- \
  python3 "$DEV/impact.py" --files src/cli.rs
assert_run "impact: cli.rs requires regenerating man pages" 0 "regenerates man pages" -- \
  python3 "$DEV/impact.py" --files src/cli.rs
assert_run "impact: --strict fails an untested HIGH change" 1 "with no test edit" -- \
  python3 "$DEV/impact.py" --strict --files components/markdown/src/markdown.rs \
    docs/content/documentation/content/linking.md
assert_run "impact: --strict passes when tests and docs move too" 0 "" -- \
  python3 "$DEV/impact.py" --strict --files components/markdown/src/markdown.rs \
    components/markdown/tests/markdown.rs docs/content/documentation/content/linking.md
assert_run "impact: json output is valid json" 0 '"risk": "CRITICAL"' -- \
  python3 "$DEV/impact.py" --json --files components/render/src/cache.rs

# --------------------------------------------------- gate reporting, with a stub cargo

# Runs the real dispatcher against a fake cargo so the reporting path is
# exercised in milliseconds. This is what catches a gate that runs the right
# commands but mis-reports the outcome.
STUB="$TMP/stub-bin"
mkdir -p "$STUB"
cat >"$STUB/cargo" <<'EOF'
#!/usr/bin/env bash
case "$1" in
  fmt)   [ "${STUB_FMT_FAILS:-0}" = 1 ] && exit 1; echo "stub: fmt ok" ;;
  check) echo "stub: check ok" ;;
  *)     echo "stub: $*" ;;
esac
exit 0
EOF
chmod +x "$STUB/cargo"

assert_run "dev.sh check: reports each step as PASS" 0 "--- cargo fmt --check: PASS" -- \
  env PATH="$STUB:$PATH" "$ROOT/scripts/dev.sh" check
assert_run "dev.sh check: succeeds when every step passes" 0 "ALL PASS" -- \
  env PATH="$STUB:$PATH" "$ROOT/scripts/dev.sh" check
assert_run "dev.sh check: a failing step is reported and fails the run" 1 "FAILED: cargo fmt --check" -- \
  env PATH="$STUB:$PATH" STUB_FMT_FAILS=1 "$ROOT/scripts/dev.sh" check
assert_run "dev.sh check: no printf usage errors leak into the report" 0 "" -- \
  bash -c "! env PATH=\"$STUB:$PATH\" \"$ROOT/scripts/dev.sh\" check 2>&1 | grep -q 'invalid option'"

# ------------------------------------------------ self-maintenance of the docs

# Every dispatcher subcommand must be listed in the usage block, in
# .claude/README.md and in CLAUDE.md. This is the mechanical half of the
# "if the command list changes, the documentation changes" rule.
missing_usage=""
missing_readme=""
while IFS= read -r c; do
  "$ROOT/scripts/dev.sh" help | grep -q "scripts/dev.sh $c" || missing_usage="$missing_usage $c"
  grep -q "scripts/dev.sh $c" "$ROOT/.claude/README.md" || missing_readme="$missing_readme $c"
done < <(sed -n '/^case "$CMD" in$/,/^esac$/p' "$ROOT/scripts/dev.sh" |
         grep -oE '^  [a-z-]+\)' | tr -d ' )' | grep -v '^help$')

if [ -z "$missing_usage" ]; then
  ok "dev.sh: every subcommand appears in its own usage block"
else
  bad "dev.sh: subcommands missing from usage" "$missing_usage"
fi
if [ -z "$missing_readme" ]; then
  ok "dev.sh: every subcommand is documented in .claude/README.md"
else
  bad "dev.sh: subcommands missing from .claude/README.md" \
    "$missing_readme — add them to the tooling table"
fi

# Every workflow file must be referenced from the .claude/README.md index,
# otherwise it is invisible to the next session.
missing_wf=""
for wf in "$ROOT"/.claude/workflows/*.md; do
  base="$(basename "$wf")"
  grep -q "workflows/$base" "$ROOT/.claude/README.md" || missing_wf="$missing_wf $base"
done
if [ -z "$missing_wf" ]; then
  ok "workflows: all are listed in .claude/README.md"
else
  bad "workflows: unlisted" "$missing_wf — add a row to the workflow table"
fi

# Every generated document must carry its do-not-edit banner.
gen_bad=""
for g in docs/architecture/COMPONENTS.md docs/performance/STATUS.md; do
  head -1 "$ROOT/$g" | grep -q "Generated by" || gen_bad="$gen_bad $g"
done
if [ -z "$gen_bad" ]; then
  ok "generated files carry a do-not-edit banner"
else
  bad "generated files missing their banner" "$gen_bad"
fi

# Each decision record must be linked from the decisions index.
adr_bad=""
for adr in "$ROOT"/docs/architecture/decisions/[0-9]*.md; do
  base="$(basename "$adr")"
  grep -q "$base" "$ROOT/docs/architecture/decisions/README.md" ||
    adr_bad="$adr_bad $base"
done
if [ -z "$adr_bad" ]; then
  ok "decision records are all in the index"
else
  bad "decision records not indexed" "$adr_bad — add a row to decisions/README.md"
fi

# ------------------------------------------------- degraded environments

# python3 is optional: a contributor without it must get a clear explanation,
# not a traceback or a silent pass.
# An interpreter too old for tomllib must say so, not raise ModuleNotFoundError.
if [ -x /usr/bin/python3 ] && ! /usr/bin/python3 -c 'import tomllib' >/dev/null 2>&1; then
  assert_run "repo_map: an old interpreter gets an explanation, not a traceback" 2 \
    "needs Python 3.11 or newer" -- /usr/bin/python3 "$DEV/repo_map.py" check
else
  ok "repo_map: old-interpreter path (skipped, no pre-3.11 python3 available)"
fi

# A PATH with a shell but no python3.
NOPY="$TMP/nopy"
mkdir -p "$NOPY"
for b in env bash dirname; do ln -sf "$(command -v "$b")" "$NOPY/$b"; done
assert_run "dev.sh: missing python3 explains itself" 1 "python3 is required for 'generate'" -- \
  env -i PATH="$NOPY" HOME="$HOME" "$ROOT/scripts/dev.sh" generate

# --------------------------------------------------------- pre-commit hook

# Mid-merge, the hook must get out of the way — git re-runs it on the merge
# commit, and blocking here strands the user in a conflicted tree.
HOOKREPO="$TMP/hookrepo"
mkdir -p "$HOOKREPO"
git -c init.defaultBranch=main init -q "$HOOKREPO"
cp "$ROOT/.githooks/pre-commit" "$HOOKREPO/pre-commit"
touch "$HOOKREPO/.git/MERGE_HEAD"
assert_run "pre-commit: skips during a merge" 0 "" -- \
  bash -c "cd '$HOOKREPO' && ./pre-commit"
rm -f "$HOOKREPO/.git/MERGE_HEAD"
assert_run "pre-commit: exits cleanly with nothing staged" 0 "" -- \
  bash -c "cd '$HOOKREPO' && ./pre-commit"

# ------------------------------------------------------- clippy ratchet logic

# Running cargo here would take minutes, so the comparison logic is exercised
# directly with synthetic counts.
cat >"$TMP/clippy_logic.py" <<'PYEOF'
import importlib.util, json, pathlib, sys, tempfile
from collections import Counter

spec = importlib.util.spec_from_file_location("cg", sys.argv[1])
cg = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cg)

tmp = pathlib.Path(tempfile.mkdtemp())
cg.BASELINE = tmp / "baseline.json"
cg.save(Counter({"clippy::a": 3, "clippy::b": 1}))
assert cg.load_baseline() == {"clippy::a": 3, "clippy::b": 1}
assert json.loads(cg.BASELINE.read_text())["total"] == 4

def check(counts):
    cg.run_clippy = lambda: (Counter(counts), [])
    return cg.main(["clippy_gate.py"])

assert check({"clippy::a": 3, "clippy::b": 1}) == 0, "unchanged must pass"
assert check({"clippy::a": 4, "clippy::b": 1}) == 1, "growth must fail"
assert check({"clippy::a": 3, "clippy::b": 1, "clippy::c": 1}) == 1, "new lint must fail"
assert check({"clippy::a": 2, "clippy::b": 1}) == 1, "reduction must be banked"
assert check({"clippy::a": 3}) == 1, "disappearance must be banked"

cg.run_clippy = lambda: (Counter(), ["error: boom"])
assert cg.main(["clippy_gate.py"]) == 1, "hard errors must fail"
print("clippy logic ok")
PYEOF
assert_run "clippy_gate: ratchet logic" 0 "clippy logic ok" -- \
  python3 "$TMP/clippy_logic.py" "$DEV/clippy_gate.py"

assert_run "clippy_gate: baseline file is valid json and current shape" 0 "" -- \
  python3 -c "
import json,pathlib
d=json.loads(pathlib.Path('$ROOT/scripts/dev/clippy-baseline.json').read_text())
assert set(d) == {'comment','total','lints'}, d.keys()
assert d['total'] == sum(d['lints'].values())
assert d['total'] > 0
"

# ------------------------------------------------------- dispatcher behaviour

assert_run "dev.sh: unknown command fails with usage" 2 "unknown command" -- \
  "$ROOT/scripts/dev.sh" definitely-not-a-command
assert_run "dev.sh: help lists the commands" 0 "quality-full" -- \
  "$ROOT/scripts/dev.sh" help
assert_run "dev.sh: hooks status is readable without installing" 0 "" -- \
  "$ROOT/scripts/dev.sh" hooks status
assert_run "dev.sh: session usage error is explicit" 2 "session start|show|end" -- \
  "$ROOT/scripts/dev.sh" session bogus

# ------------------------------------------------------------------- the real repo

assert_run "repository: architecture invariants hold" 0 "architecture OK" -- \
  python3 "$DEV/repo_map.py" check
assert_run "repository: PERF backlog is consistent" 0 "PERF backlog OK" -- \
  python3 "$DEV/perf_index.py" check

printf '\n%d passed, %d failed\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ]
