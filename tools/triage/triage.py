"""Per-miss evidence from locate_misses.sh dumps; writes <out>/triage_rows.json and prints a row per miss.
usage: python3 -I triage.py <repo> <out>/loc <cases.json>"""
import csv
import json
import math
import sys
import tomllib
from pathlib import Path

repo, loc, cases_path = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
entries = {}
for m in sorted((repo / 'data/corpus').glob('manifest*.toml')):
    for r in tomllib.loads(m.read_text())['recording']:
        entries.setdefault(r['id'], r)
rings = {}
with open(repo / 'data/dove/bells.csv', encoding='utf-8-sig') as f:
    for r in csv.DictReader(f):
        if r['Bell Role'][:1].isdigit():
            rings.setdefault(int(r['Ring ID']), []).append(r)


def nominals(ring_id):
    rows = rings.get(ring_id, [])
    return sorted((float(r['Nominal (Hz)']) for r in rows if r['Nominal (Hz)'].strip()), reverse=True), rows


c = lambda hz: 1200 * math.log2(hz)
KNOWN = [0, 70.7, -70.7, 146.7, -146.7, 217.4, -217.4]
NULL_OFFSETS = [-650, -450, 450, 650]


def present(noms, lines, shift, tol=8.0):
    return sum(any(abs(l - n - shift) <= tol for l in lines) for n in noms)


def best_present(noms, lines, shifts):
    return max(((present(noms, lines, s), -abs(s), s) for s in shifts))


rows = []
for case in json.loads(cases_path.read_text()):
    if case['rank'] == 1:
        continue
    e = entries[case['id']]
    rep = json.loads((loc / f"{case['id']}.json").read_text())
    lo, hi = rep['band_hz']
    lines = [c(p['hz']) for p in rep['peaks']]
    hz, brows = nominals(e['ring_id'])
    inband = [c(h) for h in hz if lo <= h <= hi]
    n = len(inband)
    near = [s / 2 for s in range(-20, 21)]
    far = [s / 2 for s in range(-520, 521)]
    pn, _, sn = best_present(inband, lines, near) if n else (0, 0, 0)
    pf, _, sf = best_present(inband, lines, far) if n else (0, 0, 0)
    null_n = sum(best_present([x + o for x in inband], lines, near)[0] for o in NULL_OFFSETS) / 4 if n else 0
    null_f = sum(best_present([x + o for x in inband], lines, far)[0] for o in NULL_OFFSETS) / 4 if n else 0
    ms = rep['matches']
    true = next((m for m in ms if m['ring']['tower_id'] == e['tower_id']), None)
    top = ms[0]
    tshift = true['shift_cents'] if true else sn
    wn = [c(h) + top['shift_cents'] for h in top['ring']['nominals']]
    shared = sum(any(abs(w - (x + tshift)) <= 12 for w in wn) for x in inband) if n else 0
    years = [int(r['Cast Date'][:4]) for r in brows if r['Cast Date'][:4].isdigit()]
    rows.append({
        'id': case['id'], 'rank': case['rank'], 'ringing': e.get('ringing'), 'conf': e.get('confidence'),
        'notes': e.get('notes', ''), 'source': e.get('source'), 'bells': e['bells'], 'dove_noms': len(hz),
        'inband': n, 'pn': pn, 'sn': sn, 'null_n': null_n, 'pf': pf, 'sf': sf, 'null_f': null_f,
        'peaks': len(lines), 'winner': top['ring']['place'], 'w_bells': len(top['ring']['nominals']),
        'w_shift': top['shift_cents'], 'margin': top['margin'], 'shared': shared,
        'gap': (top['score'] - true['score']) if true else None, 't_shift': true['shift_cents'] if true else None,
        't_heard': true['bells_heard'] if true else None, 't_rank400': ms.index(true) + 1 if true else None,
        'max_year': max(years) if years else None, 'title': e.get('title', '')[:60],
    })
json.dump(rows, open(loc.parent / 'triage_rows.json', 'w'), indent=1)
for r in rows:
    print(f"{r['id'][:34]:34} rk={r['rank'] or '-':>3} r400={r['t_rank400'] or '-':>4} {r['ringing'][:6]:6} {r['conf'][:3]} "
          f"n={r['inband']:2}/{r['dove_noms']:2} near={r['pn']:2}@{r['sn']:+5.1f} null={r['null_n']:4.1f} "
          f"far={r['pf']:2}@{r['sf']:+6.1f} nf={r['null_f']:4.1f} pk={r['peaks']:3} shared={r['shared']:2} "
          f"w={r['w_bells']:2}@{r['w_shift']:+5.1f} gap={r['gap'] if r['gap'] is None else round(r['gap'], 1)} "
          f"ts={r['t_shift']} yr={r['max_year']} | {r['notes'][:50]}")
