mod dove_csv;

use anyhow::Result;
use serde::Serialize;
use std::path::Path;

/// One ring of bells from Dove's Guide, with the nominal (Hz) of every bell that has one recorded.
#[derive(Debug, Clone, Serialize)]
pub struct Ring {
    pub ring_id: u32,
    pub tower_id: u32,
    pub place: String,
    pub dedication: String,
    pub county: String,
    pub country: String,
    pub lat: f64,
    pub lon: f64,
    /// Ring size as Dove states it; may exceed `nominals.len()` when bells lack measurements.
    pub bells: u32,
    /// Known nominals, highest first.
    pub nominals: Vec<f64>,
}

impl Ring {
    pub fn complete(&self) -> bool {
        self.nominals.len() as u32 >= self.bells
    }
}

// The data sits outside the crate dir; pyproject's maturin `include` puts it in the sdist.
const EMBEDDED_DOVE: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/dove/dove.csv"));
const EMBEDDED_BELLS: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/dove/bells.csv"));

pub struct Catalog {
    pub rings: Vec<Ring>,
}

impl Catalog {
    /// Loads `dove.csv` and `bells.csv` from `dir`; rings with no recorded nominal at all are dropped.
    pub fn load(dir: &Path) -> Result<Self> {
        let (dove, bells) = dove_csv::read_dir(dir)?;
        Ok(Self { rings: dove_csv::parse_rings(&dove, &bells) })
    }

    /// The Dove snapshot compiled into the binary (data/dove, CC BY-SA 4.0; see data/dove/README.md).
    pub fn embedded() -> Self {
        Self { rings: dove_csv::parse_rings(EMBEDDED_DOVE, EMBEDDED_BELLS) }
    }

    /// `dir` if given, else the embedded snapshot.
    pub fn load_or_embedded(dir: Option<&Path>) -> Result<Self> {
        dir.map_or_else(|| Ok(Self::embedded()), Self::load)
    }

    pub fn ring(&self, ring_id: u32) -> Option<&Ring> {
        self.rings.iter().find(|r| r.ring_id == ring_id)
    }

    pub fn by_tower(&self, tower_id: u32) -> impl Iterator<Item = &Ring> {
        self.rings.iter().filter(move |r| r.tower_id == tower_id)
    }
}
