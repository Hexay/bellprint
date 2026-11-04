# Bell acoustics for measurement

Scope: the partial structure of English change-ringing bells, how far each partial deviates from the
nominal, doublets, decay, strike pitch vs nominal, temperature, and where Dove's `Nominal (Hz)` values
come from. Researched 2026-11-03. All web content was treated as untrusted data.

Tags: **[V: src]** means verified on the fetched page or PDF. **[I]** means our own inference or
arithmetic. **[unverified]** means from memory and not checked.

## Sources (fetched)

- **H-build**: Hibbert, "Building a bell sound / The science and history of bell sounds" (CCCBR talk,
  2019; posted 2021-12-30). https://www.hibberts.co.uk/building-a-bell-sound/
- **H-basic**: Hibbert, "Basic principles of bell tuning" (2022 rev.). https://www.hibberts.co.uk/basic-principles-of-bell-tuning/
- **H-quality**: Hibbert, "The musical sound quality of church bells" (2008).
  https://www.hibberts.co.uk/the-musical-sound-quality-of-church-bells/
- **H-upper**: Hibbert, "The upper partials of bells" (2019). https://www.hibberts.co.uk/the-upper-partials-of-bells/
- **H-note**: Hibbert, "What note do we hear when a bell rings?" https://www.hibberts.co.uk/what-note-do-we-hear-when-a-bell-rings/
- **H-pitch**: Hibbert, "The musical pitch of bells". https://www.hibberts.co.uk/the-musical-pitch-of-bells/
- **H-strike**: Hibbert, "The strike note of bells", Ringing World 2003-06-20 p.586. https://www.hibberts.co.uk/the-strike-note-of-bells/
- **H-ident**: Hibbert, "Identifying bell partials" (2022). https://www.hibberts.co.uk/identifying-bell-partials/
- **H-doublet**: Hibbert, "Doublets or warble in bells". https://www.hibberts.co.uk/doublets-or-warble-in-bells/
- **H-calib**: Hibbert, "Frequency calibration for tonal analysis". https://www.hibberts.co.uk/frequency-calibration-for-tonal-analysis/
- **H-rec**: Hibbert, "Recording bells for analysis". https://www.hibberts.co.uk/recording-bells-for-analysis/
- **H-stretch**: Hibbert, "Stretch tuning of old-style bells". https://www.hibberts.co.uk/stretch-tuning-of-old-style-bells/
- **H-std**: Hibbert, "Bell Pitch Standards" (posted 2026-08-18, written in reply to a question from the
  Dove team). https://www.hibberts.co.uk/bell-pitch-standards/
- **H-GJbook**: Hibbert, "Gillett & Johnston tuning book entries" (2025). https://www.hibberts.co.uk/gillett-johnston-tuning-book-entries/
- **H-GJfork**: Hibbert, "Anomalies in the tuning of G&J bells" (2022/23). https://www.hibberts.co.uk/gillett-johnston-tuning-forks-anomalies/
- **H-PhD**: Hibbert W.A., *The Quantification of Strike Pitch and Pitch Shifts in Church Bells*, PhD
  thesis, Open University, 2008. https://www.hibberts.co.uk/wp-content/uploads/2021/12/wah_thesis_18_4_08.pdf
  The PDF has 353 pages; page numbers below are PDF pages.
- **BD2002**: Boutillon X., David B., "Assessing tuning and damping of historical carillon bells and their
  changes through restoration", *Applied Acoustics* 63 (2002) 901–910. https://campaners.com/pdf/pdf332.pdf
- **W2012**: Woodhouse J. et al., "The Dynamics of a Ringing Church Bell", *Adv. Acoust. Vib.* 2012,
  681787. https://www.smith.edu/sites/default/files/media/Documents/ATS%20Campus/dynamics.pdf
- **Dove-help**: Dove's Guide help pages. https://dove.cccbr.org.uk/help/bells and https://dove.cccbr.org.uk/help/conventions
- **PC1973 / PCD1983**: Perrin & Charnley, "Group theory and the bell", *J. Sound Vib.* 31:411–418 (1973).
  Perrin, Charnley & DePont, "Normal modes of the modern English church bell", *JSV* 90(1):29–49 (1983).
  Neither paper was fetched. Their data is used here only as H-PhD Table 5-1 reproduces it.
