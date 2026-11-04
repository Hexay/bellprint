#!/usr/bin/env bash
# Dump `locate --json --top 400` for every corpus miss in an eval's cases.json (research/miss_triage.md).
# usage: tools/triage/locate_misses.sh <bellprint bin> <cases.json> <out dir> [audio dir]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN=$1 CASES=$2 OUT=$3 AUD=${4:-$ROOT/data/corpus/audio}
mkdir -p "$OUT/loc"
ids=$(python3 -I -c "import json,sys;print(' '.join(c['id'] for c in json.load(open(sys.argv[1])) if c['rank']!=1))" "$CASES")
for id in $ids; do
  "$BIN" locate "$AUD/$id.flac" --json --top 400 > "$OUT/loc/$id.json" || echo "FAIL $id"
done
