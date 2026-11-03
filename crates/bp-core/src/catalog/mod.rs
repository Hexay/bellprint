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

pub struct Catalog {
    pub rings: Vec<Ring>,
}

impl Catalog {
    /// Loads `dove.csv` and `bells.csv` from `dir`; rings with no recorded nominal at all are dropped.
    pub fn load(dir: &Path) -> Result<Self> {
        Ok(Self { rings: dove_csv::load_rings(dir)? })
    }

    pub fn ring(&self, ring_id: u32) -> Option<&Ring> {
        self.rings.iter().find(|r| r.ring_id == ring_id)
    }

    pub fn by_tower(&self, tower_id: u32) -> impl Iterator<Item = &Ring> {
        self.rings.iter().filter(move |r| r.tower_id == tower_id)
    }
}
