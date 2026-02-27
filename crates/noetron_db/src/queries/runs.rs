//! Pipeline run and experiment run queries

use anyhow::Result;
use rusqlite::params;
use uuid::Uuid;
use crate::Db;

#[derive(Debug, Clone)]
pub struct PipelineRunRow {
    pub id: String,
    pub pipeline_id: String,
    pub dataset_version_id: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExperimentRunRow {
    pub id: String,
    pub pipeline_run_id: String,
    pub params_json: String,
    pub metrics_json: String,
    pub artifacts_path: Option<String>,
    pub created_at: String,
}

impl Db {
    pub fn insert_pipeline_run(&self, pipeline_id: &str, dataset_version_id: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO pipeline_runs (id, pipeline_id, dataset_version_id, status)
             VALUES (?1, ?2, ?3, 'queued')",
            params![id, pipeline_id, dataset_version_id],
        )?;
        Ok(id)
    }

    pub fn update_run_status(&self, run_id: &str, status: &str) -> Result<()> {
        if status == "completed" || status == "failed" || status == "cancelled" {
            self.conn.execute(
                "UPDATE pipeline_runs SET status = ?1, finished_at = datetime('now') WHERE id = ?2",
                params![status, run_id],
            )?;
        } else {
            self.conn.execute(
                "UPDATE pipeline_runs SET status = ?1 WHERE id = ?2",
                params![status, run_id],
            )?;
        }
        Ok(())
    }

    pub fn insert_experiment_run(
        &self,
        pipeline_run_id: &str,
        params_json: &str,
        metrics_json: &str,
        artifacts_path: Option<&str>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO experiment_runs (id, pipeline_run_id, params_json, metrics_json, artifacts_path)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, pipeline_run_id, params_json, metrics_json, artifacts_path],
        )?;
        Ok(id)
    }

    pub fn list_experiment_runs(&self, pipeline_id: &str, limit: u32) -> Result<Vec<ExperimentRunRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT er.id, er.pipeline_run_id, er.params_json, er.metrics_json,
                    er.artifacts_path, er.created_at
             FROM experiment_runs er
             JOIN pipeline_runs pr ON er.pipeline_run_id = pr.id
             WHERE pr.pipeline_id = ?1
             ORDER BY er.created_at DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![pipeline_id, limit], |row| {
            Ok(ExperimentRunRow {
                id: row.get(0)?,
                pipeline_run_id: row.get(1)?,
                params_json: row.get(2)?,
                metrics_json: row.get(3)?,
                artifacts_path: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_experiment_run(&self, id: &str) -> Result<Option<ExperimentRunRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pipeline_run_id, params_json, metrics_json, artifacts_path, created_at
             FROM experiment_runs WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(ExperimentRunRow {
                id: row.get(0)?,
                pipeline_run_id: row.get(1)?,
                params_json: row.get(2)?,
                metrics_json: row.get(3)?,
                artifacts_path: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    /// Returns all runs for a project ordered by metric value (descending).
    /// `metric_key` is the JSON key inside `metrics_json`, e.g. "accuracy".
    pub fn top_runs_by_metric(
        &self,
        pipeline_id: &str,
        metric_key: &str,
        limit: u32,
    ) -> Result<Vec<ExperimentRunRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT er.id, er.pipeline_run_id, er.params_json, er.metrics_json,
                    er.artifacts_path, er.created_at
             FROM experiment_runs er
             JOIN pipeline_runs pr ON er.pipeline_run_id = pr.id
             WHERE pr.pipeline_id = ?1
             ORDER BY json_extract(er.metrics_json, '$.' || ?2) DESC
             LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![pipeline_id, metric_key, limit], |row| {
            Ok(ExperimentRunRow {
                id: row.get(0)?,
                pipeline_run_id: row.get(1)?,
                params_json: row.get(2)?,
                metrics_json: row.get(3)?,
                artifacts_path: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}
