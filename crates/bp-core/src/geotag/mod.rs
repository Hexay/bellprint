//! GPS position embedded in the input file's own metadata, read only when asked (`--use-geotag`).
//! The file is untrusted: every read is bounds-checked against its parent and capped in size.

mod flac;
mod iso6709;
mod mp4;

use crate::geo::{GeoPoint, Location, LocationSource, Region};
use anyhow::Context;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom};
use std::path::Path;

pub use iso6709::parse as parse_iso6709;

/// Longest text field read; locations are a few dozen bytes.
const MAX_TEXT: u64 = 4096;

pub fn read_geotag(path: &Path) -> anyhow::Result<Option<GeoPoint>> {
    let f = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let len = f.metadata()?.len();
    Ok(from_reader(&mut BufReader::new(f), len))
}

pub fn from_bytes(bytes: &[u8]) -> Option<GeoPoint> {
    from_reader(&mut Cursor::new(bytes), bytes.len() as u64)
}

/// MP4/MOV/M4A/3GP (`©xyz`, Apple location key) or FLAC (Vorbis comment); `None` for anything else.
pub fn from_reader<R: Read + Seek>(r: &mut R, len: u64) -> Option<GeoPoint> {
    let head = read_at(r, 0, 8, len)?;
    if head.starts_with(b"fLaC") {
        flac::find(r, len)
    } else if mp4::is_mp4(&head) {
        mp4::find(r, len)
    } else {
        None
    }
}

/// A free-text location: ISO 6709, or decimal "lat,lon" / "lat lon".
pub fn parse_text(s: &str) -> Option<GeoPoint> {
    let s = s.trim_matches(|c: char| c.is_whitespace() || c == '\0');
    iso6709::parse(s).or_else(|| {
        let v: Vec<f64> = s.split([',', ' ']).filter(|x| !x.is_empty()).map(str::parse).collect::<Result<_, _>>().ok()?;
        match v.as_slice() {
            [lat, lon] => GeoPoint::new(*lat, *lon),
            _ => None,
        }
    })
}

/// The search area: `near` wins; otherwise, with `use_geotag`, a default-radius circle around the
/// file's geotag. With `use_geotag` the tag is read and reported even when `near` overrides it.
pub fn resolve_location(path: &Path, near: Option<&str>, use_geotag: bool) -> anyhow::Result<Location> {
    let geotag = if use_geotag { read_geotag(path)? } else { None };
    let fallback = if near.is_some() { "" } else { "; searching all towers" };
    let warning = (use_geotag && geotag.is_none())
        .then(|| format!("no geotag found in {}{fallback}", path.display()));
    let (region, location_source) = match (near, geotag) {
        (Some(n), _) => (Some(Region::parse(n)?), Some(LocationSource::Near)),
        (None, Some(p)) => (Some(Region::around(p)), Some(LocationSource::Geotag)),
        (None, None) => (None, None),
    };
    Ok(Location { region, location_source, geotag, warning })
}

/// `n` bytes at `pos`, only if they lie before `end` and `n <= MAX_TEXT`.
fn read_at<R: Read + Seek>(r: &mut R, pos: u64, n: u64, end: u64) -> Option<Vec<u8>> {
    if n > MAX_TEXT || pos.checked_add(n)? > end {
        return None;
    }
    r.seek(SeekFrom::Start(pos)).ok()?;
    let mut buf = vec![0; n as usize];
    r.read_exact(&mut buf).ok()?;
    Some(buf)
}

fn read_u32<R: Read + Seek>(r: &mut R, pos: u64, end: u64, big_endian: bool) -> Option<u32> {
    let b: [u8; 4] = read_at(r, pos, 4, end)?.try_into().ok()?;
    Some(if big_endian { u32::from_be_bytes(b) } else { u32::from_le_bytes(b) })
}
