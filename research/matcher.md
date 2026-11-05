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

Prior re-check (2026-11, replay on `runs/ab-prom3` caches; sim top-1/top-5, corpus top-1/5/20) — **keep −3**:

| prior | sim s1 | sim s2 | corpus |
|---|---|---|---|
| −3 | 144/177 | 129/169 | 46/66/80 |
| −2 | 144/176 | 128/169 | 46/68/81 |
| −1.5 | 144/176 | 126/169 | 48/67/82 |
| −1 | 140/174 | 121/167 | 48/67/82 |

−1.5 and −1 lose > 2 sim s2 top-1; −2 gains no corpus top-1. Rationale points the other way: 4 of 141
corpus clips ring a modelled front/back subset (stepney-stedman, ia-coventry-cathedral-down,
bbc-st-clement-danes, bbc-rochdale-st-chad; 7 counting B2 partial rings), spread over ≤ 8 subsets per ring:
ln(0.03/8) ≈ −5.6 (ln(0.05/8) ≈ −5.1). −3 is already generous; the +2 at −1.5 is clip-specific.

## Optional bells and the first-pass shortlist (2026-11-05, on main fc76703)
Dove's `bells.csv` "Bell Role" marks semitone bells with a `b` / `#` suffix ("6b", "2#", "5#"; 130 bells)
and an extra treble as "0" (31). They sit in 106 and 29 full-circle/lightweight rings. "NcM" roles
(18 rings) are ring bell N doubling as carillon bell M, not extras. Bands usually ring the diatonic ring
without them, and every unheard one costs ln(1 − 0.7). Subsets only drop from the ends, so an interior
semitone is always charged. 27 of 141 corpus clips come from such towers (10 already top-1). Sim rings have them in 6 (s1) and 4 (s2) clips,
and the sim rings every listed bell, extras included. That is a slight bias against both levers below.
Not changed, since the sim stays independent of the matcher.

Two levers, both replayed on main's caches (`eval --out` + `replay`; default replay is bit-identical):

| variant | sim s1 | sim s2 | corpus | corpus +gained / −lost top-1 |
|---|---|---|---|---|
| main | 144/177 | 129/169 | 46/66/80 | – |
| `p_optional_heard` 0.3 | 144/177 | 129/169 | 46/66/81 | +ia-worcester-cathedral-remembrance / −bbc-st-sepulchre |
| `p_optional_heard` 0.1 / ≈0 | 144/177 | 129/169 | 46/66/82 | same pair |
| `without_optional_log_prior` 0 | 143/177 | 128/169 | 46/67/81 | +exeter-cathedral-practice / −bbc-st-sepulchre |
| −0.25 | 143/177 | 129/169 | 47/67/81 | +exeter-cathedral-practice |
| **−0.5 (default)** | **143/177** | **129/169** | **47/68/82** | **+exeter-cathedral-practice** |
| −0.75 | 144/177 | 129/169 | 46/68/82 | – |
| −1 | 144/177 | 129/169 | 46/67/82 | – |
| −2 | 144/177 | 129/169 | 46/66/81 | – |

- `without_optional_log_prior` also scores each ring with its optional bells removed (subsets still
  apply), charged the prior. At −0.5, confirmed by a full eval, the s1 loss (sim-828-124) is a rival,
  Rothwell (0, 4b), gaining from the drop. The true ring doesn't lose. The gain is fragile:
  exeter-cathedral-practice wins by 0.07 nats. Worcester ×2 rise to rank 2 (25 → 2), winchester 10 → 5.
  `experiments/no_optional_drop.toml` turns it off.
- `p_optional_heard` (soft: an optional bell's miss costs ln(1 − q)) swaps one top-1 for another at every q.
  Not adopted (`None`).
- **The B1 targets don't move.** tewkesbury, carlisle and wakefield stay outside the top 50 under every
  variant, even combined with an unlimited shortlist. They also lose non-optional bells (tewkesbury 6 and
  10, wakefield 11, carlisle 3), so the semitone charge isn't what sinks them.

First-pass cut (coarse bell-terms-only score, top `shortlist` = 400): cases where the true ring is
cut but its full score would rank in the top 50 (unlimited-shortlist replay vs main):
s1 2/200 (sim-467-108 → 4, sim-3821-59 → 14), s2 2/200 (sim-17846-40 → 9; sim-4464-107 1 → 2 because a
rival enters), corpus 1/141 (giggleswick → 17 at 800, 18 unlimited). With 800, 1600 or unlimited:
s1 144/178, s2 128/169 (−sim-4464-107), corpus 46/66/81 (sc-westbury-muffled 8 → 14–17, three ranks
down past 20). None gains a top-1. Replay time per eval at 4 threads is about 80 s at 400, 120–200 s at
800, 180–250 s at 1600 and 580–680 s unlimited (a shared box, so the times are noisy). **Keep 400**. A
better first-pass score isn't worth building for one top-20 clip.

## Remaining misses (2.5 kHz band, 4 dB lines, 113 misses)
Per-miss decomposition (true ring vs winner at their best shifts) over the 64 misses with ≥ 75 % of true
nominals within 8 c of a line:
- The gap is in the nominal terms, not the lines: winners score about 1.4 nats per bell, as the true ring
  does, and win by having more bells (10–16 against 6–8). Strong lines left unexplained by the true ring are
  rare outside 1–3 bell "rings".
- Winners' nominals land on the true ring's **primes** (strongest, 19.6 dB median), quints and tierces.
  Primes and nominals form the same scale an octave apart, which a 12–16 bell ring in a related key covers.
- Lines are not dense (40–80): a nominal at a random position hits a line 12 % of the time, the winners' 100 %.
- Prominence doesn't separate nominals: in the 75th-percentile spectrum, hum, prime and tierce lines
  (median 14.5–14.8 dB) are stronger than nominal lines (12.4 dB).
- Per-bell partial presence doesn't separate them either: winners' heard bells show a tierce/superquint
  /octave nominal within tolerance as often as the true ring's (0.75/0.86/0.93 vs 0.76/0.89/0.92; null
  0.25/0.66/0.78).
