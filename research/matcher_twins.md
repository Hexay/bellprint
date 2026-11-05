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

## Confidence label
The report's `confidence` (`matcher::Confidence`) labels the top pick by its margin. Thresholds were
picked on sim s1+s2 (400 clips) at b8ccc0d: **high** from the lowest margin above which every 0.5-nat
bin is ≥ 95 % right (2 nats; the 1.5–2 bin is 86 %), **low** below the margin under which bins are less
than half right (1 nat; 0.5–1 is 38 %, 1–1.5 is 66 %). The corpus was not looked at while choosing, so
its rates are held out:

| label | margin | sim s1+s2 top-1 | corpus top-1 |
|---|---|---|---|
| high | ≥ 2 | 157 / 160 (98 %) | 22 / 27 (81 %) |
| medium | 1–2 | 53 / 70 (76 %) | 13 / 27 (48 %) |
| low | < 1 | 63 / 170 (37 %) | 11 / 87 (13 %) |

The label is ordered on both, but the corpus sits 17–28 points below sim at every level, so it is a
ranking of trust, not a probability. Four of the five high-confidence corpus misses are recordings where a
different ring really sounds (miss_triage.md). Eval summaries print the same split ("top-1 by
confidence"); ranking is untouched (sim and corpus `cases.json` ranks identical to b8ccc0d).

## Negative results
- **Tighter nominal σ.** At the fitted shift, true nominals sit 2.1 / 4.3 / 7.0 c from their line
  (median / 80th / 95th, n 813), which reads as σ ≈ 3.5. But `nominal_sigma_cents` 3.5 gives sim
  144 / 122, corpus 39 / 59 / 74; with the shift core, 133 / 122, 47 / 61 / 81. The measured spread is of
  lines that were hit; the 5 c Gaussian also has to carry the bells 8–12 c off (retuned, mis-recorded).
- **Dove nominal precision.** 72 % of Dove's nominals are whole Hz, the rest to 0.1 Hz, with no cluster
  on equal temperament (no derived values). Whole-Hz rounding is SD 500 / f c (≤ 1.7 c above 300 Hz),
  well inside σ 5; not worth a per-bell width.

## Line precision (2026-11-05): not what separates twins
Hypothesis: class D twins (miss_triage.md) would separate with more precise line frequencies. Baseline main
870ee77: sim s1/s2 top-1 144 / 129, corpus 46 / 66 / 80. Sim clips now save the rendered nominals
(`Clip::truth_hz`, written by `eval-sim --out`), so extraction can be scored against truth.

| measure (cents, median / p80 / p95) | value |
|---|---|
| Parabolic-on-dB bias, Hann window, clean tone (theory, worst sub-bin offset) | ≤ 0.12 at 300 Hz, ≤ 0.04 at 1 kHz |
| sim: line − rendered nominal (n 2567, s1+s2) | 0.46 / 0.90 / 1.4; signed −0.5 (doublet centroid) |
| sim: line − Dove at the clip's shift | 1.65 / 3.7 / 6.0 (the sim's 3 c Dove error) |
| corpus: line − Dove at refit shift, lines ≥ 60 % present (112 clips) | 2.0 / 5.0 / 7.9 |
| same, by prominence 3–6 / 6–12 / 12–24 / ≥ 24 dB | median 2.4 / 2.0 / 2.0 / 1.9 |
| corpus: same ring, two recordings, line − line (8 rings, 187 bells) | 1.56 / 5.0 / 9.4 |
| D clips (32): line − Dove at the matcher's true-ring shift | 2.3 / 5.2 / 11 |
| D clips: winner − true predicted nominal, shared bells (241) | 2.3 / 4.9 / 8.6; 47 % within 2 c |

- Extraction is ~10× tighter than σ 5 on sim, and corpus error does not fall with prominence, so it is
  not SNR-limited. Two recordings of a ring agree to ~1.1 c each (1.56 / √2); the rest of the 2 c to
  Dove (~1.7 c median) is Dove's figure against the bell as it sounds. Twin gaps are the same size as
  that Dove term: no extraction can split them. On shared D bells the nearest line is nearer the winner
  in 140 of 240.
