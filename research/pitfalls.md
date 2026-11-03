# Practical pitfalls for bell-frequency identification from real-world clips

Researched 2026-11-03. Tags: **[V]** = read on fetched page or source file; **[S]** = search snippet only;
**[C]** = computed here (formula given); **[I]** = inference; **[unverified]** = from memory.

## 1. Speed / pitch alteration (the global shift must absorb these)

| Cause | Ratio | Cents | Source |
|---|---|---|---|
| 44.1 kHz audio played/labelled as 48 kHz (or reverse) | 48/44.1 = 1.0884 | **±146.7** | [C] 1200·log2(48/44.1); KVR forum gives −146.7 c https://www.kvraudio.com/forum/viewtopic.php?p=1470488 |
| PAL speed-up, 24 fps film to 25 fps | 25/24 = 1.0417 | **+70.7** | [C]; "0.7067 semitone" per Wikipedia quote on Shotcut forum [S] https://forum.shotcut.org/t/please-test-the-beta-for-version-22-06/33941 |
| NTSC pull-down 24 to 23.976 (30 to 29.97) | 1001/1000 | **±1.7** | [C] |
| Combined 48/44.1 and PAL | 1.1338 | **±217** | [C] |
| TikTok "sped up" edits | ~1.25–1.30× | **+386 to +454** | [S] RNZ 2024 "25-30 percent" https://www.rnz.co.nz/news/top/525509/tiktokers-are-speeding-up-songs-but-are-we-forgetting-the-originals ; 1.25× "popular TikTok/Reels" [S] |
| Nightcore | 1.25× + extra pitch | ~+3 semitones or more beyond speed | [S] https://soundtools.io/nightcore |
| Speed-change to dodge Content ID | arbitrary | arbitrary | [S] industry exec quote that "a tempo change" got past Content ID https://www.hypebot.com/the-problem-with-content-id-start-with-dozens-of-youtube-videos-on-how-to-defeat-it/ |

- Proper resampling (e.g. YouTube re-encoding to AAC 44.1 kHz itag 140 or Opus 48 kHz itag 251) does **not**
  change pitch. Only a *mislabelled* rate does. [S] itag listings https://gist.github.com/AgentOak/34d47c65b1d28829bb17c24c04a0096f ; [I]
- **Pitch-preserving time-stretch** (TikTok/editor "speed" with pitch lock) changes duration but not partial
  frequencies. **Pitch-shift without tempo change** shifts all partials by one ratio, which bellprint's global
  shift absorbs. Formant-preserving or phase-vocoder shifts smear partials and add tens-of-ms artefacts. [I]
- A uniform shift keeps partial *ratios* intact (hum/prime/nominal ratios unchanged). Any process that
  changes ratios is not a speed change and should lower confidence. [I]

## 2. Lossy codecs

### Lowpass cutoffs (verified from source code / specs)
- **LAME MP3 default lowpass by bitrate** (`libmp3lame/lame.c` `freq_map`, Debian lame 3.100) [V]:
  8 kbps→2.0 kHz, 16→3.7, 24→3.9, 32→5.5, 40→7.0, 48→7.5, 56→10.0, 64→11.0, 80→13.5, 96→15.1, 112→15.6,
  **128→17.0**, 160→17.5, 192→18.6, 224→19.4, 256→19.7, 320→20.5 kHz. Transition width defaults to 15% of the
  lowpass. [S] https://svn.exotica.org.uk:8443/xbmc4xbox/vendor/liblame/current/doc/html/switchs.html
- **Opus audio bandwidths** (RFC 6716 Table 1) [V]: NB 4 kHz, MB 6, WB 8, SWB 12, FB 20 kHz. "Opus never codes
  audio above 20 kHz." Sweet spots: 8–12 kb/s NB speech, 16–20 WB speech, 28–40 FB speech, 48–64 FB mono
  music, 64–128 FB stereo music. https://www.rfc-editor.org/rfc/rfc6716.txt
