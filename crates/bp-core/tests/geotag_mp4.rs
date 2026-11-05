use bp_core::geo::GeoPoint;
use bp_core::geotag::from_bytes;

const OXFORD: &str = "+51.7520-001.2577+062.000/";

fn atom(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
    let mut v = ((body.len() + 8) as u32).to_be_bytes().to_vec();
    v.extend_from_slice(kind);
    v.extend_from_slice(body);
    v
}

fn cat(parts: &[Vec<u8>]) -> Vec<u8> {
    parts.concat()
}

fn ftyp() -> Vec<u8> {
    atom(b"ftyp", b"qt  \0\0\0\0qt  ")
}

fn xyz(text: &str) -> Vec<u8> {
    let mut body = (text.len() as u16).to_be_bytes().to_vec();
    body.extend_from_slice(&0x15c7u16.to_be_bytes());
    body.extend_from_slice(text.as_bytes());
    atom(b"\xa9xyz", &body)
}

fn udta_file(text: &str) -> Vec<u8> {
    let moov = atom(b"moov", &cat(&[atom(b"mvhd", &[0; 100]), atom(b"udta", &xyz(text))]));
    cat(&[ftyp(), atom(b"mdat", &[0; 64]), moov])
}

fn hdlr() -> Vec<u8> {
    atom(b"hdlr", &cat(&[vec![0; 8], b"mdta".to_vec(), vec![0; 13]]))
}

fn keys(names: &[&str]) -> Vec<u8> {
    let mut body = vec![0; 4];
    body.extend_from_slice(&(names.len() as u32).to_be_bytes());
    for n in names {
        body.extend_from_slice(&((n.len() + 8) as u32).to_be_bytes());
        body.extend_from_slice(b"mdta");
        body.extend_from_slice(n.as_bytes());
    }
    atom(b"keys", &body)
}

fn data_item(index: u32, text: &str) -> Vec<u8> {
    let data = atom(b"data", &cat(&[1u32.to_be_bytes().to_vec(), vec![0; 4], text.as_bytes().to_vec()]));
    atom(&index.to_be_bytes(), &data)
}

/// iPhone layout: QuickTime `meta` (no version/flags) with hdlr, keys and ilst.
fn apple_file(full_box: bool) -> Vec<u8> {
    let names = ["com.apple.quicktime.make", "com.apple.quicktime.location.ISO6709"];
    let ilst = atom(b"ilst", &cat(&[data_item(1, "Apple"), data_item(2, OXFORD)]));
    let prefix = if full_box { vec![0; 4] } else { Vec::new() };
    let meta = atom(b"meta", &cat(&[prefix, hdlr(), keys(&names), ilst]));
    cat(&[ftyp(), atom(b"moov", &cat(&[atom(b"mvhd", &[0; 100]), meta]))])
}

fn assert_oxford(p: Option<GeoPoint>) {
    let p = p.expect("a geotag");
    assert!((p.lat - 51.752).abs() < 1e-9 && (p.lon + 1.2577).abs() < 1e-9, "{p:?}");
}

#[test]
fn reads_udta_xyz() {
    assert_oxford(from_bytes(&udta_file(OXFORD)));
}

#[test]
fn reads_apple_keys_ilst_in_quicktime_and_iso_meta() {
    assert_oxford(from_bytes(&apple_file(false)));
    assert_oxford(from_bytes(&apple_file(true)));
}

#[test]
fn reads_moov_with_64_bit_size() {
    let inner = atom(b"udta", &xyz(OXFORD));
    let mut moov = 1u32.to_be_bytes().to_vec();
    moov.extend_from_slice(b"moov");
    moov.extend_from_slice(&((inner.len() + 16) as u64).to_be_bytes());
    moov.extend_from_slice(&inner);
    assert_oxford(from_bytes(&cat(&[ftyp(), moov])));
}

#[test]
fn ignores_other_keys_and_untagged_files() {
    let ilst = atom(b"ilst", &data_item(1, OXFORD));
    let meta = atom(b"meta", &cat(&[hdlr(), keys(&["com.apple.quicktime.make"]), ilst]));
    assert_eq!(from_bytes(&cat(&[ftyp(), atom(b"moov", &meta)])), None);
    assert_eq!(from_bytes(&cat(&[ftyp(), atom(b"moov", &atom(b"mvhd", &[0; 100]))])), None);
    assert_eq!(from_bytes(&cat(&[ftyp(), atom(b"mdat", &[0; 32])])), None);
    assert_eq!(from_bytes(b""), None);
    assert_eq!(from_bytes(b"ID3\x03\0\0\0\0\0\0garbage"), None);
    assert_eq!(from_bytes(b"RIFF\0\0\0\0WAVEfmt "), None);
}

#[test]
fn every_truncation_is_none() {
    for file in [udta_file(OXFORD), apple_file(false)] {
        for n in 0..file.len() {
            assert_eq!(from_bytes(&file[..n]), None, "truncated to {n}");
        }
    }
}

#[test]
fn rejects_malformed_sizes() {
    let mut f = udta_file(OXFORD);
    let moov_at = ftyp().len() + 72;
    for bad in [0u32, 7, u32::MAX] {
        f[moov_at..moov_at + 4].copy_from_slice(&bad.to_be_bytes());
        if bad == 0 {
            // size 0 means "to the end of the file", so the tag is still found.
            assert_oxford(from_bytes(&f));
        } else {
            assert_eq!(from_bytes(&f), None, "moov size {bad}");
        }
    }
    let mut over = xyz(OXFORD);
    over[8..10].copy_from_slice(&u16::MAX.to_be_bytes());
    let file = cat(&[ftyp(), atom(b"moov", &atom(b"udta", &over))]);
    assert_eq!(from_bytes(&file), None, "text length past the box");
    let mut many = keys(&["x"]);
    many[12..16].copy_from_slice(&u32::MAX.to_be_bytes());
    let meta = atom(b"meta", &cat(&[hdlr(), many, atom(b"ilst", &data_item(1, OXFORD))]));
    assert_eq!(from_bytes(&cat(&[ftyp(), atom(b"moov", &meta)])), None, "huge key count");
}

#[test]
fn caps_nesting_depth() {
    let mut inner = xyz(OXFORD);
    for _ in 0..64 {
        inner = atom(b"udta", &inner);
    }
    assert_eq!(from_bytes(&cat(&[ftyp(), atom(b"moov", &inner)])), None);
}

#[test]
fn rejects_bad_coordinates_in_valid_boxes() {
    for text in ["+91.0000-001.2577/", "+51.7520-181.0000/", "+00.0000+000.0000/", "51.7520-001.2577/", ""] {
        assert_eq!(from_bytes(&udta_file(text)), None, "{text:?}");
    }
}

#[test]
fn survives_corrupted_bytes() {
    let base = apple_file(false);
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    for _ in 0..5000 {
        let mut f = base.clone();
        for _ in 0..4 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let i = (state as usize) % f.len();
            f[i] = (state >> 32) as u8;
        }
        let _ = from_bytes(&f);
    }
}
