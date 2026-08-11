#!/usr/bin/env bash
# Build the zola binary for benchmarking with a pinned, reproducible profile.
#
# Why this script exists instead of a plain `cargo build --release`:
#
#   1. A developer's global ~/.cargo/config.toml can override the workspace
#      release profile (lto, codegen-units, panic) and inject target rustflags.
#      On the machine this program was developed on, `-C lto=thin` in
#      [target.aarch64-apple-darwin].rustflags makes build scripts fail to
#      compile at all, and `panic = "abort"` silently changes the binary.
#   2. rustc-wrapper=sccache introduces cache-hit/miss variance in build time
#      (not in the benchmark itself, but it makes "did the binary change?"
#      harder to reason about).
#
# So we neutralise RUSTFLAGS and the wrapper and pin the profile explicitly to
# what components' Cargo.toml asks for.
#
# Usage:
#   scripts/perf/build.sh                  # release (benchmark binary)
#   scripts/perf/build.sh profiling        # release + debug symbols (samply)
set -euo pipefail

PROFILE="${1:-release}"
cd "$(dirname "$0")/../.."

export RUSTFLAGS=""
export RUSTC_WRAPPER=""
export CARGO_PROFILE_RELEASE_LTO=true
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_PANIC=unwind
export CARGO_PROFILE_RELEASE_STRIP=true
export CARGO_PROFILE_RELEASE_INCREMENTAL=false

case "$PROFILE" in
  release)
    echo "[build] release (pinned: lto=true codegen-units=1 panic=unwind strip=true)"
    exec cargo build --release
    ;;
  profiling)
    echo "[build] profiling (release + full debug symbols, no strip)"
    exec cargo build --profile profiling
    ;;
  *)
    echo "unknown profile: $PROFILE (expected: release | profiling)" >&2
    exit 2
    ;;
esac
