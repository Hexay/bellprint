# Real-corpus miss triage (start brief for the next rounds)

Date 2026-11-04, baseline main at b8ccc0d (Welch mean, 3 dB prominence, 120-line cap, shift core σc 6).
Corpus 141 recordings, top-1/5/20 46/66/80, so **95 misses**. Every miss was triaged, none sampled.

## Method

For each miss, the baseline binary's `locate --json --top 400` (400 = the full shortlist; its lines match
`clips.json` exactly) against the true ring's Dove nominals (`bells.csv`, in-band ones only):

- **net present** = true nominals within ±8 c of a line at the best shift within ±10 c, minus the same
  count for the nominals moved ±450 / ±650 c (chance hits from the line list; 1–7 of a ring).
- **share** = true nominals with a winner nominal within 12 c, each ring at its fitted shift.
- **gap** = winner score − true score, when the true ring is in the 400-ring shortlist.
- **near hint** = would the true tower rank first among candidates within 10 / 30 km of it (`--near`)?

Classes are exclusive and assigned in this order (first match wins):

| class | rule |
|---|---|
| A1 no Dove nominals | ≤ 1 nominal in `bells.csv` |
| A2 chime / clock | `ringing = chiming` or a clock / chime note |
| A3 ring changed / other tower | manifest note: Dove tuning or augmentation after the recording, recorded from another tower; or a tower ≤ 0.2 km away wins |
| B2 partial ring | named subset beyond the subset rule (light 6 of 12, ringing up, 19-bell ring partly heard) |
| C shift | true ring fitted ≥ 12 c from every known shift (outside 2σc of the core) |
| B1 too little signal | net present < 40 % of in-band bells |
| D near twin | signal present, share ≥ 75 % |
| E other | signal present, share < 75 % |

## Split (95 misses)

| class | n | rank 2–5 / 6–50 / >50 | evidence | examples |
|---|---|---|---|---|
| D near twin | 32 | 16 / 12 / 4 | median share 100 %, median gap 1.2 nats, median margin 0.3; same bell count in 16 | witley-close, ia-croome, marsden-practice, woolpit, ballarat ×4 |
| B1 too little signal | 20 | 2 / 1 / 17 | median net 1.4 of 8 bells (6–14); winners are small rings (fewer bells in 15) | southwark ×2, dorchester ×2, bridport, sevenoaks, carlisle, tewkesbury |
| A2 chime / clock | 13 | 1 / 2 / 10 | a few bells struck repeatedly; 10 never shortlisted | wells-clock, ledbury-noon, rothwell-clock, totnes-6am |
| A1 no Dove nominals | 10 | – / – / 10 | 6 rings with none, 4 with a single nominal | askrigg, wylam, marsh-gibbon, kensington, bolsover |
| E other | 7 | 1 / 5 / 1 | share 33–70 %, median gap 2.8; 4 winners have fewer bells | stepney-stedman, bbc-oxton, sc-oxford-st-ebbe, bbc-winchester |
| A3 ring changed / other tower | 6 | – / – / 6 | Magdalen (tuned 2011), St Albans (2010), Pembroke (2026), Rochdale (8 → 10), Canterbury (central tower), Oxford University Church (Lincoln College, 150 m, wins by 3.5 nats) | |
| C shift | 4 | – / 1 / 3 | true ring at +15, −12.5, +25, −22.5 c; share 100 %, gap 3.4–6.2 nats | bbc-south-lopham, alfriston, bbc-brixham, bbc-westminster-abbey |
| B2 partial ring | 3 | – / – / 3 | net 4–7 of 14–19 bells: a subset the matcher doesn't model | perth-swan-bells, dublin-christ-church, sc-wellington-cathedral |

29 misses (A1–A3) can't be matched against this Dove snapshot: the attainable corpus top-1 is 112, so the
baseline is 46 / 112 on matchable clips.

## What would fix it, ranked by clips recoverable

Tested by `replay` on the baseline `clips.json` (corpus +gained / −lost top-1; sim s1 top-1 vs 144):

| # | fix | recoverable | evidence |
|---|---|---|---|
| 1 | **Location hint** (`--near`, or a geotag/metadata prior) | 40 top-1 at 30 km (D 30, E 4, A2 3, B1 2, C 1); 48 at 10 km | twins are rarely neighbours: 32 / 32 D clips rank first within 10 km. Nothing in the audio separates them at today's line precision. Report a `--near 30` benchmark alongside the no-hint one |
| 2 | Noise-robust extraction on the 8 B1 clips with partial signal (net ≥ 30 %: tewkesbury, carlisle, wakefield, hereford, giggleswick, lyndhurst, edith-weston, port-sunlight-wedding) | 0 by extraction (extraction_ab.md §14: nominals of rung bells are already lines; misses are unrung Dove bells, distance, near twins) | the remaining 12 B1 clips are at chance (binaural Dorchester ×2, trains at Southwark ×2, bridport soundwalk, distant/one-bell clips); no line list can fix them |
| 3 | Subset prior `subset_log_prior` −3 → −1.5 | +2 / −0 (stepney-stedman, ia-bournemouth) | sim s1 144 → 144; needs sim s2 before adopting |
| 4 | `p_bell_heard` 0.7 → 0.5 | +3 / −1 | sim s1 142; borderline under the A/B rule |
| 5 | Corpus labels: exclude or tag A1–A3 in the benchmark (report top-1 of 112 matchable) | 0 recovered, 29 removed from the denominator | data, not code |
| 6 | Chime mode (score any 3–5 bell subset, no penalty for silent bells, when the clip is a strike/chime) | ≤ 3 (kirkbymoorside rank 5, norton-fitzwarren 14, wedmore 26) | the other 10 chimes never reach the shortlist: quarter bells outside the Dove ring |

Levers that recover **nothing** (corpus top-1 unchanged or worse):

- Shift prior: core σc 4 (+0 / −0, sim 135), core weight 0.9 (+0 / −0, sim 138). C clips trail by 1.3–3.0
  nats beyond their whole prior cost, so no prior closes them; earlier broad floors also failed
  (matcher.md).
- Longer subsets, `max_bells_dropped` 4 → 10: B2 clips stay outside the top 50.
- Tighter `nominal_sigma_cents` 3.5 (+2 / −4), `true_harmonic_tol_cents` 6 (+1 / −1, sim 133),
  `full_strength_db` 12 (+1 / −4), `background_share` 0.8 (+2 / −5), no line terms (+2 / −11, sim 77).
- Large tape shifts: the previous round's "shifted-present" clips (Sevenoaks, Wakefield, Rochdale, York
  evening) don't beat a null that is also maximised over ±260 c, so they are not shift misses.

## Confidence of misses

The report's confidence label (matcher_twins.md, "Confidence label") puts 76 of the 95 misses at low, 14 at
medium and 5 at high. Four of the five high-confidence misses are A2/A3: a different ring really is
sounding (Oxford University Church vs Lincoln College, Canterbury from the central tower, St Albans and
Rothwell); the fifth is bbc-winchester (E, rank 10).

## Earlier triage

The 956ee52 triage (17 / 141 top-1, extraction vs matcher split, 72 matcher / 39 extraction) is superseded
by the classes above; see `git log -p research/miss_triage.md` before 2026-11-04.

Regenerate: `tools/triage/locate_misses.sh <bin> <cases.json> <out>` writes the per-miss dumps;
`python3 -I tools/triage/triage.py`, `classify.py` (class rules, incl. the hand-set A3 / B2 lists),
`near.py` and `levers.py` (on `replay --out` dirs) print the tables.
