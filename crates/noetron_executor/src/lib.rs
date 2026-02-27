//! noetron_executor — spawn Python pipelines and ingest run results into the DB.
//!
//! The executor:
//! 1. Calls `noetron_ir::sync::write_pipeline` to ensure the `.py` is up-to-date.
//! 2. Spawns `python -m noetron_runtime.pipeline.runner` as a subprocess.
//! 3. Streams stdout/stderr line by line (forwarded to the UI via events).
//! 4. On exit, reads the `metrics.json` / `params.json` produced by AutoLogger.
//! 5. Writes an `ExperimentRun` record to the SQLite DB via `noetron_db`.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;
use serde_json::Value;
use noetron_db::Db;

// ── Public types ─────────────────────────────────────────────────────────────

/// Callback invoked for each line of subprocess output.
pub type LogLine = Box<dyn Fn(bool, &str) + Send + Sync>;

/// Configuration for a single pipeline execution.
pub struct ExecutionRequest {
    /// Path to the pipeline `.py` file (already written by IR sync).
    pub script_path: PathBuf,
    /// `.aiproj/experiments` directory.
    pub experiments_dir: PathBuf,
    /// Optional human name for the run.
    pub run_name: Option<String>,
    /// Path to the project's SQLite DB (`.aiproj/db/project.db`).
    pub db_path: PathBuf,
    /// Database pipeline_id for this pipeline.
    pub pipeline_id: Uuid,
    /// Optional dataset version ID linked to this run.
    pub dataset_version_id: Option<Uuid>,
    /// Python interpreter to use (defaults to `python3`).
    pub python: Option<String>,
}

/// Result returned after a successful run.
pub struct ExecutionResult {
    pub run_id: Uuid,
    pub metrics: serde_json::Value,
    pub params: serde_json::Value,
    pub exit_code: i32,
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Execute a pipeline script and record the run in the DB.
///
/// `on_log(is_stderr, line)` is called for every line of subprocess output.
pub async fn execute(req: ExecutionRequest, on_log: LogLine) -> Result<ExecutionResult> {
    let python = req.python.as_deref().unwrap_or("python3");

    // Build subprocess command
    let mut cmd = Command::new(python);
    cmd.arg("-m")
        .arg("noetron_runtime.pipeline.runner")
        .arg("--script").arg(&req.script_path)
        .arg("--experiments-dir").arg(&req.experiments_dir);

    if let Some(name) = &req.run_name {
        cmd.arg("--run-name").arg(name);
    }

    // Add the project root to PYTHONPATH so noetron_runtime is importable
    if let Some(parent) = req.script_path.ancestors().nth(4) {
        let existing = std::env::var("PYTHONPATH").unwrap_or_default();
        let python_path = if existing.is_empty() {
            parent.to_string_lossy().into_owned()
        } else {
            format!("{}:{}", parent.display(), existing)
        };
        cmd.env("PYTHONPATH", python_path);
    }

    cmd.stdout(std::process::Stdio::piped())
       .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn Python subprocess")?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    // Stream stdout
    let mut run_id_str: Option<String> = None;
    {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(id) = line.strip_prefix("NOETRON_RUN_ID=") {
                run_id_str = Some(id.to_string());
            }
            on_log(false, &line);
        }
    }
    // Stream stderr
    {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            on_log(true, &line);
        }
    }

    let status = child.wait().await?;
    let exit_code = status.code().unwrap_or(-1);

    // Derive run_id from the output line; fall back to a fresh UUID
    let run_id: Uuid = run_id_str
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or_else(Uuid::new_v4);

    // Read metrics.json and params.json written by AutoLogger
    let run_dir = req.experiments_dir.join(run_id.to_string());
    let metrics = read_json_file(run_dir.join("metrics.json"))?;
    let params  = read_json_file(run_dir.join("params.json"))?;

    // Persist to the SQLite DB
    if exit_code == 0 {
        persist_run(&req.db_path, run_id, req.pipeline_id, req.dataset_version_id, &metrics, &params)?;
    }

    Ok(ExecutionResult { run_id, metrics, params, exit_code })
}

// ── DB persistence ────────────────────────────────────────────────────────────

fn persist_run(
    db_path: &Path,
    run_id: Uuid,
    pipeline_id: Uuid,
    dataset_version_id: Option<Uuid>,
    metrics: &Value,
    params: &Value,
) -> Result<()> {
    let db = Db::open(db_path)?;
    let metrics_json = serde_json::to_string(metrics)?;
    let params_json  = serde_json::to_string(params)?;

    db.conn.execute(
        "INSERT INTO experiment_runs
         (id, pipeline_id, dataset_version_id, status, metrics_json, params_json, started_at, finished_at)
         VALUES (?1, ?2, ?3, 'completed', ?4, ?5, datetime('now'), datetime('now'))",
        rusqlite::params![
            run_id.to_string(),
            pipeline_id.to_string(),
            dataset_version_id.map(|u| u.to_string()),
            metrics_json,
            params_json,
        ],
    ).context("Failed to insert experiment run")?;

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_json_file(path: PathBuf) -> Result<Value> {
    if !path.exists() {
        return Ok(Value::Object(serde_json::Map::new()));
    }
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("Reading {}", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("Parsing JSON from {}", path.display()))
}
