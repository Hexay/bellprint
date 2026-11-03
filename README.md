# bellprint

[![CI](https://github.com/Hexay/bellprint/actions/workflows/ci.yml/badge.svg)](https://github.com/Hexay/bellprint/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Work out which church tower you can hear in a recording, from the pitches of its bells.

**Status: early research software.** On simulated recordings it ranks the right tower first about half the
time. On real recordings it mostly fails so far (3 of 44). See [Accuracy](#accuracy).

## The idea

[Dove's Guide](https://dove.cccbr.org.uk/) lists the rings of bells in about 7,000 towers, and for most bells
the frequency of its *nominal*, the partial that sets its pitch. A viral post claimed that a tower's tenor (its
lowest bell) is enough to pick it out from a few seconds of video. It isn't. Of 5,976 English rings with a
tenor frequency there are only 1,553 distinct values, and a typical tenor matches 19 towers to within ±0.5 Hz.

The whole ring is far more distinctive. bellprint uses the number of bells, their intervals and their absolute
pitch, matched against the per-bell nominals in Dove (46,983 bells; 6,362 rings with every bell measured).

## Quick start

```bash
cargo run --release -p bp-cli -- locate recording.flac
cargo run --release -p bp-cli -- locate recording.mp3 --near 53.2,-1.6,30 --start 12 --end 40
```

`--near lat,lon[,km]` restricts the search to a circle. `--start` and `--end` select the part of the file where
bells are ringing, which matters (see below). `--json` prints the full report. From Python (`tools/dev.sh py`):
`bellprint.locate("recording.flac")` returns the same report as a dict.

Audio is decoded natively (WAV, FLAC, MP3, AAC/M4A, Ogg Vorbis).

## How it works

1. **Lines.** Average the clip's power spectrum (0.7 s Hann windows) and keep the lines that stand out from a
   moving-median floor.
2. **Bell model.** Each bell predicts lines at its hum, prime, tierce, quint, nominal, superquint and octave
   nominal, each with a tolerance in cents taken from published bell measurements
   ([research/acoustics.md](research/acoustics.md)).
3. **Match.** Score every Dove ring. It gains for each of its nominals that is present, pays for each that is
   missing, and is judged on how well its predicted partials explain the observed lines (a density ratio
   against background, so large rings don't win by covering everything). A single pitch shift is fitted per
   clip, with priors at the offsets that sample-rate (44.1/48 kHz) and frame-rate (24/25 fps) mix-ups produce.
4. **Rank.** Return candidate towers with relative probabilities.

An experimental per-blow extractor (`--extraction blows`) detects strikes and keeps only lines that rise at
blows. It currently does worse than the long-term spectrum.

## Accuracy

| Benchmark | Setup | Result |
|---|---|---|
| Simulated, 200 random Dove rings | 8 s of rounds, 10 dB SNR, distractor tones, ±8 c speed error, ±3 c Dove error, 5 % retuned rings; whole catalogue searched | Top-1 **46 %**, top-5 70 %, top-20 87 %, median rank 2 |
| Real recordings, 44 named towers ([data/corpus](data/corpus/README.md)) | Whole clips (13 s to 17 min), no `--near` | Top-1 **3 / 44**, top-5 5 / 44 |

What these show:

- The simulator is independent of the matcher's bell model but is still kinder than reality: no reverb, no
  automatic gain control, and steady ringing for the whole clip.
- Real clips fail mostly because the whole file is averaged: speech, traffic and silence swamp the bells.
  Selecting the ringing (`--start/--end`) and a region (`--near`) both help. Choosing that window automatically
  is the next piece of work.
- Chimes, clock bells and carillons are mostly absent from Dove, so those recordings can't match.
- The viral claim has not been reproduced. The original post could not be traced
  ([research/prior_work.md](research/prior_work.md)).

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md). Background research with sources is in [research/](research/README.md).

## Responsible use

This tool can place a recording, and so the person who made it, on a map. Use it on your own recordings, on
material published for this purpose, or with the recordist's consent. Don't use it to find, track or expose
people.

## Credits and data

- Bell data: [Dove's Guide for Church Bell Ringers](https://dove.cccbr.org.uk/), © Central Council of Church
  Bell Ringers, [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). The files in `data/dove/` keep
  that licence; see [data/dove/README.md](data/dove/README.md). Not endorsed by the Central Council.
- Bell acoustics: Bill Hibbert's published measurements and thesis, among the sources in [research/](research/).
- Structure follows [dopplerprint](https://github.com/Hexay/dopplerprint).
- Real-recording corpus: links only, to recordings by their owners. No audio is redistributed.

## Licence

Code: MIT, see [LICENSE](LICENSE). Data keeps its own licence (above).
