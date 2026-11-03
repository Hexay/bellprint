"""Why does a corpus recording miss? Compare its detected lines with the true ring's Dove nominals.

usage: python3 -I tools/diagnose.py <corpus id> [--bin path] [-- extra bellprint args]
Prints, at the shift that best aligns the true nominals, each nominal's nearest line (cents, prominence),
and where the true tower ranked.
"""

import csv
import json
import math
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def entry(rec_id: str) -> dict:
    for m in sorted((ROOT / 'data/corpus').glob('manifest*.toml')):
        for r in tomllib.loads(m.read_text())['recording']:
            if r['id'] == rec_id:
                return r
    sys.exit(f'no corpus entry {rec_id}')


def ring_nominals(ring_id: int) -> list[float]:
    with open(ROOT / 'data/dove/bells.csv', encoding='utf-8-sig') as f:
        rows = [r for r in csv.DictReader(f) if r['Ring ID'] == str(ring_id) and r['Nominal (Hz)'].strip()
                and r['Bell Role'][:1].isdigit()]
    return sorted((float(r['Nominal (Hz)']) for r in rows), reverse=True)


def cents(a: float, b: float) -> float:
    return 1200 * math.log2(a / b)


def main() -> None:
    args = sys.argv[1:]
    extra = args[args.index('--') + 1:] if '--' in args else []
    args = args[:args.index('--')] if '--' in args else args
    binary = args[args.index('--bin') + 1] if '--bin' in args else str(ROOT / 'target/fastrel/bellprint')
    rec = entry(args[0])
    audio = ROOT / 'data/corpus/audio' / f"{rec['id']}.flac"
    out = subprocess.run([binary, *extra, 'locate', str(audio), '--json', '--top', '300'], cwd=ROOT,
                         check=True, capture_output=True, text=True).stdout
    report = json.loads(out)
    peaks = report['peaks']
    noms = ring_nominals(rec['ring_id'])
    print(f"{rec['id']}: {rec['place']} ring {rec['ring_id']} ({len(noms)} nominals), {len(peaks)} lines, "
          f"windows {[(round(w['start_s']), round(w['end_s'])) for w in report['windows']]}")

    def misses(shift: float) -> float:
        return sum(min(abs(cents(p['hz'], n) - shift) for p in peaks) for n in noms)
    best = min(range(-250, 251), key=misses)
    print(f'best alignment shift {best:+d} c')
    for n in noms:
        p = min(peaks, key=lambda p: abs(cents(p['hz'], n) - best))
        print(f'  nominal {n:7.1f}  nearest line {p["hz"]:7.1f}  off {cents(p["hz"], n) - best:+6.1f} c  '
              f'prom {p["prominence_db"]:4.1f} dB')
    ranks = [i + 1 for i, m in enumerate(report['matches']) if m['ring']['tower_id'] == rec['tower_id']]
    top = report['matches'][0]
    print(f"true tower rank {ranks[0] if ranks else '>300'}; top pick {top['ring']['place']} "
          f"score {top['score']:.1f} shift {top['shift_cents']:+.0f}")
    if ranks:
        t = report['matches'][ranks[0] - 1]
        print(f"true score {t['score']:.1f} shift {t['shift_cents']:+.0f} heard {t['bells_heard']}")


if __name__ == '__main__':
    main()
