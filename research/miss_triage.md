# Real-corpus miss triage (extraction vs matcher)

Date 2026-11-04, binary at 956ee52 (mean long-term spectrum, 80-line cap). Corpus: all 141 recordings,
top-1 17, so 124 misses. Every miss was triaged, not a sample.

## Method

For each miss, `locate --json` with the cap lifted (`max_peaks = 400`, `min_prominence_db = 3`), then the
first 80 lines (what the matcher saw) were checked against the true ring's Dove nominals:

- **present** = nominals with a line within ±8 c, at the best common shift within ±10 c of zero;
- **null** = the same count with the nominals moved to ±450 or ±700 c (chance hits from a dense list);
- **extraction miss** if present − null < 40 % of the bells; **matcher miss** if present ≥ 75 %.
- Non-matcher rows were re-checked over shifts up to ±240 c against a null of jittered nominals
  (`shifted-present`: old tapes and resampled uploads).

A dense line list makes chance hits common: at 80 lines the null is about a third of all nominals. Treat
single-clip counts as ±1.

## Split (124 misses)

| class | n | meaning |
|---|---|---|
| matcher | 72 | ≥ 75 % of nominals are in the top 80 lines (median line rank 9–40), yet a wrong ring scores higher |
| extraction | 39 | the nominals are not in the list at near-zero shift |
| shifted-present | 6 | nominals present only at a large shift (York evening, Wakefield, South Lopham, …) |
| no Dove nominals | 6 | ring has no `Nominal (Hz)` in bells.csv (askrigg, shirenewton, heckington, willesborough, wylam, marsh-gibbon) |
| partial | 1 | bbc-oxton |

So most misses are **matcher** misses: the lines are there. Of the 39 extraction misses about half have
causes no extraction can fix (clock chimes on bells outside Dove, rings retuned after the recording,
recorded from the wrong tower, bells barely audible).

## Representative rows

present/null = nominals within ±8 c in the top 80 / chance level; rank = true tower's rank (eval keeps 50).

