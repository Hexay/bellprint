"""True-ring fitted shift, as offset from the nearest known shift, for hits and ranked misses, and by source.
usage: python3 -I shifts.py <eval cases.json>   (true ring outside the reported 50 is left out)"""
import json
import statistics
import sys
from collections import defaultdict

KNOWN = (0.0, 70.7, -70.7, 146.7, -146.7, 217.4, -217.4)
BINS = [(0, 6), (6, 10), (10, 15), (15, 20), (20, 30), (30, 99)]
SOURCES = ("bbc", "ia", "sc", "fsg")

cases = [c for c in json.load(open(sys.argv[1])) if c.get("true_shift_cents") is not None]
for c in cases:
    c["off"] = min((c["true_shift_cents"] - k for k in KNOWN), key=abs)

for label, hit in (("hits", True), ("ranked misses", False)):
    xs = [c for c in cases if (c["rank"] == 1) == hit]
    hist = "  ".join(f"[{lo},{hi}) {sum(lo <= abs(c['off']) < hi for c in xs)}" for lo, hi in BINS)
    print(f"{label} (n {len(xs)}): {hist}")
    for c in sorted(xs, key=lambda c: -abs(c["off"])):
        if abs(c["off"]) >= 10:
            print(f"   {c['id']:40} off {c['off']:+6.2f}  gap {c['true_gap']:5.2f}  rank {c['rank']}")

by = defaultdict(list)
for c in cases:
    src = c["id"].split("-")[0]
    by[src if src in SOURCES else "other"].append(c["off"])
for k, v in sorted(by.items()):
    a = [abs(x) for x in v]
    print(f"{k:6} n {len(v):3}  median |off| {statistics.median(a):5.2f}  >=10 c {sum(x >= 10 for x in a)}")