- **libopus automatic bandwidth switch thresholds** (`src/opus_encoder.c`, xiph/opus main) [V], value ±
  hysteresis in bps (equivalent rate):
  voice: NB↔MB 9000±700, MB↔WB 9000±700, WB↔SWB 13500±1000, SWB↔FB 14000±2000.
  music: WB↔SWB 11000±1000, SWB↔FB 12000±2000.
  So a voice-classified signal below ~13 kbps is capped at **8 kHz**, and below ~14 kbps at **12 kHz**. [V+I]
- **WhatsApp voice notes**: Opus at **16 kHz** sample rate (container metadata "Sampling rate: 16.0 KHz",
  "Writing library: WhatsApp") [S] https://ffmpeg.org/pipermail/ffmpeg-user/2019-July/044853.html .
  So content above **8 kHz** is gone. Bitrate 16 kbps per a low-authority blog [S], 24 kbps per a call-codec
  reimplementation [S] https://docs.rs/crate/whatsapp-rust/latest/source/agent_docs/voip_audio_codecs.md
- **YouTube** itag 140 = AAC-LC ~128 kbps 44.1 kHz; itag 251 = Opus ≤160 kbps 48 kHz [S]. A forum claim says
  YouTube's AAC cuts above ~15 kHz [S, anecdotal]. Treat 15–16 kHz as the usable ceiling [I].

### Fake or missing partials
- **HE-AAC SBR**: the high band is regenerated "from the low frequency data" plus envelope side info. The
  core runs at **half the output sample rate** [V] https://wiki.multimedia.cx/index.php/SBR . Above the
  crossover (bitrate-dependent; no authoritative number found [S]), spectral peaks are *transposed copies of
  low-band peaks*, so they sit at wrong frequencies with plausible energy. An SBR copy of the nominal can
  look like a superquint or octave that is not there. [I]
- **Opus CELT band folding**: "the shape of the bands is only coded if there is at least 1/2 bit per sample
  available ... If not, then no bit is allocated and folding is used instead" (RFC 6716 §5.3.4.3) [V]. Folded
  bands carry copied/noise spectral shape, so their peaks are not real partials. [I]
- **Spectral holes**: MP3/AAC at ≤128 kbps zero out masked bins frame by frame (birdies), so weak upper
  partials flicker in and out. [unverified]
- **Frequency precision of strong tones** survives transform codecs: MDCT quantisation perturbs amplitude
  and phase, but a stable sinusoid's peak frequency over a long window is preserved to well under 1 cent if
  the bin is coded. [I, needs a test: encode a synthetic bell at 64/96/128 kbps and measure drift]

## 3. Capture chain: phones, apps, calls

- **Android**: the `VOICE_RECOGNITION` source must have noise reduction and AGC disabled (CDD 4.1, per SO quote).
  `UNPROCESSED` gives raw capture where supported. Default recorder/camera sources may apply processing. [S]
  https://source.android.com/docs/core/audio/implement-pre-processing
- **iPhone Voice Memos AGC/limiter**: no Apple documentation found. [S, unverified]
- **Call/VoIP noise suppression kills sustained tones**: the WebRTC NS "can mistake a sustained note for
  background noise, since it is stationary" (discuss-webrtc). A Cisco patent says tones and music-on-hold "might
  appear as noise ... and might be suppressed". Browser defaults have `noiseSuppression`, `echoCancellation`
  and `autoGainControl` all on. [S] https://groups.google.com/g/discuss-webrtc/c/PDZYMzAI-_Y ;
  https://docs.nanocosmos.de/docs/webrtc/nanostream_webrtc_speech_music
  A bell's decaying hum and nominal are exactly the stationary tonal content NS removes. Expect the
  long-decay partials (hum, prime) to be attenuated first. [I]
- **AGC/limiting**: changes envelope, not frequency. It damages decay-rate features and onset amplitudes, and
  clipping adds harmonics/intermod (fake peaks at sums/differences of partials). [I]
- **Wind**: broadband low-frequency noise, mostly below a few hundred Hz, competing with hums of heavy tenors
  (a 20 cwt tenor nominal ~ 300–350 Hz, hum ~ 75–90 Hz) [unverified figures]. Phone wind filters are high-pass. [I]

