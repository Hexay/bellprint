mod corpus;
mod sim;

pub use corpus::{CorpusEntry, eval_corpus, load_manifest, load_manifests, manifest_toml};
pub use sim::{SimEvalConfig, eval_sim, pick_rings};

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
