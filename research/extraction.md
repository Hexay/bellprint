# Extracting bell frequencies from audio: algorithms and parameters

Scope: onset/strike detection, per-blow nominal estimation, Hibbert's Wavanal, striking-analysis tools
(HawkEar/CAS/Bagley), polyphonic/inharmonic methods, and separating bells by blow timing in change ringing.
Researched 2026-11-03. Tags: **[V]** means verified on the cited page this session. **[unverified]** means
recalled from memory. **Inference** means my own derivation.

## 1. Sources (fetched)

| # | Source | Used for |
|---|---|---|
| S1 | B. Hibbert, *Wavanal documentation* v1.7, 24/10/2004, https://www.hibberts.co.uk/wp-content/uploads/2021/12/wavdoc.pdf | Transform params, resolution |
| S2 | B. Hibbert, "Tonal analysis from recordings", 2022 (upd. 2023), https://www.hibberts.co.uk/tonal-analysis-from-recordings/ | Workflow on recordings of rounds |
| S3 | B. Hibbert, "Programs available from this website" (Wavanal 5.10, Rounds, Tuner, Pitcher), https://www.hibberts.co.uk/?p=141 | Tool inventory |
| S4 | I. McCallion, "Guidance when recording bells for striking analysis", Oct 2012, https://www.12bell.org.uk/wp-content/uploads/2012/11/Recording-Bells-for-Striking-Analysis.doc.docx | HawkEar input requirements |
| S5 | National 12-Bell Striking Contest, "Strikeometer" page, https://www.12bell.org.uk/hawkear/ | HawkEar history, CAS |
| S6 | S. J. Flockton, "SJF Model – Birmingham Eliminator 2nd April 2011", https://www.12bell.org.uk/hawkear-files/SJF%20Model.doc | Measured blow intervals |
| S7 | SRCY HawkEar site, https://hawkear.srcy.org.uk/ and a touch report https://hawkear.srcy.org.uk/ym.20260510-1516.4.guzh/index.html | Handstroke uplift, quality figures |
| S8 | Channel Islands Branch, "Hawkear at ADM", 2017, https://cibells.com/category/hawkear/ | HawkEar failure modes |
| S9 | M. Marolt, "Automatic transcription of bell chiming recordings", IEEE T-ASL manuscript T-ASL-03052-2010, https://lgm.fri.uni-lj.si/papers/8992340.pdf | NMF + onset bell transcription |
| S10 | J. O. Smith, *Spectral Audio Signal Processing*, "Optimal Peak-Finding in the Spectrum", https://www.dsprelated.com/freebooks/sasp/Optimal_Peak_Finding_Spectrum.html | QIFFT, zero-padding |
| S11 | M. Abe & J. O. Smith, "Design criteria for simple sinusoidal parameter estimation based on quadratic interpolation of FFT magnitude peaks", AES Conv. preprint 6256, 2004, https://aes.org/publications/elibrary-page/?id=12913 | QIFFT bias (via S10) |
| S12 | ringbell.co.uk, "Ringing Speed", https://ringbell.co.uk/methods/speed.htm | Rounds timing rule of thumb |
| S13 | Bello et al., "A Tutorial on Onset Detection in Music Signals", IEEE TSAP 13(5):1035–1047, 2005 (bibliographic details from search results; full text not fetched) | HFC, spectral flux, complex domain |
| S14 | Böck & Widmer, "Maximum filter vibrato suppression for onset detection" (SuperFlux), DAFx-13, 2013; madmom docs https://madmom.readthedocs.io/en/v0.16/modules/features/onsets.html (search summary) | Onset detector defaults |
| S15 | Adelaide Uni. project wiki "2014S1-06 Bell Ringing Robot: Hawkear", https://projectswiki.eleceng.adelaide.edu.au/projects/index.php/Projects:2014S1-06_Bell_Ringing_Robot:_Hawkear | Bagley sensor systems |
| S16 | B. Hibbert, "Doublets or warble in bells", 2022, https://www.hibberts.co.uk/doublets-or-warble-in-bells/ | Doublets |

Note: hibberts.co.uk had an expired TLS certificate on 2026-11-03, so WebFetch failed. I fetched it with `curl -k`.

## 2. Hibbert's Wavanal: what it actually does [V, S1–S3]

- It is a magnitude-only FFT ("without regard to phase") with a **Hamming window**. The transform length comes
  from the user's **frequency interval**. If the selection is shorter than 1/interval, it is zero-padded. If it
  is longer, the excess is dropped. [V S1 §4.3.1, §6.3]
