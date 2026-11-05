//! ISO 6709 Annex H point strings as phones and QuickTime write them: "+51.7520-001.2577+062.000/".

use crate::geo::GeoPoint;

/// Latitude and longitude of an ISO 6709 string; altitude, CRS and the trailing '/' are optional and ignored.
pub fn parse(s: &str) -> Option<GeoPoint> {
    let s = s.trim_matches(|c: char| c.is_whitespace() || c == '\0');
    let (lat, rest) = component(s, 2)?;
    let (lon, rest) = component(rest, 3)?;
    if !(rest.is_empty() || rest.starts_with(['+', '-', '/', 'C'])) {
        return None;
    }
    GeoPoint::new(lat, lon)
}

/// One signed coordinate whose integer part has `deg` digits (±DD.D), `deg + 2` (±DDMM.M) or `deg + 4`
/// (±DDMMSS.S); the fraction belongs to the last unit.
fn component(s: &str, deg: usize) -> Option<(f64, &str)> {
    let sign = match s.as_bytes().first()? {
        b'+' => 1.0,
        b'-' => -1.0,
        _ => return None,
    };
    let body = &s[1..];
    let end = body.find(|c: char| !(c.is_ascii_digit() || c == '.')).unwrap_or(body.len());
    let num = &body[..end];
    let int_digits = num.find('.').unwrap_or(num.len());
    let x: f64 = num.parse().ok()?;
    let value = match int_digits.checked_sub(deg)? {
        0 => x,
        2 => {
            let d = (x / 100.0).trunc();
            minutes(d, x - d * 100.0, 0.0)?
        }
        4 => {
            let d = (x / 10_000.0).trunc();
            let m = ((x - d * 10_000.0) / 100.0).trunc();
            minutes(d, m, x - d * 10_000.0 - m * 100.0)?
        }
        _ => return None,
    };
    Some((sign * value, &body[end..]))
}

fn minutes(d: f64, m: f64, s: f64) -> Option<f64> {
    (m < 60.0 && s < 60.0).then_some(d + m / 60.0 + s / 3600.0)
}
