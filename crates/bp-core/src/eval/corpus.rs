use super::Case;
use super::sim::case;
use crate::audio::Audio;
use crate::catalog::Catalog;
use crate::locate::{LocateOptions, locate_audio};
use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A row of `data/corpus/manifest.toml`; fields not needed for scoring are ignored.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorpusEntry {
    pub id: String,
    pub tower_id: u32,
    #[serde(default)]
    pub ringing: String,
    #[serde(default)]
    pub confidence: String,
    #[serde(default)]
    pub bells: u32,
}

#[derive(Deserialize, Serialize)]
struct Manifest {
    recording: Vec<CorpusEntry>,
}

pub fn load_manifest(path: &Path) -> Result<Vec<CorpusEntry>> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(toml::from_str::<Manifest>(&text)?.recording)
}

pub fn manifest_toml(entries: Vec<CorpusEntry>) -> Result<String> {
    Ok(toml::to_string(&Manifest { recording: entries })?)
}

/// Locates every entry with audio at `<audio_dir>/<id>.flac`; entries without audio are skipped.
pub fn eval_corpus(
    catalog: &Catalog,
    entries: &[CorpusEntry],
    audio_dir: &Path,
    opts: &LocateOptions,
) -> Vec<Case> {
    entries
        .par_iter()
        .filter_map(|e| {
            let audio = Audio::load(&audio_dir.join(format!("{}.flac", e.id))).ok()?;
            let report = locate_audio(catalog, &audio, opts);
            let note = format!("{} {} {:.0}s", e.ringing, e.confidence, audio.duration_s());
            Some(case(e.id.clone(), e.tower_id, e.bells as usize, &report, note))
        })
        .collect()
}