- No ring-level coherence of partial offsets: the spread of per-ring mean offsets (hum 24, prime 15 c) is
  the same as that of control ratios (18–23 c).
- **Era does separate them.** Offsets by Dove cast year: from 1926 hum IQR −3…+5 c and prime −3…+4
  (87 %/93 % within 20 c); 1896–1925 72 %/69 %; before 1896 near the control level (23–53 %). The
  all-era hum and prime tolerances (40, 30 c) were 3–4× too wide for modern rings. That let a bigger ring's
  5 c nominals claim the true primes.

## Era tolerances
Hum and prime of bells cast from `true_harmonic_from` = 1896 (Taylor's switch, acoustics.md §2) use
`true_harmonic_tol_cents` = 10 (founder SDs 6.5–11.3 c, modern tuning tolerance ±10 c). Older or undated
bells keep the all-era tolerances. Tested against an era-aware sim (hum and prime by cast year, from the
acoustics.md founder table; see its Implications), on which main's matcher scores 117 / 117 top-1:

| | sim s1 | sim s2 | corpus top-1/5/20 |
|---|---|---|---|
| all-era tolerances (main) | 117 | 117 | 28 / 47 / 63 |
| true-harmonic from 1926 | 135 | 122 | 31 / 48 / 62 |
| **true-harmonic from 1896** | **147** | **129** | **31 / 51 / 67** |

σ 7 / 10 / 15 c gives corpus top-1 31 / 31 / 30, so the gain doesn't hinge on the width.

## Misses on the era-tolerance baseline (2026-11-04)
Fast loop: `eval-sim`/`eval-corpus --out D` also write `D/clips.json` (lines + truth); `bellprint
[--matcher x.toml] replay D/clips.json` re-ranks them without audio and reproduces the full eval exactly.
Decomposing the 103 corpus misses with a ranked rival (true ring vs winner at their best shifts):
- Big rings are no longer the main winners: the winner has more bells in 30, as many in 26, **fewer in 47**.
- The gap is in the nominal terms (median +3.8 nats to the winner) more than the line terms (+0.9).
- Winner hits land on true nominals 397, quints 122, none 123, superquints 34, primes only 29 (of 735).
- True rings lose by **unheard bells**: at the best shift 84 % of true nominals have a line within 3σ,
  against 46 % for the same rings moved off-scale (±250, ±450 c). So about (0.84 − 0.46) / 0.54 ≈ 70 %
  of bells are heard, not the assumed 97 %: each silent bell cost ln 0.03 = −3.5 nats where ln 0.3 = −1.2
  is right. Sim rings ring every bell loudly (≈ 100 % in reach), so sim barely cares.

| `p_bell_heard` | sim s1 | sim s2 | corpus top-1/5/20 |
|---|---|---|---|
| 0.97 (`experiments/heard_0.97.toml`) | 147 | 129 | 32 / 53 / 70 |
| 0.9 | 147 | 129 | 34 / 56 / 71 |
| 0.8 | 147 | 128 | 34 / 57 / 74 |
| **0.7 (measured)** | **146** | **127** | **34 / 59 / 76** |

## Negative results
- **Explaining away within a ring** (each nominal's LR tested against local line density plus the
  density of the ring's other bells' partials at it, weight w): sim s1/s2 152/136 at w 0.5, but corpus
  32/48/62 (w 0.5), 25/44/56 (w 0.97); hum and prime only, w 0.97: 30/48/67. In a diatonic ring about half
  of the true ring's own nominals sit within 12 c of another bell's partial too (sim: 172 of 342 true
  hits vs 217 of 389 winner hits), so it discounts true rings almost as much as rivals.
- **Ring-size prior.** Not needed: with the true ring moved off-scale, E[1 − p + p·LR] per bell is 0.39
  (p 0.97) / 0.56 (p 0.7), below 1, and the best-shift per-bell nominal score falls with ring size
  (6: −0.19, 8: −0.14, 10: −0.28, 12: −0.22 nats at p 0.7). Chance hits don't favour big rings. Big
  rings that win do so on on-scale structure (same scale, octave-related keys), which a size prior
  can't target. Winners of 10–12 bells are over-represented against Dove's size mix, but so are
  10–16 bell true rings in the corpus (56 of 141).
- **Wider shift prior for tapes and LPs** (a broad Gaussian floor of weight 0.02–0.05 and σ 100–250 c
  under the resample peaks): corpus top-1 28 → 27–28, no top-5/20 gain. The grid already covers ±257 c
  continuously, and the shifted true rings (Rochdale +179, Wakefield +113, Sevenoaks −257) trail by 9–15
  nats, which the prior (at most 3–5 nats) can't close. It also helps wrong rings at large shifts.
- **Era-dependent tolerances against an era-less sim** (earlier round; old-style hum +90 c and prime −70 c
  with ~70–85 c windows): +2 corpus top-1, sim 101 → 74–95. Superseded by the era-aware sim above, which
  widens nothing and only tightens true-harmonic bells.
- **Capping the per-nominal likelihood ratio** at 1 or 2 (to stop big rings collecting hits): corpus top-1
  17 → 11. The nominal terms carry real evidence; the bias was in the line terms.
- **Probability calibration**: sim is under-confident (mean top p ≈ 0.48 at 66 % right), the corpus
  over-confident (≈ 0.38 at 18 %). One temperature can't serve both, so `Match::margin` (score lead over the
  next candidate) is reported instead.
