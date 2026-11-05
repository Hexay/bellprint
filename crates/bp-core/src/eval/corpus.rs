use super::{Case, Clip};
use crate::audio::Audio;
use crate::audio::resample::resample;
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
    #[serde(default)]
    pub url: String,
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

/// Every `manifest*.toml` in `dir`, concatenated in file-name order. A recording listed in more than
/// one manifest (same url) is kept once, so it isn't scored twice.
pub fn load_manifests(dir: &Path) -> Result<Vec<CorpusEntry>> {
    let mut paths: Vec<_> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.starts_with("manifest") && name.ends_with(".toml")
        })
        .collect();
    paths.sort();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for p in paths {
        out.extend(load_manifest(&p)?.into_iter().filter(|e| e.url.is_empty() || seen.insert(e.url.clone())));
    }
    Ok(out)
}

pub fn manifest_toml(entries: Vec<CorpusEntry>) -> Result<String> {
    Ok(toml::to_string(&Manifest { recording: entries })?)
}

/// Locates every entry with audio at `<audio_dir>/<id>.flac`, resampled to `resample_hz` if given (a
/// robustness probe); entries without audio are skipped.
pub fn eval_corpus(
    catalog: &Catalog,
    entries: &[CorpusEntry],
    audio_dir: &Path,
    resample_hz: Option<u32>,
    opts: &LocateOptions,
) -> Vec<(Case, Clip)> {
    entries
        .par_iter()
        .filter_map(|e| {
            let audio = Audio::load(&audio_dir.join(format!("{}.flac", e.id))).ok()?;
            let audio = match resample_hz {
                Some(hz) => resample(&audio, hz),
                None => audio,
            };
            let report = locate_audio(catalog, &audio, opts);
            let note = format!("{} {} {:.0}s", e.ringing, e.confidence, audio.duration_s());
            let clip = Clip::new(e.id.clone(), e.tower_id, e.bells as usize, &report, note);
            Some((clip.case(&report.matches), clip))
        })
        .collect()
}
