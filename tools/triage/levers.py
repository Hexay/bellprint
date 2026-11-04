"""Per matcher variant: misses that become top-1 (by triage class) and hits lost.
usage: python3 -I levers.py <out> <baseline cases.json>, with each variant's `replay --out` at <out>/rep/<name>/corpus"""
import json
import sys
from collections import Counter
from pathlib import Path

t = Path(sys.argv[1])
base = {c['id']: c['rank'] for c in json.loads(Path(sys.argv[2]).read_text())}
cls, k = {}, None
for line in (t / 'classes.txt').read_text().splitlines():
    if line.startswith('## '):
        k = line[3:].split(':')[0].split()[0]
    elif line.strip():
        cls[line.split()[0]] = k
for d in sorted((t / 'rep').iterdir()):
    new = {c['id']: c['rank'] for c in json.loads((d / 'corpus/cases.json').read_text())}
    gained = [i for i in new if new[i] == 1 and base[i] != 1]
    lost = [i for i in new if new[i] != 1 and base[i] == 1]
    by = Counter(cls.get(i, '?') for i in gained)
    print(f"{d.name:14} +{len(gained)} -{len(lost)} gained {dict(by)} {gained} lost {lost}")
