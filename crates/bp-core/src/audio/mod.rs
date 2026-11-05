pub mod blows;
mod decode;
pub mod longterm;
pub mod onsets;
pub mod resample;
pub mod spectrum;
pub mod window;

use anyhow::Result;
use std::path::Path;

/// Mono PCM in [-1, 1].
#[derive(Clone)]
pub struct Audio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl Audio {
    pub fn load(path: &Path) -> Result<Self> {
        decode::decode_mono(path)
    }

    pub fn duration_s(&self) -> f64 {
        self.samples.len() as f64 / self.sample_rate as f64
    }

    /// Clamped sub-range in seconds; `None` end means to the end of the file.
    pub fn slice(&self, start_s: f64, end_s: Option<f64>) -> Audio {
        let n = self.samples.len();
        let at = |t: f64| ((t.max(0.0) * self.sample_rate as f64) as usize).min(n);
        let (a, b) = (at(start_s), end_s.map_or(n, at));
        Audio { samples: self.samples[a..b.max(a)].to_vec(), sample_rate: self.sample_rate }
    }

    pub fn write_wav(&self, path: &Path) -> Result<()> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(path, spec)?;
        for &s in &self.samples {
            w.write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)?;
        }
        w.finalize()?;
        Ok(())
    }
}
