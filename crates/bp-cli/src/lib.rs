//! Argument parsing and output only; all work happens in bp-core. Shared with bp-py's `main()`.

use anyhow::{Context, Result};
use bp_core::audio::spectrum::PeakParams;
use bp_core::catalog::Catalog;
use bp_core::eval::{self, Case, Clip, CorpusEntry, SimEvalConfig};
use bp_core::geo::Region;
use bp_core::locate::{Extraction, LocateOptions, locate_audio};
use bp_core::matcher::MatchParams;
use bp_core::sim::{Pattern, SimParams, simulate};
use clap::{Args, Parser, Subcommand};
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "bellprint", about = "Identify the bell tower heard in a recording")]
struct Cli {
    /// Directory with a newer Dove's dove.csv and bells.csv; defaults to the snapshot built in.
    #[arg(long, global = true)]
    dove: Option<PathBuf>,
    /// TOML overriding matcher parameters (fields of `MatchParams`).
    #[arg(long, global = true)]
    matcher: Option<PathBuf>,
    /// TOML overriding long-term peak picking (fields of `PeakParams`).
    #[arg(long, global = true)]
    peaks: Option<PathBuf>,
    /// Line extraction: long_term | blows (experimental).
    #[arg(long, global = true, value_parser = parse_enum::<Extraction>, default_value = "long_term")]
    extraction: Extraction,
    /// Analyse only the clip's most bell-like windows (experimental).
    #[arg(long, global = true)]
    auto_window: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Args, Clone)]
struct SimArgs {
    #[arg(long, default_value_t = 8.0)]
    duration: f64,
    #[arg(long, default_value_t = 10.0)]
    snr_db: f64,
    #[arg(long, value_parser = parse_enum::<Pattern>, default_value = "rounds")]
    pattern: Pattern,
    #[arg(long, default_value_t = 8.0)]
    speed_sd_cents: f64,
}