- "For an interval of 0.5Hz, two seconds of data is required. For an interval of 0.1Hz, ten seconds." [V S1
  §6.3]
- Resolution: ripple spacing = fs / N. Examples: "5000 samples … at 44 kHz will give ripples of 8 or 9 Hz",
  and "200,000 samples at 22 kHz (~10 s) gives … about 0.1 Hz. Therefore, long sound samples are needed."
  [V S1 §6.2]
- If the selection is much shorter than 1/interval, "peaks … will be broadened, and the accuracy … less than
  anticipated". [V S1 §6.3] So Wavanal relies on long windows, not interpolation. No interpolation method is
  documented.
- Recording: "at least 3 seconds" is needed for accuracy. Digitise at "16-bit mono … 44.1 or 48 kHz". "Bells
  have no significant partials above about 5 kHz". [V S1 §2, §6.1]
- Wavanal auto-sets "start sample" to the moment of clapper strike when it loads a file. [V S1 §5.1.1]
- "Find Decays" runs short "splash" transforms (start→start+splash, start→start+2·splash, …) and then longer
  sliding transforms (start+k·step → length+k·step). Splash decay is defaulted to **0.25 s**. [V S1 §5.x]
- Bells recorded rung "up" in the ringing room give partial frequencies "comparable" to chimed bells. Attack,
  decay and relative amplitudes differ. [V S1 §2.1]
- **Working on a recording of ringing** (S2): Hibbert no longer splits files. He selects each bell's blow in
  the peal recording, starting "with the start of the bell sound … as long as possible, but not include the
  next bell sound". He runs the transform with default Start/End Frequency/Interval and then picks partials. If
  the lower partials are lost in noise, he estimates the strike pitch, doubles it to get a candidate nominal,
  and confirms it by "the evenly spaced rim partials above the nominal". He then restricts the nominal search
  to a band around the candidate. [V S2]
- Wavanal's default interval value is not stated in the docs I read. [unverified]
- Sibling programs: *Rounds* turns recordings of single bells into change ringing (useful for synthetic test
  data). *Tuner* is a tuning spectrum analyser. *Pitcher* checks strike pitch. [V S3]

## 3. Striking-analysis tools: how blow times are found

**HawkEar** (Ian McCallion; first contest use Birmingham eliminator, April 2011) [V S5]
- It "obtains the strike information directly from a sound recording of the ringing". It replaced clapper
  sensors (2007), which "proved difficult to set up and were unreliable". [V S5]
- Input requirements [V S4]: 44100 Hz, mono, WAV preferred (MP3 acceptable), no automatic level control, peak
  about 20 dB below clipping, a dry mic above the bells. It "can normally cope if the sound level difference
  between bells is within 10dB". The ringing chamber is "poor third-best and probably will not work".
- **Supervised per tower:** "It is essential to supply a training recording consisting of one whole pull of
  each bell, in the order treble to tenor … leave a few seconds between each bell." [V S4] So HawkEar learns
  each bell's spectral signature from isolated blows. It does not infer bells blind.
- **Row-structure prior:** "Any recording that does not start with silence followed by rounds cannot be used."
  [V S4] Field reports say it "won't analyse anything if … some bells strike in more rows than others". A very
  slow tenor cover gets assigned "as leading for the next row". It "can't manage half muffled ringing". [V S8]
  So HawkEar assumes every bell strikes exactly once per row and cuts rows from timing.
- Its signal-processing internals (detector, bands, thresholds) are **not published** on any page I could
  find. A third-party summary ("each bell has a unique signature … HawkEar attempts to spot this signature")
  appeared only in a search snippet. [unverified]
- Output is blow times only. Interpretation comes from the "strikeometer" models (McBurnie, Pipe, Flockton)
  and from **CAS** (CAS 1.4, Java, analyses HawkEar output; downloadable from S5). [V S5] Abel simulator can
  export striking data for CAS. [search snippet, abelsim.co.uk; unverified]

**Bagley (David Bagley):** sensor-based. The *Odd Struckness Meter* uses Hall-effect sensors on the bell and
clapper and measures swing times. It uses no audio. [V S15]

**Ringing Room, "Ding":** I found no audio analysis in either. Ringing Room is an online simulator and
generates its own blows. [unverified] "Ding" turned up nothing relevant.

## 4. Timing numbers for change ringing

Measured, 12 bells, Birmingham eliminator 2011, seven teams [V S6]:

