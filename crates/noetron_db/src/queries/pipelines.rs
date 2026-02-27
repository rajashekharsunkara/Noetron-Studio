//! Pipeline and pipeline stage queries

use anyhow::Result;
use rusqlite::params;
use uuid::Uuid;
use crate::Db;

#[derive(Debug, Clone)]
pub struct PipelineRow {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub domain: String,
    pub version: u32,
    pub ir_json: String,
    pub code_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct PipelineStageRow {
    pub id: String,
    pub pipeline_id: String,
    pub name: String,
    pub stage_type: String,
    pub params_json: String,
    pub position: u32,
}

impl Db {
    pub fn insert_pipeline(&self, project_id: &str, name: &str, domain: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO pipelines (id, project_id, name, domain) VALUES (?1, ?2, ?3, ?4)",
            params![id, project_id, name, domain],
        )?;
        Ok(id)
    }

    pub fn update_pipeline_ir(&self, pipeline_id: &str, ir_json: &str, code_path: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE pipelines SET ir_json = ?1, code_path = ?2,
             version = version + 1, updated_at = datetime('now')
             WHERE id = ?3",
            params![ir_json, code_path, pipeline_id],
        )?;
        Ok(())
    }

    pub fn get_pipeline(&self, id: &str) -> Result<Option<PipelineRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, name, domain, version, ir_json, code_path,
                    created_at, updated_at
             FROM pipelines WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], map_pipeline)?;
        Ok(rows.next().transpose()?)
    }

    pub fn list_pipelines(&self, project_id: &str) -> Result<Vec<PipelineRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, name, domain, version, ir_json, code_path,
                    created_at, updated_at
             FROM pipelines WHERE project_id = ?1 ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id], map_pipeline)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn upsert_pipeline_stages(&self, pipeline_id: &str, stages: &[PipelineStageRow]) -> Result<()> {
        self.conn.execute("DELETE FROM pipeline_stages WHERE pipeline_id = ?1", params![pipeline_id])?;
        for stage in stages {
            self.conn.execute(
                "INSERT INTO pipeline_stages (id, pipeline_id, name, stage_type, params_json, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![stage.id, pipeline_id, stage.name, stage.stage_type, stage.params_json, stage.position],
            )?;
        }
        Ok(())
    }

    pub fn list_pipeline_stages(&self, pipeline_id: &str) -> Result<Vec<PipelineStageRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pipeline_id, name, stage_type, params_json, position
             FROM pipeline_stages WHERE pipeline_id = ?1 ORDER BY position",
        )?;
        let rows = stmt.query_map(params![pipeline_id], |row| {
            Ok(PipelineStageRow {
                id: row.get(0)?,
                pipeline_id: row.get(1)?,
                name: row.get(2)?,
                stage_type: row.get(3)?,
                params_json: row.get(4)?,
                position: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}

fn map_pipeline(row: &rusqlite::Row<'_>) -> rusqlite::Result<PipelineRow> {
    Ok(PipelineRow {
        id: row.get(0)?,
        project_id: row.get(1)?,
        name: row.get(2)?,
        domain: row.get(3)?,
        version: row.get(4)?,
        ir_json: row.get(5)?,
        code_path: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}
