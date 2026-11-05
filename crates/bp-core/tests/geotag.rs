use bp_core::audio::Audio;
use bp_core::catalog::Catalog;
use bp_core::geo::{DEFAULT_RADIUS_KM, GeoPoint, LocationSource};
use bp_core::geotag::{from_bytes, parse_iso6709, parse_text, resolve_location};
use bp_core::locate::{LocateOptions, locate_audio};
use std::path::PathBuf;

fn close(p: Option<GeoPoint>, lat: f64, lon: f64) -> bool {
    p.is_some_and(|p| (p.lat - lat).abs() < 1e-6 && (p.lon - lon).abs() < 1e-6)
}

#[test]
fn iso6709_forms() {
    assert!(close(parse_iso6709("+51.7520-001.2577+062.000/"), 51.752, -1.2577));
    assert!(close(parse_iso6709("+51.7520-001.2577/"), 51.752, -1.2577));
    assert!(close(parse_iso6709("-33.8688+151.2093"), -33.8688, 151.2093));
    assert!(close(parse_iso6709("+5145.12-00115.462/"), 51.752, -(1.0 + 15.462 / 60.0)));
    assert!(close(parse_iso6709("+514507.2-0011527.72/"), 51.752, -(1.0 + 15.0 / 60.0 + 27.72 / 3600.0)));
    assert!(close(parse_iso6709("+51.7520-001.2577+062.000CRSWGS_84/"), 51.752, -1.2577));
    assert!(close(parse_iso6709(" +51.7520-001.2577/\0"), 51.752, -1.2577));
}

#[test]
fn iso6709_rejects_out_of_range_and_malformed() {
    for s in [
        "+90.0001+000.0000/",
        "-90.5000+010.0000/",
        "+10.0000+180.0001/",
        "+10.0000-200.0000/",
        "+5160.00-00100.00/",
        "+514560.0-0010000.0/",
        "+00.0000+000.0000/",
        "51.7520-001.2577/",
        "+51.7520/",
        "+517.520-001.2577/",
        "+51.7520-01.2577/",
        "+51.7520-001.2577x",
        "+51..7520-001.2577/",
        "+NaN-001.2577/",
        "",
        "+",
    ] {
        assert_eq!(parse_iso6709(s), None, "{s:?}");
    }
    assert!(parse_iso6709("+90.0000+180.0000/").is_some());
    assert!(parse_iso6709("-90.0000-180.0000/").is_some());
}

#[test]
fn geopoint_range() {
    assert!(GeoPoint::new(51.0, -1.0).is_some());
    for (lat, lon) in [(90.1, 0.5), (-91.0, 0.5), (1.0, 180.5), (1.0, -181.0), (f64::NAN, 1.0), (1.0, f64::INFINITY)] {
        assert_eq!(GeoPoint::new(lat, lon), None, "{lat},{lon}");
    }
}

#[test]
fn decimal_text() {
    assert!(close(parse_text("51.752, -1.2577"), 51.752, -1.2577));
    assert!(close(parse_text("51.752 -1.2577"), 51.752, -1.2577));
    assert_eq!(parse_text("51.752"), None);
    assert_eq!(parse_text("151.752,-1.2577"), None);
}

fn flac(comments: &[&str], truncate_to: Option<usize>) -> Vec<u8> {
    let mut vc = 3u32.to_le_bytes().to_vec();
    vc.extend_from_slice(b"bp\0");
    vc.extend_from_slice(&(comments.len() as u32).to_le_bytes());
    for c in comments {
        vc.extend_from_slice(&(c.len() as u32).to_le_bytes());
        vc.extend_from_slice(c.as_bytes());
    }
    let mut f = b"fLaC".to_vec();
    f.extend_from_slice(&[0, 0, 0, 34]);
    f.extend_from_slice(&[0; 34]);
    f.extend_from_slice(&(0x8400_0000 | vc.len() as u32).to_be_bytes());
    f.extend_from_slice(&vc);
    f.truncate(truncate_to.unwrap_or(f.len()));
    f
}

#[test]
fn flac_vorbis_comment() {
    assert!(close(from_bytes(&flac(&["TITLE=Rounds", "location=+51.7520-001.2577/"], None)), 51.752, -1.2577));
    assert!(close(from_bytes(&flac(&["GPS=51.752,-1.2577"], None)), 51.752, -1.2577));
    assert_eq!(from_bytes(&flac(&["TITLE=Rounds", "COMMENT=+51.7520-001.2577/"], None)), None);
    let full = flac(&["LOCATION=+51.7520-001.2577/"], None);
    for n in 0..full.len() {
        assert_eq!(from_bytes(&full[..n]), None, "truncated to {n}");
    }
}

fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
    let p = std::env::temp_dir().join(format!("bp-geotag-{}-{name}", std::process::id()));
    std::fs::write(&p, bytes).unwrap();
    p
}

fn tagged(test: &str) -> PathBuf {
    temp_file(&format!("{test}.flac"),&flac(&["LOCATION=+51.7520-001.2577/"], None))
}

#[test]
fn geotag_sets_the_region_when_asked() {
    let path = tagged("sets");
    let l = resolve_location(&path, None, true).unwrap();
    assert_eq!(l.location_source, Some(LocationSource::Geotag));
    let r = l.region.unwrap();
    assert!((r.lat - 51.752).abs() < 1e-9 && r.radius_km == DEFAULT_RADIUS_KM);
    assert!(l.warning.is_none());

    let off = resolve_location(&path, None, false).unwrap();
    assert!(off.region.is_none() && off.geotag.is_none() && off.location_source.is_none());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn near_wins_over_geotag() {
    let path = tagged("near");
    let l = resolve_location(&path, Some("52.0,0.1,5"), true).unwrap();
    assert_eq!(l.location_source, Some(LocationSource::Near));
    assert_eq!(l.region.unwrap().radius_km, 5.0);
    assert!(close(l.geotag, 51.752, -1.2577));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn missing_geotag_warns_and_searches_everywhere() {
    let path = temp_file("plain.flac", &flac(&["TITLE=Rounds"], None));
    let l = resolve_location(&path, None, true).unwrap();
    assert!(l.region.is_none() && l.location_source.is_none() && l.geotag.is_none());
    assert!(l.warning.unwrap().contains("no geotag"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn report_json_names_the_location_source() {
    let path = tagged("report");
    let location = resolve_location(&path, None, true).unwrap();
    std::fs::remove_file(path).unwrap();
    let audio = Audio { samples: vec![0.0; 8000], sample_rate: 8000 };
    let report = locate_audio(&Catalog::embedded(), &audio, &LocateOptions { location, top: 3, ..Default::default() });
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["location_source"], "geotag");
    assert!((json["geotag"]["lat"].as_f64().unwrap() - 51.752).abs() < 1e-9);
    assert_eq!(json["region"]["radius_km"], DEFAULT_RADIUS_KM);
    assert!(json.get("warning").is_none());

    let none = locate_audio(&Catalog::embedded(), &audio, &LocateOptions { top: 3, ..Default::default() });
    let json = serde_json::to_value(&none).unwrap();
    assert!(json["location_source"].is_null() && json["geotag"].is_null() && json["region"].is_null());
}
