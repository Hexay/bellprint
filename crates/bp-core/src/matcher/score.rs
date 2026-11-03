use super::model::{MatchParams, PARTIALS, cents, ratio_cents};
use crate::audio::spectrum::Peak;

/// Observed lines in cents, sorted, with strength in (0, 1].
pub(super) struct Lines {
    cents: Vec<f64>,
    strength: Vec<f64>,
}

impl Lines {
    pub fn from_peaks(peaks: &[Peak], full_strength_db: f64) -> Self {
        let mut v: Vec<(f64, f64)> =
            peaks.iter().map(|p| (cents(p.hz), (p.prominence_db / full_strength_db).min(1.0))).collect();
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self { cents: v.iter().map(|x| x.0).collect(), strength: v.iter().map(|x| x.1).collect() }
    }

    /// Best `strength * gaussian(distance)` among lines within 3 sigma of `x`.
    fn best_near(&self, x: f64, sigma: f64) -> f64 {
        let lo = self.cents.partition_point(|&c| c < x - 3.0 * sigma);
        let mut best: f64 = 0.0;
        for i in lo..self.cents.len() {
            let d = self.cents[i] - x;
            if d > 3.0 * sigma {
                break;
            }
            best = best.max(self.strength[i] * (-0.5 * (d / sigma).powi(2)).exp());
        }
        best
    }
}

/// A ring prepared for scoring: nominals and every predicted partial, in cents.
pub(super) struct RingModel {
    nominals: Vec<f64>,
    /// (cents, sigma) sorted by cents.
    partials: Vec<(f64, f64)>,
    max_sigma: f64,
}

impl RingModel {
    pub fn new(nominals_hz: &[f64], p: &MatchParams) -> Self {
        let nominals: Vec<f64> = nominals_hz.iter().map(|&h| cents(h)).collect();
        let mut partials: Vec<(f64, f64)> = nominals
            .iter()
            .flat_map(|&n| {
                PARTIALS.iter().map(move |q| (n + ratio_cents(q.ratio), q.tol_cents.hypot(p.nominal_sigma_cents)))
            })
            .collect();
        partials.sort_by(|a, b| a.0.total_cmp(&b.0));
        let max_sigma = partials.iter().map(|x| x.1).fold(0.0, f64::max);
        Self { nominals, partials, max_sigma }
    }

    /// Density (per cent) of this ring's partials, shifted by `shift`, at `x`: an equal-weight Gaussian
    /// mixture, so broad partials and big rings spread their evidence thin (Occam).
    fn density(&self, x: f64, shift: f64) -> f64 {
        let y = x - shift;
        let lo = self.partials.partition_point(|l| l.0 < y - 3.0 * self.max_sigma);
        let mut sum = 0.0;
        for &(c, s) in &self.partials[lo..] {
            let d = c - y;
            if d > 3.0 * self.max_sigma {
                break;
            }
            sum += (-0.5 * (d / s).powi(2)).exp() / (s * (std::f64::consts::TAU).sqrt());
        }
        sum / self.partials.len() as f64
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RingScore {
    pub score: f64,
    pub shift_cents: f64,
    pub bells_heard: usize,
}

fn bell_terms(m: &RingModel, lines: &Lines, shift: f64, p: &MatchParams) -> (f64, usize) {
    let mut total = 0.0;
    let mut heard = 0;
    for &n in &m.nominals {
        let hit = lines.best_near(n + shift, p.nominal_sigma_cents);
        heard += usize::from(hit > 0.5);
        total += (1.0 - p.p_bell_heard + p.p_bell_heard * hit * p.hit_gain).ln();
    }
    (total, heard)
}

/// Per line: log ratio of (background share + ring share) density against pure background, weighted by strength.
fn line_terms(m: &RingModel, lines: &Lines, shift: f64, p: &MatchParams) -> f64 {
    let a = p.background_share;
    lines
        .cents
        .iter()
        .zip(&lines.strength)
        .map(|(&x, &w)| w * (a + (1.0 - a) * p.band_cents * m.density(x, shift)).ln())
        .sum()
}

/// Best score over the global pitch shift. `full` adds how well the ring explains every observed line.
/// `grid` is `model::shift_grid`: (shift, log prior) pairs.
pub(super) fn score_ring(m: &RingModel, lines: &Lines, p: &MatchParams, grid: &[(f64, f64)], full: bool) -> RingScore {
    let mut best = RingScore { score: f64::NEG_INFINITY, shift_cents: 0.0, bells_heard: 0 };
    for &(shift, prior) in grid {
        let (mut s, heard) = bell_terms(m, lines, shift, p);
        s += prior;
        if full {
            s += line_terms(m, lines, shift, p);
        }
        if s > best.score {
            best = RingScore { score: s, shift_cents: shift, bells_heard: heard };
        }
    }
    best
}
