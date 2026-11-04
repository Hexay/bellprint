# Contributing

Issues and pull requests are welcome. For anything bigger than a fix, open an issue first so the approach
can be agreed before you write it.

## Setup and the dev loop

Needs Rust ≥ 1.90 and, for the Python package, Python ≥ 3.11 with [uv](https://docs.astral.sh/uv/).
`cargo run --release -p bp-cli -- locate ...` runs the same `bellprint` command as the Python package
(`tools/dev.sh py` builds that into `.venv`).

| Command | Does |
|---|---|
| `tools/dev.sh check` | clippy on the whole workspace (CI runs it with `-D warnings`) |
| `tools/dev.sh test [filter]` | bp-core tests |
| `tools/dev.sh py` | `maturin develop` the `bellprint` package into `.venv` |
| `tools/sweep.sh experiments/x.toml[:blows\|:long_term] -- --n 200` | simulated A/B → `runs/<ts>-sweep/` |
| `tools/dev.sh corpus-eval` | real-recording benchmark → `runs/<ts>-corpus/summary.md` |
| `tools/sim_codec.sh [n] [seed]` | simulated recordings through MP3/AAC/Opus, scored per codec |
| `bellprint [--matcher x.toml] replay <eval out>/clips.json` | re-rank an eval's saved lines without audio (matcher A/B in seconds) |

The real corpus is listed in `data/corpus/manifest.toml`; fetch its audio with
`python3 -I tools/fetch_corpus.py` (needs yt-dlp and ffmpeg). The audio is not redistributed. A change that
claims an accuracy gain should come with an `experiments/<name>.toml` and before/after numbers in the PR.

## Rules the code holds to

- All computation lives in `bp-core`. `bp-cli` and `bp-py` are thin wrappers.
- One module, one responsibility; keep files under 300 lines.
- `sim` is the independent truth: it must not reuse the matcher's partial model (`matcher::model`).
- Pitch is compared in cents, with one global shift per clip for speed and resampling errors.
- Files under `data/` are untrusted input (run Python on them with `python3 -I`).
- Comments only for non-obvious logic, gotchas and TODOs.

By contributing you agree your code is released under the MIT licence (`LICENSE`). Dove's data under
`data/dove/` keeps its CC BY-SA 4.0 licence.
