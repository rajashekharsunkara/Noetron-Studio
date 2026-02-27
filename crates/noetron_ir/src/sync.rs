//! Sync — keeps the IR in sync with files on disk.
//!
//! The sync engine watches the pipeline's `.py` file and data config `.toml`.
//! When either changes, it re-parses and emits an `IrChanged` event so the
//! no-code UI panels can re-render the affected fields.
//!
//! When the IR changes (user edits a form field), it regenerates the file.
//! Both directions are handled here.

use anyhow::Result;
use std::path::Path;
use crate::entities::{DataConfigIr, PipelineIr};
use crate::{codegen, parser};

/// Write the pipeline IR to the given `.py` file path.
pub fn write_pipeline(ir: &PipelineIr, py_path: &Path) -> Result<()> {
    let code = codegen::generate_pipeline(ir);
    std::fs::write(py_path, &code)?;
    tracing::debug!("Wrote pipeline '{}' to {}", ir.name, py_path.display());
    Ok(())
}

/// Read the pipeline `.py` file and parse it back into an IR.
pub fn read_pipeline(py_path: &Path, pipeline_name: &str, domain: &str) -> Result<PipelineIr> {
    let source = std::fs::read_to_string(py_path)?;
    Ok(parser::parse_pipeline(&source, pipeline_name, domain))
}

/// Write the data config IR to the given `.toml` file path.
pub fn write_data_config(ir: &DataConfigIr, toml_path: &Path) -> Result<()> {
    let text = codegen::generate_data_config(ir);
    std::fs::write(toml_path, &text)?;
    tracing::debug!("Wrote data config to {}", toml_path.display());
    Ok(())
}

/// Read the data config `.toml` file and parse it back into an IR.
pub fn read_data_config(toml_path: &Path) -> Result<DataConfigIr> {
    let source = std::fs::read_to_string(toml_path)?;
    Ok(parser::parse_data_config(&source))
}

/// Round-trip: write then re-read. Returns `true` if the re-parsed IR
/// is fully structured (no `Custom` fragments), `false` otherwise.
pub fn round_trip_check(ir: &PipelineIr, py_path: &Path) -> Result<bool> {
    write_pipeline(ir, py_path)?;
    let reparsed = read_pipeline(py_path, &ir.name, &ir.domain)?;
    Ok(reparsed.is_fully_structured())
}
