# Matcher round 4: near-twin rings and the shift prior (2026-11-04)

Baseline: main at ece17c2 (Welch-mean spectrum, era tolerances, `p_bell_heard` 0.7): sim s1/s2 top-1
147/128, corpus top-1/5/20 40/60/76. Method as in matcher.md: `replay` on the baseline binary's
`clips.json`, decomposed with a numpy port of `score.rs` that reproduces every true-vs-winner order.

## Misses on the mean-spectrum baseline
Corpus rank of the true tower: 1 → 40, 2–5 → 20, 6–20 → 16, 21–50 → 11, not in the top 50 → 54.
Decomposing the 95 misses with a ranked winner (true ring vs winner, each at its best shift and subset):
- Twins, mostly: the median true ring shares 83 % of its bells with the winner (a winner nominal within
  12 c at the two best shifts); 53 of 95 share ≥ 75 %. Winners have more bells in 29, as many in 27,
  fewer in 39.
- Median gap 4.7 nats: shared bells +0.2, winner-only bells +0.8, true-only bells 0 (mean −0.9: mostly
  unheard), line terms +1.0, shift prior and subset 0. No single term; each rival bell outside the true
  ring is scored at full weight and usually lands on some line.
- **The winners use the shift.** True rings align within ±5 c in 58 % of misses, winners in 36 %; 46 of
  95 winners sit 5–40 c off, against 20 true rings. A twin in a slightly different tuning slides onto
  the true lines and the prior (σ 15 c) barely charges it (10 c costs 0.2 nats).
- Sim misses are pure twins: all bells shared in 52 of 53, median gap 0.9 nats.

## Shift prior with a narrow core
Without the prior, the shift at which the true ring's nominal terms peak, over corpus clips where it
hears ≥ 75 % of bells (96 clips): 47 % within 2.5 c, 70 % within 5 c, 84 % within 10 c, 8 % at a
resample shift; near-zero robust SD 3.7 c. Real clips mostly play at true speed; sim draws speed error
at SD 8 c (`SimParams::speed_sd_cents`), so its shifts have SD 9 c.

Around each known shift the prior is now a mixture `w·N(0, σc) + (1 − w)·N(0, 15)` scaled so the broad
part alone is the old prior (`shift_core_weight`, `shift_core_sigma_cents`). Maximum likelihood on the
residuals: corpus w 0.70, σc 4 c (ll +34 over the broad Gaussian); sim w 0.85, σc 8 c.

| core (w 0.7) | sim s1 | sim s2 | corpus top-1/5/20 |
|---|---|---|---|
| none (main, `experiments/no_shift_core.toml`) | 147 | 128 | 40 / 60 / 76 |
| σc 4 (corpus ML) | 137 | 127 | 46 / 67 / 80 |
| σc 5 | 140 | 130 | 45 / 65 / 79 |
| **σc 6 (kept)** | **145** | **130** | **45 / 66 / 79** |
| σc 8 (sim ML) | 142 | 130 | 43 / 62 / 77 |

σc 6 is the narrowest core that keeps sim within the A/B rule; the corpus profile likelihood puts it
4.6 nats below σc 4. The 4 c the corpus measures costs sim s1 10 because sim's speed error is twice the
real one: if `speed_sd_cents` were set from the corpus (≈ 4 c), σc 4 would be the consistent choice.
Gains are spread: of 18 corpus clips that move ≥ 5 ranks or into top-1, 17 move up (Witney, Salehurst,
Llandaff, St Sepulchre, Coventry to top-1); South Lopham, a tape at a large shift, falls 20 → 46.

## Margin as confidence (report only)
`Case::top_margin` (score lead of the top pick, nats) against top-1 hit rate, σc 6:

| margin | < 0.5 | 0.5–1 | 1–2 | 2–4 | 4–8 | ≥ 8 |
|---|---|---|---|---|---|---|
| sim s1+s2 | 41/105 | 22/63 | 58/75 | 72/74 | 70/71 | 12/12 |
| corpus | 6/55 | 6/33 | 12/27 | 14/19 | 4/4 | 3/3 |

A margin ≥ 2 nats is right 98 % of the time on sim, 81 % on the corpus; below 1 nat 38 % vs 14 %. The
two disagree under 2 nats, so no single margin → probability map is offered; ranking is unchanged.

## Negative results
- **Tighter nominal σ.** At the fitted shift, true nominals sit 2.1 / 4.3 / 7.0 c from their line
  (median / 80th / 95th, n 813), which reads as σ ≈ 3.5. But `nominal_sigma_cents` 3.5 gives sim
  144 / 122, corpus 39 / 59 / 74; with the shift core, 133 / 122, 47 / 61 / 81. The measured spread is of
  lines that were hit; the 5 c Gaussian also has to carry the bells 8–12 c off (retuned, mis-recorded).
- **Dove nominal precision.** 72 % of Dove's nominals are whole Hz, the rest to 0.1 Hz, with no cluster
  on equal temperament (no derived values). Whole-Hz rounding is SD 500 / f c (≤ 1.7 c above 300 Hz),
  well inside σ 5; not worth a per-bell width.

Regenerate: scripts outside the repo, the procedure above on `replay` output and Dove `bells.csv`.
