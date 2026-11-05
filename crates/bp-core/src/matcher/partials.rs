use super::model::{MatchParams, PARTIALS, cents, partial_tol, ratio_cents};
use super::score::{Lines, advance};
use crate::catalog::Ring;
use std::ops::Range;

struct Partial {
    cents: f64,
    sigma: f64,
    /// `sigma * sqrt(2π)`, the Gaussian's normaliser.
    norm: f64,
    bell: usize,
}

/// Sets of a ring's bells ("runs") whose partials are each scored as an equal-weight Gaussian mixture.
pub(super) struct Runs {
    /// Per bell: bit r set when run r has it.
    bell_mask: Vec<u64>,
    /// Partials per run.
    count: Vec<f64>,
    /// Distinct 3 × widest-partial-sigma of the runs, with the runs that have each: a run ignores
    /// partials farther than this from a line.
    reaches: Vec<(f64, u64)>,
    /// Per run: an earlier run of the same size, whose line term it reuses when their sums agree.
    twin: Vec<Option<usize>>,
}

/// Every predicted partial of every bell of a ring, in cents, sorted.
pub(super) struct RingPartials {
    partials: Vec<Partial>,
}

/// Grid shifts bounded together when pruning.
const BLOCK: usize = 8;
/// Slack (nats) on pruning decisions, far above the float error of a score yet far below any gap that
/// matters: a shift is skipped only when its bound is this much below an exact score.
const PRUNE_SLACK: f64 = 1e-6;
/// Slack (cents) on bounding distances, against rounding in `x - shift`.
const CENTS_SLACK: f64 = 1e-6;

/// Shared per-call constants of the line term w · ln(a + scale · sum / count).
struct Terms<'a> {
    runs: &'a Runs,
    lines: &'a Lines,
    a: f64,
    scale: f64,
    ln_a: f64,
    reach: f64,
}

impl RingPartials {
    pub fn new(ring: &Ring, p: &MatchParams) -> Self {
        let mut partials: Vec<Partial> = (0..ring.nominals.len())
            .flat_map(|bell| {
                let (n, year) = (cents(ring.nominals[bell]), ring.cast_years[bell]);
                PARTIALS.iter().map(move |q| {
                    let sigma = partial_tol(q, year, p).hypot(p.nominal_sigma_cents);
                    Partial { cents: n + ratio_cents(q.ratio), sigma, norm: sigma * std::f64::consts::TAU.sqrt(), bell }
                })
            })
            .collect();
        // Stable, so ties stay in bell order as in a sort of any one run's partials.
        partials.sort_by(|a, b| a.cents.total_cmp(&b.cents));
        Self { partials }
    }

    /// `sets` of this ring's bells (at most 64), each scored as its own mixture.
    pub fn runs(&self, sets: &[&[usize]], n_bells: usize) -> Runs {
        assert!(sets.len() <= 64, "run masks are u64");
        let mut bell_mask = vec![0u64; n_bells];
        let mut reaches: Vec<(f64, u64)> = Vec::new();
        for (r, bells) in sets.iter().enumerate() {
            bells.iter().for_each(|&b| bell_mask[b] |= 1 << r);
            let max_sigma = self.partials.iter().filter(|q| bells.contains(&q.bell)).map(|q| q.sigma).fold(0.0, f64::max);
            let reach = 3.0 * max_sigma;
            match reaches.iter_mut().find(|x| x.0 == reach) {
                Some(x) => x.1 |= 1 << r,
                None => reaches.push((reach, 1 << r)),
            }
        }
        let twin = (0..sets.len()).map(|r| (0..r).rev().find(|&t| sets[t].len() == sets[r].len())).collect();
        Runs { bell_mask, count: sets.iter().map(|b| (b.len() * PARTIALS.len()) as f64).collect(), reaches, twin }
    }

    /// Per run, per grid shift: each line's log ratio of (background share + run share) density
    /// against pure background, weighted by strength and summed. Big runs and broad partials spread
    /// their evidence thin (Occam). `base` is each run's other score terms per shift; shifts whose
    /// upper bound can't reach the run's best `base + line term` are left at -inf. The rest are exact:
    /// every run sums the same floats in the same order as if it were scored alone.
    pub fn line_terms(&self, runs: &Runs, lines: &Lines, grid: &[(f64, f64)], base: &[&[f64]], p: &MatchParams) -> Vec<Vec<f64>> {
        let a = p.background_share;
        let reach = runs.reaches.iter().map(|r| r.0).fold(0.0, f64::max);
        let t = Terms { runs, lines, a, scale: (1.0 - a) * p.band_cents, ln_a: a.ln(), reach };
        let n = runs.count.len();
        let mut out = vec![vec![f64::NEG_INFINITY; grid.len()]; n];
        let mut best = vec![f64::NEG_INFINITY; n];
        let score = |g: usize, active: u64, out: &mut Vec<Vec<f64>>, best: &mut Vec<f64>| {
            self.exact(&t, grid[g].0, active, |r, v| {
                out[r][g] = v;
                best[r] = best[r].max(base[r][g] + v);
            });
        };
        // Seed each run's best at its best shift before line terms, so most blocks prune.
        let seeds: Vec<usize> = base.iter().map(|b| (0..b.len()).fold(0, |m, g| if b[g] > b[m] { g } else { m })).collect();
        for (r, &g) in seeds.iter().enumerate() {
            if !best[r].is_finite() {
                let mask = seeds.iter().enumerate().filter(|x| *x.1 == g).fold(0u64, |m, x| m | 1 << x.0);
                score(g, mask, &mut out, &mut best);
            }
        }
        for block in blocks(grid) {
            let (lo, hi) = (grid[block.start].0, grid[block.end - 1].0);
            let bound = self.bound(&t, lo, hi);
            let active = (0..n)
                .filter(|&r| block.clone().map(|g| base[r][g]).fold(f64::NEG_INFINITY, f64::max) + bound[r] + PRUNE_SLACK >= best[r])
                .fold(0u64, |m, r| m | 1 << r);
            if active != 0 {
                block.for_each(|g| score(g, active, &mut out, &mut best));
            }
        }
        out
    }

