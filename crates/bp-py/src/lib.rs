use bp_core::catalog::Catalog;
use bp_core::geotag::resolve_location;
use bp_core::locate::{LocateOptions, locate_audio};
use pyo3::prelude::*;
use std::path::PathBuf;

/// Rank towers for a recording; returns the report as a JSON string (the Python wrapper decodes it).
#[pyfunction]
#[pyo3(signature = (path, dove_dir=None, near=None, start=0.0, end=None, top=10, use_geotag=false))]
#[allow(clippy::too_many_arguments)]
fn locate_json(
    py: Python<'_>,
    path: PathBuf,
    dove_dir: Option<PathBuf>,
    near: Option<String>,
    start: f64,
    end: Option<f64>,
    top: usize,
    use_geotag: bool,
) -> anyhow::Result<String> {
    py.detach(|| {
        let catalog = Catalog::load_or_embedded(dove_dir.as_deref())?;
        let location = resolve_location(&path, near.as_deref(), use_geotag)?;
        if let Some(w) = &location.warning {
            eprintln!("warning: {w}");
        }
        let clip = bp_core::audio::Audio::load(&path)?.slice(start, end);
        let opts = LocateOptions { location, top, ..LocateOptions::default() };
        Ok(serde_json::to_string(&locate_audio(&catalog, &clip, &opts))?)
    })
}

/// Runs the CLI with `argv` (argv[0] is the program name).
#[pyfunction]
fn main(argv: Vec<String>) -> anyhow::Result<()> {
    bp_cli::run(argv.into_iter().map(Into::into))
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(locate_json, m)?)?;
    m.add_function(wrap_pyfunction!(main, m)?)?;
    Ok(())
}
