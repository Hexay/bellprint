use super::Match;
use serde::Serialize;

/// Coarse confidence in the top pick, from its score lead over the next candidate (`Match::margin`).
/// Thresholds were chosen on the sim and checked on the real corpus; hit rates per label are in
/// research/matcher_twins.md ("Confidence label").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

pub const HIGH_MARGIN_NATS: f64 = 2.0;
pub const MEDIUM_MARGIN_NATS: f64 = 1.0;

impl Confidence {
    pub fn from_margin(margin: f64) -> Self {
        if margin >= HIGH_MARGIN_NATS {
            Self::High
        } else if margin >= MEDIUM_MARGIN_NATS {
            Self::Medium
        } else {
            Self::Low
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// Confidence in the first of `matches` (ranked best first); `None` when there are none.
pub fn top_confidence(matches: &[Match]) -> Option<Confidence> {
    matches.first().map(|m| Confidence::from_margin(m.margin))
}
