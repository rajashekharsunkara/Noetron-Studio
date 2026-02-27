//! Schema migrations — applied in order, tracked in `_migrations` table.

use anyhow::Result;
use rusqlite::Connection;

const MIGRATIONS: &[(&str, &str)] = &[
    ("001_init", MIGRATION_001),
];

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )?;

    for (id, sql) in MIGRATIONS {
        let applied: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM _migrations WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )?;
        if !applied {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT INTO _migrations (id) VALUES (?1)",
                rusqlite::params![id],
            )?;
            tracing::info!("Applied DB migration: {id}");
        }
    }
    Ok(())
}

// ── Migration 001 — Full Schema ───────────────────────────────────────────────

const MIGRATION_001: &str = "
-- Projects
CREATE TABLE projects (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    domain      TEXT NOT NULL,
    description TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Datasets
CREATE TABLE datasets (
    id         TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    domain     TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE dataset_versions (
    id             TEXT PRIMARY KEY,
    dataset_id     TEXT NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    content_hash   TEXT NOT NULL,
    storage_path   TEXT NOT NULL,
    diff_summary   TEXT,
    profile_json   TEXT,
    created_at     TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(dataset_id, version_number)
);

-- Pipelines
CREATE TABLE pipelines (
    id         TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    domain     TEXT NOT NULL,
    version    INTEGER NOT NULL DEFAULT 1,
    ir_json    TEXT NOT NULL DEFAULT '{}',
    code_path  TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE pipeline_stages (
    id          TEXT PRIMARY KEY,
    pipeline_id TEXT NOT NULL REFERENCES pipelines(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    stage_type  TEXT NOT NULL,
    params_json TEXT NOT NULL DEFAULT '{}',
    position    INTEGER NOT NULL
);

-- Pipeline Runs
CREATE TABLE pipeline_runs (
    id                 TEXT PRIMARY KEY,
    pipeline_id        TEXT NOT NULL REFERENCES pipelines(id),
    dataset_version_id TEXT NOT NULL REFERENCES dataset_versions(id),
    status             TEXT NOT NULL DEFAULT 'queued',
    started_at         TEXT NOT NULL DEFAULT (datetime('now')),
    finished_at        TEXT
);

-- Experiment Runs (auto-logged)
CREATE TABLE experiment_runs (
    id              TEXT PRIMARY KEY,
    pipeline_run_id TEXT NOT NULL REFERENCES pipeline_runs(id) ON DELETE CASCADE,
    params_json     TEXT NOT NULL DEFAULT '{}',
    metrics_json    TEXT NOT NULL DEFAULT '{}',
    artifacts_path  TEXT,
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Models
CREATE TABLE models (
    id           TEXT PRIMARY KEY,
    project_id   TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,
    domain       TEXT NOT NULL,
    architecture TEXT,
    created_at   TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE model_versions (
    id                TEXT PRIMARY KEY,
    model_id          TEXT NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    version_number    INTEGER NOT NULL,
    experiment_run_id TEXT NOT NULL REFERENCES experiment_runs(id),
    export_format     TEXT NOT NULL,
    file_path         TEXT NOT NULL,
    card_json         TEXT,
    created_at        TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(model_id, version_number)
);

-- Indices
CREATE INDEX idx_dataset_versions_dataset     ON dataset_versions(dataset_id);
CREATE INDEX idx_pipeline_stages_pipeline     ON pipeline_stages(pipeline_id);
CREATE INDEX idx_pipeline_runs_pipeline       ON pipeline_runs(pipeline_id);
CREATE INDEX idx_pipeline_runs_status         ON pipeline_runs(status);
CREATE INDEX idx_experiment_runs_pipeline_run ON experiment_runs(pipeline_run_id);
CREATE INDEX idx_model_versions_model         ON model_versions(model_id);
";
