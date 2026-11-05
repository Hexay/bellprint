use super::Ring;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize)]
struct DoveRow {
    #[serde(rename = "TowerID")]
    tower_id: u32,
    #[serde(rename = "RingID")]
    ring_id: u32,
    #[serde(rename = "Place")]
    place: String,
    #[serde(rename = "Dedicn")]
    dedication: String,
    #[serde(rename = "County")]
    county: String,
    #[serde(rename = "Country")]
    country: String,
    #[serde(rename = "Lat", deserialize_with = "csv::invalid_option")]
    lat: Option<f64>,
    #[serde(rename = "Long", deserialize_with = "csv::invalid_option")]
    lon: Option<f64>,
    #[serde(rename = "Bells")]
    bells: String,
    #[serde(rename = "Hz", deserialize_with = "csv::invalid_option")]
    tenor_hz: Option<f64>,
}

#[derive(Deserialize)]
struct BellRow {
    #[serde(rename = "Ring ID", deserialize_with = "csv::invalid_option")]
    ring_id: Option<u32>,
    #[serde(rename = "Bell Role")]
    role: String,
    #[serde(rename = "Nominal (Hz)", deserialize_with = "csv::invalid_option")]
    nominal: Option<f64>,
    #[serde(rename = "Cast Date", default)]
    cast: String,
}

/// First four-digit year in Dove's free-text cast date ("1765", "c1599", "1890s").
fn cast_year(s: &str) -> Option<u16> {
    let b = s.as_bytes();
    (0..b.len().saturating_sub(3))
        .find(|&i| b[i..i + 4].iter().all(u8::is_ascii_digit) && !b.get(i + 4).is_some_and(u8::is_ascii_digit))
        .and_then(|i| s[i..i + 4].parse().ok())
}

fn reader(bytes: &[u8]) -> csv::Reader<&[u8]> {
    csv::ReaderBuilder::new().flexible(true).from_reader(bytes)
}

pub(super) fn read_dir(dir: &Path) -> Result<(Vec<u8>, Vec<u8>)> {
    let read = |name: &str| {
        let p = dir.join(name);
        std::fs::read(&p).with_context(|| format!("opening {}", p.display()))
    };
    Ok((read("dove.csv")?, read("bells.csv")?))
}

/// Ring bells have numeric roles ("1", "6b"); Sanctus, clock and disused bells are excluded.
fn is_ring_bell(role: &str) -> bool {
    role.chars().next().is_some_and(|c| c.is_ascii_digit())
}

/// Semitone bells ("6b", "2#") and an extra treble ("0"): hung beside the diatonic ring, which bands
/// usually ring without them.
fn is_optional_bell(role: &str) -> bool {
    role == "0" || role.ends_with(['b', '#'])
}

/// Dove writes sizes like "3 U/R" for unringable rings.
fn leading_int(s: &str) -> Option<u32> {
    let digits: String = s.trim().chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

type Bell = (f64, Option<u16>, bool);

/// (nominal Hz, cast year, optional) of every ring bell, by ring.
fn bells_by_ring(bells_csv: &[u8]) -> HashMap<u32, Vec<Bell>> {
    let mut out: HashMap<u32, Vec<Bell>> = HashMap::new();
    for row in reader(bells_csv).deserialize::<BellRow>() {
        let Ok(row) = row else { continue };
        if let (Some(id), Some(hz)) = (row.ring_id, row.nominal)
            && is_ring_bell(&row.role)
            && hz > 0.0
        {
            out.entry(id).or_default().push((hz, cast_year(&row.cast), is_optional_bell(&row.role)));
        }
    }
    out
}

/// Rings from the bytes of Dove's `dove.csv` and `bells.csv`.
pub(super) fn parse_rings(dove_csv: &[u8], bells_csv: &[u8]) -> Vec<Ring> {
    let mut bells = bells_by_ring(bells_csv);
    let mut rings = Vec::new();
    for row in reader(dove_csv).deserialize::<DoveRow>() {
        // Dove's CSV has a BOM and occasional odd rows; skip what doesn't parse.
        let Ok(row) = row else { continue };
        let (Some(lat), Some(lon)) = (row.lat, row.lon) else { continue };
        let mut b = bells.remove(&row.ring_id).unwrap_or_default();
        if b.is_empty()
            && let Some(t) = row.tenor_hz.filter(|t| *t > 0.0)
        {
            b.push((t, None, false));
        }
        if b.is_empty() {
            continue;
        }
        b.sort_by(|x, y| y.0.total_cmp(&x.0));
        let hz: Vec<f64> = b.iter().map(|x| x.0).collect();
        let cast_years = b.iter().map(|x| x.1).collect();
        let optional = b.iter().map(|x| x.2).collect();
        rings.push(Ring {
            ring_id: row.ring_id,
            tower_id: row.tower_id,
            place: row.place,
            dedication: row.dedication,
            county: row.county,
            country: row.country,
            lat,
            lon,
            bells: leading_int(&row.bells).unwrap_or(hz.len() as u32),
            nominals: hz,
            cast_years,
            optional,
        });
    }
    rings
}
