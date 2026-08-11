#!/usr/bin/env bash
# Canonical entry point for working on Zola.
#
#   scripts/dev.sh doctor          what this machine can do
#   scripts/dev.sh check           fast feedback: format + type check
#   scripts/dev.sh quality         the gate: format + lint + tests
#   scripts/dev.sh quality-full    quality + generated files + tooling tests
#   scripts/dev.sh generate        rewrite every generated document
#   scripts/dev.sh impact [args]   what did I change, how risky, what docs
#   scripts/dev.sh clippy [args]   lint ratchet: --list, --update
#   scripts/dev.sh map             workspace map + architecture invariants
#   scripts/dev.sh perf-index      PERF-* backlog integrity (check|open|json)
#   scripts/dev.sh test-tooling    tests for the scripts in this directory
#   scripts/dev.sh session <cmd>   start | show | end  (see .claude/README.md)
#   scripts/dev.sh hooks <cmd>     install | uninstall | status
#   scripts/dev.sh perf [args]     forwards to scripts/perf/run.sh
#
# None of this is required to contribute to Zola: `cargo build --all` and
# `cargo test --all` remain the whole story. This script exists so that "is my
# branch healthy?" has exactly one answer, and so that answer matches CI.
#:end-usage
set -uo pipefail

cd "$(dirname "$0")/.."
ROOT="$(pwd)"

# A global ~/.cargo/config.toml that injects target rustflags (lto, panic) makes
# the workspace fail to compile under clippy and silently changes what a release
# build measures. Gate runs neutralise it so a local gate means the same thing as
# a CI gate. Set ZOLA_DEV_KEEP_RUSTFLAGS=1 if you know what yours are doing.
if [ "${ZOLA_DEV_KEEP_RUSTFLAGS:-0}" != "1" ]; then
  export RUSTFLAGS="${ZOLA_DEV_RUSTFLAGS:-}"
fi

FAILURES=()

step() {
  local label="$1"
  shift
  printf '\n=== %s\n' "$label"
  # `printf '--- ...'` is parsed as an option by bash's builtin; keep the
  # leading dashes inside an argument.
  if "$@"; then
    printf '%s\n' "--- $label: PASS"
  else
    printf '%s\n' "--- $label: FAIL"
    FAILURES+=("$label")
  fi
}

