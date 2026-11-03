# Prior work: geolocating audio from bells / acoustic fingerprints

Researched 2026-11-03 via web search + fetch. Web pages treated as untrusted data.
Tags: **[V]** = read on the fetched page; **[S]** = from search-result snippet only (page not opened);
**[I]** = my inference; **[unverified]** = from memory, not checked.

## 1. Bells as OSINT / forensic evidence

### Verified
- **Bellingcat Challenge 81 "Urban Symphony"** (puzzle by "noarno", Bellingcat Volunteer Community,
  "Acoustic Adventures" series). Task: from an audio clip, "identify the city and the name of the ringing
  tower"; the clip is described as "an audio fingerprint of a city". No date or solution on the page. [V]
  https://challenge.bellingcat.com/challenge/81
- No Bellingcat investigation using bell pitch for geolocation was found. Bellingcat's church-related work
  found (e.g. Gaza church strike debunk, 2023) is visual geolocation. [S]
  https://www.bellingcat.com/news/2023/10/26/separating-fact-from-fiction-on-social-media-in-times-of-conflict/
- Fact-checks of viral "bell" videos exist (Montreal church bells vs. call to prayer; Ayodhya bell
  misattribution) but use provenance and visual checks, not acoustic matching. [S]
  https://www.thecanadianpressnews.ca/fact_checking/montreal-church-bells-ring-hourly-chimes-unrelated-to-muslim-prayer-seen-in-video-basilica-says/article_314914d3-92e7-5c7f-839a-f7608af91dc9.html
- No court/forensic case using bell frequencies was found.

### The viral "Dove's Guide + a few seconds of video found a Derbyshire church" claim: UNVERIFIED
I could not find the original post or any replication. Searched (2026-11-03), all with no hit:
- `Dove's Guide church bells video identified Derbyshire church tweet` (extended)
- `"Dove's Guide" bells identified church from video few seconds ringers twitter viral` (extended)
- `bellringer identified church just from sound of bells video "Derbyshire"` (extended)
- `"Dove" bells "identified the church" OR "identified the tower" ... hacker news OR mastodon OR twitter`
- `geoguessr OSINT church bells pitch identify tower Dove database bluesky` (extended)
- `"worked out" which church from bells in video using Dove's Guide tenor pitch ringer post` (extended)
- `reddit bellringing identify tower from recording bells frequency Dove`
- `ringingworld.co.uk tower identified from video bells sound social media`

