# bellprint research index

Web research gathered 2026-11. Each file tags claims as verified (with source) or inference, and ends with
"Implications for bellprint".

| File | Covers |
|---|---|
| [acoustics.md](acoustics.md) | Partial ratios and cents windows (true-harmonic vs old-style), doublets, decay, strike note, temperature, Dove measurement accuracy, retuning |
| [decay.md](decay.md) | Measured per-partial decay (hum, prime, tierce, nominal), clapper damping, the sim's decay model |
| [extraction.md](extraction.md) | Onset detection, per-blow nominal estimation, Wavanal, HawkEar and other striking tools, inharmonic/high-resolution pitch methods, blow-timing numbers |
| [extraction_ab.md](extraction_ab.md) | Measured A/B of line extraction on sim and corpus: prominence, aggregation, sim partial decay |
| [data_sources.md](data_sources.md) | Dove fields and fill rates, Hibbert data, carillon DBs, German Glockenbücher, Dutch/French inventories, BellBoard/Felstead, service times, licences |
| [prior_work.md](prior_work.md) | Bells as OSINT/forensic evidence, the viral Derbyshire claim (unverified), ENF forensics, audio geolocation, MIR bell papers |
| [matcher.md](matcher.md) | Why real recordings missed (bigger rings claiming partial lines), partial spreads measured on the corpus, ring subsets, negative results |
| [matcher_twins.md](matcher_twins.md) | Round 4: near-twin misses, the narrow-core shift prior, margin vs hit rate, tighter nominal σ (rejected) |
| [perf.md](perf.md) | Matcher/eval profile and bit-identical speed-ups (sim eval 112 → 15 s, corpus 130 → 21 s) |
| [pitfalls.md](pitfalls.md) | Codec cutoffs, AGC/noise suppression, speed/pitch alteration, ringing chamber vs outside, multiple towers, chimes/clocks, electronic bells |

Known tension: pitfalls.md says to lean on hum/prime/tierce; acoustics.md shows hum/prime deviate
60–155 c in old-style bells. Resolve by using era-dependent windows (acoustics.md) for those partials.
