#!/usr/bin/env bash
# Entry point for the Zola large-site performance harness.
#
#   scripts/perf/run.sh build              build the benchmark binary
#   scripts/perf/run.sh baseline           full scenario × size matrix
#   scripts/perf/run.sh quick              small matrix (smoke test, ~1 min)
#   scripts/perf/run.sh proxy <site>       build a content-faithful proxy of a real site
#   scripts/perf/run.sh site <path>        benchmark an external site (or $ZOLA_PERF_SITE)
#   scripts/perf/run.sh threads <path>     parallel efficiency sweep
#   scripts/perf/run.sh scaling <file...>  scaling analysis of result JSON
#   scripts/perf/run.sh ab <a-bin> <b-bin> <site>...   interleaved A/B comparison
#   scripts/perf/run.sh equivalence <baseline-bin> <candidate-bin> [site]
#
# Nothing here ever writes into an external site: builds always target a temp dir.
set -euo pipefail

cd "$(dirname "$0")/../.."
PERF="scripts/perf"
CMD="${1:-help}"
shift || true

case "$CMD" in
  build)
    exec "$PERF/build.sh" "$@"
    ;;
  baseline)
    exec python3 "$PERF/bench.py" matrix --runs 3 --warmup 1 --name baseline-matrix "$@"
    ;;
  quick)
    exec python3 "$PERF/bench.py" matrix --scenarios simple-pages,mixed-realistic \
      --sizes 100,500 --runs 2 --warmup 1 --name quick "$@"
    ;;
  proxy)
    src="${1:?usage: run.sh proxy <path-to-site> [extra args]}"; shift || true
    exec python3 "$PERF/make_proxy_site.py" --source "$src" "$@"
    ;;
  site)
    if [ $# -gt 0 ]; then
      exec python3 "$PERF/bench.py" site --path "$1" "${@:2}"
    fi
    exec python3 "$PERF/bench.py" site
    ;;
  threads)
    exec python3 "$PERF/bench.py" threads "$@"
    ;;
  scaling)
    exec python3 "$PERF/scaling.py" "$@"
    ;;
  ab)
    a="${1:?usage: run.sh ab <a-bin> <b-bin> <site>...}"; shift
    b="${1:?usage: run.sh ab <a-bin> <b-bin> <site>...}"; shift
    [ $# -gt 0 ] || { echo "usage: run.sh ab <a-bin> <b-bin> <site>..." >&2; exit 2; }
    sites=()
    for s in "$@"; do sites+=(--site "$s"); done
    exec python3 "$PERF/ab.py" --a "$a" --b "$b" --rounds 3 --warmup "${sites[@]}"
    ;;
  equivalence)
    base="${1:?usage: run.sh equivalence <baseline-bin> <candidate-bin> [site]}"
    cand="${2:?usage: run.sh equivalence <baseline-bin> <candidate-bin> [site]}"
    site="${3:-benchmarks/sites/mixed-realistic-1000}"
    exec python3 "$PERF/compare_output.py" --site "$site" --baseline "$base" --candidate "$cand"
    ;;
  help|*)
    sed -n '2,20p' "$0"
    ;;
esac
