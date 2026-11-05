//! FLAC metadata blocks: a `LOCATION`, `GPS` or `GEOTAG` Vorbis comment.

use super::{MAX_TEXT, parse_text, read_at, read_u32};
use crate::geo::GeoPoint;
use std::io::{Read, Seek};

const MAX_BLOCKS: usize = 128;
const MAX_COMMENTS: u32 = 1024;
const VORBIS_COMMENT: u8 = 4;
const KEYS: [&str; 3] = ["LOCATION", "GPS", "GEOTAG"];

pub fn find<R: Read + Seek>(r: &mut R, len: u64) -> Option<GeoPoint> {
    let mut pos = 4;
    for _ in 0..MAX_BLOCKS {
        let h = read_u32(r, pos, len, true)?;
        let (last, kind, size) = (h >> 31 == 1, (h >> 24) as u8 & 0x7f, (h & 0x00ff_ffff) as u64);
        let end = pos + 4 + size;
        if end > len {
            return None;
        }
        if kind == VORBIS_COMMENT {
            return comments(r, pos + 4, end);
        }
        if last {
            return None;
        }
        pos = end;
    }
    None
}

/// Little-endian vendor string, comment count, then length-prefixed "KEY=value" comments.
fn comments<R: Read + Seek>(r: &mut R, start: u64, end: u64) -> Option<GeoPoint> {
    let vendor = read_u32(r, start, end, false)? as u64;
    let mut pos = start + 4 + vendor;
    let count = read_u32(r, pos, end, false)?.min(MAX_COMMENTS);
    pos += 4;
    for _ in 0..count {
        let len = read_u32(r, pos, end, false)? as u64;
        let next = pos + 4 + len;
        if next > end {
            return None;
        }
        if len <= MAX_TEXT {
            let text = read_at(r, pos + 4, len, end)?;
            let hit = std::str::from_utf8(&text).ok().and_then(|c| c.split_once('=')).and_then(|(k, v)| {
                KEYS.iter().any(|key| k.eq_ignore_ascii_case(key)).then(|| parse_text(v)).flatten()
            });
            if hit.is_some() {
                return hit;
            }
        }
        pos = next;
    }
    None
}