## 4. Propagation and recording position

- **Air absorption** (ISO 9613-1 formula, implemented here and validated against the 10 °C/70 % RH table from
  the old ISO 9613-2: 1 kHz 3.7, 2 kHz 9.7, 4 kHz 32.8, 8 kHz 117 dB/km
  https://help.emd.dk/knowledgebase/content/windPRO4.1/Appendix_A_DECIBEL.pdf ) [C]:

  | dB/km | 500 Hz | 1 kHz | 2 kHz | 4 kHz | 8 kHz |
  |---|---|---|---|---|---|
  | 10 °C, 70 % | 1.9 | 3.7 | 9.7 | 33.2 | 118.8 |
  | 20 °C, 70 % | 2.8 | 5.0 | 9.1 | 23.1 | 77.9 |

  At 500 m, 8 kHz loses 40–60 dB extra versus a few dB at 1 kHz. Distant recordings lose the upper partials
  (superquint, octave, upper partials of trebles) before the nominal. Spherical spreading (−6 dB per doubling)
  is frequency-independent. [C+I]
- **Louvres, inside vs outside**: louvres and tower walls act as a directional, frequency-dependent filter.
  Recordings in the ringing chamber are dominated by low partials coupled through the floor and by room
  modes, and may barely contain trebles' nominals. Room modes colour amplitudes, not frequencies, so peak
  frequencies are still usable. [I]
- **Doppler from swinging**: ringing sources describe "a doppler effect ... as the strike occurs whilst the
  bell is still moving as it approaches top dead centre" [S] https://en.wikipedia.org/wiki/Ring_of_bells .
  No measured value was found. [C/I] Rim speed at the strike is ~0.5–2 m/s, so 1200·log2(1+v/343) ≈ **2.5–10
  cents**. The shift is signed and varies per stroke and per listener direction, so the hand/back stroke
  difference shows as a wobble. It averages out over many blows. Chimed or hammered bells have none.
- **Multiple towers audible**: town-centre clips can contain two rings, or a ring plus a clock. Sets of
  nominals that do not fit one diatonic ring (e.g. two tonics, too many bells) are the indicator. [I]

## 5. What is actually ringing

- **Dove coverage of clock/chime bells**: Dove's lead (Dickon Love, Bell-historians list, 2 Jun 2026): "We do
  not record which bells are used as quarter or hour bells unless they are installed as such as their
  principle use", and Ellacombe racks are recorded "only provided we are advised of its presence". [V]
  https://lists.ringingworld.co.uk/pipermail/bell-historians/2026-June/022004.html
- Dove marks hanging: italics for bells not hung for full-circle ringing, `U` not hung, `D` hung dead, `S`
  swing chiming, `NS` non-standard. Accidentals (e.g. 2♯, 6♭) are listed after the main ring. The listed
  frequency is the nominal unless stated. Additional bells are included "where we know about them". [V]
  https://dove.cccbr.org.uk/help/bells
  Consequences [I]: (a) an hour bell is often the tenor of the ring, so its nominal is in Dove but
  unlabelled; (b) separate clock/quarter bells (e.g. a Westminster-quarters set) may be missing entirely;
  (c) a 4-note Westminster chime gives only 4 nominals, a weak fingerprint.
- **Hammer vs clapper**: Hibbert found "no perceivable difference" between a 1.5 kg and a 540 g hammer on a
  clock bell [S] https://www.hibberts.co.uk/identifying-bell-partials/ . Strike position changes partial
  *amplitudes* and so the perceived strike note, but not mode *frequencies*. A clock hammer on the outside
  of a stationary bell and a swinging clapper give the same partial frequencies, minus the swing Doppler. [I]
- **Electronic/digital bells**: real UK installs exist, e.g. St Michael's Stoke Gifford (digital system from
  8 Dec 2018, four loudspeakers aimed at the louvres) [S]
  https://www.bradleystokematters.co.uk/community-news/digital-bells-an-appealing-option-for-st-michaels-church-stoke-gifford/ .
  Vendors: Smith of Derby digital chiming ("wedding peals, change ringing, chimes, tunes"), McShane Apollo
  (built-in synthesizer, 64-note carillon), Phoenix GABRIEL [S]. Carter Voce "DigiToll" (2003) played a
  **recording of St John's, Dublin** [S] https://lists.ringingworld.co.uk/pipermail/bell-historians/2003-April/002654.html .
  A sampled system will match the *source* tower, not the one it is installed in. [I]
