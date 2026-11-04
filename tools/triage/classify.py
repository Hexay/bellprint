"""Assign each miss in triage_rows.json one class (rules: research/miss_triage.md).
usage: python3 -I classify.py <out>/triage_rows.json > <out>/classes.txt"""
import json
import math
import sys
from collections import defaultdict

rows = json.load(open(sys.argv[1]))
KNOWN = [(0.0, 0.70), (70.7, 0.05), (-70.7, 0.05), (146.7, 0.06), (-146.7, 0.06), (217.4, 0.04), (-217.4, 0.04)]


def prior(s, c=0.7, sc=6.0, sb=15.0):
    best = -math.inf
    for mu, w in KNOWN:
        d = s - mu
        if abs(d) <= 40:
            v = c * (sb / sc) * math.exp(-0.5 * (d / sc) ** 2) + (1 - c) * math.exp(-0.5 * (d / sb) ** 2)
            best = max(best, math.log(w) + math.log(v))
    return best


P0 = prior(0.0)
# Rings the manifest notes or Dove dates show changed after the recording, or recorded from elsewhere.
CHANGED = {'bbc-oxford-magdalen', 'bbc-st-albans-erin', 'bbc-pembroke', 'bbc-rochdale-st-chad',
           'bbc-canterbury-grandsire-cinques', 'oxford-university-church'}
PARTIAL = {'sc-wellington-cathedral', 'perth-swan-bells', 'dublin-christ-church'}
out = defaultdict(list)
for r in rows:
    n = r['inband']
    net = r['pn'] - r['null_n']
    share = r['shared'] / n if n else 0
    r['net'], r['share'] = net, share
    if r['t_shift'] is not None:
        r['shift_cost'] = (P0 - prior(r['t_shift'])) - (P0 - prior(r['w_shift']))
    else:
        r['shift_cost'] = None
    if r['dove_noms'] <= 1:
        k = 'A1 no Dove nominals'
    elif r['ringing'] == 'chiming' or 'clock' in r['notes'].lower() or 'chime' in r['notes'].lower():
        k = 'A2 chime/clock'
    elif r['id'] in CHANGED:
        k = 'A3 ring changed / wrong tower'
    elif r['id'] in PARTIAL:
        k = 'B2 partial ring beyond subset rule'
    elif r['t_shift'] is not None and min(abs(r['t_shift'] - mu) for mu, _ in KNOWN) >= 12:
        k = 'C shift prior'
    elif net < 0.4 * n:
        k = 'B1 too little signal'
    elif share >= 0.75:
        k = 'D near twin'
    else:
        k = 'E other'
    out[k].append(r)
for k in sorted(out):
    print(f'## {k}: {len(out[k])}')
    for r in sorted(out[k], key=lambda r: r['rank'] or 99):
        g = '-' if r['gap'] is None else f"{r['gap']:.1f}"
        sc = '-' if r['shift_cost'] is None else f"{r['shift_cost']:.1f}"
        print(f"  {r['id'][:32]:32} rank {r['rank'] or '>50':>3} r400 {r['t_rank400'] or '-':>4} net {r['net']:4.1f}/{n if (n:=r['inband']) else 0:2} "
              f"share {r['share']:.2f} w{r['w_bells']:2} gap {g:>5} shiftcost {sc:>5} ts {r['t_shift']} ws {r['w_shift']} margin {r['margin']:.1f}")
