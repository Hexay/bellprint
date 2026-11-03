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

/// Dove writes sizes like "3 U/R" for unringable rings.
fn leading_int(s: &str) -> Option<u32> {
    let digits: String = s.trim().chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn nominals_by_ring(bells_csv: &[u8]) -> HashMap<u32, Vec<f64>> {
    let mut out: HashMap<u32, Vec<f64>> = HashMap::new();
    for row in reader(bells_csv).deserialize::<BellRow>() {
        let Ok(row) = row else { continue };
        if let (Some(id), Some(hz)) = (row.ring_id, row.nominal)
            && is_ring_bell(&row.role)
            && hz > 0.0
        {
            out.entry(id).or_default().push(hz);
        }
    }
    out
}

/// Rings from the bytes of Dove's `dove.csv` and `bells.csv`.
pub(super) fn parse_rings(dove_csv: &[u8], bells_csv: &[u8]) -> Vec<Ring> {
    let mut nominals = nominals_by_ring(bells_csv);
    let mut rings = Vec::new();
    for row in reader(dove_csv).deserialize::<DoveRow>() {
        // Dove's CSV has a BOM and occasional odd rows; skip what doesn't parse.
        let Ok(row) = row else { continue };
        let (Some(lat), Some(lon)) = (row.lat, row.lon) else { continue };
        let mut hz = nominals.remove(&row.ring_id).unwrap_or_default();
        if hz.is_empty()
            && let Some(t) = row.tenor_hz.filter(|t| *t > 0.0)
        {
            hz.push(t);
        }
        if hz.is_empty() {
            continue;
        }
        hz.sort_by(|a, b| b.total_cmp(a));
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
        });
    }
    rings
}