- **Ringing simulators** (Abel, Beltower, Virtual Belfry): Abel ships "real tower or handbell sound" and lets
  users record their own bells [S] https://www.abelsim.co.uk/doc/bellsnds.htm . Videos of simulator practice,
  handbell or Ringing Room sessions, and stock sound effects in edited videos produce real-looking bell
  spectra from a different (or no) tower. [I]
- **Electronic bell tells** [I, to validate]: identical spectra across blows of the same bell (real swing
  ringing varies in strike amplitude and Doppler), sample-loop repetition, a hard bandlimit from the playback
  chain (loudspeaker roll-off below ~100 Hz removes the hum of heavy bells), and perfectly equal-tempered
  nominals.

## Verified vs inference
- Verified numbers: LAME table, Opus bandwidths and libopus thresholds, RFC folding rule, Dove recording
  policy, the 48/44.1 and 25/24 ratios (arithmetic), and air absorption (ISO formula validated against table).
- Snippet-only: WhatsApp 16 kHz, YouTube itags, WebRTC NS behaviour, TikTok 1.25–1.30×, the digital-bell installs.
- Inference: Doppler magnitude, codec frequency precision, SBR/folding fake partials, electronic-bell tells.

## Implications for bellprint
- **Global shift search range**: cover at least **−230 to +230 cents** by default (48/44.1 ±147 combined
  with PAL ±71, plus 10 c slack). Add an opt-in "sped-up" mode to about **+500 c** (TikTok 1.25–1.3×). Put a
  prior on the known ratios {0, ±1.7, ±70.7, ±146.7, ±217.4 c} and penalise others.
- **Shift ambiguity**: ±147 c is about 1.5 semitones, so a ring in D shifted by a mislabelled rate can land
  near a ring in E♭/E. Always report the best fit at each prior shift plus the margin.
- **Per-input codec ceiling**: read codec/bitrate from the container (ffprobe). Ignore partials above
  min(codec lowpass, 0.9·Nyquist). Use 8 kHz for WhatsApp/Opus-WB, 12 kHz for Opus-SWB, the LAME table for
  MP3, and ~15 kHz for YouTube AAC. Ignore anything above the SBR crossover for HE-AAC (detect via `aac_he`
  profile).
- **Base identity on nominal + hum/prime/tierce**, which survive all codecs and distance. Use superquint and
  octave only as optional confirmation, never as hard requirements.
- **Check partial ratios before matching**: if hum:prime:nominal ratios are wrong, flag non-uniform
  processing (phase vocoder, SBR, folding) instead of fitting.
- **Detect call/voice-processed audio** (16 kHz rate, Opus voice mode, abrupt tonal gating) and warn that
  sustained partials may be suppressed.
- **Doppler tolerance**: use a ~10 c per-blow tolerance for swing ringing, averaged over blows. For
  hammered/chimed bells, expect tighter clustering.
- **Out-of-Dove sources**: output "no confident match" when nominals fit a ≤5-note chime or Westminster
  pattern. Clock and quarter bells are often missing from Dove.
- **Electronic-bell detector**: flag near-identical repeated spectra, missing hum below ~100 Hz on a heavy
  ring, and exact equal temperament. Do not geolocate on these.
- **Multiple sources**: allow the matcher to explain the nominal set as two subsets (ring + clock, or
  two rings) before declaring a single-tower match.
- **Add a codec test harness**: encode a synthetic or clean bell clip through LAME 64/128, Opus 16/32/96,
  HE-AAC 48, and WhatsApp-like Opus 16 kHz, and measure nominal drift in cents. Put the results in this file.
