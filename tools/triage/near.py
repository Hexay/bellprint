"""Per triage class: misses whose true tower would rank first among candidates within R km of it (a --near hint).
usage: python3 -I near.py <triage dir> <baseline cases.json>"""
import json
import math
import sys
from collections import Counter
from pathlib import Path

t = Path(sys.argv[1])
tower = {c['id']: c['tower_id'] for c in json.loads(Path(sys.argv[2]).read_text())}
cls, k = {}, None
for line in (t / 'classes.txt').read_text().splitlines():
    if line.startswith('## '):
        k = line[3:].split()[0]
    elif line.strip():
        cls[line.split()[0]] = k


def km(a, b):
    return 6371 * math.hypot(math.radians(a['lat'] - b['lat']),
                             math.radians(a['lon'] - b['lon']) * math.cos(math.radians(a['lat'])))


tot, hit = Counter(), {r: Counter() for r in (10, 30)}
for f in sorted((t / 'loc').glob('*.json')):
    i = f.stem
    k = cls.get(i[:32], '?')
    tot[k] += 1
    ms = json.loads(f.read_text())['matches']
    pos = next((j for j, m in enumerate(ms) if m['ring']['tower_id'] == tower[i]), None)
    if pos is None:
        continue
    for r in (10, 30):
        if all(km(m['ring'], ms[pos]['ring']) > r for m in ms[:pos]):
            hit[r][k] += 1
for k in sorted(tot):
    print(k, tot[k], 'top-1 within 10 km:', hit[10][k], ' within 30 km:', hit[30][k])