- Not fetched, cited only through Hibbert: Lehr, "Partial groups in the bell sound", JASA 79(6) 1986;
  Rossing, *Acoustics of Bells* (1984); Terhardt & Seewann, *Acustica* 54 (1984); Eggen & Houtsma
  (1986). Fletcher & Rossing and Rossing's *Science of Percussion Instruments* could not be accessed.

## 1. Partial names and nominal ratios

Each bell has dozens of partials. H-PhD found up to 135 in one bell [V: H-build]. The ones that matter:

| Partial | Mode (nodal meridians) | Type | Ideal cents from nominal | Ideal ratio |
|---|---|---|---|---|
| Hum | 4 | rim (I-2) | −2400 | 0.25 |
| Prime | 4 | group II | −1200 | 0.5 |
| Tierce | 6 | rim (I-3) | −900 (minor 3rd above prime) | 0.5946 |
| Quint | 6 | group II | −500 (5th above prime) | 0.7492 |
| Nominal | 8 | rim (I-4) | 0 | 1 |
| Superquint | 10 | rim (I-5) | "roughly a fifth" | see §3 |
| Octave nominal | 12 | rim (I-6) | "roughly an octave" | see §3 |
| I-7, I-8 … | 14, 16 … | rim | determined by octave nominal | see §3 |

Sources: [V: H-ident table, H-basic]. There are also three weak partials between the nominal and the
superquint (II-4, III-3, III-2), at about +395 to +536 cents [V: H-basic Taylor/Mears tables; H-PhD Table 5-1].
- The partials are always in the same order for a bell of conventional shape [V: H-ident]: hum < prime
  < tierce < quint < nominal. Long-waisted and archaic bells can break this rule [V: H-ident].
- The quint is quiet in most bells because the clapper hits near one of its nodes [V: H-ident, H-quality]. One Taylor
  treble showed no measurable quint doublet [V: H-ident]. In H-PhD's survey the quint was missing in
  285 of 2,066 bells (1,781 measured), against ≥2,056 for every other low partial [V: H-PhD Table 5-2].
- Typical relative amplitudes in the first 250 ms of a full-circle bell (arbitrary units, averaged over
  15 bells): hum 2.79, prime 5.09, tierce 8.35, quint 1.00, nominal 10.60, superquint 8.79, octave
  nominal 7.10, I-7 4.94, I-8 4.03, I-9 3.30, I-10 2.05, I-11 1.52 [V: H-PhD Table 6-5].

## 2. Low partials: true-harmonic vs old-style (cents relative to nominal)

**Verified population statistics.** Deviation from true-harmonic in cents is (measured − ideal). The
hum ideal is −2400 and the prime ideal is −1200 [V: H-build founder table]:

| Founder / era | n | Hum mean | Hum SD | Prime mean | Prime SD |
|---|---|---|---|---|---|
| Taylor > 1895 | 1277 | +0.6 | 10.1 | −2.0 | 11.3 |
| Mears/Whitechapel > 1926 | 498 | −1.5 | 8.9 | −1.1 | 6.5 |
| Hemony 1651–68 | 61 | −0.4 | 4.3 | +0.4 | 6.4 |
| Dobson 1808–28 | 63 | +11.0 | 40.4 | −33.8 | 63.3 |
| Knight/Catlin 1575–1748 | 89 | +62.2 | 58.6 | −37.1 | 74.9 |
| Wells 1770–1826 | 19 | +93.8 | 56.9 | −52.6 | 83.9 |
| Rudhall 1685–1835 | 324 | +98.5 | 62.0 | −73.3 | 88.7 |
| Mears < 1851 | 290 | +130.0 | 59.4 | −127.9 | 86.2 |
| Briant 1794–1825 | 41 | +60.7 | 51.0 | −153.7 | 95.5 |

- In old-style bells the hum is sharp by up to a whole tone and the prime flat by up to a whole tone.
  Some primes are sharp instead [V: H-build, H-quality]. Naylor Vickers steel bells have hums up to two whole
  tones (+400 cents) sharp [V: H-build].
