# Data sources for bellprint (beyond and within Dove)

Researched 2026-11-03. **V** = verified this session (page fetched or local file read). **U** = unverified or inferred.
All web content was treated as untrusted. Local CSVs were read with `python3 -I`.

## Summary

| Source | Per-bell Hz / partials | Key fields | Licence / terms | Access |
|---|---|---|---|---|
| Dove `bells.csv` (local) | **Nominal Hz** on 46,983 of 64,136 bells. No other partials | weight, note, diameter, founder, cast date, turnings, canons | CC BY-SA 4.0 (V) | CSV download (V) |
| Dove `dove.csv` (local) | Tenor Hz only (6,398 of 7,262 rings) | Wt, Note, TuneYr, Contractor, Lat/Long, UR, Practice | CC BY-SA 4.0 (V) | CSV download (V) |
| Dove `towers.csv`, `frames.csv`, `founders.csv`, `changes.csv` | no | all collections (chimes, carillons), frames, founders, changelog since 2019 | CC BY-SA 4.0 (V) | CSV download (V). **Not yet local** |
| Erzbistum Köln Glockenbücher (G. Hoffs) | **Yes: Schlagton plus about 13 partials**, in 1/16-semitone offsets | founder, year, diameter, rim thickness, weight, rib | "(Ausdruck gestattet)", printing allowed. **No open licence** (V) | about 40 PDFs, one per deanery (V) |
| Limburger Glockenbuch (2017, 1,197 pp) | U, probably yes (same genre) | U | Print publication (V: DDB record) | Book. No online data found |
| hibberts.co.uk (Bill Hibbert) | **Yes, cents tables** for hum, prime, tierce and nominal on articles about individual rings | per-ring articles. Recording archive statistics by founder | No licence stated (V). Default copyright | HTML articles only. Archive not downloadable (V) |
| TowerBells.org (Carl Zimmerman) | U, probably not per-bell (bourdon note and weight) | carillons and chimes worldwide, foundries | U (no licence page found) | HTML only. Broken TLS cert (V) |
| GCNA tower directory | no (V: none stated) | location, short description | U | PDF, last updated June 2014 (V) |
| CSBI (formerly British Carillon Society) | no (V) | name and town of 15 carillons | none stated | HTML |
| French POP / Palissy | **no note** (V, sample notice) | founder, year, location, protection status | **Etalab 2.0** (open, attribution) (V) | open data on data.gouv.fr (V: site statement) |
| Belgian (Campano, Flemish inventory 2011) | U | inventory sheet template, carillon counts | U | PDF. Sheet URL returned 404 |
| Dutch (NKV, "Klokkenbestand") | none found | — | — | No national online register found (V: searches) |
| Swiss (cantonal Denkmalpflege, e.g. AG) | not in hits (U) | date, weight, diameter, inscription | U | per-canton web search |
| Austria (glockenarchiv.at) | U | bell count per site, "Gussdaten", audio and video | none shown (V) | HTML |
| createsoundscape (DE, Beratungsausschuss) | no (U) | **audio and video of over 5,000 church bell sets**, history | U | web map |
| BellBoard | no | performance, date, duration, **dove-tower-id**, dove-ring-id, tenor, methods | © Ringing World. No API terms (V) | **XML API** (V) |
| Felstead DB | no | per tower: date and method of each peal (V) | none stated | HTML only. No bulk export found |
| A Church Near You | no | service times and events (U from site purpose) | **All rights reserved. No copying or reproduction** (V) | No API found |

## (a) Dove fields: local fill rates (V)

`dove.csv`: 7,262 rows, full-circle rings only. `RingType` is "Full-circle ring" for 7,124 and "Lightweight ring" for 138. By country: England 6,779, Wales 230, Australia 74, USA 50.

| Field | Non-empty | % | Use |
|---|---|---|---|
| Bells | 7,262 | 100 | Hard filter: the number of distinct strike pitches heard must be ≤ Bells, and usually equal |
| Wt (lb, tenor) | 7,260 | 100 | Fallback prior on tenor pitch when Hz is missing (weight scales roughly with f⁻³) |
| Note | 7,166 | 98.7 | Coarse tenor pitch class when Hz is missing. Uses `♭` and `#` glyphs |
| Hz (tenor) | 6,398 | 88.1 | Primary match key |
| Lat/Long | 7,141 | 98.3 | Location prior when the user gives a location (also SNLat/SNLong on 639 rows) |
| OvhaulYr | 4,714 | 64.9 | Recency of the data. Pairs with TuneYr |
| Contractor | 4,500 | 62.0 | Taylor 1,601, Whitechapel 436, Whites 297 |
| Practice | 3,473 | 47.8 | Weekday string such as "Wed". **Time-of-week prior** |
| TuneYr | 1,267 | 17.4 | Year of the last retune. A pre-retune recording will not match the current Hz |
| UR (`u/r`) | 964 | 13.3 | Unringable: **strong negative prior** for full-circle audio |
| Simulator (`T`) | 262 | 3.6 | Silent practice possible. Weak signal |
| Semitones | 115 | 1.6 | Ring contains a sharp or flat bell ("2#", "6b"). **Affects diatonic-scale templates** |
| GF / Toilet / App | 2,625 / 2,592 / 1,630 | — | Not useful |

