# bellprint

![Three panels: the spectrum of a real recording at Oxford, St Ebbe with its bells' nominals marked; a map of the 4 towers whose tenor shares that pitch; and a map where matching the whole ring ranks St Ebbe first.](docs/img/header.png)

[![CI](https://github.com/Hexay/bellprint/actions/workflows/ci.yml/badge.svg)](https://github.com/Hexay/bellprint/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/bellprint.svg)](https://pypi.org/project/bellprint/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Work out which church tower you can hear in a recording, from the pitches of its bells.

**Status: early research software.** On simulated recordings it puts the right tower first half the time; on
real recordings, about one time in ten. See [Accuracy](#accuracy).

## Why the whole ring

[Dove's Guide](https://dove.cccbr.org.uk/) lists about 7,000 rings of bells and, for most bells, the frequency
of its *nominal*, the partial that sets its pitch. The biggest bell (the tenor) on its own is not a fingerprint:
5,976 English rings with a recorded tenor share just 1,553 distinct frequencies, and a typical tenor matches 19
towers to within ±0.5 Hz.

A whole ring is far more distinctive: how many bells, the intervals between them, and their absolute pitch.
bellprint matches all of that against Dove's per-bell nominals (46,983 bells; 6,362 rings with every bell
measured).

## Quick start

```bash
pip install bellprint
bellprint locate recording.flac
bellprint locate recording.mp3 --near 51.75,-1.26,20 --start 12 --end 40 --json
```

| Option | Does |
|---|---|
| `--near lat,lon[,km]` | Only consider towers in this circle (default radius 30 km) |
| `--start`, `--end` | Seconds of the file to analyse; pick the part where bells ring |
| `--top N` | How many candidates to list |
| `--json` | Full report: spectral lines, candidates, scores |

Audio is decoded natively: WAV, FLAC, MP3, AAC/M4A, Ogg Vorbis. A snapshot of Dove's data is built in; point
`--dove` at a directory with a newer `dove.csv` and `bells.csv` to use those instead. From Python,
`bellprint.locate("recording.flac", near="51.75,-1.26,20")` returns the same report as a dict. Prebuilt wheels
cover Linux, macOS and Windows; from a checkout, `cargo run --release -p bp-cli -- locate ...` does the same.

## How it works

1. **Lines.** Take the 75th percentile of the clip's power spectrum over time, which keeps sustained bell
   partials and damps brief noises such as speech or traffic. Keep the lines that stand out from a
   moving-median floor.
2. **Bell model.** Each bell predicts lines at its hum, prime, tierce, quint, nominal, superquint and octave
   nominal, with tolerances in cents from published bell measurements ([research/acoustics.md](research/acoustics.md)).
3. **Match.** Score every Dove ring on how many of its nominals are present and how well its predicted
   partials explain the observed lines, compared with chance. One pitch shift is fitted per clip, with priors
   at the offsets that 44.1/48 kHz and 24/25 fps mix-ups produce.
4. **Rank.** List candidate towers with relative probabilities.

Two experimental options did worse than this default and stay opt-in: `--extraction blows` (keep only lines
that rise when a bell strikes) and `--auto-window` (analyse only the most bell-like 10 s windows).

## Accuracy

| Benchmark | Setup | Right tower first | In top 5 |
|---|---|---|---|
| Simulated, 200 random Dove rings | 8 s of rounds, 10 dB SNR, distractor tones, ±8 c speed and ±3 c Dove error, 5 % of rings retuned | **51 %** | 77 % |
| Real recordings of 141 named towers ([data/corpus](data/corpus/README.md)) | Whole clips as published, no `--near` | **18 / 141** | 32 / 141 |

- The simulator shares no code or constants with the matcher's bell model, but it is kinder than reality: no
  reverb, no automatic gain control, and ringing for the whole clip.
- Every real hit is far above chance (about 1 in 7,000 per clip), but most real clips still miss.
  Chimes, clock bells and carillons are largely absent from Dove and can't match at all.
- The viral claim that a few seconds of the tenor identify a tower has not been reproduced, and its original
  post could not be traced ([research/prior_work.md](research/prior_work.md)).

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md). Sourced background research is in [research/](research/README.md);
the header is drawn by `tools/readme_figure.py` from a real corpus clip.

## Responsible use

This tool can place a recording, and so the person who made it, on a map. Use it on your own recordings, on
material published for this purpose, or with the recordist's consent. Don't use it to find, track or expose
people.

## Credits and data

- Bell data: [Dove's Guide for Church Bell Ringers](https://dove.cccbr.org.uk/), © Central Council of Church
  Bell Ringers, [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/). The files in `data/dove/` keep
  that licence ([data/dove/README.md](data/dove/README.md)). Not endorsed by the Central Council.
- Bell acoustics: Bill Hibbert's published measurements and thesis, among the sources in [research/](research/).
- Real-recording corpus: links to recordings on Freesound, archive.org (including Radio Aporee), BBC Sound
  Effects, SoundCloud and Vimeo, credited per entry in the manifests. No audio is redistributed.
- Structure follows [dopplerprint](https://github.com/Hexay/dopplerprint).

## Licence

Code: MIT, see [LICENSE](LICENSE). Data keeps its own licence (above).