- Typical ranges across all bells (the ranges used in H-PhD's pitch experiments): hum −2400 to −2200,
  prime −1400 to −1200, tierce −900 to −800, quint −500 to −300 [V: H-PhD Table 6-5; H-build table].
- Tierce: it ranges from −900 (a major sixth below the nominal) to nearly a minor sixth below (about −800). The tuner
  cannot move it independently [V: H-quality, H-upper]. Measured examples: Taylor 1980s −909, PCD bell
  −909.8, Mears 1859 −868 [V: H-basic, H-PhD Table 5-1]. In the Perpignan carillon the tierce sits at
  ratio 1.195 to nominal/2, i.e. 9 cents above a tempered minor third [V: BD2002].
- Quint: true-harmonic is about −500 (Taylor −491). Old-style is often far from it (Mears 1859 −374) [V: H-basic].
- Hum, prime and quint can be varied independently of the upper partials. Tierce and upper partials cannot [V: H-upper].
- Correlations across 2,066 bells: hum–quint +0.63, hum–tierce +0.46, prime–tierce +0.39. The low
  partials have |r| < 0.35 with the upper partials [V: H-PhD Table 5-3].
- The modern tuning tolerance per partial is about ±10 cents. Hemony achieved about 3 cents [V: H-basic]. Since 1925 nominals have been tuned
  to "±5 or ±10 cents" against A=432, 435 or 440 [V: H-std].
- Era boundary: Taylor went true-harmonic in 1896. Whitechapel kept casting old-style designs into
  the 1920s and 1930s, with some true-harmonic bells from 1923. Warner never achieved it consistently
  [V: H-build]. Retuning an old-style bell rarely makes it fully true-harmonic. For example, Lyminge 5th
  (Mears 1785, tuned 2002) went from hum −2203 and prime −1290 to hum −2324 and prime −1273 [V: H-build].

## 3. Upper partials (superquint, octave nominal, I-7…): one-parameter family

**Verified [H-PhD Table 5-4, 2,066 bells; 888 for I-7 and above].** The cents of each upper partial
fit `cents_p = m · cents_onom + c`:

| Partial | m | c | resid SD (cents) | at onom = 1140 | at onom = 1200 | at onom = 1280 |
|---|---|---|---|---|---|---|
| Superquint | 0.557 | −0.2 | 4.9 | 635 (×1.443) | 668 (×1.471) | 713 (×1.509) |
| Octave nominal | 1 | 0 | 0 | 1140 (×1.932) | 1200 (×2.000) | 1280 (×2.095) |
| I-7 | 1.363 | 0.2 | 8.9 | 1554 (×2.454) | 1636 (×2.572) | 1745 (×2.740) |
| I-8 | 1.668 | 0.1 | 17.3 | 1902 (×3.00) | 2002 (×3.18) | 2135 (×3.43) |
| I-9 | 1.929 | −0.2 | 26.3 | 2199 | 2315 | 2469 |
| I-10 | 2.156 | −1.3 | 34.8 | 2457 | 2586 | 2758 |
| I-11 | 2.362 | −9.0 | 42.3 | 2684 | 2825 | 3014 |

The column cents and ratios are [I] arithmetic from the fitted parameters.
- Octave nominals of 1140 to 1280 cents are "commonly encountered". The scatter plots run from about 1050
  to 1400 in extreme cases [V: H-note, H-PhD figs 5-4..5-10]. Thick bells (e.g. trebles of 10s and 12s)
  have compressed partials. Thin bells have stretched ones [V: H-upper].
- Examples: Taylor 1980s bell: superquint 706, octave nominal 1268, I-7 1731, I-8 2118. Mears 1859:
  666, 1203, 1647, 2018 [V: H-basic]. PCD bell: 708.1, 1270.3, 1729.8, 2117.4 [V: H-PhD Table 5-1].
- The upper partials move together. You cannot tune them independently, as shown by the Berlin
  Freedom Bell, where removing 1.4 t of metal sharpened all of them together [V: H-upper].
- **[I] Consequence: modern Taylor bells have an octave nominal of about 2.08 × nominal (+70 cents), not
  2.00. A superquint fixed at 1.5 is right for those bells but about 30 cents sharp for typical older
  bells (about 1.47).**

## 4. Doublets and beating

- Any departure from axisymmetry splits each mode into two frequencies. Both sound unless the clapper
  hits a node, and they beat at the difference frequency [V: H-doublet, H-PhD §3.2.5, W2012].
- **Measured splits on one modern English bell** (PCD bell, nominal 1171 Hz) [V: H-PhD Table 5-1]: hum 0,
  prime 0, tierce 0, quint 0.32 Hz, nominal 0.6 Hz, superquint 3.2 Hz, octave nominal 4.3 Hz, I-7
  7.1 Hz, I-8 3.6 Hz, I-9 0 Hz. Some higher modes split by 11 to 33 Hz.
- Other examples: Dorking service bell (Wilnar 1626) nominal 831.7/832.3 Hz, a 0.6 Hz split [V: H-ident]. In
  Wellesley carillon bell 9 (G&J 1931) the prime is 867.6/868.5 Hz, a 0.9 Hz split. G&J books record ½ to 1½ Hz beats
  on hum and prime [V: H-GJbook]. Big Ben's nominal of about 335 Hz is "doubletted" [V: H-pitch].
- Splits grow with partial rank. Low partials beat slowly, so high-resolution FFTs are needed to
  separate them [V: DAFx 2019, Rau/Das/Canfield-Dafilou, https://ccrma.stanford.edu/~orchi/Documents/DAFx19_paper_34.pdf].
- Hibbert's convention is to treat a doublet as one partial, at the frequency of the louder half [V: H-PhD §3.1].
- **[I]** Typical splits are well under 1 Hz on hum through nominal and a few Hz on the upper partials.
  A nominal split of 0.6 Hz is 0.9 to 2 cents, which is below Dove's precision.

## 5. Decay

Measured per-partial values, sources that disagree, and the sim model: see [decay.md](decay.md).
- Free decay (BD2002 Fig. 3, read off): Q ≈ 4000 for the hum, ≈ 3000 for prime, tierce and upper
  partials, but ≈ 1200 for the nominal. At a 1 kHz nominal: hum T60 ≈ 35 s, prime ≈ 13 s, tierce ≈ 11 s,
  nominal ≈ 2.6 s [I]. Hibbert's test sounds (hum 13.5 s, all else 2.7 s) agree on hum and nominal only.
- **Full-circle ringing**: the clapper bounces for about 0.8 s (small bell) to about 2 s (tenor). While it
  bounces the bell decays as if chimed. Once the clapper comes to rest against the bell, friction damps
  the vibration "very quickly" [V: W2012].
- "Hum, tierce and prime … often last for quite a while (many seconds)". Short-lived high partials are
  hard to measure with forks [V: H-basic].
- Impure bronze deadens tone, probably because some important partials decay fast [V: H-PhD ch.4].
- Full-circle and swing-chimed bells have more prominent high partials than carillon-chimed bells
  [V: H-PhD §3.3].

## 6. Strike note vs nominal

- The strike note is a virtual pitch, about an octave below the nominal (nominal/2). It need not
  exist as a frequency in the sound [V: H-pitch, H-build]. Mainly the nominal, superquint and octave
  nominal generate it [V: H-pitch, after Eggen & Houtsma].
- Size bands [V: H-note, H-quality]:
  - Nominal 500 to 1500/1600 Hz (about 2 t down to 500 kg): most listeners hear nominal/2.
  - Below 500 Hz: a secondary strike from I-7, about a fourth higher, takes over for about 2:1 of listeners.
  - Above about 1600 Hz: the prime or hum sets the pitch.
  - Above about 2500 Hz: the nominal is irrelevant to pitch.
  Example: Great Bede's strikes are 204.8 Hz and 269.5 Hz [V: H-pitch].
- Pitch shift: 0.1 to 0.2 cents per cent of octave-nominal change. Across the 1140–1280 range this is 14 to 28 cents.
  Shifts of 20 to 30 cents are common [V: H-note, H-pitch]. Trebles stretched for this reason sit 30.5 ± 9.6 cents sharp
  (8s) and 34.7 ± 8.4 cents sharp (10s) of a tempered scale on the tenor nominal [V: H-stretch].
- **[I] For bellprint:** we measure partial frequencies, not pitch, so virtual pitch does not matter
  directly. It does mean that interval ratios between nominals in a ring are not equal-tempered and the
  `Note` field is not a substitute for Hz. Dove's `Nominal (Hz)` already includes the stretch.

## 7. Temperature, corrosion, long-term stability

- The temperature coefficient of bronze bell partials is −1.5×10⁻⁴ per °C, i.e. **−0.26 cents/°C**
  (−2.6 cents per 10 °C) [V: H-PhD §3.1.1 citing Terhardt & Seewann 1984 §2.1, and Jones; arithmetic I].
  Van Heuven gives −4×10⁻⁴ per °C, i.e. **−0.69 cents/°C** [V: BD2002]. The sources disagree.
  Use 0.26–0.7 c/°C.
- All partials of all bells shift by the same proportion [V: H-PhD]. **[I]** A 0–30 °C seasonal swing
  therefore shifts everything by 8 to 21 cents. One global scale factor absorbs this.
- Bronze tuning is stable over decades. Hibbert matched 50- and 100-year-old founder figures against
  recordings "with very good correspondence" [V: H-PhD §3.1.1]. A century of oxide lowered Perpignan
  carillon partials by about 8 cents, slightly less for the hum [V: BD2002]. Corroded small bells in polluted air
  go noticeably flat (Bournville trebles) [V: H-GJfork]. Rusted steel bells changed little between
  1993 and 2017 [V: H-build].

## 8. How nominals are measured; accuracy of Dove `Nominal (Hz)`

- Dove: "The frequency listed is that of the nominal, unless otherwise specified". Notes assume A=440
  and equal temperament. Dove points to Hibbert's Wavanal, which "has brought the measurement of a
  bell's nominal within the capability of anyone with a good sound recorder" [V: Dove-help]. Dove
  team members do send recordings for analysis (Tim Jackson, Kirkby Stephen) [V: H-stretch]. The
  fetched pages give no per-bell provenance. **[I]** Values are a mix of founder tuning
  figures and Wavanal analyses of ringer recordings.
- Methods: founders beat partials against tuning forks, frequency generators or strobe discs. Modern
  practice uses spectrum analysers [V: H-basic]. Taylor bought its first forks and tuning machine around
  1894–96 [V: H-build].
- **Fork errors**: all 2,044 G&J forks were measured. The error cycles by octave (A forks flattest, E♭
  sharpest) and spans up to 25 cents across an octave. Extreme forks are about 75 cents flat [V: H-GJfork].
  Uncalibrated forks can be "up to 10 Hz out" [V: H-calib].
  **[I] Founder-record Hz for G&J bells (1907–1950s) may be off by 10 to 25 cents from the true
  nominal.**
- Recording-based accuracy: Wavanal's FFT bin is 1/t Hz. A 2 s sample gives ±0.25 Hz, quoted as
  integer or .5. A 10 s sample gives 0.1 Hz [V: H-PhD §1.3]. Calibrated laptops and phones are accurate to
  <0.05 Hz [V: H-calib, H-rec]. Full-circle recordings are "less accurate due to the doppler shift".
  Stationary hard-struck bells are best [V: H-rec]. Doppler shows as line broadening or splitting [V: H-PhD §3.3].
- **[I] Local check on our data** (`bells.csv`, 46,983 nominals): 33,617 integer, 10,020 ending .5,
  3,346 other tenths. This fits mostly 2 s Wavanal-style or rounded founder values. Assume ±0.5 Hz
  quantisation, i.e. ±3.5 cents at 250 Hz and ±0.9 cents at 1 kHz per Hz.
- **[I] Doppler estimate:** in full-circle ringing the radiating surface moves at about 2–4 m/s near the bottom of the swing.
  v/c ≈ 0.6–1.2%, which is ±10–20 cents of frequency modulation within one stroke. Measuring at the strike, when the bell
  is near the balance point and slow, minimises it.
- **Retuning / staleness**: in `dove.csv`, 1,267 of 7,262 rings have a `TuneYr`, spread from the 1890s
  to the 2020s with about 160–170 per decade since 1990 [I: local count]. That is roughly 2% of rings
  retuned per decade, or a few hundred rings per decade. Tuning lowers the strike note unless it is very
  heavy, so a ring is tuned down to its flattest bell [V: H-build]. **[I]** A Dove nominal
  measured before an undocumented retune can be tens of cents sharp of the current bell. The Lyminge
  example moved hum/prime by up to 121 cents, and nominals move by the "flattest bell" alignment, typically
  <50 cents [unverified magnitude].

## Implications for bellprint

- **Model the octave nominal as a fitted ratio**, about 1.93–2.10 × nominal (centre 2.0–2.08). It sits
  near 2.08 for post-1895 Taylor bells. Derive superquint, I-7 and I-8 from it using the H-PhD slopes
  (0.557, 1.363, 1.668) instead of fixed ratios. That reduces three free tolerances to one per-bell parameter.
- **Superquint ratio**: use about 1.47 (668 c) with a ±45 c window (635–713 c), not a fixed 1.5.
- **Per-partial windows** (cents from nominal, centre / half-width):
  - true-harmonic (cast ≥1896 Taylor, ≥1926 Whitechapel/G&J): hum −2400 ±30, prime −1200 ±35,
    tierce −905 ±20, quint −495 ±25.
  - old-style / untuned: hum −2400…−2150 (centre −2270), prime −1450…−1150 (centre −1330), tierce
    −900…−800 (centre −865), quint −500…−300.
  - Pick the class from `Cast Date` and `Founder` in bells.csv when available. Otherwise use the union.
- **Simulator hum and prime** (`sim/bell.rs`, `Tuning`), [C] from the §2 founder table, pooled by n
  (mean of means; SD = √(mean within-founder variance + variance of the means)):
  - true-harmonic, Taylor > 1895 and Mears/Whitechapel > 1926 (n = 1775): hum +0 ± 10 c, prime −2 ± 10 c;
  - old-style, Dobson, Knight/Catlin, Wells, Rudhall, Mears < 1851 and Briant (n = 826): hum +97 ± 68 c,
    prime −89 ± 93 c.
  - A bell is true-harmonic if cast ≥ 1926, old-style if cast < 1896, and either with even odds between
    (Taylor switched in 1896, Whitechapel only in the 1920s) or when Dove has no cast date. Retuning old
    bells is ignored: per §2 it rarely makes them fully true-harmonic.
- **Weight the nominal highest, then superquint, tierce and octave nominal.** The quint is weak and
  often missing, so give it near-zero weight and never penalise its absence. Hum and prime are
  diagnostic of tuning style but carry ±100+ c spread in old bells.
- **Expect nominal-level agreement of ±10 c in the best case.** Dove quantisation (±0.25–0.5 Hz),
  temperature (0.26–0.7 c/°C) and recorder calibration together are about ±5–10 c. Old G&J fork-based
  figures can be off by up to 25 c and retunes add more. Use a scoring kernel about 10 c wide with heavy
  tails out to about 50 c.
- **Keep the single global pitch shift.** Temperature, recorder clock and corrosion all scale every
  partial equally. Bound it to about ±30 c, or ±1% if Doppler-smeared clips are allowed.
- **Inter-bell intervals are more robust than absolute Hz**, since temperature and clock errors cancel.
  Do not assume equal temperament: stretched trebles run 30 ± 10 c sharp.
- **Analysis window**: for full-circle ringing, energy after about 1–2 s is clapper-damped, and each bell
  sounds for only about 0.25 s in changes. The long-term spectrum should rely on nominal and tierce
  (loudest in the first 250 ms). The hum builds up only on swung or long-ringing bells and on rounds or
  call-changes with gaps.
- **Frequency resolution**: doublets are <1 Hz on hum through nominal, so treat a doublet as one peak.
  Clips of ≥2 s give ±0.25 Hz bins. Resolving doublets needs ≥10 s of a stationary bell, which we will
  not get.
- **Large tenors** (nominal < 500 Hz): the partials are still measurable even though the perceived pitch moves
  to I-7. Small trebles (nominal > 1600 Hz) push the upper partials towards 3–5 kHz, so make sure the
  recorder band does not cut them (44.1 kHz recommended).
- **Staleness**: treat a `TuneYr`/`OvhaulYr` after the source date of the nominal as a reason to widen
  the tolerance for that ring. Without per-bell provenance, allow about 2% of rings to be stale per decade.
