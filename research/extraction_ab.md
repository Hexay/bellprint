# Line extraction: A/B results on bellprint

Measured counterpart of [extraction.md](extraction.md), which holds the literature survey.

## 9. Measured on bellprint (2026-11-04, main 4e4a7c9: p75, 2.5 kHz, out-of-band nominals unobservable)

One parameter at a time via `--peaks`; corpus is all 141 clips, sim `eval-sim --n 200`. Baseline: sim seed 1
132/172, seed 2 122/164, corpus top-1/5/20 28/43/61. Keep rule: corpus better, neither seed down > 2 top-1.

| change | seed 1 | seed 2 | corpus | verdict |
|---|---|---|---|---|
| `min_prominence_db` 9 | 132/171 | 121/164 | 23/34/51 | worse |
| `min_prominence_db` 4 | 131/172 | 122/164 | 28/47/63 | **kept** |
| `min_prominence_db` 3 | 131/173 | 121/162 | 28/49/64 | same trend; 4 kept as the margin over floor noise in 8 s clips |
| aggregate mean | 132/170 | 117/167 | 33/52/66 | seed 2 −5 |
| p90 / p85 / p60 | 129, 127, 127 | 121, 119, 117 | 29/51/64, 33/48/62, 26/38/51 | sim −3 to −5 |
| log-mean (mean of dB) | 131/171 | 115/165 | 21/32/50 | worse |
| trimmed mean ≤ p90 / ≤ p75 | 132, 130 | 116, 116 | 29/45/61, 24/40/54 | seed 2 −6 |
| `min_hz` 150 / 250 | 132, 132 | 122, 118 | 27/43/60, 27/43/59 | worse: hums carry evidence |
| `max_peaks` 50 / 120 | 132, 132 | 122, 122 | 25/43/57, 28/43/61 | cap rarely binds (most clips 40–70 lines) |
| tonal frame gate, 3 / 6 / 10 dB | 132, 132, 132 | 120, 122, 122 | 29/41/59, 30/42/63, 30/42/63 | per clip 8 up / 9 down; nothing on top of mean, p85, p90 or prom 4 |
| `--extraction blows` | 84/128 | 85/125 | 19/35/47 | no clip class wins (17 up / 38 down) |

- **Corpus wants more transient weight than sim.** Corpus top-1 ranks mean ≈ p85 > p90 > p75 > p60 >
  log-mean. A nominal decays within a second or two of each strike and the hum rings on, so higher
  aggregates favour nominals; long `sc-` practice clips with gaps between touches gain most (sc-rye 14 → 1,
  sc-llanfeugan 43 → 1 under p90/mean). The 8 s sim, which rings throughout, loses 3–6 top-1 for every
  aggregate above p75. Resolved in §10: the sim's decay was off.
- **Several clips have 0 lines at p75** (stow-on-the-wold, lyndhurst, ia-scarborough-st-mary,
  ia-lincoln-cathedral-package, several chimes): the bells sound in fewer than a quarter of the frames. Even
  with the mean, lyndhurst has only 11 lines over 6 dB.
- **Frame gate** (tried): per 0.74 s frame, mean prominence of its 15 strongest lines over a ±150 c moving
  mean; keep frames within N dB of the 90th-percentile frame. It restores lines on stow (0 → 22) and helps
  bristol-redcliffe (traffic, 43 → 5), but loses as many elsewhere. It never changes the sim (no frame is
  dropped). Not kept.
- **Lower prominence threshold.** Corpus top-5/20 rise monotonically from 9 dB down to 3 dB, with sim flat.
  Faint nominals of distant rings (ia-inveraray-distant miss → 3, bbc-bangor-comgall miss → 11) sit at
  4–6 dB. Top-1 is unchanged and per clip it is 17 up / 18 down, so the gain is mainly in the top-5/20.

## 10. Partial decay in the sim; mean aggregation (2026-11-04, on main 3a03682)

Same protocol as §9. "Old sim" is main's; "new sim" gives partials Hibbert's measured decays (hum T60
13.5 s, every other partial 2.7 s; research/acoustics.md §5) in place of a frequency-scaled T60 (nominal
≈ 6 s at 1 kHz).

