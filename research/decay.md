# Partial decay of bells: measured values

Per-partial decay times for the sim (`sim/bell.rs`). Researched 2026-11-04 to find a second source for
the Hibbert decay model held on branch `worktree-agent-a55a007c19444bfca` (449e08e: hum T60 13.5 s, every
other partial 2.7 s). Tags as in [acoustics.md](acoustics.md): **[V: src]** verified on the fetched PDF,
**[I]** our arithmetic or a value read off a figure.

Conversions [I]: amplitude ∝ e^(−t/τ); T60 = 6.91 τ; dB/s = 60/T60; Q = π f τ, so T60 = 2.2 Q/f.

## Sources

- **H-PhD**: Hibbert, PhD thesis, Open University 2008 (acoustics.md Sources). PDF p.154 (thesis p.138)
  and App. 10 (PDF p.331 on).
- **BD2002**: Boutillon & David, *Applied Acoustics* 63 (2002) 901–910. https://campaners.com/pdf/pdf332.pdf
- **W2012**: Woodhouse et al., *Adv. Acoust. Vib.* 2012, 681787. https://www.smith.edu/sites/default/files/media/Documents/ATS%20Campus/dynamics.pdf
- **DAFx17**: Canfield-Dafilou & Werner, "Modal audio effects: a carillon case study", DAFx-17, Edinburgh,
  https://www.dafx17.eca.ed.ac.uk/papers/DAFx17_paper_22.pdf
- **DAFx19**: Rau, Das & Canfield-Dafilou, "Improved carillon synthesis", DAFx-19, Birmingham.
  https://ccrma.stanford.edu/~orchi/Documents/DAFx19_paper_34.pdf
- **CCCBR-SC**: CCCBR Towers & Belfries Committee, "Sound control in bell towers – guidance notes".
  https://cccbr.org.uk/wp-content/uploads/2016/03/sound-control-notes.pdf
- **H-basic**: Hibbert, "Basic principles of bell tuning" (acoustics.md Sources).
- Searched, no decay figures: Perrin, Charnley & DePont 1983 (Hibbert's scan, image only) and their 2024
  AAS follow-up (Whyte, Perrin & Halkon, modes only); Orr, *Herit. Sci.* 9:16 (2021), which says its
  intensities "do not represent decay times"; Young et al., Physics Today 2026 (Liberty Bell; loss factors
  mentioned, none given). Not reachable: Rossing & Perrin, "Vibrations of bells", *Appl. Acoust.* 20 (1987)
  41–70, whose Table 5 lists "decay times of the principal modes" (ResearchGate 403); Fletcher & Rossing
  ch. 21 and Debut et al., *Herit. Sci.* 2021 (both behind a login redirect).

## What each source gives

**H-PhD (synthetic test sounds, not a measurement table).** "The hum (a relatively long lasting partial)
decayed to 60% of its initial value in 1s. The rest of the partials decayed to 60% of their initial values
in 200mS. These values were chosen to represent partial envelope decays measured in a number of bell
recordings." [V: H-PhD p.154]. App. 10 lists the same parameters per partial (`Hum_dec` 1, all others 5,
"the reciprocal of the time taken for the amplitude … to decay to 60%"), and every test sound is 0.25 s
long [V: H-PhD App. 10]. τ = 1.96 s / 0.39 s → T60 13.5 s / 2.7 s [I]. No bells, partial-wise data or
ringing style are given, and only the first 250 ms of the envelope is heard.

**BD2002 (46-bell Perpignan carillon, struck with its own clapper, accelerometer, matrix-pencil fit).**
"As expected, upper partials decay faster than lower ones" (§6). Fig. 3 plots Q per partial against
frequency; read off the figure [I], for the range of English ring nominals:

| partial | frequency in Fig. 3 | Q | T60 at a 1 kHz nominal [I] |
|---|---|---|---|
| hum | 300–1000 Hz | 4800 falling to 3300 | ≈ 35 s (Q 4000 at 250 Hz) |
| fundamental (prime) | 450–1000 Hz | 2800–3300 | ≈ 13 s |
| tierce | 500–1000 Hz | 3000–3400 | ≈ 11 s |
| nominal | 600–1600 Hz | **1000–1400** | ≈ 2.6 s |
| twelve (superquint), upper octave | 1–4 kHz | 2500–3500 | 3–4.5 s |

