# Matcher: why real recordings missed, and what changed (2026-11)

## The miss pattern
In about half of corpus misses every true nominal sits on a detected line within 12 c, yet the true ring is
outranked, usually by a 10–16 bell ring in the same key. Decomposing the score showed the per-line terms
decide it: the true 8 explained its own lines for +0 to +6 nats, the winner +12 to +20. Across 77 misses the
winner's nominals landed on the true ring's nominals (355), octave nominals (123), quints (99) and
superquints (75). The true ring's non-nominal partials were modelled 30–70 c wide, so its own octave-nominal
and superquint lines barely counted, and a bigger ring's narrow (σ = nominal_sigma) nominals claimed them.

## Partial spread on real recordings
At the best shift, on recordings where ≥ 75 % of true nominals hit a line within 8 c (offsets from the ideal
ratio, nearest line within ±120 c, partials that collide with another nominal skipped):

| Partial | Median offset | IQR | Old tolerance | New tolerance |
|---|---|---|---|---|
| hum | +2 | −7…+30 | 70 | 40 |
| prime | +1 | −13…+9 | 50 | 30 |
| tierce | +10 | 0…+23 | 30 | 18 (ratio +5 c) |
| quint | −2 | −51…+15 | 50 | 45 |
| superquint | +14 (vs 1.471) | +2…+25 | 45 | 25 (ratio 1.4815, i.e. 0.557 × onom, acoustics.md §3) |
| octave nominal | −1 | −16…+12 | 55 | 35 |

True nominals sit within 2.5 c of a line (median) and 6.8 c (80th percentile), hence `nominal_sigma_cents`
5 rather than 8. I-7 and I-8 do not stand out from a control offset (lines are dense up there), so they are
not modelled.

## Ring subsets
Bands often ring the back 8 of 10 or 12, or 6 of 8. `score_ring_or_subset` also scores every contiguous
run that drops up to 4 bells from the front or back (keeping ≥ 5), charged `subset_log_prior` = −3. Sim
rings are complete, so sim is neutral to it; on the corpus it adds top-1 and top-5 (`experiments/no_subsets.toml`).

## Negative results
- **Era-dependent tolerances** (cast year from bells.csv; true-harmonic ≥ 1900 tight, old-style hum +90 c
  and prime −70 c with ~70–85 c windows, after acoustics.md §2): +2 top-1 on the corpus, but sim top-1 fell
  from 101 to 74–95 of 200. Sim draws every bell's partials around the ideal with no era, so the change can't
  be validated against it. Revisit only together with an era-aware sim.
- **Capping the per-nominal likelihood ratio** at 1 or 2 (to stop big rings collecting hits): corpus top-1
  17 → 11. The nominal terms carry real evidence; the bias was in the line terms.
- **Probability calibration**: sim is under-confident (mean top p ≈ 0.48 at 66 % right), the corpus
  over-confident (≈ 0.38 at 18 %). One temperature can't serve both, so `Match::margin` (score lead over the
  next candidate) is reported instead.
