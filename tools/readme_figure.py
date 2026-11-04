"""Render docs/img/header.png, the README header: one real corpus clip, tenor-only vs whole-ring lookup.

Display only: matches, shift and windows come from `bellprint locate --json`; the spectrum is recomputed
here just to draw it. Run:
  uv run --no-project --with matplotlib --with numpy --with soundfile python3 -I tools/readme_figure.py
"""

import csv
import json
import os
import subprocess
import sys
import tomllib
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np
import soundfile as sf

ROOT = Path(__file__).resolve().parent.parent
CLIP = sys.argv[1] if len(sys.argv) > 1 else "sc-oxford-st-ebbe"
BIN = Path(os.environ.get("BELLPRINT_BIN", ROOT / "target/fastrel/bellprint"))
OUT = ROOT / "docs/img/header.png"

SURFACE, INK, INK2, MUTED = "#fcfcfb", "#0b0b0b", "#52514e", "#898781"
AXIS, CLOUD, BLUE, RED = "#c3c2b7", "#cfcdc6", "#2a78d6", "#e34948"
TENOR_TOL_HZ = 0.5
HEARD_CENTS = 9.4  # where the matcher's 8 c Gaussian falls to 0.5 (matcher/score.rs bell_terms)
BOUNDS = (-5.9, 1.9, 49.9, 55.9)  # lon/lat box: England and Wales
# Keep in step with README.md's Accuracy table.
FOOTER = (
    "Benchmarks (README, Accuracy): simulated, 200 random Dove rings: right tower first 51 %.  "
    "Real recordings of 141 named towers: right tower first for 18."
)

plt.rcParams.update({"font.family": ["Liberation Sans", "DejaVu Sans"], "text.color": INK})


def manifest_entry(clip):
    for path in sorted((ROOT / "data/corpus").glob("manifest*.toml")):
        with open(path, "rb") as f:
            for c in tomllib.load(f)["recording"]:
                if c["id"] == clip:
                    return c
    raise SystemExit(f"{clip}: not in any data/corpus/manifest*.toml")


def locate(audio):
    out = subprocess.run([str(BIN), "locate", str(audio), "--json", "--top", "5"], check=True,
                         capture_output=True, text=True).stdout
    return json.loads(out)


def dove_towers():
    """One row per tower: (tower_id, lat, lon, tenor Hz or None)."""
    towers = {}
    with open(ROOT / "data/dove/dove.csv", encoding="utf-8-sig", newline="") as f:
        for r in csv.DictReader(f):
            if not (r["Lat"] and r["Long"]):
                continue
            hz = float(r["Hz"]) if r["Hz"] else None
            prev = towers.get(r["TowerID"])
            towers[r["TowerID"]] = (float(r["Lat"]), float(r["Long"]), hz if hz else (prev[2] if prev else None),
                                    r["Place"], r["Dedicn"])
    return towers