| Quantity | Range |
|---|---|
| Average inter-blow interval | **192–205 ms** |
| Average handstroke-lead interval (last bell of backstroke row → first of next handstroke row) | **430–465 ms** (≈2.2–2.3 × interval) |
| Equivalent peal speed (5040) | 3h25m – 3h38m |
| Overall SD of blow error | 24.1–39.4 ms (12.0–19.2 % of interval) |

Other verified figures:
- HawkEar "Ringing Quality" on 12-bell Cambridge Maximus: 23–30 ms (12–15.5 %). 288 changes in about 11m25s,
  which is 2.38 s per row. [V S7] The handstroke gap comes from "all ringers leaving more time from
  backstroke to handstroke than from handstroke to backstroke". [V S7]
- Rule of thumb: rounds at 2 s per bell-cycle with no handstroke gap gives a peal speed of 2h48m/5040. That
  means a 1/3 s gap on 6 bells and 1/4 s on 8 bells. [V S12]
- Peals take "two and a half to over four hours depending on the weight of the bells". [search snippet,
  Wikipedia "Peal"; unverified] Worcester 12-bell peals average about 3h54m (Cinques) and 4h01m (Maximus).
  [search snippet, worcesterbells.org.uk; unverified]

Derived model (**Inference**, checked against S6): with open handstroke leads, a whole pull (2 rows) of n bells
spans 2n + 1 intervals. So interval d = 2·T_row / (2n + 1), where T_row = peal time / 5040.

| Case | T_row | d (blow interval) | Handstroke gap (≈2d) |
|---|---|---|---|
| 12 bells, 3h30 (S6 check: predicts 200 ms, measured 192–205) | 2.50 s | 200 ms | 400 ms |
| 12 bells, 4h00 (heavy) | 2.86 s | 229 ms | 457 ms |
| 8 bells, 3h00 | 2.14 s | 252 ms | 504 ms |
| 8 bells, 3h20 (heavy-ish) | 2.38 s | 280 ms | 560 ms |
| 6 bells, 2h45 | 1.96 s | 302 ms | 604 ms |
| 6 bells, 3h00 (heavy 6) | 2.14 s | 330 ms | 660 ms |

- Each bell recurs every 2 rows with a period of about 2·T_row = 3.9–5.7 s, split unevenly between
  handstroke and backstroke. **Inference**
- Blow-time jitter is about 25–40 ms SD for good bands (S6, S7). Blows less than 20 ms apart are heard as
  simultaneous (ringingteachers.org snippet, unverified). An onset detector therefore needs about 10 ms time
  resolution. Clusters can be gated with a tolerance of about ±0.3·d. **Inference**
- Weight → speed has no published formula (search found none). Tenor weight only bounds d to roughly
  190–330 ms. Treat d as **estimated from the clip** (autocorrelation of the onset function), not as a prior.
  **Inference**

## 5. Onset (strike) detection

- Standard detection functions [S13, bibliographic via search]: energy, **high-frequency content (HFC)**
  (energy weighted linearly by bin index), **spectral flux** (half-wave-rectified frame-to-frame magnitude
  increase), phase deviation, and **complex domain** (amplitude plus phase prediction error).
- Marolt used **complex-domain ODF + peak picking** for church bells, noting it "performs well with church
  bells, because of their sharp and percussive onsets". [V S9]
- SuperFlux (log-magnitude flux with a 3-bin max filter along frequency, at 200 fps) suppresses false onsets
  from vibrato or beating partials. 200 fps and 3 bins come from madmom/yup defaults and the sister ISMIR-13
  paper (search summary), not from the DAFx paper itself. [partly unverified] Beating doublets [S16] produce
  amplitude modulation that plain flux mistakes for onsets, which is why this matters. **Inference**
- Recommended settings (**Inference**, informed by S1/S13/S14):

| Parameter | 44.1 kHz | 48 kHz | Note |
|---|---|---|---|
| STFT window | 2048 (46 ms) Hann | 2048 (43 ms) Hann | short enough for d ≥ 190 ms |
| Hop | 441 (10 ms) or 220 (5 ms) | 480 / 240 | ≥100 fps; jitter is 25–40 ms |
| Band | 300 Hz – 5 kHz | same | S1: no significant partials > 5 kHz. The band skips hum/prime and most rope/room rumble |
| ODF | log-mag SuperFlux (max-filter 3 bins), or complex domain | | |
| Peak pick | adaptive threshold (moving median + δ), min spacing ≈ 0.4·d (start with 80 ms) | | refine after estimating d |
| Per-band variant | flux restricted to ±50 cents around each candidate nominal | | "energy in nominal band": gives per-bell onset trains directly |