What turned up was only @DovesGuide on X announcing features (e.g. https://x.com/DovesGuide/status/1809971554966479330),
Dove's own pages, and a Dickon Love (Dove lead) tweet that the snippet does not tie to this claim
(https://twitter.com/dickonlove/status/1865795182962040917, not opened). X, Bluesky and Mastodon are poorly
indexed by the search tool, so absence of results is weak evidence. Treat the anecdote as plausible but
unsourced. Do not cite it as a precedent.

### Adjacent data work (found incidentally)
- `frankbueltge/n-1` PR #78 (merged 2026-10-29): analysis of "the 64,136 bells" in Dove (CC BY-SA 4.0).
  It reports that bells in a ring agree closely with their own tenor, but rings sit a **median 22 cents from
  the note they are named for**. [V] https://github.com/frankbueltge/n-1/pull/78
  It is not peer-reviewed, but it supports matching on Hz, not on note names.
- Dove's own scope: "The frequency listed is that of the nominal, unless otherwise specified"; note names
  assume A=440 equal temperament and are *derived from ring position*, not measured. [V]
  https://dove.cccbr.org.uk/help/bells

## 2. ENF (electric network frequency) forensics: the closest analogue

The method matches a measured frequency trace against a reference database, the same shape as bellprint's
problem. ENF matches a *time series* of one frequency; bellprint matches a *vector* of nominals up to one
global ratio.

### Verified
- **Grigoras, C. (2005)** "Digital audio recording analysis: the Electric Network Frequency (ENF)
  Criterion", *Int. J. Speech, Language and the Law* 12(1):63–76, doi:10.1558/sll.2005.12.1.63. The author
  was at the Ministry of Justice in Bucharest. The method matches mains hum in a recording against a reference
  grid-frequency database to check editing and recording time. [S, full text paywalled (HTTP 402)]
  https://journal.equinoxpub.com/IJSLL/article/view/9977
- Reference databases are kept by the UK Metropolitan Police (since 2005) and the Bavarian police (since 2010). [V]
  https://en.wikipedia.org/wiki/Electrical_network_frequency_analysis
- **Huijbregtse & Geradts (2009)** "Using the ENF criterion for determining the time of recording of short
  digital audio recordings", IWCF 2009, LNCS 5718:116–124. They propose the **correlation coefficient** as more
  robust than squared error. Ambiguity comes from **fixed frequency offsets** during recording and
  **self-similarity of the database**, "particularly for recordings shorter than 10 minutes". [V via Wikipedia; S for paper]
- **Cooper (2011)**, IJSLL: the comparison metrics in use are variants of MSE or the correlation
  coefficient, compared empirically. [S] https://journal.equinoxpub.com/IJSLL/article/view/5980
- **Hua, Wang, Ye, Zhang (2020)** "Reliability of Power System Frequency on Time-Stamping Digital
  Recordings", arXiv:2011.00176. "The length of test recording is the most important inherent factor, followed
  by the length of reference". SNR is an important external factor and temporal resolution has little impact. [V]
  https://arxiv.org/abs/2011.00176
- Practical numbers (Heaton, explainer citing Hua et al. 2017): GB grid nominal 50 Hz, normal range
  **49.8–50.2 Hz**. ENF rarely changes by more than **0.001 Hz/s**. It is estimated once per second to
  3 decimals (mHz) using an STFT over 8–16 s windows plus **quadratic peak interpolation**. Matching
  slides the trace across the reference and takes the max Pearson r. Harmonics at 100/150 Hz are halved when
  50 Hz is weak. **~10 min** is "typically given as a lower bound". Case: Croydon Crown Court 2012
  (firearms), where Dr Alan Cooper (Met) matched hum to National Grid data and the defendants were convicted. [V]
  https://robertheaton.com/enf/
- European ENF is held within a 200 mHz band around 50 Hz (Baksteen, TU Delft thesis). Its chapters cover
  STFT + zero-padding + quadratic interpolation and cross-correlation vs MSE matching. [S; PDF body not extractable]
  https://diamhomes.ewi.tudelft.nl/~kvuik/numanal/baksteen_afs.pdf

### Lessons transferable to bellprint [I]
- A "fixed frequency offset" in ENF corresponds to bellprint's global speed/pitch shift. ENF treats it as a
  nuisance that causes ambiguity, which is a warning that a free global shift plus a small vector will alias
  across towers.
- Database self-similarity is the dominant failure mode. Rings tuned to the same note and scale differ by
  only a few cents (see 22-cent finding above), so uniqueness must be *measured* (best vs second-best margin),
  not assumed.
- Evidence length (number of distinct bells heard and number of blows averaged) matters more than
  resolution. This matches Hua et al.'s ranking of test length first.
- Report a likelihood ratio or margin, not just the top hit. Forensic ENF practice frames results this way.

## 3. Audio geolocation ("where was this recorded")

### Verified
- **Kumar, Elizalde, Raj (CMU), "Audio Content based Geotagging in Multimedia", arXiv:1606.02816 (2016).**
  Uses a city-tagged Flickr set from Lei et al. (ICASSP 2012): 1,079 videos (540 train / 539 test), clips up to
  90 s, 18 cities reduced to 10. Semi-NMF bases come from 10 UrbanSound8K classes, with MFCC features
  (60-dim). Best **MAP 0.563** vs 0.478 BoAW and 0.491 GMM-supervector baselines. This is city-level
  classification, not point location. [V] https://ar5iv.arxiv.org/html/1606.02816
- **MediaEval Placing Task** (2010s): predicts video location from text, image and audio. Sevillano et
  al. (Information Sciences 2015, Ramon Llull) report geotagging within 1 km "at least 4 times more precisely"
  than audiovisual participants in MediaEval 2011. [S]
  https://merit.url.edu/en/publications/look-listen-and-find-a-purely-audiovisual-approach-to-online-vide-4/
- **Chasmai, Liu, Maji, Van Horn (UMass), "Audio Geolocation: A Natural Sounds Benchmark",
  arXiv:2505.18726 (2025).** Converts audio to spectrograms and applies image-geolocation methods. Uses
  iNatSounds wildlife audio and combines species-range priors with retrieval, and argues for fusing audio with
  visual cues. [S] https://arxiv.org/abs/2505.18726v1
- None of these use tonal fingerprints of fixed sound sources. All are learned soundscape statistics that
  reach city or region scale at best. [I]

## 4. Bell identification / MIR

### Verified
- **Marolt, M. (2012)** "Automatic transcription of bell chiming recordings", *IEEE TASLP* 20(3):844–853.
  Detects which bells were played and their onsets. Estimates the number of bells and their spectra from the
  covariance of the TF representation using bell-acoustics priors, then NMF with selective sparsity. Notes that
  "the sounds of bells are inharmonic and their tuning not known in advance". Tested on Slovenian
  field recordings of *pritrkavanje* (rim-struck stationary bells). No accuracy figure on the project page. [V]
  https://lgm.fri.uni-lj.si/research/transcription-of-bell-chiming-recordings/
- **Marolt (2009)** SMC Porto, pp.137–142: NMF with selective sparsity for bell chiming. [V, same page]
- **Lefeber, van Kranenburg, Marolt (2011)**, TVNM 61:147–161: identification of 18th-century bell-playing
  clock repertoire by automatic comparison. [V title only]
- **Fleischer (DAGA 2009)**: listeners hear a 5-partial reduced spectrum as a bell. The perceptual core of a bell
  is a handful of partials. [S] https://www.pub.dega-akustik.de/NAG_DAGA_2009/data/articles/000092.pdf
- Trifonov & Georgieva (Bulgarian bell archive, math.bas.bg) state the goal of finding bells by sound. [S]
  https://www.math.bas.bg/bells/Publications/Bell_sound-Varna-eng.pdf
- Bill Hibbert's *Wavanal* is recommended by Dove for measuring nominals. Hibbert's site covers partial
  identification. [V via Dove help] https://www.hibberts.co.uk/identifying-bell-partials/
- Generic datasets with a coarse "church bells" class: **ESC-50** (Piczak, ACM MM 2015: 2,000 × 5 s clips, 50
  classes × 40, CC BY-NC; note "some recordings ... bandlimited" class-dependently) [S]
  https://github.com/karolpiczak/ESC-50 ; ETH Acoustic Event Dataset (71 church-bell clips, 11.8 min) [S];
  AudioSet "church bell" label [S] https://research.google.com/audioset/eval/church_bell.html
  These are useful only as negative/positive data for a "is there a bell here" gate, not for identity. [I]
- No published system that identifies *which tower* from recorded bell pitches was found. The closest is a
  personal idea note (bzamecnik/ideas, bell_classification.md). [S]

## Verified vs inference summary
- Verified: the ENF method and limits, Kumar et al. numbers, the Marolt papers, Dove's nominal convention, the
  22-cent median offset claim (single unreviewed source), and the Bellingcat audio-tower puzzle.
- Inference: transferability of ENF lessons, and that bellprint's approach is novel in the literature.
- Unverified: the Derbyshire viral claim (not found anywhere).

## Implications for bellprint
- **The approach looks novel.** No paper or tool matches measured bell nominals against Dove. Cite ENF
  (Grigoras 2005, Huijbregtse & Geradts 2009) as the methodological precedent.
- **Score uniqueness, not just fit.** Output the best-vs-second-best margin or a likelihood ratio. ENF's main
  failure mode is database self-similarity plus a free offset, which is exactly bellprint's global shift.
- **Correlation-style scoring over interval patterns.** Shift-invariant comparison of log-frequency
  (cents) intervals is the analogue of ENF's correlation coefficient. Fit the global shift as a nuisance
  parameter and penalise large shifts.
- **Use Hz, never note names.** Dove notes are derived from ring position and rings sit ~22 c from nominal
  note names. Match on the `Nominal (Hz)` field only.
- **More blows beat finer resolution.** Aggregate many strikes per bell before matching, mirroring
  ENF's "length matters most".
- **Add a bell-presence gate** trained or tested on ESC-50/AudioSet church-bell vs non-bell clips before
  attempting identification.
- **Marolt's pipeline is reusable** for counting bells and separating per-bell spectra in a change-ringing
  clip (covariance → number of bells → sparse NMF), with ringing-order priors replacing chiming rules.
- **Do not cite the Derbyshire anecdote** until someone produces the post.
