//! ISO BMFF / QuickTime box walker for `moov/udta/©xyz` and the Apple `moov/meta` keys + ilst location.

use super::{MAX_TEXT, iso6709, read_at, read_u32};
use crate::geo::GeoPoint;
use std::io::{Read, Seek};

const MAX_DEPTH: usize = 4;
const MAX_CHILDREN: usize = 1024;
const MAX_KEYS: u32 = 1024;
const MAX_KEY_NAME: u64 = 256;
const XYZ: [u8; 4] = *b"\xa9xyz";
const APPLE_LOCATION_KEY: &[u8] = b"com.apple.quicktime.location.ISO6709";
const TOP_LEVEL: [&[u8; 4]; 8] = [b"ftyp", b"moov", b"mdat", b"free", b"skip", b"wide", b"pnot", b"uuid"];
const UTF8: u32 = 1;

#[derive(Clone, Copy)]
struct Atom {
    kind: [u8; 4],
    body: u64,
    end: u64,
}

pub fn is_mp4(head: &[u8]) -> bool {
    head.len() >= 8 && TOP_LEVEL.iter().any(|k| head[4..8] == k[..])
}

pub fn find<R: Read + Seek>(r: &mut R, len: u64) -> Option<GeoPoint> {
    let moov = children(r, 0, len).into_iter().find(|b| &b.kind == b"moov")?;
    search(r, moov, 0)
}

/// The box at `pos`, if its header and declared size fit before `limit`.
fn header<R: Read + Seek>(r: &mut R, pos: u64, limit: u64) -> Option<Atom> {
    let size32 = read_u32(r, pos, limit, true)?;
    let kind: [u8; 4] = read_at(r, pos + 4, 4, limit)?.try_into().ok()?;
    let (size, header_len) = match size32 {
        0 => (limit - pos, 8),
        1 => {
            let hi = read_u32(r, pos + 8, limit, true)? as u64;
            let lo = read_u32(r, pos + 12, limit, true)? as u64;
            (hi << 32 | lo, 16)
        }
        s => (s as u64, 8),
    };
    (size >= header_len && size <= limit - pos).then_some(Atom { kind, body: pos + header_len, end: pos + size })
}

/// Child boxes in `start..end`, stopping at the first malformed one.
fn children<R: Read + Seek>(r: &mut R, start: u64, end: u64) -> Vec<Atom> {
    let mut out = Vec::new();
    let mut pos = start;
    while out.len() < MAX_CHILDREN {
        let Some(b) = header(r, pos, end) else { break };
        out.push(b);
        pos = b.end;
    }
    out
}

fn search<R: Read + Seek>(r: &mut R, parent: Atom, depth: usize) -> Option<GeoPoint> {
    if depth > MAX_DEPTH {
        return None;
    }
    children(r, parent.body, parent.end).into_iter().find_map(|b| match &b.kind {
        &XYZ => user_data_text(r, b),
        b"udta" => search(r, b, depth + 1),
        b"meta" => meta(r, b),
        _ => None,
    })
}

/// QuickTime user-data text: u16 length, u16 language, then the string.
fn user_data_text<R: Read + Seek>(r: &mut R, b: Atom) -> Option<GeoPoint> {
    let len_lang = read_u32(r, b.body, b.end, true)?;
    let text = read_at(r, b.body + 4, (len_lang >> 16) as u64, b.end)?;
    iso6709::parse(std::str::from_utf8(&text).ok()?)
}

fn meta<R: Read + Seek>(r: &mut R, b: Atom) -> Option<GeoPoint> {
    // ISO `meta` is a FullBox (4 bytes of version/flags before its children); QuickTime's isn't,
    // and starts straight with its `hdlr` child.
    let quicktime = read_at(r, b.body + 4, 4, b.end).is_some_and(|k| k == b"hdlr");
    let body = if quicktime { b.body } else { b.body + 4 };
    let kids = children(r, body, b.end);
    let key = kids.iter().find(|k| &k.kind == b"keys").and_then(|k| location_key_index(r, *k));
    kids.into_iter().filter(|k| &k.kind == b"ilst").find_map(|ilst| ilst_location(r, ilst, key))
}

/// 1-based index of the Apple location key in a `keys` box.
fn location_key_index<R: Read + Seek>(r: &mut R, b: Atom) -> Option<u32> {
    let count = read_u32(r, b.body + 4, b.end, true)?.min(MAX_KEYS);
    let mut pos = b.body + 8;
    for i in 1..=count {
        let size = read_u32(r, pos, b.end, true)? as u64;
        if size < 8 || pos.checked_add(size)? > b.end {
            return None;
        }
        let name_len = size - 8;
        if name_len <= MAX_KEY_NAME && read_at(r, pos + 8, name_len, b.end)? == APPLE_LOCATION_KEY {
            return Some(i);
        }
        pos += size;
    }
    None
}

/// The ilst item named by `key` (or a literal `©xyz` item) holding UTF-8 `data`.
fn ilst_location<R: Read + Seek>(r: &mut R, ilst: Atom, key: Option<u32>) -> Option<GeoPoint> {
    children(r, ilst.body, ilst.end)
        .into_iter()
        .filter(|item| item.kind == XYZ || key.is_some_and(|k| item.kind == k.to_be_bytes()))
        .find_map(|item| {
            let data = children(r, item.body, item.end).into_iter().find(|d| &d.kind == b"data")?;
            if read_u32(r, data.body, data.end, true)? & 0x00ff_ffff != UTF8 {
                return None;
            }
            let text = read_at(r, data.body + 8, (data.end - data.body).checked_sub(8)?.min(MAX_TEXT), data.end)?;
            iso6709::parse(std::str::from_utf8(&text).ok()?)
        })
}