`bells.csv`: 64,136 rows. By `Collection Type`: full-circle ring 45,834, chime 12,100, other 2,229, tubular chime 1,161, lightweight 1,111, plus carillon and clock chime.

| Field | Non-empty | % | Use |
|---|---|---|---|
| Nominal (Hz) | 46,983 | 73.3 | **Per-bell match key.** 9,430 rings have at least one nominal. By collection: full-circle 40,047, chime 4,091, carillon 595, clock chime 282, tubular 243 |
| Note | 55,244 | 86.1 | Pitch class for bells without Hz |
| Weight (lbs) / (approx) | 54,506 / 9,041 | 85 | Size prior. Flags estimated weights |
| Diameter (in) | 59,769 | 93.2 | With weight, gives a rough check on expected nominal |
| Cast Date | 63,020 | 98.3 | Free text such as "c1599". **Proxy for tuning style**: pre-1890s untuned or old-style, then Simpson "true-harmonic" tuning |
| Founder | 62,417 | 97.3 | Taylor 12,875, Mears & Stainbank 6,017, Warner 3,647, G&J 2,736. **Partial-profile prior** (hum and prime offsets differ by founder and era) |
| Turnings | 29,054 | 45.3 | "unturned", "quarter", and so on. Small changes to partials |
| Canons | 52,714 | 82.2 | Not useful |
| Cracked | 798 | 1.2 | "cracked" 335, "welded" 463. Cracked bells sound anomalous: down-weight |
| Shape / Material | 64,136 | 100 | Tubular (1,161), hemispherical (249), steel (232) and ceramic (80) have **different partial structures**: exclude them from the church-bell partial model |
| Bell Role | 64,136 | 100 | Bell number within the ring (1 = treble) |
| Frame ID | 44,485 | 69.4 | Join to `frames.csv` |

Other Dove files on https://dove.cccbr.org.uk/downloads (V): `towers.csv` (all collection types), `founders.csv`, `frames.csv`, `regions.csv`, and `changes.csv` (updates since 2019-01-01, so bellprint can track changes incrementally). The page also says automated fetching is OK but the number and order of columns may change, while column headings stay stable. Read the CSVs by header name, never by position.

## (b) Whiting Society: https://www.whitingsociety.org.uk/ (V)

Offers more than 300 old ringing books as free downloads ("mostly out of copyright", no explicit licence), articles, methods, and a Gorbals (Glasgow) foundry output list. **No tower lists and no tuning or frequency data.** Footer: "© The Whiting Society of Ringers, 2013". Not relevant to matching.

## (c) Bill Hibbert: https://www.hibberts.co.uk/ (V; TLS cert expired, fetched with curl -k)

- Category "Individual sets of bells" (`/category/individual-sets-of-bells/`) has per-ring articles such as Merthyr Tydfil, Poynton, Westminster Abbey tenor, two van Aerschodt carillons, and "Strike notes and tuning of old-style bells" (31 historic rings of 8 and 10). These contain **tables of partials in cents relative to the tenor nominal**. Example (Merthyr Tydfil, 1893 tuning, tenor nominal 710 Hz): one row per bell with Hum, Prime, Tierce and Nominal, e.g. `treble -2332 -1369 – 1222`.
- `/bell-recording-collection/` shows only **statistics** of his recording archive by founder and half-century (Taylor 2,830, Whitechapel 2,487, Gillett 1,769, and more; over 10k bells in total, U: inferred by summing the table). **The recordings and analyses are not downloadable.**
- `/gillett-johnston-tuning-book-entries/` documents the G&J tuning-book format (bell number, tower, diameter, soundbow thickness, weights, partials). These are scans of the archive, not a dataset.
- WAVANAL is his partial-analysis software. The PhD thesis is on ORO: https://oro.open.ac.uk/44498. It is based on about 1,800 to 2,000 bells and gives a model of strike pitch from the upper partials (V: search snippets).
- No licence is stated, so default copyright applies. Use as **validation ground truth** (a few dozen rings) and for partial-ratio models. Do not bulk-ingest. Contact the author for archive access.