## 6. Per-blow frequency estimation

Resolution needed (**Inference**: Δf = f·(2^(c/1200) − 1)):

| Nominal f | ±1 cent | ±5 cents | ±10 cents |
|---|---|---|---|
| 300 Hz | 0.17 Hz | 0.87 Hz | 1.74 Hz |
| 500 Hz | 0.29 Hz | 1.45 Hz | 2.90 Hz |
| 1000 Hz | 0.58 Hz | 2.89 Hz | 5.79 Hz |
| 2000 Hz | 1.16 Hz | 5.78 Hz | 11.6 Hz |

- The window is bounded by the blow interval. Bin spacing = 1/T: a 150 ms window gives a 6.7 Hz bin, and a
  2 s window gives 0.5 Hz. [V S1 formula]
- **QIFFT** (parabola on the dB magnitude of three bins around the peak) is approximately the ML estimator for
  isolated sinusoids. The interpolation error is inaudible when the zero-padding factor is ≥5 for any window.
  [V S10, citing S11] Minimum zero-padding to keep bias ≲1 % of fs/M: rectangular 2.1, generalized Hamming
  (Hann/Hamming) 1.2, Blackman needs none. For bias ≈0.1 %: rectangular 4.1, Hamming 2.4, Blackman about
  1.8–2.4 (the S10 table and the zpfmin test differ). [V S10] Parabolic interpolation is unbiased exactly on a
  bin or exactly halfway. [V S10/CCRMA]
- With a Hann window, 150 ms, zero-pad ×4 (44.1 kHz: 6615 samples → 32768-point FFT; 48 kHz: 7200 → 32768),
  QIFFT bias is <0.1 % of 6.7 Hz (≈0.007 Hz). That is negligible. **Inference** The real error comes from
  **interference** from other bells' still-ringing partials within about 2 bins (±13 Hz) and from **doublets**
  [S16]. Bias from AM/FM is treated in Abe & Smith part III (CCRMA STAN-M-116, not fetched).
- A Gaussian window with QIFFT on log magnitude is exactly unbiased for a stationary sinusoid. [unverified]
- Analysis offset after the strike: Hibbert starts the selection *at* the strike [V S2]. The splash decays
  with a default constant of 0.25 s [V S1]. Recommend starting **20–40 ms after onset**, which skips the
  broadband clapper transient but keeps the nominal's strongest early energy, and ending ≥20 ms before the next
  detected onset. **Inference**
- **Blow-synchronous spectral differencing** (**Inference**, the same idea as Marolt's delta spectrogram
  [V S9]): per blow, P_after(f) − P_before(f), where P_before is a short window ending just before the onset.
  This cancels the sustained partials of other bells and leaves the new bell's spectrum. Averaging over all
  blows of one bell (N ≈ 50–100 in a 2–3 min clip) raises SNR by about √N.
- After assignment, refine precision with a **long window per bell**: either concatenate or coherently
  combine the post-onset segments of that bell's blows, or run a 2–10 s window on the whole clip and take the
  peak inside the ±50 cent band predicted by the per-blow estimate. Wavanal-equivalent resolution is 0.5 Hz at
  2 s and 0.1 Hz at 10 s [V S1]. **Inference**

## 7. Polyphonic / inharmonic methods

- **Marolt (S9)** [V]. CQT magnitude spectrogram: max window 100 ms, step 25 ms, 60 bins/octave. Delta
  spectrogram over Nd = 4 frames. Partial "synchronicity patterns" are clustered with modified intelligent
  k-means, using a bell-partial-position prior, over 2.5 s segments (2–8 s all similar, <2 s unreliable).
  Cluster centres initialise NMF (bases are bell spectra). A Markov-chain model combines NMF activations,
  complex-domain onsets and "bells are seldom struck at the same time". Activation width is 150 ms. A bell
  counted as found if its nominal was within **50 cents** of annotation. Results: about **97 % P/R** finding
  bells (30 recordings, 110 bells, 3–5 bells each) and about **90 % P/R** transcription (1123 onsets). Using
  the delta spectrogram mattered more than spectral flattening. Published form appears to be IEEE TASLP,
  2012. [unverified]
