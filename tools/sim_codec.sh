#!/usr/bin/env bash
# Simulate a batch, transcode it through lossy codecs, and evaluate each codec with eval-corpus.
# usage: tools/sim_codec.sh [n] [seed] [codec ...]   codecs: wav mp3_64 aac_96 opus_32 opus_16k
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
N="${1:-100}"; SEED="${2:-1}"; shift $(( $# < 2 ? $# : 2 ))
CODECS=("${@:-wav mp3_64 aac_96 opus_32 opus_16k}")
read -r -a CODECS <<<"${CODECS[*]}"
BIN="$ROOT/target/fastrel/bellprint"
DIR="$ROOT/data/sim/codec-$N-$SEED"
"$ROOT/tools/dev.sh" build
(cd "$ROOT" && "$BIN" sim-batch --n "$N" --seed "$SEED" --out "$DIR/wav" >/dev/null)

encode() {  # codec in out_stem
  case "$1" in
    mp3_64)   ffmpeg -loglevel error -y -i "$2" -c:a libmp3lame -b:a 64k "$3.mp3" ;;
    aac_96)   ffmpeg -loglevel error -y -i "$2" -c:a aac -b:a 96k "$3.m4a" ;;
    opus_32)  ffmpeg -loglevel error -y -i "$2" -c:a libopus -b:a 32k "$3.ogg" ;;
    # WhatsApp-like voice note: 16 kHz wideband Opus.
    opus_16k) ffmpeg -loglevel error -y -i "$2" -ar 16000 -c:a libopus -b:a 16k -application voip "$3.ogg" ;;
  esac
}

for c in "${CODECS[@]}"; do
  if [[ $c != wav ]]; then
    mkdir -p "$DIR/$c"
    for f in "$DIR"/wav/*.wav; do
      out="$DIR/$c/$(basename "$f" .wav)"
      [[ -f $out.flac ]] && continue
      encode "$c" "$f" "$out"
      # Decode back to FLAC: keeps the codec's damage, and symphonia has no Opus decoder.
      enc=("$out".mp3 "$out".m4a "$out".ogg); for e in "${enc[@]}"; do
        [[ -f $e ]] && ffmpeg -loglevel error -y -i "$e" -ar 44100 "$out.flac" && rm "$e"
      done
    done
  else
    mkdir -p "$DIR/wav-flac"
    for f in "$DIR"/wav/*.wav; do
      out="$DIR/wav-flac/$(basename "$f" .wav).flac"
      [[ -f $out ]] || ffmpeg -loglevel error -y -i "$f" "$out"
    done
    c=wav-flac
  fi
  line="$(cd "$ROOT" && "$BIN" eval-corpus --manifest "$DIR/wav/manifest.toml" --audio "$DIR/$c" \
    --out "runs/$(date +%Y%m%d-%H%M%S)-codec-$c" | sed -n 3p)"
  printf '%-10s %s\n' "$c" "$line"
done