#[derive(Subcommand)]
enum Cmd {
    /// Rank the towers that could have produced a recording.
    Locate {
        audio: PathBuf,
        /// Restrict to a circle: "lat,lon[,radius_km]".
        #[arg(long)]
        near: Option<String>,
        #[arg(long, default_value_t = 0.0)]
        start: f64,
        #[arg(long)]
        end: Option<f64>,
        #[arg(long, default_value_t = 10)]
        top: usize,
        #[arg(long)]
        json: bool,
    },
    /// Render a synthetic recording of one Dove ring.
    Sim {
        ring_id: u32,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[command(flatten)]
        sim: SimArgs,
    },
    /// Simulate random rings and report how often the true tower ranks first.
    EvalSim {
        #[arg(long, default_value_t = 200)]
        n: usize,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long)]
        country: Option<String>,
        #[arg(long, default_value_t = 3)]
        min_bells: usize,
        #[arg(long)]
        out: Option<PathBuf>,
        #[command(flatten)]
        sim: SimArgs,
    },
    /// Write simulated recordings plus a corpus-style manifest (for codec tests via eval-corpus).
    SimBatch {
        #[arg(long, default_value_t = 100)]
        n: usize,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long)]
        out: PathBuf,
        #[command(flatten)]
        sim: SimArgs,
    },
    /// Locate every recording in the real corpus.
    EvalCorpus {
        /// A manifest file, or a directory whose `manifest*.toml` files are all used.
        #[arg(long, default_value = "data/corpus")]
        manifest: PathBuf,
        #[arg(long, default_value = "data/corpus/audio")]
        audio: PathBuf,
        /// Resample every clip to this rate before locating (sample-rate robustness check).
        #[arg(long)]
        resample_hz: Option<u32>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Re-rank the lines an eval saved (`<out>/clips.json`) with the current matcher, without audio.
    Replay {
        clips: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, String> {
    serde_json::from_value(serde_json::Value::String(s.replace('-', "_"))).map_err(|e| e.to_string())
}

fn sim_params(a: &SimArgs) -> SimParams {
    SimParams {
        duration_s: a.duration,
        snr_db: a.snr_db,
        pattern: a.pattern,
        speed_sd_cents: a.speed_sd_cents,
        ..SimParams::default()
    }
}

fn toml_params<T: Default + serde::de::DeserializeOwned>(path: Option<&PathBuf>) -> Result<T> {
    let Some(p) = path else { return Ok(T::default()) };
    let text = std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
    Ok(toml::from_str(&text)?)
}

/// Writes the summary, and with `out` the cases and (if given) the clips' lines for `replay`.
fn emit(title: &str, cases: &[Case], clips: Option<&[Clip]>, out: Option<PathBuf>) -> Result<()> {
    let summary = eval::summarize(cases);
    let md = eval::markdown(title, &summary, cases);
    match out {
        Some(dir) => {
            std::fs::create_dir_all(&dir)?;
            std::fs::write(dir.join("summary.md"), &md)?;
            std::fs::write(dir.join("cases.json"), serde_json::to_string_pretty(cases)?)?;
            if let Some(clips) = clips {
                std::fs::write(dir.join("clips.json"), serde_json::to_string(clips)?)?;
            }
            println!("{}", md.lines().take(3).collect::<Vec<_>>().join("\n"));
            println!("wrote {}", dir.display());
        }
        None => print!("{md}"),
    }
    Ok(())
}

pub fn run<I: IntoIterator<Item = OsString>>(args: I) -> Result<()> {
    let cli = Cli::parse_from(args);
    let catalog = Catalog::load_or_embedded(cli.dove.as_deref()).context("loading Dove data (see --dove)")?;
    let base = LocateOptions {
        matcher: toml_params::<MatchParams>(cli.matcher.as_ref())?,
        peaks: toml_params::<PeakParams>(cli.peaks.as_ref())?,
        extraction: cli.extraction,
        auto_window: cli.auto_window,
        ..LocateOptions::default()
    };
    let eval_opts = LocateOptions { top: 50, ..base.clone() };
    match cli.cmd {
        Cmd::Locate { audio, near, start, end, top, json } => {
            let region = near.as_deref().map(Region::parse).transpose()?;
            let opts = LocateOptions { region, top, ..base };
            let clip = bp_core::audio::Audio::load(&audio)?.slice(start, end);
            let report = locate_audio(&catalog, &clip, &opts);
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("{:.1} s, {} spectral lines", report.duration_s, report.peaks.len());
                if let (Some(c), Some(top)) = (report.confidence, report.matches.first()) {
                    println!("confidence in the top pick: {} (lead {:.1} nats)", c.label(), top.margin);
                }
                for (i, m) in report.matches.iter().enumerate() {
                    let r = &m.ring;
                    println!(
                        "{:>2}. p={:.3} lead {:.1} {} ({}), {} [{} bells, {} heard, shift {:+.0}c] tower {} {:.5},{:.5}",
                        i + 1, m.probability, m.margin, r.place, r.dedication, r.county, r.nominals.len(),
                        m.bells_heard, m.shift_cents, r.tower_id, r.lat, r.lon
                    );
                }
            }
        }
        Cmd::Sim { ring_id, out, seed, sim } => {
            let ring = catalog.ring(ring_id).with_context(|| format!("no ring {ring_id}"))?;
            let (audio, truth) = simulate(ring, &sim_params(&sim), seed);
            audio.write_wav(&out)?;
            println!("{}", serde_json::to_string(&truth)?);
        }
        Cmd::EvalSim { n, seed, country, min_bells, out, sim } => {
            let cfg = SimEvalConfig { n, seed, sim: sim_params(&sim), complete_only: true, min_bells, country };
            let (cases, clips): (Vec<_>, Vec<_>) = eval::eval_sim(&catalog, &cfg, &eval_opts).into_iter().unzip();
            emit("Simulated recordings", &cases, Some(&clips), out)?;
        }
        Cmd::SimBatch { n, seed, out, sim } => {
            let cfg = SimEvalConfig { n, seed, sim: sim_params(&sim), complete_only: true, min_bells: 3, country: None };
            std::fs::create_dir_all(&out)?;
            let mut entries = Vec::new();
            for (ring, s) in eval::pick_rings(&catalog, &cfg) {
                let id = format!("sim-{}-{s}", ring.ring_id);
                let (audio, _) = simulate(ring, &cfg.sim, s);
                audio.write_wav(&out.join(format!("{id}.wav")))?;
                entries.push(CorpusEntry {
                    url: String::new(),
                    id,
                    tower_id: ring.tower_id,
                    ringing: format!("{:?}", cfg.sim.pattern).to_lowercase(),
                    confidence: "sim".into(),
                    bells: ring.nominals.len() as u32,
                });
            }
            std::fs::write(out.join("manifest.toml"), eval::manifest_toml(entries)?)?;
            println!("wrote {n} recordings to {}", out.display());
        }
        Cmd::EvalCorpus { manifest, audio, resample_hz, out } => {
            let entries =
                if manifest.is_dir() { eval::load_manifests(&manifest)? } else { eval::load_manifest(&manifest)? };
            let (cases, clips): (Vec<_>, Vec<_>) =
                eval::eval_corpus(&catalog, &entries, &audio, resample_hz, &eval_opts).into_iter().unzip();
            emit("Real corpus", &cases, Some(&clips), out)?;
        }
        Cmd::Replay { clips, out } => {
            let text = std::fs::read_to_string(&clips).with_context(|| format!("reading {}", clips.display()))?;
            let clips: Vec<Clip> = serde_json::from_str(&text)?;
            emit("Replay", &eval::replay(&catalog, &clips, &eval_opts), None, out)?;
        }
    }
    Ok(())
}