| extraction | old sim s1 / s2 | new sim s1 / s2 (s3 / s4) | corpus |
|---|---|---|---|
| p75 (main) | 131/172, 122/164 | 124/169, 120/162 (124/163, 130/169) | 28/47/63 |
| p75, < 20 lines → mean | 131/172, 122/164 | – | 28/49/66 |
| p75, < 20 lines → p85/p90/p95 | 131/172, 122/164 | – | 28/47/64 |
| p85 | – | 131/173, 121/166 (122/162, 130/171) | 33/48/62 (§9) |
| mean | 130/170, 118/166 | 133/172, 121/165 (125/162, 129/173) | 30/52/68 |
| **mean, < 20 lines → p75/p90** | – | **133/172, 121/165** | **30/53/69** |

- **The sim/corpus disagreement was the sim's decay.** With nominals that die in seconds, the new sim
  ranks aggregates as the corpus does (mean ≥ p85 > p75). The new sim alone costs p75 7 and 2 top-1 on
  seeds 1 and 2. Kept on the sources, not the score.
- **Kept: Welch mean, with a p75/p90 fallback below 20 lines.** Against main on the new sim: +9/+1 top-1
  (seeds 3, 4: +1/−1); corpus 28/47/63 → 30/53/69, 32 clips up / 19 down. On the old sim the mean is
  −1/−4. The fallback only touches clips the mean leaves near-empty (no sim clip has < 22 lines):
  ia-inveraray-distant, distant bells under noise, goes 4 → 17 lines, miss → 3.
- **Few-frame clips.** Of the 22 corpus clips with < 20 lines at p75, the mean fallback gave lines to all
  and a rank to three (lincoln-cathedral-package 5, york-minster-midday 2, kirkbymoorside-chime 16). The
  rest are chimes on bells outside Dove's ring, near-silent clips (bridport, scarborough) or clips whose
  recovered lines miss the nominals (stow: best alignment −250 c, 2 of 8 within 10 c). Thresholds
  12/20/30 gave identical results. A p85–p95 ladder restores too few lines (stow 21, lincoln 18) to rank
  anything.
- **Fallback criterion is line count.** On a single tone under a loud noise burst, the mean's floor-ripple
  lines outnumber p75's one true line, so the fallback keeps the mean; with a ring's worth of lines it
  works (`tests/longterm.rs`).
- Not modelled, no figure found: clapper damping after the 0.8–2 s bounce in full-circle ringing
  (acoustics.md §5, W2012). It would shorten every partial further, the hum included.

## 11. Decay from Boutillon & David 2002; mean re-run (2026-11-04, on main 232786b, era sim)

§10's sim gave prime and tierce the nominal's 2.7 s; a second-source search
([decay.md](decay.md)) found that only hum and nominal are supported. "New sim" here is T60 = 2.2 Q/f
with BD2002's Q: nominal 1200, hum 4000, all else 3000 (nominal ≈ 2.6 s, prime ≈ 13 s at a 1 kHz nominal).
"Old sim" is main's frequency law (nominal ≈ 6.4 s). Mean = the §10 commit, fallback p75/p90 below 20 lines.

| extraction | old sim s1 / s2 | new sim s1 / s2 | corpus top-1/5/20 |
|---|---|---|---|
| p75 (main) | 147/177, 129/170 | 141/171, 123/168 | 32/53/70 |
| mean | 143/178, 128/170 | 139/173, 124/167 | 37/55/74 |

- **The new sim costs every extraction 4–6 top-1**, mostly from the faster nominal.
- **With prime and tierce long-lived the sim no longer prefers the mean**: −2/+1 top-1 against p75
  (old sim −4/−1), where §10's sim gave +9/+1. The corpus still prefers the mean, +5/+2/+4.
- Clapper damping (W2012, single source) remains unmodelled and is the likeliest remaining gap: it would
  cut the long-lived prime, tierce and hum that keep the percentile competitive in the sim.
