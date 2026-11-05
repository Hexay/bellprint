# Matcher and eval performance (2026-11)

Pure speed-ups: `eval-sim --n 200` seeds 1 and 2 and the full `eval-corpus` write byte-identical
`cases.json` and `clips.json` before and after. fastrel build, `RAYON_NUM_THREADS=4`, on a shared devbox
(load 5–10), so wall-clock is noisy; user-space instruction counts (`perf stat -e instructions:u`) are not.

| Run | Before (wall / instr) | After (wall / instr) |
|---|---|---|
| eval-sim seed 1 | 112 s / 6818 G | 15.4 s / 849 G |
| eval-sim seed 2 | 109 s / 7032 G | 15.5 s / 873 G |
| eval-corpus (141 clips) | 130 s / 6642 G | 21.2 s / 1330 G |
| locate fsg-st-pauls-nlm (perf record) | 1.8 s | 0.64 s |
| locate ap-worcester-remembrance (perf record) | 5.4 s | 2.5 s |

## Where the time went (perf, before)

Per clip, matching was ~95 % of sim-eval time and ~60 % of a long corpus clip's: the first-pass
nominal terms (Student-t `powf`, `ln`) ~55 %, the shortlist's line terms (Gaussian `exp`, `ln`) ~35 %.
Decode and the frame FFTs only matter on long clips (Worcester: ~10 % each before, ~35 % each after).

## What changed (sim seed 1 instructions after each step)

1. A bell's nominal term doesn't depend on which run it is scored in: compute per bell, sum per run.
   Wall 115 → 63 s.
2. Line terms of all a ring's runs in one pass, each partial's Gaussian computed once per line and
   shift; runs of equal size reuse the `ln` when their sums agree. → 2850 G.
3. Nominal terms once per distinct (nominal, optional) bell per clip: Dove has ~6.3k distinct
   nominals among ~47k bells. → 2083 G.
4. Frame FFTs in parallel (long clips, single `locate`; no change in instructions).
5. Exact branch and bound over shifts for line terms (`matcher/partials.rs`): blocks of 8 shifts get
   an upper bound (each partial's Gaussian at its nearest point of the block's span); a block is
   scored only if bound + best base term can reach the run's best exact score (slack 1e-6 nats).
   → 939 G. Blocks of 4 and 16 were worse (1058, 1014 G).
6. Sim renderer steps four oscillators per pass (same per-sample add order). → 895 G.
7. Runs from the front of a group share one running sum. → 849 G.

Left: decode (symphonia FLAC) and the 32k-point FFTs on long clips, the sim renderer, and the
first-pass Student-t `powf`. A table-based pdf was not tried: it would change scores.