- **Oracle replays (sim, baseline clips.json).** Line nearest each rendered nominal moved onto it (zero
  extraction error): 136 / 131 (−8 / +2). Moved onto Dove × speed (zero Dove error): 176 / 166. The sim
  ceiling is Dove error, not line precision.
- **Hann ratio estimator** (Grandke, `2(c − a)/(a + 2b + c)` on magnitudes, exact for one tone): sim
  144 / 130, corpus 45 / 65 / 82. Lost bbc-st-sepulchre, gained none; D ranks 3 up, 10 down. Sim s1 error
  0.48 c vs 0.47 (no better): doublets and neighbouring lines, not window bias, set the residual. Not kept.
  Zero-padded refinement converges on the same exact-tone estimate, so it was not run.
- **Narrower σ** not tried: precision did not improve, the precondition for it.

Regenerate: scripts outside the repo, the procedure above on `replay` output and Dove `bells.csv`.

## Nominal likelihood shape (2026-11-05): heavy tail kept, but not because the residuals are heavy
Question: does a likelihood that matches the Dove-vs-heard error (median 2.0, p95 7.9 c) beat the 5 c
Gaussian? Knobs: `nominal_shape` (gaussian / student_t / laplace), `nominal_t_nu`, a wide Gaussian
mixed in (`nominal_outlier_weight`, `_sigma_cents`), `nominal_reach_sigmas` (how far a nominal may claim a
line; was a fixed 3 σ). σ stays 5, so the shift grid step is unchanged except where σ is varied.

**Fitted shape (in-sample: corpus top-1 hits only, 46 clips, 448 nominals).** Shift refit per clip, signed
offset to the nearest line within ±40 c; each offset modelled as π·shape + (1 − π)·nearest chance line
(Poisson at the local ±300 c line density). Even clips fit, odd clips held out:

| shape | fit, all | ll all | ll odd (even fit) |
|---|---|---|---|
| Gaussian | σ 3.2, π 0.96 | −1233.2 | −622.8 |
| Student-t | σ 3.1, ν 29, π 0.96 | −1233.1 | −622.9 |
| Laplace | b 2.7, π 0.98 | −1242.8 | −624.8 |
| Gaussian + wide | 10 % σ 1.4 + 90 % σ 3.4 | −1232.4 | −623.9 |

The "heavy tail" (p95 ≈ 3.5× median) is about 4 % chance lines, not a fat Dove error. The fitted
Student-t's ν goes to about 30, which is a Gaussian. Per-clip RMS (p10/50/90 2.1/3.1/4.2 c) matches a
bootstrap from the pooled offsets (2.2/3.2/4.2), so there is no ring-level heterogeneity either. On misses
(descriptive only) the fit is the same: σ 3.9, π 0.82.

**Replay screen** (`replay` on `runs/ab-robust` clips, which reproduce the full evals; vs main a4b851c
baseline sim 143/177, 129/169, corpus 47/68/82; corpus even/odd = index in clips.json):

| variant | sim s1 | sim s2 | corpus all | even | odd |
|---|---|---|---|---|---|
| main (Gaussian σ 5, reach 3 σ) | 143/177 | 129/169 | 47/68/82 | 25/34/41 | 22/34/41 |
| **Student-t σ 5 ν 1.5, reach 5 σ (kept)** | **145/177** | **128/167** | **49/70/86** | **26/35/41** | **23/35/45** |
| Student-t σ 5 ν 1, reach 5 σ | 144/177 | 128/166 | 49/70/85 | 26/35/41 | 23/35/44 |
| Student-t σ 5 ν 2, reach 5 σ | 143/177 | 127/167 | 49/69/86 | 26/34/41 | 23/35/45 |
| Student-t σ 5 ν 2, reach 3 σ | 143/178 | 127/168 | 49/68/86 | 26/34/41 | 23/34/45 |
| Student-t σ 5 ν 2, reach 8 σ | 142/177 | 127/167 | 49/69/86 | 26/34/41 | 23/35/45 |
| Student-t σ 5 ν 3, reach 5 σ | 143/177 | 127/168 | 48/67/84 | 26/33/41 | 22/34/43 |
| Gaussian σ 5 + 20 % σ 10 | 143/177 | 129/169 | 48/69/84 | 26/34/41 | 22/35/43 |

