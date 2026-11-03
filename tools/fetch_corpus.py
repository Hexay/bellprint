"""Download the real-recording benchmark corpus listed in data/corpus/manifest.toml.

Each recording becomes data/corpus/audio/<id>.flac (mono, 44.1 kHz), trimmed to start_s/end_s when set.
Existing files are skipped. YouTube goes through yt-dlp; Freesound uses the public HQ preview (originals
need an API key); archive.org uses the first audio file (else video) listed in the item's metadata.

Run: python3 -I tools/fetch_corpus.py [--only id,id] [--manifest PATH] [--cookies cookies.txt]
--cookies is passed to yt-dlp for hosts that hit YouTube's "confirm you're not a bot" wall.
"""
import argparse
import json
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UA = {'User-Agent': 'Mozilla/5.0 (bellprint corpus fetcher)'}
ARCHIVE_MEDIA = ('.flac', '.wav', '.mp3', '.ogg', '.m4a', '.mp4', '.ogv')


def http_get(url: str) -> bytes:
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=60) as resp:
        return resp.read()


def freesound_preview_url(page_url: str) -> str:
    page = http_get(page_url).decode('utf-8', 'replace')
    m = re.search(r'<meta property="og:audio" content="([^"]+)"', page)
    if not m:
        raise RuntimeError('no og:audio preview on page')
    url = m[1][m[1].rfind('https://'):]
    return url.replace('-lq.mp3', '-hq.mp3')


def archive_file_url(page_url: str) -> str:
    ident = urllib.parse.urlparse(page_url).path.rstrip('/').split('/')[-1]
    meta = json.loads(http_get(f'https://archive.org/metadata/{ident}'))
    for ext in ARCHIVE_MEDIA:
        for f in meta.get('files', []):
            if f['name'].lower().endswith(ext):
                return f'https://archive.org/download/{ident}/{urllib.parse.quote(f["name"])}'
    raise RuntimeError('no audio/video file in archive item')


def download(rec: dict, workdir: Path, cookies: Path | None) -> Path:
    src = rec['source']
    # 'ytdlp' = any site yt-dlp supports (Vimeo, SoundCloud, ...); 'direct' = url is the media file itself.
    if src in ('youtube', 'ytdlp'):
        cmd = ['yt-dlp', '-q', '--no-warnings', '--no-playlist', '-f', 'bestaudio/best',
               '-o', str(workdir / 'src.%(ext)s')]
        if cookies:
            cmd += ['--cookies', str(cookies)]
        subprocess.run(cmd + [rec['url']], check=True, capture_output=True, text=True)
        return next(workdir.glob('src.*'))
    resolve = {'freesound': freesound_preview_url, 'archive': archive_file_url, 'direct': lambda u: u}
    url = resolve[src](rec['url'])
    dest = workdir / ('src' + Path(urllib.parse.urlparse(url).path).suffix)
    dest.write_bytes(http_get(url))
    return dest


def to_flac(src: Path, dest: Path, start: float | None, end: float | None) -> None:
    cmd = ['ffmpeg', '-nostdin', '-loglevel', 'error', '-y']
    if start is not None:
        cmd += ['-ss', str(start)]
    cmd += ['-i', str(src)]
    if end is not None:
        cmd += ['-t', str(end - (start or 0))]
    cmd += ['-vn', '-ac', '1', '-ar', '44100', '-c:a', 'flac', str(dest)]
    subprocess.run(cmd, check=True, capture_output=True, text=True)


def fetch_one(rec: dict, audio_dir: Path, cookies: Path | None) -> str:
    dest = audio_dir / f"{rec['id']}.flac"
    if dest.exists():
        return 'skip'
    with tempfile.TemporaryDirectory(prefix='bellprint-') as tmp:
        part = Path(tmp) / 'out.flac'
        to_flac(download(rec, Path(tmp), cookies), part, rec.get('start_s'), rec.get('end_s'))
        shutil.move(part, dest)
    return 'ok'


def error_line(exc: Exception) -> str:
    text = getattr(exc, 'stderr', None) or str(exc)
    return (text.strip().splitlines() or ['?'])[-1][:120]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument('--manifest', type=Path, default=ROOT / 'data/corpus/manifest.toml')
    ap.add_argument('--only', default='', help='comma-separated recording ids')
    ap.add_argument('--cookies', type=Path, help='Netscape cookies.txt for yt-dlp (YouTube bot check)')
    args = ap.parse_args()

    recs = tomllib.loads(args.manifest.read_text(encoding='utf-8'))['recording']
    only = {s for s in args.only.split(',') if s}
    audio_dir = args.manifest.parent / 'audio'
    audio_dir.mkdir(parents=True, exist_ok=True)

    counts = {'ok': 0, 'skip': 0, 'FAIL': 0}
    for rec in recs:
        if only and rec['id'] not in only:
            continue
        try:
            status, detail = fetch_one(rec, audio_dir, args.cookies), ''
        except Exception as exc:  # one bad URL must not stop the batch
            status, detail = 'FAIL', error_line(exc)
        counts[status] += 1
        print(f"{status:4} {rec['id']:<28} {rec['source']:<9} {detail}", flush=True)
    print('summary: ' + ' '.join(f'{k}={v}' for k, v in counts.items()))
    return 1 if counts['FAIL'] else 0


if __name__ == '__main__':
    sys.exit(main())