| id | type | s | lines | present/null | rank | class | cause |
|---|---|---|---|---|---|---|---|
| ia-march-st-wendreda | ? | 1200 | 165 | 6/6 / 1.8 | >50 | matcher | all 6 at 19–30 dB, rank ≤ 20; Barrow upon Humber wins |
| ia-ballarat-pb-major-2 | method | 62 | 181 | 8/8 / 4.2 | >50 | matcher | all 8 within 6 c; Shoreditch at −20 c wins |
| ballarat-town-hall-pbmajor | method | 60 | 226 | 8/8 / 3.8 | >50 | matcher | same tower, same failure |
| woolpit | method | 307 | 177 | 5/5 / 1.8 | >50 | matcher | |
| sc-willingdon | method | 1200 | 164 | 6/6 / 2.5 | >50 | matcher | |
| sc-windsor-curfew-tower | method | 1200 | 170 | 8/8 / 3.5 | >50 | matcher | |
| marsden-practice | ? | 297 | 210 | 10/10 / 4.0 | 47 | matcher | |
| aston-cantlow | ? | 206 | 208 | 6/6 / 2.2 | 39 | matcher | camcorder |
| sc-buckfast-abbey | method | 370 | 110 | 13/14 / 6.8 | >50 | matcher | |
| ia-lincoln-cathedral-package | ? | 184 | 85 | 12/13 / 5.2 | >50 | matcher | speech over ringing, still found |
| southwark-cathedral | ? | 120 | 183 | 1/13 / 3.0 | >50 | extraction | trains over Borough Market |
| ia-southwark-borough-market | ? | 328 | 127 | 1/13 / 0.5 | >50 | extraction | trains, market |
| stow-on-the-wold | ? | 91 | 119 | 2/8 / 2.0 | >50 | extraction | starlings in foreground |
| lyndhurst | ? | 166 | 129 | 3/8 / 1.2 | >50 | extraction | birdsong, gunshots |
| carlisle-cathedral | ? | 124 | 110 | 7/13 / 5.0 | >50 | extraction | wind distortion |
| dorchester-abbey (+ia- copy) | ? | 450 | 137 | 2/8 / 1.8 | >50 | extraction | binaural; fetch downmixes to mono first |
| edith-weston | ? | 63 | 230 | 3/6 / 1.0 | >50 | extraction | |
| kinver-tolling | tolling | 29 | 225 | 3/8 / 1.8 | >50 | extraction | distant, one bell tolled |
| bbc-sevenoaks | ? | 172 | 243 | 1/8 / 2.8 | >50 | extraction | old tape: best alignment at +228 c |
| bbc-oxford-magdalen | method | 396 | 198 | 2/10 / 3.2 | >50 | extraction | Dove tuning 2011, after the recording |
| bbc-st-albans-erin | method | 205 | 193 | 4/13 / 4.0 | >50 | extraction | Dove tuning 2010, after the recording |
| bbc-canterbury-grandsire-cinques | mixed | 391 | 220 | 2/14 / 2.8 | >50 | extraction | recorded from the central tower |
| ia-bridport-st-mary | ? | 315 | 9 | 0/8 / 0 | >50 | extraction | soundwalk away to a weir: 9 lines in all |
| ia-inveraray-distant | ? | 34 | 16 | 4/10 / 1.8 | >50 | extraction | bells barely audible |
| ap-dublin-johns-lane | ? | 360 | 22 | 0/10 / 0.5 | >50 | extraction | probably one Angelus bell |
| wells-cathedral-clock | chiming | 82 | 163 | 1/10 / 1.5 | >50 | extraction | quarter bells not in Dove |
| ia-ledbury-noon | chiming | 388 | 40 | 1/10 / 1.2 | >50 | extraction | clock chime |
| ia-york-minster-evening | ? | 338 | 74 | 10/14 at shift | >50 | shifted | 20:30 chime, shifted alignment |

## Patterns checked

- **80-line cap.** 98 of 141 clips hit it, but lifting it does not help: present − null over all 141 clips
  is 325 / 408 / 424 / 387 at the first 20 / 40 / 80 / all lines. Lines past 80 are mostly chance hits.
- **Low-prominence nominals.** Where present, nominals sit at 11–30 dB and median rank 9–40; prominence is
  not what loses them.
- **Codec cutoffs.** Not a factor. Every clip except the near-silent ones (bridport, 267 Hz) has lines up to
  2–8 kHz, and the misses' nominals are below 1.5 kHz.
- **Speech, traffic, birds, wind.** The bulk of the fixable extraction misses (Southwark ×2, Stow, Lyndhurst,
  Carlisle, Scarborough, Hereford). These sounds are loud only part of the time, which is what a per-bin
  percentile across frames is for.
- **Long clips (20 min).** Mostly matcher misses, not extraction: the nominals are present.
- **Doublets / upper partials crowding.** No evidence: present nominals are within ±8 c (most ±3 c).
- **Speed/tape shift.** 6 clips present only at ±60–230 c (BBC LPs, re-uploads). The matcher's shift prior
  decides those.
- **Stereo downmix.** `tools/fetch_corpus.py` uses `-ac 1`, so spaced or binaural pairs comb-filter before
  bellprint sees them. This can't be fixed in bp-core without keeping both channels.

## Conclusions

1. The matcher is the bottleneck: 72 of 124 misses have ≥ 75 % of the nominals in the list.
2. Extraction work only pays on noisy clips. Robust aggregation (per-bin percentile) is the lever; changing
   the cap or the prominence threshold is not.
3. About 20 misses can't be fixed by either side (no Dove nominals, chimes outside the ring, retuned rings,
   wrong tower). They put a ceiling on corpus top-1 well under 141.

Regenerate: the triage scripts live outside the repo; the procedure above is what they did, using
`bellprint locate --json --peaks <toml>` and Dove `bells.csv`.