Before main's optional-bell change (baseline 144/177, 129/169, 46/66/80) the other variants were screened:

| variant | sim s1 | sim s2 | corpus |
|---|---|---|---|
| Student-t σ 5 ν 4, reach 5 σ | 143/178 | 129/168 | 46/66/82 |
| Student-t σ 3.2 ν 30 (the fit) | 140/169 | 124/164 | 44/64/77 |
| Student-t σ 3.5 ν 4, reach 6 σ | 142/174 | 128/168 | 46/63/81 |
| Laplace b 2.7 (the fit), reach 6 σ | 132/163 | 117/156 | 41/60/76 |
| Laplace b 3.5, reach 5 σ | 139/172 | 126/165 | 46/63/80 |
| Gaussian σ 3.2 + 5 % σ 6.8 (the fit) | 140/170 | 124/166 | 45/64/78 |
| Gaussian σ 3.5 + 20 % σ 8 | 141/174 | 126/168 | 46/64/81 |
| Gaussian σ 5 + 10 % σ 15 | 143/177 | 130/169 | 46/66/81 |
| Gaussian σ 5, reach 5 σ | 144/177 | 129/169 | 46/66/80 |
| Per ring: one bell of unknown index mislisted, prior 0.3, σ 15 | 143/177 | 129/168 | 46/65/80 |

- **Shapes fitted to the hits lose** (sim −4 to −12, corpus −2 to −5). Narrow is what loses: every
  σ ≈ 3 variant falls, as σ 3.5 did before. The term has to carry more than the hits' offsets: the
  shift is searched, not known; twins sit 2–5 c apart; and bells retuned since Dove's measurement are
  not in a hits-only fit.
- **What wins is a heavier tail at the same σ.** ν ≤ 2 gives +2 corpus top-1 at every reach tried, +1 in
  each half. The density is 15 % lower at the centre and 1.2× / 7× / 9× the Gaussian at 10 / 15 / 20 c.
  So a nominal 10–25 c off still counts for something, and the term is less peaked between twins. Reach
  alone (Gaussian to 5 σ) changes nothing, and a light tail (ν ≥ 3, a 10–20 % wide component) gets
  +0 to +1. ν 1.5 was picked among ν 1–2 on sim. On the corpus they tie, so this is a choice made on
  the corpus, with only 2 clips of evidence.
- Side effect: with the tail, silent bells pick up partial credit from chance lines up to 25 c off, so
  subsets matter less. A clean back 6 of 8 (pipeline test) is rank 10 with or without subsets; under
  the Gaussian term subsets still lift it. The test now checks the mechanism under the Gaussian term.
- The per-ring "one bad bell" mixture was neutral (corpus −1 top-5) and was dropped. Per-bell
  `p_bell_heard` already floors a mislisted bell at ln(1 − p).

**Full evals of the kept default** (bit-identical ranks to the replay): sim s1 145/177, s2 128/167
(rule floor 142/127), corpus 49/70/86 (even 26/35/41, odd 23/35/45). Corpus top-1 gained:
belfast-st-thomas (2 → 1), stepney-stedman (3 → 1); none lost. Corpus ranks move up in 17 clips and down
in 9 (≥ 5 ranks: reading-st-giles 14 → 5, norton-fitzwarren 19 → 9, stirling-holy-rude 29 → 15,
wormhill 30 → 16, east-leake 21 → 13, ballarat-town-hall-grandsire out → 43; aston-cantlow 8 → 14,
oxford-st-ebbe 12 → 18, oxton 22 → 27). Sim top-1 s1 +5 −3, s2 +3 −4. `experiments/nominal_gaussian.toml`
restores the old term.
