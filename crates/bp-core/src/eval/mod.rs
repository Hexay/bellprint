mod corpus;
mod replay;
mod sim;

pub use corpus::{CorpusEntry, eval_corpus, load_manifest, load_manifests, manifest_toml};
pub use replay::{Clip, replay};
pub use sim::{SimEvalConfig, eval_sim, pick_rings};

use crate::matcher::Confidence;
use serde::Serialize;

/// One evaluated recording.
#[derive(Debug, Clone, Serialize)]
pub struct Case {
    pub id: String,
    pub tower_id: u32,
    pub bells: usize,
    /// 1-based rank of the true tower; `None` if outside the reported list.
    pub rank: Option<usize>,
    pub top_tower_id: Option<u32>,
    pub top_place: String,
    pub top_probability: f64,
    pub top_margin: f64,
    pub top_shift_cents: Option<f64>,
    /// The true ring's fitted shift and its score deficit to the top pick, when it is in the list.
    pub true_shift_cents: Option<f64>,
    pub true_gap: Option<f64>,
    pub confidence: Option<Confidence>,
    pub peaks: usize,
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub n: usize,
    pub top1: usize,
    pub top5: usize,
    pub top20: usize,
    pub median_rank: Option<usize>,
}

pub fn summarize(cases: &[Case]) -> Summary {
    let within = |k: usize| cases.iter().filter(|c| c.rank.is_some_and(|r| r <= k)).count();
    // Unranked cases sort last, so the median is honest about misses.
    let mut ranks: Vec<usize> = cases.iter().map(|c| c.rank.unwrap_or(usize::MAX)).collect();
    ranks.sort_unstable();
    let median_rank = ranks.get(ranks.len() / 2).copied().filter(|&r| r != usize::MAX);
    Summary { n: cases.len(), top1: within(1), top5: within(5), top20: within(20), median_rank }
}

/// "high 22/27, medium 13/27, low 11/87": top-1 hits over cases per confidence label.
pub fn by_confidence(cases: &[Case]) -> String {
    [Confidence::High, Confidence::Medium, Confidence::Low]
        .iter()
        .map(|&k| {
            let with: Vec<&Case> = cases.iter().filter(|c| c.confidence == Some(k)).collect();
            format!("{} {}/{}", k.label(), with.iter().filter(|c| c.rank == Some(1)).count(), with.len())
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Markdown table of a summary and its cases, worst first.
pub fn markdown(title: &str, s: &Summary, cases: &[Case]) -> String {
    let pct = |k: usize| 100.0 * k as f64 / s.n.max(1) as f64;
    let mut out = format!(
        "# {title}\n\nn = {}; top-1 {} ({:.0}%), top-5 {} ({:.0}%), top-20 {} ({:.0}%), median rank {}\n\n",
        s.n,
        s.top1,
        pct(s.top1),
        s.top5,
        pct(s.top5),
        s.top20,
        pct(s.top20),
        s.median_rank.map_or("-".into(), |r| r.to_string())
    );
    out += &format!("top-1 by confidence: {}\n\n", by_confidence(cases));
    out += "| id | bells | rank | top pick | p | peaks | note |\n|---|---|---|---|---|---|---|\n";
    let mut sorted: Vec<&Case> = cases.iter().collect();
    sorted.sort_by_key(|c| std::cmp::Reverse(c.rank.unwrap_or(usize::MAX)));
    for c in sorted {
        out += &format!(
            "| {} | {} | {} | {} | {:.2} | {} | {} |\n",
            c.id,
            c.bells,
            c.rank.map_or("miss".into(), |r| r.to_string()),
            c.top_place,
            c.top_probability,
            c.peaks,
            c.note
        );
    }
    out
}
