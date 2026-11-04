#!/usr/bin/env bash
# A/B the working-tree matcher against HEAD's: freeze both binaries, run sim + corpus with each.
# usage: tools/ab_matcher.sh [n_sim]   → runs/<ts>-ab/{new,old}/...
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
N="${1:-200}"
OUT="$ROOT/runs/$(date +%Y%m%d-%H%M%S)-ab"
FILES=(crates/bp-core/src/matcher/model.rs crates/bp-core/src/matcher/score.rs)
mkdir -p "$OUT/new" "$OUT/old" "$OUT/saved"
cd "$ROOT"

"$ROOT/tools/dev.sh" build && cp target/fastrel/bellprint "$OUT/new/bellprint"
for f in "${FILES[@]}"; do cp "$f" "$OUT/saved/$(basename "$f")"; git show "HEAD:$f" > "$f"; done
restore() { for f in "${FILES[@]}"; do cp "$OUT/saved/$(basename "$f")" "$f"; done; }
trap restore EXIT
"$ROOT/tools/dev.sh" build && cp target/fastrel/bellprint "$OUT/old/bellprint"
restore; trap - EXIT

for v in new old; do
  s=$SECONDS; sim="$("$OUT/$v/bellprint" eval-sim --n "$N" --seed 1 --out "$OUT/$v/sim" | sed -n 3p)"; ts=$((SECONDS - s))
  s=$SECONDS; corpus="$("$OUT/$v/bellprint" eval-corpus --out "$OUT/$v/corpus" | sed -n 3p)"; tc=$((SECONDS - s))
  printf '%s sim (%ss):    %s\n%s corpus (%ss): %s\n' "$v" "$ts" "$sim" "$v" "$tc" "$corpus"
done
# A pure speed-up must leave every case (ranks, probabilities) byte-identical, not just the summary.
for k in sim corpus; do
  cmp -s "$OUT/new/$k/cases.json" "$OUT/old/$k/cases.json" && echo "$k cases.json: identical" || echo "$k cases.json: DIFFER"
done
echo "$OUT"
