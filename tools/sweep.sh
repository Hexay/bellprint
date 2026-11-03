#!/usr/bin/env bash
# Run eval-sim once per matcher TOML on the same seeds; print one summary line each.
# usage: tools/sweep.sh experiments/a.toml[:long_term|:blows] ... [-- extra eval-sim args]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cfgs=(); while [[ $# -gt 0 && $1 != "--" ]]; do cfgs+=("$1"); shift; done; [[ ${1:-} == "--" ]] && shift
TS="$(date +%Y%m%d-%H%M%S)"
# Build once and run a frozen copy, so source edits during a long sweep can't leak into it.
"$ROOT/tools/dev.sh" build
mkdir -p "$ROOT/runs/$TS-sweep"
BIN="$ROOT/runs/$TS-sweep/bellprint"
cp "$ROOT/target/fastrel/bellprint" "$BIN"
cd "$ROOT"
for entry in "${cfgs[@]}"; do
  # entry: config.toml[:extraction]
  c="${entry%%:*}"; ex="long_term"; [[ $entry == *:* ]] && ex="${entry#*:}"
  name="$(basename "$c" .toml)-$ex"
  line="$("$BIN" --matcher "$c" --extraction "$ex" eval-sim --out "runs/$TS-sweep/$name" "$@" | sed -n 3p)"
  printf '%-24s %s\n' "$name" "$line"
done
