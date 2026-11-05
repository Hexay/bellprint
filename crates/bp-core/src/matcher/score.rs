use super::model::{MatchParams, PARTIALS, cents, partial_tol, ratio_cents};
use super::nominal::NominalPdf;
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

    /// Best pdf (per cent) of the offset to a line within `pdf.reach` of `x`, discounted only for faint
    /// lines: real nominals are often just 10–12 dB above the floor. `lo` is the first line at or above
    /// `x - pdf.reach`.
    fn best_near(&self, lo: usize, x: f64, pdf: &NominalPdf) -> f64 {
        let mut best: f64 = 0.0;
        for i in lo..self.cents.len() {
            let d = self.cents[i] - x;
            if d > pdf.reach {
                break;
            }
            best = best.max(pdf.scaled((self.strength[i] / 0.4).min(1.0), d));
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
    /// (cents, sigma) sorted by cents.
    partials: Vec<(f64, f64)>,
    max_sigma: f64,
}

impl RingModel {
    /// `cast_years` runs parallel to `nominals_hz`.
    pub fn new(nominals_hz: &[f64], cast_years: &[Option<u16>], p: &MatchParams) -> Self {
        let nominals: Vec<f64> = nominals_hz.iter().map(|&h| cents(h)).collect();
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
        Self { nominals, partials, max_sigma }
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
    let pdf = NominalPdf::new(p);
    let bad = (p.ring_outlier_weight > 0.0).then(|| NominalPdf::wide(p.ring_outlier_sigma_cents));
    let mut total = vec![0.0; grid.len()];
    let mut heard = vec![0usize; grid.len()];
    // Σ over bells of exp(bad term − good term): see MatchParams::ring_outlier_weight.
    let mut swaps = vec![0.0; if bad.is_some() { grid.len() } else { 0 }];
    for &n in &m.nominals {
        let (mut cur, mut cur_bad) = (Cursor::default(), Cursor::default());
        let mut prev = f64::INFINITY;
        for (g, &(shift, _)) in grid.iter().enumerate() {
            if shift < prev {
                (cur, cur_bad) = (Cursor::default(), Cursor::default());
            }
            prev = shift;
            let x = n + shift;
            // Out of band the bell is unobservable, not missed: charging it would sink light rings.
            if x < lines.band.0 || x > lines.band.1 {
                continue;
            }
            let (t, hit) = cur.term(lines, x, &pdf, p);
            heard[g] += usize::from(hit);
            total[g] += t;
            if let Some(bad) = &bad {
                swaps[g] += (cur_bad.term(lines, x, bad, p).0 - t).exp();
            }
        }
    }
    let (w, k) = (p.ring_outlier_weight, m.nominals.len() as f64);
    for (t, s) in total.iter_mut().zip(&swaps) {
        *t += (1.0 - w + w * s / k).ln();
    }
    (total, heard)
}

/// Line cursors for one nominal riding a rising run of the shift grid.
#[derive(Default)]
struct Cursor {
    near: usize,
    below: usize,
    upto: usize,
}

impl Cursor {
    /// ln(1 - p + p LR) for a nominal at `x` (LR = pdf of its best line / local line density), and
    /// whether LR > 1. With no line in reach that is ln(1 - p) exactly.
    fn term(&mut self, lines: &Lines, x: f64, pdf: &NominalPdf, p: &MatchParams) -> (f64, bool) {
        let half = DENSITY_HALF_CENTS;
        self.near = advance(&lines.cents, self.near, |&c| c < x - pdf.reach);
        let best = lines.best_near(self.near, x, pdf);
        if best == 0.0 {
            return ((1.0 - p.p_bell_heard).ln(), false);
        }
        // A hit in a crowded spectrum is weak evidence; see MatchParams::p_bell_heard.
        self.below = advance(&lines.cents, self.below, |&c| c < x - half);
        self.upto = advance(&lines.cents, self.upto, |&c| c <= x + half);
        let lr = best / line_density(self.below, self.upto, half);
        ((1.0 - p.p_bell_heard + p.p_bell_heard * lr).ln(), lr > 1.0)
    }
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
