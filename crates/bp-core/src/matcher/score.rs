use super::model::{MatchParams, PARTIALS, cents, partial_tol, ratio_cents};
use crate::audio::spectrum::Peak;

/// Observed lines in cents, sorted, with strength in (0, 1].
pub(super) struct Lines {
    cents: Vec<f64>,
    strength: Vec<f64>,
    /// Analysed band in cents; nothing outside it could have been seen.
    band: (f64, f64),
}

/// `v.partition_point(pred)` found by scanning up from `from`, which must not be past it: a cursor for
/// a predicate whose threshold only rises between calls. Same index as the binary search, far cheaper.
fn advance<T>(v: &[T], mut from: usize, pred: impl Fn(&T) -> bool) -> usize {
    while from < v.len() && pred(&v[from]) {
        from += 1;
    }
    from
}

impl Lines {
    pub fn from_peaks(peaks: &[Peak], full_strength_db: f64, band_hz: (f64, f64)) -> Self {
        let mut v: Vec<(f64, f64)> =
            peaks.iter().map(|p| (cents(p.hz), (p.prominence_db / full_strength_db).min(1.0))).collect();
        v.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self {
            cents: v.iter().map(|x| x.0).collect(),
            strength: v.iter().map(|x| x.1).collect(),
            band: (cents(band_hz.0), cents(band_hz.1)),
        }
    }

    /// Best Gaussian pdf (per cent) of the offset to a line within 3 sigma of `x`, discounted only for
    /// faint lines: real nominals are often just 10–12 dB above the floor. `lo` is the first line at or
    /// above `x - 3 sigma`.
    fn best_near(&self, lo: usize, x: f64, sigma: f64) -> f64 {
        let norm = 1.0 / (sigma * std::f64::consts::TAU.sqrt());
        let mut best: f64 = 0.0;
        for i in lo..self.cents.len() {
            let d = self.cents[i] - x;
            if d > 3.0 * sigma {
                break;
            }
            let s = (self.strength[i] / 0.4).min(1.0);
            best = best.max(s * norm * (-0.5 * (d / sigma).powi(2)).exp());
        }
        best
    }
}

/// Lines per cent within ±`half` cents (add-one smoothed), from the line counts below and up to that
/// window: the rate of chance coincidences there.
fn line_density(below: usize, upto: usize, half: f64) -> f64 {
    (upto - below + 1) as f64 / (2.0 * half)
}

const DENSITY_HALF_CENTS: f64 = 300.0;

/// A ring prepared for scoring: nominals and every predicted partial, in cents.
pub(super) struct RingModel {
    nominals: Vec<f64>,
    /// Probability each nominal is heard: `p_bell_heard`, or `p_optional_heard` for optional bells.
    heard: Vec<f64>,
    /// (cents, sigma) sorted by cents.
    partials: Vec<(f64, f64)>,
    max_sigma: f64,
}

impl RingModel {
    /// `cast_years` and `optional` run parallel to `nominals_hz`.
    pub fn new(nominals_hz: &[f64], cast_years: &[Option<u16>], optional: &[bool], p: &MatchParams) -> Self {
        let nominals: Vec<f64> = nominals_hz.iter().map(|&h| cents(h)).collect();
        let heard = optional
            .iter()
            .map(|&o| if o { p.p_optional_heard.unwrap_or(p.p_bell_heard) } else { p.p_bell_heard })
            .collect();
        let mut partials: Vec<(f64, f64)> = nominals
            .iter()
            .zip(cast_years)
            .flat_map(|(&n, &year)| {
                PARTIALS.iter().map(move |q| {
                    (n + ratio_cents(q.ratio), partial_tol(q, year, p).hypot(p.nominal_sigma_cents))
                })
            })
            .collect();
        partials.sort_by(|a, b| a.0.total_cmp(&b.0));
        let max_sigma = partials.iter().map(|x| x.1).fold(0.0, f64::max);
        Self { nominals, heard, partials, max_sigma }
    }

    /// Density (per cent) of this ring's partials at `y` (already unshifted): an equal-weight Gaussian
    /// mixture, so broad partials and big rings spread their evidence thin (Occam). `lo` is the first
    /// partial at or above `y - 3 max_sigma`.
    fn density(&self, lo: usize, y: f64) -> f64 {
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

/// Per grid shift: the summed nominal terms and the bells heard. Loops nominal-outer so line cursors
/// can ride each rising run of the grid; each shift still sums its nominals in order, so the floats
/// are bit-identical to a shift-outer loop.
fn bell_terms(m: &RingModel, lines: &Lines, grid: &[(f64, f64)], p: &MatchParams) -> (Vec<f64>, Vec<usize>) {
    let (sigma, half) = (p.nominal_sigma_cents, DENSITY_HALF_CENTS);
    let mut total = vec![0.0; grid.len()];
    let mut heard = vec![0usize; grid.len()];
    for (&n, &ph) in m.nominals.iter().zip(&m.heard) {
        // ln(1 - p + p * 0) exactly: the term for a nominal with no line in reach.
        let miss = (1.0 - ph).ln();
        let (mut near, mut below, mut upto) = (0, 0, 0);
        let mut prev = f64::INFINITY;
        for (g, &(shift, _)) in grid.iter().enumerate() {
            if shift < prev {
                (near, below, upto) = (0, 0, 0);
            }
            prev = shift;
            let x = n + shift;
            // Out of band the bell is unobservable, not missed: charging it would sink light rings.
            if x < lines.band.0 || x > lines.band.1 {
                continue;
            }
            near = advance(&lines.cents, near, |&c| c < x - 3.0 * sigma);
            let best = lines.best_near(near, x, sigma);
            if best == 0.0 {
                total[g] += miss;
                continue;
            }
            // A hit in a crowded spectrum is weak evidence; see MatchParams::p_bell_heard.
            below = advance(&lines.cents, below, |&c| c < x - half);
            upto = advance(&lines.cents, upto, |&c| c <= x + half);
            let lr = best / line_density(below, upto, half);
            heard[g] += usize::from(lr > 1.0);
            total[g] += (1.0 - ph + ph * lr).ln();
        }
    }
    (total, heard)
}

/// Per line: log ratio of (background share + ring share) density against pure background, weighted by strength.
fn line_terms(m: &RingModel, lines: &Lines, shift: f64, p: &MatchParams) -> f64 {
    let a = p.background_share;
    let mut lo = 0;
    lines
        .cents
        .iter()
        .zip(&lines.strength)
        .map(|(&x, &w)| {
            let y = x - shift;
            lo = advance(&m.partials, lo, |l| l.0 < y - 3.0 * m.max_sigma);
            w * (a + (1.0 - a) * p.band_cents * m.density(lo, y)).ln()
        })
        .sum()
}

/// Best score over the global pitch shift. `full` adds how well the ring explains every observed line.
/// `grid` is `model::shift_grid`: (shift, log prior) pairs.
pub(super) fn score_ring(m: &RingModel, lines: &Lines, p: &MatchParams, grid: &[(f64, f64)], full: bool) -> RingScore {
    let (bells, heard) = bell_terms(m, lines, grid, p);
    let mut best = RingScore { score: f64::NEG_INFINITY, shift_cents: 0.0, bells_heard: 0 };
    for (g, &(shift, prior)) in grid.iter().enumerate() {
        let mut s = bells[g] + prior;
        if full {
            s += line_terms(m, lines, shift, p);
        }
        if s > best.score {
            best = RingScore { score: s, shift_cents: shift, bells_heard: heard[g] };
        }
    }
    best
}
