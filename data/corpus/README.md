# Real-recording benchmark corpus

Real recordings of church bells, each tied to a Dove's Guide tower (`tower_id`) and ring (`ring_id`). Use them to
benchmark bellprint's tower identification. The list lives in `manifest.toml`. Fetch the audio with:

    python3 -I tools/fetch_corpus.py [--only id,id] [--cookies cookies.txt]

Each recording is saved as mono 44.1 kHz FLAC at `audio/<id>.flac`, trimmed to `start_s`/`end_s` when they are set.

## Sources and licences

- **archive.org**: 21 items, mostly Radio aporee sound-map field recordings, plus London Sound Survey (Southwark) and
  Flickr video mirrors (Ballarat, Perth). Licences: CC-BY-SA-3.0, CC-BY-NC-3.0, CC-BY-NC-SA-3.0, PDM-1.0, CC-BY-2.0,
  CC-BY-SA-2.0. `woolpit` has no licence stated.
- **Freesound**: 22 sounds under CC0-1.0, CC-BY-3.0/4.0 or CC-BY-NC-3.0/4.0. The script fetches the public HQ preview
  (128 kbps mp3), because original files need an API key.
- **YouTube**: 17 videos under the standard YouTube licence.

Each entry's `licence` field holds its exact licence. **The audio is not redistributed.** `audio/` is gitignored
(`data/corpus/.gitignore`), so each user fetches it locally.

## Selection rules

- Tower identity comes only from the title, description or tags naming the church or place, matched to Dove on place,
  dedication and county. `identity_evidence` quotes that text.
- The Dove `Hz`/`Note` columns and the `bells.csv` nominals were never read during selection or verification, so the
  answer does not leak into the benchmark.
- Confidence:
  - `low`: more than one ring could be sounding, for example clock chimes that may use bells outside the Dove ring.
  - `medium`: the identity is clear, but the recording has a caveat: it is distant, walk-past or edited, other towers
    are nearby, or for YouTube the identity rests on the title alone.
  - `high`: everything else.

## Counts (high and medium only, 59 entries; low entries are excluded from benchmark totals)

| | whole manifest | fetched here (42) |
|---|---|---|
| ringing | unknown 42, method 7, chiming 5, tolling 3, call_changes 1, mixed 1 | unknown 32, method 5, chiming 3, tolling 2 |
| country | England 45, Ireland (island) 4, Australia 4, Wales 2, Scotland 2, USA 2 | England 36, Australia 3, Ireland 2, Wales 1 |
| ring size | 3:2, 4:2, 5:1, 6:13, 8:23, 10:5, 12:11, 16:2 | 6:10, 8:17, 10:3, 12:10, 16:2 |

Confidence across all 62 entries: high 29, medium 30, low 3 (`rothwell-clock`, `wells-cathedral-clock`,
`york-minster-midday`).

## Known caveats

- **YouTube is bot-walled from the build host.** Every video page returned "Sign in to confirm you're not a bot",
  whatever the yt-dlp player client. The 17 YouTube entries therefore rest on the title alone: descriptions and
  chapters were not checked, and every one is `medium`. To fetch them, rerun the script with `--cookies` pointing to a
  cookies.txt exported from a signed-in browser. This gap is why no 3-, 4- or 5-bell rings, no Scotland or USA
  entries, and no call-change entries have been fetched yet.
- `ringing` is `unknown` for most field recordings, because the uploader did not say what was rung. Don't use it as
  ground truth.
- Some ring sizes differ from what is heard. `stepney-stedman` rings only the back 8 of 10, `alfriston` rings 5 of 6,
  and `perth-swan-bells` is ringing up with an unknown subset of 16. Chiming and tolling entries sound one bell or a
  few.
- Several recordings are short or edited: `shirenewton` (13 s loop), `perth-swan-bells` (16 s) and `alfriston`
  (loop cut). Others are distant or noisy (`crediton-distant`, `rode-distant`, `bristol-redcliffe`, `lyndhurst`).
  In City of London and Oxford recordings, neighbouring towers may also be audible.
- `exeter-cathedral-practice` is capped at the first 20 min of a 32 min practice, so it includes gaps between touches.
- The two `ballarat-town-hall-*` entries are the same tower, recorded on different peal days.
