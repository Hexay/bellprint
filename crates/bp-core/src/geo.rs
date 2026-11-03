use serde::Serialize;

const EARTH_RADIUS_KM: f64 = 6371.0;

pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let dp = p2 - p1;
    let dl = (lon2 - lon1).to_radians();
    let a = (dp / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_KM * a.sqrt().asin()
}

/// A circular search area.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Region {
    pub lat: f64,
    pub lon: f64,
    pub radius_km: f64,
}

impl Region {
    pub fn distance_km(&self, lat: f64, lon: f64) -> f64 {
        haversine_km(self.lat, self.lon, lat, lon)
    }

    pub fn contains(&self, lat: f64, lon: f64) -> bool {
        self.distance_km(lat, lon) <= self.radius_km
    }

    /// Parses "lat,lon" or "lat,lon,radius_km" (radius defaults to 30 km).
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        let v: Vec<f64> = s.split(',').map(|x| x.trim().parse()).collect::<Result<_, _>>()?;
        match v.as_slice() {
            [lat, lon] => Ok(Self { lat: *lat, lon: *lon, radius_km: 30.0 }),
            [lat, lon, r] => Ok(Self { lat: *lat, lon: *lon, radius_km: *r }),
            _ => anyhow::bail!("expected lat,lon[,radius_km], got {s:?}"),
        }
    }
}