summary() {
  printf '\n'
  if [ ${#FAILURES[@]} -eq 0 ]; then
    echo "ALL PASS"
    return 0
  fi
  echo "FAILED: ${FAILURES[*]}"
  echo
  echo "Fix the first failure and re-run. Do not report work as done while this"
  echo "command fails."
  return 1
}

# The usage block is the comment header, up to the :end-usage marker, so the
# two can never drift apart.
usage() { sed -n '2,/^#:end-usage/p' "$0" | sed -e 's/^#:end-usage//' -e 's/^# \{0,1\}//'; }

has_clippy() { cargo clippy --version >/dev/null 2>&1; }
# The generators parse Cargo manifests with tomllib, which landed in 3.11.
has_python() { python3 -c 'import sys, tomllib' >/dev/null 2>&1; }

require_python() {
  if has_python; then
    return 0
  fi
  if command -v python3 >/dev/null 2>&1; then
    echo "'$1' needs Python 3.11 or newer (for tomllib); found $(python3 -V 2>&1)." >&2
  else
    echo "python3 is required for '$1' but is not installed." >&2
  fi
  echo "    Install a newer python3, or skip this check — it is not needed to" >&2
  echo "    build or test Zola." >&2
  return 1
}

cmd_check() {
  step "cargo fmt --check" cargo fmt --check
  step "cargo check --workspace --all-targets" cargo check --workspace --all-targets
  summary
}

quality_steps() {
  step "cargo fmt --check" cargo fmt --check
  if has_clippy && has_python; then
    step "clippy (no new warnings)" python3 scripts/dev/clippy_gate.py
  elif has_clippy; then
    step "cargo clippy --workspace --all-targets" cargo clippy --workspace --all-targets
  else
    printf '\n=== clippy: SKIPPED (not installed; rustup component add clippy)\n'
  fi
  step "cargo test --workspace" cargo test --workspace
}

cmd_quality() {
  quality_steps
  summary
}

cmd_quality_full() {
  quality_steps
  if has_python; then
    step "generated files up to date" "$ROOT/scripts/dev.sh" generate --check
    step "tooling tests" "$ROOT/scripts/dev.sh" test-tooling
    printf '\n=== change impact (informational)\n'
    python3 scripts/dev/impact.py || true
  else
    printf '\n=== generated files / tooling tests: SKIPPED (python3 not installed)\n'
  fi
  summary
}

cmd_generate() {
  require_python generate || return 1
  local mode="${1:-write}"
  if [ "$mode" = "--check" ]; then
    local rc=0
    python3 scripts/dev/repo_map.py check || rc=1
    python3 scripts/dev/perf_index.py check || rc=1
    return $rc
  fi
  python3 scripts/dev/perf_index.py generate || return 1
  python3 scripts/dev/repo_map.py generate || return 1
}

cmd_session() {
  local sub="${1:-show}"
  shift || true
  local dir=".claude/context"
  local file="$dir/session.md"
  mkdir -p "$dir"
  case "$sub" in
    start)
      if [ -f "$file" ]; then
        echo "$file already exists — finish or archive the current session first:"
        echo "    scripts/dev.sh session end"
        return 1
      fi
      {
        sed -e "s|{{DATE}}|$(date -u +%Y-%m-%d)|" .claude/templates/session.md
        echo
        echo '## Collected state'
        echo
        echo '```'
        echo "date        $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "branch      $(git branch --show-current 2>/dev/null || echo '(detached)')"
        echo "commit      $(git rev-parse HEAD 2>/dev/null || echo '?')"
        echo "worktree    $(if [ -n "$(git status --porcelain)" ]; then echo dirty; else echo clean; fi)"
        echo "rustc       $(rustc --version 2>/dev/null)"
        echo "cargo       $(cargo --version 2>/dev/null)"
        echo "os          $(uname -srm)"
        echo "cpus        $(getconf _NPROCESSORS_ONLN 2>/dev/null || echo '?')"
        echo '```'
        echo
        echo '### Open PERF items'
        echo
        if has_python; then
          python3 scripts/dev/perf_index.py open 2>/dev/null ||
            echo '* (perf index unavailable)'
        else
          echo '* (python3 not installed)'
        fi
      } >"$file"
      echo "wrote $file — fill in Objective before you start changing code."
      ;;
    show)
      if [ -f "$file" ]; then cat "$file"; else
        echo "no active session. Start one with: scripts/dev.sh session start"
      fi
      ;;
    end)
      if [ ! -f "$file" ]; then
        echo "no active session to end."
        return 1
      fi
      echo "Before ending, the session record must answer all of these:"
      echo
      sed -n '/^## Handoff checklist/,$p' .claude/templates/session.md
      echo
      echo "Durable findings do not live in $file. Move them to:"
      echo "  * docs/performance/OPTIMIZATIONS.md   measured results of a PERF item"
      echo "  * docs/performance/HOTSPOTS.md        a new or revised backlog item"
      echo "  * docs/architecture/decisions/        a decision with lasting consequences"
      echo "  * CHANGELOG.md                        anything a Zola user would notice"
      echo
      echo "Then archive the session file:"
      echo "    mv $file $dir/\$(date -u +%Y-%m-%d)-<slug>.md"
      ;;
    *)
      echo "usage: scripts/dev.sh session start|show|end" >&2
      return 2
      ;;
  esac
}

cmd_hooks() {
  local sub="${1:-status}"
  case "$sub" in
    install)
      git config core.hooksPath .githooks
      echo "git hooks enabled from .githooks/ (pre-commit runs 'scripts/dev.sh check')."
      echo "Disable again with: scripts/dev.sh hooks uninstall"
      ;;
    uninstall)
      git config --unset core.hooksPath 2>/dev/null || true
      echo "repository hooks disabled; git uses .git/hooks again."
      ;;
    status)
      local path
      path="$(git config core.hooksPath || true)"
      if [ "$path" = ".githooks" ]; then
        echo "enabled (core.hooksPath=.githooks)"
      else
        echo "not enabled. Opt in with: scripts/dev.sh hooks install"
      fi
      ;;
    *)
      echo "usage: scripts/dev.sh hooks install|uninstall|status" >&2
      return 2
      ;;
  esac
}

CMD="${1:-help}"
shift || true

case "$CMD" in
  doctor)       exec scripts/dev/doctor.sh "$@" ;;
  check)        cmd_check ;;
  quality)      cmd_quality ;;
  quality-full) cmd_quality_full ;;
  generate)     cmd_generate "$@" ;;
  impact)       require_python impact && exec python3 scripts/dev/impact.py "$@" ;;
  map)          require_python map && exec python3 scripts/dev/repo_map.py "${1:-check}" ;;
  perf-index)   require_python perf-index && exec python3 scripts/dev/perf_index.py "${1:-check}" ;;
  clippy)       require_python clippy && exec python3 scripts/dev/clippy_gate.py "$@" ;;
  test-tooling) exec scripts/dev/tests/run.sh "$@" ;;
  session)      cmd_session "$@" ;;
  hooks)        cmd_hooks "$@" ;;
  perf)         exec scripts/perf/run.sh "$@" ;;
  help|--help|-h) usage ;;
  *)
    echo "unknown command: $CMD" >&2
    usage >&2
    exit 2
    ;;
esac
