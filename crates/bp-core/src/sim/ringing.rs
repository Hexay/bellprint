use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pattern {
    Rounds,
    PlainHunt,
    CallChanges,
    /// Tenor alone, slowly.
    Tolling,
}

/// The rows rung: each row is a permutation of bell indices (0 = treble).
fn rows<R: Rng>(n: usize, pattern: Pattern, count: usize, rng: &mut R) -> Vec<Vec<usize>> {
    let mut row: Vec<usize> = (0..n).collect();
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        out.push(row.clone());
        match pattern {
            Pattern::Rounds | Pattern::Tolling => {}
            Pattern::PlainHunt => {
                let first = i % 2;
                for j in (first..n.saturating_sub(1)).step_by(2) {
                    row.swap(j, j + 1);
                }
            }
            Pattern::CallChanges if n > 1 && i % 3 == 2 => {
                let j = rng.random_range(0..n - 1);
                row.swap(j, j + 1);
            }
            Pattern::CallChanges => {}
        }
    }
    out
}

/// Strike times `(bell index, seconds)` over `dur_s`, starting part-way into the ringing; times may be
/// negative so the clip opens with earlier blows still sounding.
pub fn strikes<R: Rng>(n: usize, pattern: Pattern, tenor_hz: f64, dur_s: f64, rng: &mut R) -> Vec<(usize, f64)> {
    if pattern == Pattern::Tolling {
        let gap = rng.random_range(2.0..4.0);
        let t0 = rng.random_range(0.0..gap);
        return (0..).map(|k| (n - 1, t0 + k as f64 * gap)).take_while(|s| s.1 < dur_s).collect();
    }
    // Heavier (lower) rings strike slower; a 2 s whole pull is typical for 8 bells.
    let blow = 0.18 + 45.0 / tenor_hz * rng.random_range(0.85..1.15);
    let jitter = Normal::new(0.0, 0.012).unwrap();
    let row_len = n as f64 * blow;
    let count = (dur_s / row_len) as usize + 4;
    let lead_in = rng.random_range(2.0 * row_len..4.0 * row_len);
    let count = count + (lead_in / row_len) as usize + 1;
    let mut out = Vec::new();
    let mut t = -lead_in;
    for (r, row) in rows(n, pattern, count, rng).iter().enumerate() {
        for &b in row {
            if t < dur_s {
                out.push((b, t + jitter.sample(rng)));
            }
            t += blow;
        }
        // Handstroke gap: one blow of silence before every other row.
        if r % 2 == 1 {
            t += blow;
        }
    }
    out
}
