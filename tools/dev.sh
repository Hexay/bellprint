#!/usr/bin/env bash
# Dev runner: check | test [filter] | py | build | sim-eval [args] | corpus-eval [args] | locate [args]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}"
MANIFEST="$ROOT/Cargo.toml"
BIN="$ROOT/target/fastrel/bellprint"
TS="$(date +%Y%m%d-%H%M%S)"

build() { cargo build --manifest-path "$MANIFEST" -q --profile fastrel -p bp-cli 2>&1 | tail -n 40; }

case "${1:-}" in
  check) cargo clippy --manifest-path "$MANIFEST" --workspace --all-targets -q 2>&1 | tail -n 60 ;;
  test)  shift; cargo test --manifest-path "$MANIFEST" -q -p bp-core ${1:+"$1"} 2>&1 | tail -n 40 ;;
  py)    [[ -d $ROOT/.venv ]] || uv venv -q "$ROOT/.venv"
         (cd "$ROOT" && VIRTUAL_ENV="$ROOT/.venv" maturin develop -q --release 2>&1 | tail -n 20) ;;
  build) build ;;
  sim-eval)    shift; build; (cd "$ROOT" && "$BIN" eval-sim --out "runs/$TS-sim" "$@") ;;
  corpus-eval) shift; build; (cd "$ROOT" && "$BIN" eval-corpus --out "runs/$TS-corpus" "$@") ;;
  locate)      shift; build; (cd "$ROOT" && "$BIN" locate "$@") ;;
  *) echo "usage: tools/dev.sh check|test [filter]|py|build|sim-eval|corpus-eval|locate ..." >&2; exit 2 ;;
esac