- **Parametric high-resolution methods** (Prony, matrix pencil, ESPRIT) fit sums of damped exponentials.
  They resolve components closer than 1/T and estimate decay rates, so they suit doublets and short windows.
  They are costly (SVD) and fragile with many components or model-order errors (Potts/Tasche/Volkmer review,
  Frontiers Appl. Math. Stat. 2016, via search). [partly unverified] No bell-specific application was found.
  A handbell paper (ipb.ac.rs, "frequency-zooming ARMA") says very close modal frequencies "require
  high-resolution spectral analysis tools". [search snippet, unverified]
- Practical use for bellprint (**Inference**): ESPRIT or matrix pencil on a **narrowband, mixed-down,
  decimated** signal (±30 Hz around a candidate nominal, decimated to about 100–200 Hz, model order 2–6) over
  a 150–300 ms post-onset window. This resolves doublets and the neighbour-bell interferer cheaply. Use it as
  an optional refinement, not the primary detector.

## 8. Verified facts vs inferences: summary

- Verified: Wavanal uses long Hamming-windowed FFTs, 0.5 Hz ↔ 2 s and 0.1 Hz ↔ 10 s, no documented
  interpolation, 44.1/48 kHz, nothing significant above 5 kHz. HawkEar needs a per-tower training pull of each
  bell and a recording starting with rounds, and requires every bell to strike every row. 12-bell interval is
  192–205 ms with a 430–465 ms handstroke lead. Good-band jitter is 24–40 ms SD. Marolt's NMF + onset method
  reaches 97 %/90 %. QIFFT with zero-pad ≥5 is effectively unbiased.
- Inference: the interval model d = 2·T_row/(2n+1) (it matched S6 within 4 %), the parameter tables in §5–6,
  blow-synchronous differencing, and narrowband ESPRIT.

## 9–10. Measured on bellprint

A/B results (peak picking, aggregation, sim decay): see [extraction_ab.md](extraction_ab.md).


## Implications for bellprint

- **Keep the long-term spectrum for precision. Add a per-blow stage for assignment.** The long-term peaks
  mix every bell's partials, and partial coincidences between bells are inherent to scale tuning. Onset timing
  is the clean way to decide which peaks belong to one bell.
- Pipeline: (1) band-limited (300 Hz–5 kHz) log SuperFlux ODF at 10 ms hop → onsets. (2) Estimate the blow
  interval d and row period from ODF autocorrelation; expect d ≈ 190–330 ms and a handstroke gap ≈ 2d.
  (3) Per onset, spectral difference (post 20–190 ms minus pre) → dominant new peak in the nominal band, using
  Hann + zero-pad ×4 + QIFFT. (4) Cluster blow nominals in **log-frequency (cents)**, with ≤50 cents
  tolerance as in Marolt, into n bells. (5) Per cluster, refine with a long window or narrowband
  ESPRIT → nominal ±1–2 Hz. (6) Match against Dove.
- **Use rounds structure as a free prior.** In rounds the bells strike in pitch order (treble highest →
  tenor). Within each row, blow slot k should carry the k-th highest nominal. A check on that ordering
  validates n and the assignment, and it gives the bell count n directly: onsets per row period = n (2n+1 per
  whole pull with an open handstroke lead). This narrows Dove candidates by ring size before frequency
  matching. This is the same assumption HawkEar makes, so it fails on half-muffled ringing, firing, and partial
  rows.
- Without rounds (methods, call changes), cluster by frequency only. Timing still gives n (blows per row) and
  the speed (d) as weak priors.
- Score matches in cents per bell (all n nominals plus the global shift), not on a pooled peak list. With
  per-bell nominals, the partial model only needs to decide which cluster peak is the nominal. Use the
  superquint/rim partial spacing check from Hibbert S2 for that.
- Precision targets: ±5 cents is 0.9 Hz at 300 Hz and 2.9 Hz at 1 kHz. Per-blow 150 ms QIFFT is fine for
  clustering. Final values should come from aggregated multi-blow or long windows (≥2 s ⇒ 0.5 Hz grid,
  Wavanal-equivalent).
- Test data: Hibbert's *Rounds* program (S3) synthesises change ringing from single-bell recordings, which
  gives ground-truth timing and bell labels for an automated regression harness.
- Expect beating doublets [S16]. Do not treat a single FFT peak per partial as truth. Allow for two peaks a few
  Hz apart and take the stronger one (or the energy centroid) as the nominal, consistent with how Dove values
  are measured. [unverified: Dove measurement convention, check with the acoustics agent]