def spectrum(audio, windows, win_s=0.7):
    """Mean power spectrum (dB) over the ringing windows, flattened by a moving-median floor."""
    info = sf.info(str(audio))
    n = int(win_s * info.samplerate)
    hann = np.hanning(n)
    acc, count = np.zeros(n // 2 + 1), 0
    for w in windows:
        x, _ = sf.read(str(audio), start=int(w["start_s"] * info.samplerate),
                       stop=int(w["end_s"] * info.samplerate), always_2d=True)
        x = x.mean(axis=1)
        for i in range(0, len(x) - n, n // 2):
            acc += np.abs(np.fft.rfft(x[i:i + n] * hann)) ** 2
            count += 1
    hz = np.fft.rfftfreq(n, 1 / info.samplerate)
    db = 10 * np.log10(acc / count + 1e-20)
    k = 61
    pad = np.pad(db, k // 2, mode="edge")
    floor = np.median(np.lib.stride_tricks.sliding_window_view(pad, k), axis=1)
    return hz, db - floor


def heard(nominals, shift, peaks):
    peak_c = 1200 * np.log2(np.array([p["hz"] for p in peaks]))
    return [np.min(np.abs(peak_c - (1200 * np.log2(n) + shift))) <= HEARD_CENTS for n in nominals]


def panel_head(fig, x, y, num, title, sub):
    fig.text(x, y, f"{num}  {title}", fontsize=21, color=INK)
    fig.text(x, y - 0.037, sub, fontsize=13.5, color=INK2)


def draw_spectrum(ax, hz, db, nominals, shift, is_heard):
    lo, hi = min(nominals) * 0.93, max(nominals) * 1.05
    sel = (hz >= lo) & (hz <= hi)
    ax.plot(hz[sel], db[sel], color=MUTED, lw=1.1, zorder=2)
    top, base = db[sel].max(), min(0, db[sel].min())
    tenor = min(nominals)
    for n, h in zip(nominals, is_heard):
        f = n * 2 ** (shift / 1200)
        is_tenor = n == tenor
        color = RED if is_tenor else (BLUE if h else AXIS)
        ax.vlines(f, base, top * 1.04, color=color, lw=2.2 if is_tenor else 1.4,
                  ls="-" if is_tenor else (0, (4, 3)), zorder=1)
        ax.text(f, top * 1.07, "✓" if h else "✗", ha="center", va="bottom", fontsize=14,
                color=color if h else MUTED)
    ax.text(tenor * 2 ** (shift / 1200), top * 1.2, f"tenor {tenor:.1f} Hz", color=RED, fontsize=14,
            ha="left", va="bottom")
    ax.set_xlim(lo, hi)
    ax.set_ylim(base, top * 1.3)
    ax.set_yticks([])
    for s in ("top", "right", "left"):
        ax.spines[s].set_visible(False)
    ax.spines["bottom"].set_color(AXIS)
    ax.tick_params(axis="x", colors=INK2, labelsize=12, length=0, pad=6)
    ax.set_xlabel("pitch (Hz)", color=INK2, fontsize=13)
    ax.set_facecolor(SURFACE)


def draw_map(ax, towers):
    lat = np.array([t[0] for t in towers.values()])
    lon = np.array([t[1] for t in towers.values()])
    ax.scatter(lon, lat, s=1.3, color=CLOUD, linewidths=0, zorder=1)
    ax.set_xlim(BOUNDS[0], BOUNDS[1])
    ax.set_ylim(BOUNDS[2], BOUNDS[3])
    ax.set_aspect(1 / np.cos(np.radians(54.5)))
    ax.axis("off")


def dot(ax, lon, lat, color, size=60, marker="o", z=3):
    ax.scatter(lon, lat, s=size, color=color, marker=marker, edgecolors=SURFACE, linewidths=1.6, zorder=z)


def main():
    entry = manifest_entry(CLIP)
    audio = ROOT / "data/corpus/audio" / f"{CLIP}.flac"
    report = locate(audio)
    matches = report["matches"]
    rank = next(i for i, m in enumerate(matches, 1) if m["ring"]["tower_id"] == entry["tower_id"])
    best = matches[rank - 1]
    ring, shift = best["ring"], best["shift_cents"]
    is_heard = heard(ring["nominals"], shift, report["peaks"])
    if sum(is_heard) != best["bells_heard"]:
        sys.exit(f"heard-bell rule disagrees with locate: {sum(is_heard)} vs {best['bells_heard']}")
    tenor = min(ring["nominals"])

    towers = dove_towers()
    in_box = {k: t for k, t in towers.items()
              if BOUNDS[0] <= t[1] <= BOUNDS[1] and BOUNDS[2] <= t[0] <= BOUNDS[3]}
    same_tenor = [t for t in towers.values() if t[2] is not None and abs(t[2] - tenor) <= TENOR_TOL_HZ]
    outside = [t for t in same_tenor if t not in in_box.values()]
    if outside:
        sys.exit(f"{len(outside)} same-tenor towers fall outside the map box")
    # locate analyses the whole clip unless --auto-window chose windows.
    windows = report["windows"] or [{"start_s": 0.0, "end_s": report["duration_s"]}]
    hz, db = spectrum(audio, windows)
    secs = sum(w["end_s"] - w["start_s"] for w in windows)

    fig = plt.figure(figsize=(17.6, 9.9), dpi=100, facecolor=SURFACE)
    fig.text(0.04, 0.905, "Which tower is that? Ask every bell, not just the biggest.", fontsize=33, color=INK)
    fig.text(0.04, 0.855, f"Dove's Guide gives the pitch of almost every bell in {len(towers):,} towers. "
             "bellprint matches the whole ring against it.", fontsize=17, color=INK2)

    place = f"{ring['place']}, {ring['dedication']}"
    panel_head(fig, 0.04, 0.745, "1", "Listen",
               f"{place}: {best['bells_heard']} of {ring['bells']} nominals found in a {secs:.0f} s clip.")
    fig.text(0.04, 0.672, "Lines: Dove's nominals for this ring.  Blue found, grey missed, red the tenor.",
             fontsize=12, color=MUTED)
    ax1 = fig.add_axes([0.04, 0.15, 0.31, 0.47])
    draw_spectrum(ax1, hz, db, ring["nominals"], shift, is_heard)

    panel_head(fig, 0.39, 0.745, "2", "The tenor alone",
               f"{len(same_tenor)} towers have a tenor within ±{TENOR_TOL_HZ} Hz of {tenor:.1f} Hz.")
    ax2 = fig.add_axes([0.38, 0.09, 0.28, 0.6])
    draw_map(ax2, in_box)
    for t in same_tenor:
        dot(ax2, t[1], t[0], BLUE)

    p = best["probability"] * 100
    panel_head(fig, 0.69, 0.745, "3", "The whole ring",
               f"Top 5 after matching all {ring['bells']} bells. Ranked #{rank}, p = {p:.1f} %.")
    ax3 = fig.add_axes([0.68, 0.09, 0.28, 0.6])
    draw_map(ax3, in_box)
    for m in matches:
        if m is not best:
            dot(ax3, m["ring"]["lon"], m["ring"]["lat"], BLUE)
    dot(ax3, ring["lon"], ring["lat"], RED, size=420, marker="*", z=4)
    ax3.annotate(f"{ring['place']},\n{ring['dedication']}", (ring["lon"], ring["lat"]), xytext=(-14, 10),
                 textcoords="offset points", color=RED, fontsize=15, va="bottom", ha="right")

    fig.text(0.04, 0.045, FOOTER, fontsize=13.5, color=INK2)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(OUT, dpi=200, facecolor=SURFACE)
    print(f"{OUT}: {CLIP} rank {rank} p {p:.1f}% heard {best['bells_heard']}/{ring['bells']} "
          f"tenor {tenor} Hz same-tenor {len(same_tenor)} top5 {[m['ring']['place'] for m in matches]}")


if __name__ == "__main__":
    main()
