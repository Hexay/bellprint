# Real-recording benchmark corpus

Real recordings of church bells, each tied to a Dove's Guide tower (`tower_id`) and ring (`ring_id`), for
benchmarking tower identification. `eval-corpus` reads every `manifest*.toml` here and scores a recording once
even if two manifests list the same URL. Fetch the audio with:

    python3 -I tools/fetch_corpus.py --manifest data/corpus/<manifest>.toml [--only id,id] [--cookies cookies.txt]

Each recording is saved as mono 44.1 kHz FLAC at `audio/<id>.flac`, trimmed to `start_s`/`end_s` when set.
**The audio is not redistributed**: `audio/` is gitignored and each user fetches it locally.

## Manifests

| File | Sources | How the tower is identified |
|---|---|---|
| `manifest.toml` | archive.org (mostly Radio Aporee), Freesound, YouTube | Title, description or tags name the church |
| `manifest_commons_archive.toml` | archive.org | Item text names the church |
| `manifest_geotagged.toml` | Radio Aporee (via its archive.org mirror), geotagged Freesound | Nearest Dove tower to the geotag (≤300 m, or ≤1 km if the text also names it) |
| `manifest_web.toml` | BBC Sound Effects (RemArc licence, research use), SoundCloud, Vimeo | Title or catalogue text names the church |

Each entry's `licence` field holds its exact licence, and `identity_evidence` quotes the text or geotag used.
Current counts by confidence, source and fetched audio:

    for f in data/corpus/manifest*.toml; do python3 -I -c "import tomllib,sys,collections as c; r=tomllib.load(open(sys.argv[1],'rb'))['recording']; print(sys.argv[1], len(r), dict(c.Counter(x['confidence'] for x in r)))" $f; done

## Selection rules

- Tower identity never comes from Dove's `Hz`/`Note` columns or the `bells.csv` nominals, which were not read
  during selection or verification, so the answer does not leak into the benchmark.
- Confidence: `low` when more than one ring could be sounding (clock chimes, carillons, nearby towers);
  `medium` when the identity is clear but the recording has a caveat (distant, walk-past, edited, title-only);
  `high` otherwise.
- Sites' robots.txt is respected: BellBoard and Freesound search pages were not crawled, and aporee.org's map is
  reached through its archive.org mirror.

## Known caveats

- **YouTube is bot-walled from the build host**, so its 17 entries in `manifest.toml` rest on the title alone and
  have no audio here. Fetch them with `--cookies` and a cookies.txt from a signed-in browser. They hold the only
  3-, 4- and 5-bell rings and the Scotland and USA entries.
- `ringing` is `unknown` for most field recordings, because uploaders rarely say what was rung.
- Some recordings ring a subset of the bells (`stepney-stedman` back 8 of 10, `alfriston` 5 of 6), are short or
  looped (`shirenewton`, `perth-swan-bells`), distant, or in cities where neighbouring towers are audible.
- `exeter-cathedral-practice` is capped at the first 20 min of a 32 min practice; `ap-worcester-remembrance`
  runs 45 min.
- Most of the 4,000-odd Aporee bell recordings are not tower ringing (clocks, shops, meditation bells); only
  those within reach of a Dove tower were considered.
