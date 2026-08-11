#!/usr/bin/env bash
# Reports what this machine can do. Exits non-zero only when something *required*
# to build and test Zola is missing — a contributor who never runs a benchmark
# must never be blocked by a missing profiler.
set -uo pipefail

cd "$(dirname "$0")/../.."

missing_required=0
missing_recommended=0

say() { printf '%-12s %-10s %s\n' "$1" "$2" "$3"; }

report() {
  local tier="$1" name="$2" why="$3"
  shift 3
  if command -v "$name" >/dev/null 2>&1; then
    local version
    version="$("$@" 2>/dev/null | head -1)"
    say "$tier" "ok" "$name — ${version:-present}"
  else
    case "$tier" in
      required) missing_required=$((missing_required + 1)) ;;
      recommended) missing_recommended=$((missing_recommended + 1)) ;;
    esac
    say "$tier" "MISSING" "$name — $why"
  fi
}

echo "Toolchain"
echo "---------"
report required    git    "version control"                              git --version
report required    rustc  "the compiler"                                 rustc --version
report required    cargo  "the build tool"                               cargo --version

if cargo fmt --version >/dev/null 2>&1; then
  say required ok "rustfmt — $(cargo fmt --version 2>/dev/null)"
else
  missing_required=$((missing_required + 1))
  say required MISSING "rustfmt — required by the format gate; rustup component add rustfmt"
fi

if cargo clippy --version >/dev/null 2>&1; then
  say recommended ok "clippy — $(cargo clippy --version 2>/dev/null)"
else
  missing_recommended=$((missing_recommended + 1))
  say recommended MISSING "clippy — the lint gate; rustup component add clippy"
fi

if python3 -c 'import sys, tomllib' >/dev/null 2>&1; then
  say recommended ok "python3 — $(python3 --version 2>&1)"
elif command -v python3 >/dev/null 2>&1; then
  missing_recommended=$((missing_recommended + 1))
  say recommended OLD "python3 — $(python3 --version 2>&1); the generators need 3.11+ (tomllib)"
else
  missing_recommended=$((missing_recommended + 1))
  say recommended MISSING "python3 — generators, benchmark harness and reports"
fi

echo
echo "Performance work (only needed for benchmarking and profiling)"
echo "-------------------------------------------------------------"
report optional hyperfine "benchmark runner used by scripts/perf/bench.py; brew install hyperfine" hyperfine --version
report optional samply    "sampling profiler used for CPU profiles; cargo install samply"          samply --version
report optional shellcheck "lints the shell scripts under scripts/; brew install shellcheck"       shellcheck --version

echo
echo "Environment"
echo "-----------"
say info "" "os          $(uname -srm)"
if command -v getconf >/dev/null 2>&1; then
  say info "" "cpus        $(getconf _NPROCESSORS_ONLN 2>/dev/null || echo '?')"
fi
say info "" "repo        $(pwd)"
say info "" "branch      $(git branch --show-current 2>/dev/null || echo '(detached)')"
say info "" "commit      $(git rev-parse --short HEAD 2>/dev/null || echo '?')"
if [ -n "$(git status --porcelain 2>/dev/null)" ]; then
  say info "" "worktree    dirty — benchmark results measured now are not reproducible from a commit"
else
  say info "" "worktree    clean"
fi

# A global cargo config that injects target rustflags is invisible until a build
# fails in a confusing way, so name it here.
cargo_cfg="${CARGO_HOME:-$HOME/.cargo}/config.toml"
if [ -f "$cargo_cfg" ] && grep -qE '^\s*rustflags' "$cargo_cfg"; then
  echo
  echo "Note"
  echo "----"
  echo "$cargo_cfg sets rustflags for this machine."
  echo "Those flags leak into every build here: they can make clippy fail to compile"
  echo "the workspace, and they change what a release build measures."
  echo "scripts/dev.sh and scripts/perf/build.sh clear RUSTFLAGS for that reason."
  echo "A plain 'cargo clippy' run outside those scripts may fail; use:"
  echo "    RUSTFLAGS= cargo clippy --workspace --all-targets"
fi

echo
if [ "$missing_required" -gt 0 ]; then
  echo "FAIL: $missing_required required tool(s) missing. Install them before building."
  exit 1
fi
if [ "$missing_recommended" -gt 0 ]; then
  echo "OK with gaps: $missing_recommended recommended tool(s) missing."
  echo "You can build and test; some gates in 'scripts/dev.sh quality' will be skipped."
  exit 0
fi
echo "OK: everything needed to build, test and benchmark is present."