    /// Exact line terms at `shift` of the `active` runs, passed to `emit(run, term)`.
    fn exact(&self, t: &Terms, shift: f64, active: u64, mut emit: impl FnMut(usize, f64)) {
        let n = t.runs.count.len();
        let (mut acc, mut sums, mut terms) = (vec![-0.0; n], vec![0.0; n], vec![0.0; n]);
        let mut lo = 0;
        for (&x, &w) in t.lines.cents.iter().zip(&t.lines.strength) {
            let y = x - shift;
            lo = advance(&self.partials, lo, |q| q.cents < y - t.reach);
            sums.fill(0.0);
            for q in &self.partials[lo..] {
                if q.cents - y > t.reach {
                    break;
                }
                let mut m = t.runs.bell_mask[q.bell] & active;
                for &(r, rm) in &t.runs.reaches {
                    if q.cents < y - r || q.cents - y > r {
                        m &= !rm;
                    }
                }
                if m != 0 {
                    let e = (-0.5 * ((q.cents - y) / q.sigma).powi(2)).exp() / q.norm;
                    while m != 0 {
                        sums[m.trailing_zeros() as usize] += e;
                        m &= m - 1;
                    }
                }
            }
            for r in (0..n).filter(|&r| active >> r & 1 == 1) {
                let sum = sums[r];
                terms[r] = match t.runs.twin[r] {
                    Some(tw) if active >> tw & 1 == 1 && sums[tw].to_bits() == sum.to_bits() => terms[tw],
                    // a + scale * 0 is exactly a.
                    _ if sum == 0.0 => w * t.ln_a,
                    _ => w * (t.a + t.scale * (sum / t.runs.count[r])).ln(),
                };
                acc[r] += terms[r];
            }
        }
        (0..n).filter(|&r| active >> r & 1 == 1).for_each(|r| emit(r, acc[r]));
    }

    /// Per run: an upper bound on its line terms at every shift in [lo, hi]. Each partial's Gaussian
    /// is taken at its nearest point of the line's span of unshifted positions, and every partial a
    /// run could reach from that span counts; the log term rises with the sum.
    fn bound(&self, t: &Terms, lo: f64, hi: f64) -> Vec<f64> {
        let n = t.runs.count.len();
        let (mut acc, mut sums) = (vec![0.0; n], vec![0.0; n]);
        let mut first = 0;
        for (&x, &w) in t.lines.cents.iter().zip(&t.lines.strength) {
            let (y0, y1) = (x - hi - CENTS_SLACK, x - lo + CENTS_SLACK);
            first = advance(&self.partials, first, |q| q.cents < y0 - t.reach);
            sums.fill(0.0);
            for q in &self.partials[first..] {
                if q.cents > y1 + t.reach {
                    break;
                }
                let mut m = t.runs.bell_mask[q.bell];
                for &(r, rm) in &t.runs.reaches {
                    if q.cents < y0 - r || q.cents > y1 + r {
                        m &= !rm;
                    }
                }
                let d = (y0 - q.cents).max(q.cents - y1).max(0.0);
                let e = (-0.5 * (d / q.sigma).powi(2)).exp() / q.norm;
                while m != 0 {
                    sums[m.trailing_zeros() as usize] += e;
                    m &= m - 1;
                }
            }
            for r in 0..n {
                acc[r] += if sums[r] == 0.0 { w * t.ln_a } else { w * (t.a + t.scale * (sums[r] / t.runs.count[r])).ln() };
            }
        }
        acc
    }
}

/// Up to `BLOCK` consecutive grid indices of one rising run of shifts each.
fn blocks(grid: &[(f64, f64)]) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    for g in 0..grid.len() {
        match out.last_mut() {
            Some(b) if b.len() < BLOCK && grid[g].0 > grid[g - 1].0 => b.end = g + 1,
            _ => out.push(g..g + 1),
        }
    }
    out
}

#[cfg(test)]
#[path = "partials_tests.rs"]
mod tests;