Fig. 2 agrees in dB/s at the low notes: hum ≈ 2–5, fundamental ≈ 5, tierce ≈ 8–10, nominal ≈ 25 dB/s
[I]. The text summarises the Q curve as "roughly 4500 at low frequencies to 2500 at 2 kHz and increasing
again to 4000 at 10 kHz" [V], but the figure shows the nominal well below the other partials at the same
frequency. acoustics.md §5 used the summary curve, giving the nominal ≈ 7 s at 1 kHz; that was the
basis of the old sim law and is about 2.5× too long for the nominal.

**DAFx17 (Lurie carillon, 60 bells, one recording each, fitted exponential per partial).** Fig. 4 plots
"Decay Rate" (no unit on the axis; τ of their e^(−t/τ) model, read as seconds) against frequency for bells 2, 20 and 50 [I, read off]: bell 2 lowest partial ≈ 100 Hz τ ≈ 20 s,
next two ≈ 200–250 Hz τ ≈ 5–6 s, the loudest group at ≈ 400–600 Hz τ ≈ 1–2 s. Low partials outlast the
loud ones by 3–10×.

**DAFx19 (Stanford carillon, C3 bell, prime 129 Hz, LDV + microphones).** Fig. 3b, "Extrapolated mode
decay rates" in s, per mode number [I]: mode 1 τ ≈ 15 s, mode 2 ≈ 3.5 s, most modes up to ≈ 70 at 0.5–1.5 s,
higher modes 0.1–0.2 s. "The low frequency modes are louder and decay more slowly than the high frequency
modes" [V §3.4].

**W2012 (lab bell full-circle; Great St Mary's, Cambridge, 5th and tenor of 12).** Single-strike lab
bell: 624 Hz Q = 1300, 981 Hz Q = 1000, 1310 Hz Q = 2000 [V]. In full-circle ringing the clapper bounces
for ≈ 0.8 s (lab), "over 1 s" (5th), "almost 2 s" (tenor) [V §2.2]. "While the clapper is bouncing, the
bell sound is able to ring on roughly as it would if chimed; it only switches to the faster decay when the
clapper comes to rest" [V]. Fig. 4: after rest the 624 Hz mode falls ≈ 45 dB in ≈ 0.15 s, ≈ 300 dB/s [I];
Fig. 3a shows every mode stopping at ≈ 1 s.

**Qualitative, English bells.** "The high frequency partials (nominal, quint) tend to be high-intensity and
short duration whereas the low frequency partials (hum, prime, & tierce) are generally lower intensity but
of long duration" [V: CCCBR-SC p.7]. "Hum, tierce and prime … often last for quite a while (many
seconds)" [V: H-basic].

## Verdict

- **Hum long, nominal short: supported by two independent sources.** BD2002's nominal Q ≈ 1200 gives
  T60 ≈ 2.6 s at 1 kHz, matching Hibbert's 2.7 s. BD2002's hum (≈ 35 s free) is longer than Hibbert's
  13.5 s, but both put it 5–13× the nominal.
- **Prime and tierce at 2.7 s: contradicted.** BD2002 (Q ≈ 3000, ≈ 11–13 s), DAFx17 and DAFx19 (second
  and third modes 3–5× the loud mid group), CCCBR-SC and H-basic all have prime and tierce long-lived.
  Hibbert's lumping is a 250 ms test-sound simplification.
- **Model adopted for the A/B (`sim/bell.rs`):** T60 = 2.2 Q/f per partial with BD2002's Q: nominal 1200,
  hum 4000, every other partial 3000. Quint and I-7/I-8 have no measured Q; they take 3000 [I]. Per-bell
  lognormal jitter (sd 0.25) stands for BD2002's bell-to-bell scatter. Against the old sim law only the
  nominal changes materially (6.4 → 2.6 s at 1 kHz); the hum goes 22 → 35 s at 250 Hz.
- **Clapper damping in full-circle ringing: one source (W2012), not modelled.** It would end every
  partial, hum included, 0.8–2 s after each blow. No second measurement found; Hibbert's 250 ms sounds
  say nothing about it.
- Every source is free decay of a stationary bell except W2012; none measures an English ring's hum,
  prime or tierce decay while ringing full-circle.
