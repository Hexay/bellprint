use super::model::{MatchParams, cents};
use super::nominal::NominalPdf;
use crate::audio::spectrum::Peak;
use crate::catalog::Ring;

/// Observed lines in cents, sorted, with strength in (0, 1].
pub(super) struct Lines {
    pub cents: Vec<f64>,
    pub strength: Vec<f64>,
    /// Analysed band in cents; nothing outside it could have been seen.
    band: (f64, f64),
}

/// `v.partition_point(pred)` found by scanning up from `from`, which must not be past it: a cursor for
/// a predicate whose threshold only rises between calls. Same index as the binary search, far cheaper.
pub(super) fn advance<T>(v: &[T], mut from: usize, pred: impl Fn(&T) -> bool) -> usize {
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

/// Every bell's nominal term at every grid shift. A bell's term doesn't depend on which other bells
/// are scored with it, so it is computed once per ring and summed per scored subset.
pub(super) struct BellTerms {
    /// Per bell, per grid shift: ln(1 - p + p * LR), or 0.0 out of band (adds nothing to a sum that
    /// starts at +0.0, so subset sums keep their floats).
    terms: Vec<Vec<f64>>,
    /// Per bell, per grid shift: LR > 1.
    heard: Vec<Vec<bool>>,
}

impl BellTerms {
    /// Loops grid-inner so line cursors can ride each rising run of the grid.
    pub fn new(ring: &Ring, lines: &Lines, grid: &[(f64, f64)], p: &MatchParams) -> Self {
        let (pdf, half) = (NominalPdf::new(p), DENSITY_HALF_CENTS);
        let mut terms = Vec::with_capacity(ring.nominals.len());
        let mut heard = Vec::with_capacity(ring.nominals.len());
        for (&hz, &optional) in ring.nominals.iter().zip(&ring.optional) {
            let n = cents(hz);
            let ph = if optional { p.p_optional_heard.unwrap_or(p.p_bell_heard) } else { p.p_bell_heard };
            // ln(1 - p + p * 0) exactly: the term for a nominal with no line in reach.
            let miss = (1.0 - ph).ln();
            let (mut t, mut h) = (vec![0.0; grid.len()], vec![false; grid.len()]);
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
                near = advance(&lines.cents, near, |&c| c < x - pdf.reach);
                let best = lines.best_near(near, x, &pdf);
                if best == 0.0 {
                    t[g] = miss;
                    continue;
                }
                // A hit in a crowded spectrum is weak evidence; see MatchParams::p_bell_heard.
                below = advance(&lines.cents, below, |&c| c < x - half);
                upto = advance(&lines.cents, upto, |&c| c <= x + half);
                let lr = best / line_density(below, upto, half);
                h[g] = lr > 1.0;
                t[g] = (1.0 - ph + ph * lr).ln();
            }
            terms.push(t);
            heard.push(h);
        }
        Self { terms, heard }
    }

    /// Per grid shift: the terms of `bells` summed in the order given, and how many were heard.
    fn sum(&self, bells: &[usize], shifts: usize) -> (Vec<f64>, Vec<usize>) {
        let (mut total, mut heard) = (vec![0.0; shifts], vec![0usize; shifts]);
        for &b in bells {
            total.iter_mut().zip(&self.terms[b]).for_each(|(s, &t)| *s += t);
            heard.iter_mut().zip(&self.heard[b]).for_each(|(s, &h)| *s += usize::from(h));
        }
        (total, heard)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RingScore {
    pub score: f64,
    pub shift_cents: f64,
    pub bells_heard: usize,
}

/// Best score of `bells` over the global pitch shift. `line_terms` (of the same bells, per grid
/// shift; `partials::RingPartials::line_terms`) adds how well they explain every observed line.
/// `grid` is `model::shift_grid`: (shift, log prior) pairs.
pub(super) fn score_ring(terms: &BellTerms, bells: &[usize], line_terms: Option<&[f64]>, grid: &[(f64, f64)]) -> RingScore {
    let (total, heard) = terms.sum(bells, grid.len());
    let mut best = RingScore { score: f64::NEG_INFINITY, shift_cents: 0.0, bells_heard: 0 };
    for (g, &(shift, prior)) in grid.iter().enumerate() {
        let mut s = total[g] + prior;
        if let Some(l) = line_terms {
            s += l[g];
        }
        if s > best.score {
            best = RingScore { score: s, shift_cents: shift, bells_heard: heard[g] };
        }
    }
    best
}