## (d) Carillons and chimes

- **TowerBells.org** (http://www.towerbells.org/data/Data_Top.html, V): a huge static HTML site with indexes by region and type (e.g. `data/IXNAcoll.html`) and a bellfoundries section. The TLS cert is for securedata.net, so it must be fetched over plain http. Per-site pages typically give bourdon note, weight, founder and bell count (U: not opened). **No per-bell Hz seen. No licence found.** A Mechelen thesis plans to migrate it to a modern database (V: title only; the page returned 401).
- **GCNA** (https://gcna.org/tower-directories, V): a PDF directory of North American carillons, last updated June 2014, with location and short description. It points to the World Carillon Federation (bell count and players) and to TowerBells.
- **CSBI** (formerly British Carillon Society; https://carillonsbi.org/?p=34, V): lists 15 carillons by name and town only.
- Dove `bells.csv` already holds 595 carillon nominals and 4,091 chime nominals (V). For the British Isles that is more useful than any of the above.

## (e) Continental European bell inventories

- **Erzbistum Köln Glockenbücher**: https://thema.erzbistum-koeln.de/glockenbuch/glockenbuecher/NN_*.pdf. There are about 40 deanery PDFs. Bonn alone is 308 pages, "bearbeitet von Gerhard Hoffs", "(Ausdruck gestattet)" (V).
  - **Confirmed** per-bell table: Glockengießer, Gußjahr, Metall, Durchmesser (mm), Schlagringstärke (mm), Proportion, Gewicht (kg), Konstruktion (Rippe), then **Schlagton/Nominal, Nominalquarte, Unteroktav-Vertreter (hum), Prim-Vertreter, Terz, Quint-Vertreter, Oktave, Dezime, Undezime, Duodezime, Tredezime, Quattuordezime, Doppeloktav-Vertreter**.
  - Notation is **note plus sixteenth-semitone offset**, e.g. `g'-7`, `b'-1`, `c''±o`. The book includes a "Tabelle der Sechzehntelwerte plus Schwingungszahl": `c'±o = 258,6 Hz`, which implies **a' = 435 Hz** (V by arithmetic: 258.6 × 2^(9/12) ≈ 434.9). Conversion: f = f(note @ a'=435) × 2^(n/192).
  - This is the richest per-bell partial data found anywhere. Licence: none beyond permission to print. CC BY-SA compatibility is **unknown**, so ask before redistributing derived data.
- **Limburger Glockenbuch** (Foersch and Nickel, Bischöfliches Ordinariat Limburg, 2017, 1,197 pp; V: Deutsche Digitale Bibliothek record). Print only. Partial content is U.
- **Other German diocesan books** use the same Schlagton notation, e.g. a museum-digital entry reading "Schlagton: f''+6" (V: snippet). Oldenburg Glockenatlas 2022 (PDF returned 403, U). **Deutsches Glockenmuseum** (glockenmuseum.de): no online database found.
- **createsoundscape**: audio and video of over 5,000 church bell sets in a web map, run by the Beratungsausschuss für das Deutsche Glockenwesen (V: evangelisch.de 2024 article). Possibly useful as a **test-audio corpus**. Licence U.
- **France: POP/Palissy** (https://pop.culture.gouv.fr): about 7,436 protected bells (V: ministry figure). The sample notice PM33002985 has founder, year, location and status, but **no note and no dimensions** (V). **Etalab 2.0** licence (open, attribution only), with open data on data.gouv.fr (V). The French Inventaire général "IM" notices are similar.
- **Belgium**: Campano (Wallonia) publishes a bell inventory sheet template (PDF returned 404, U). There is also a Flemish carillon inventory from 2011 (PDF report) and a list of carillons from the Mechelen school. No per-bell database found.
- **Netherlands**: no "Nationaal Klokkenbestand" or NKV online database found (V: three searches). Local open-day folders list founder and slagtoon, e.g. "ais1".
- **Switzerland**: cantonal heritage inventories (e.g. ag.ch/denkmalpflege) give date, weight, diameter and inscription. BABS publishes a model inventory form. No Schlagton database found.
- **Austria**: https://www.glockenarchiv.at lists sites with bell counts, plus "Gussdaten" and audio and video. The operator, per-bell fields and terms are not stated (V). There is also glockeninstitut.at, and the diocese of St. Pölten keeps a register of 2,681 bells (V: snippet).

## (f) Peal records

- **BellBoard** (https://bb.ringingworld.co.uk; API help at `/help/api.php`, V): XML via `&fmt=xml` or `Accept: application/xml`. Endpoints: `view.php?id=N`, `search.php` (POST; `changed_since=YYYY-MM-DD`), `export.php` (GET, full details), `list.php?last_changed`. Page size is limited (HTTP 413 above 10,000). XML elements: `place` with `dove-tower-id` and `towerbase-id`, `ring` with `dove-ring-id` and `tenor`, `date`, `duration`, `title`, `method`, `changes`, `ringers`, `footnote` and `timestamp`. **No start time of day** in the XML (V, one record). Footnotes sometimes mention occasions (U). Terms: "Copyright © 2012–26, The Ringing World", no API licence, so ask bellboard@ringingworld.co.uk before bulk mirroring.
- **Felstead** (https://felstead.cccbr.org.uk): for each tower, the date and method of every tower-bell peal (more than 300k peals, U). HTML only. No export or licence found (V). Run by the CCCBR (ICT and Peal Records committees).
- **PealBase**: paid export of an individual's own peals (V: snippet). Not useful.

## (g) Service times

- **A Church Near You** (https://www.achurchnearyou.com/terms/, V): operated by the Archbishops' Council. Its terms say "All such rights are reserved" and prohibit you "to reproduce, duplicate, copy or re-sell any part of our site". **No API and no data licence. Scraping is not permitted** without consent. Contact the CofE digital team for a data agreement.

## Verified vs inferred (key points)

- V: Dove CC BY-SA 4.0. V: the extra Dove CSVs exist. V: Köln partial tables in 1/16-semitone offsets at a' = 435. V: BellBoard XML with Dove IDs and no time of day. V: ACNY all rights reserved. V: POP is Etalab 2.0 and has no note field in the sample.
- U: TowerBells per-site fields. U: Limburg partial content. U: Austrian archive fields. U: createsoundscape licence. U: the size of Hibbert's archive (summed from his table).

## Implications for bellprint

- **Ingest Dove only for v1.** Use `bells.csv` Nominal (Hz) as the per-bell key and `dove.csv` Hz as the tenor fallback. Also download `towers.csv` (chimes and carillons), `founders.csv` and `changes.csv`. Key on column headers, not positions.
- **Share-alike obligation**: any distributed index or table derived from Dove must be CC BY-SA 4.0. Credit "Dove's Guide, CCCBR", link the licence, state changes, and do not imply endorsement. Code can stay under its own licence. Keep the data artefact separate.
- **Priors to use**:
  - `Bells`: hard limit on the number of distinct pitches.
  - `UR = u/r`: strong down-weight.
  - `RingType`/`Collection Type`: matches the audio mode (rounds and changes vs chime tunes).
  - `Lat/Long`: optional location prior.
  - `Practice`: weekday prior if a timestamp is given.
  - `TuneYr` and `Cast Date`: tolerance widening (pre-retune recordings; untuned or old-style bells).
  - `Semitones`: non-diatonic templates.
  - `Founder` + `Cast Date`: expected hum and prime offsets.
  - `Cracked`: down-weight.
  - `Shape`/`Material`: exclude non-standard partial models.
- **Missing Hz**: back off to `Note` plus `Wt`/`Diameter` with a wide tolerance (about ±50 cents for Note). Do not treat a missing value as a mismatch.
- **Validation**: hand-transcribe a few dozen Hibbert cents tables (published only, used as test fixtures, not redistributed) to test partial extraction and the strike-pitch model.
- **Continental extension (later)**: parse the Köln Glockenbuch PDFs (pypdf text works) into per-bell partials using `f = f_note(a'=435) × 2^(n/192)`. **Get permission from the Erzbistum Köln and Hoffs first**, because the licence is not open and must not be mixed into the CC BY-SA bundle without clearance. POP/Palissy (Etalab) is open but has no pitch, so it adds location metadata only.
- **Skip**: Whiting (no data), GCNA, CSBI, Dutch sources (no pitch), ACNY (all rights reserved; time-of-day priors would need a licensed source), Felstead (no export).
- **BellBoard**: optional activity prior (which towers ring, and how often, by date) via the XML `export.php` and `changed_since`, joined on `dove-tower-id`. It has no time of day. Ask The Ringing World before bulk mirroring and rate-limit requests.
- **Test audio**: candidates are the CCCBR Library of Bell Recordings (bellrecordings.cccbr.org.uk) and createsoundscape. Check the licences before use (U).
