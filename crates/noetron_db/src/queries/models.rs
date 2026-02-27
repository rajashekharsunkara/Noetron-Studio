//! Model and model version queries

use anyhow::Result;
use rusqlite::params;
use uuid::Uuid;
use crate::Db;

#[derive(Debug, Clone)]
pub struct ModelRow {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub domain: String,
    pub architecture: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct ModelVersionRow {
    pub id: String,
    pub model_id: String,
    pub version_number: u32,
    pub experiment_run_id: String,
    pub export_format: String,
    pub file_path: String,
    pub card_json: Option<String>,
    pub created_at: String,
}

impl Db {
    pub fn insert_model(
        &self,
        project_id: &str,
        name: &str,
        domain: &str,
        architecture: Option<&str>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO models (id, project_id, name, domain, architecture)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, project_id, name, domain, architecture],
        )?;
        Ok(id)
    }

    pub fn insert_model_version(
        &self,
        model_id: &str,
        experiment_run_id: &str,
        export_format: &str,
        file_path: &str,
        card_json: Option<&str>,
    ) -> Result<String> {
        let version_number = self.next_model_version(model_id)?;
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO model_versions
             (id, model_id, version_number, experiment_run_id, export_format, file_path, card_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, model_id, version_number, experiment_run_id, export_format, file_path, card_json],
        )?;
        Ok(id)
    }

    pub fn next_model_version(&self, model_id: &str) -> Result<u32> {
        let max: Option<u32> = self.conn.query_row(
            "SELECT MAX(version_number) FROM model_versions WHERE model_id = ?1",
            params![model_id],
            |row| row.get(0),
        )?;
        Ok(max.unwrap_or(0) + 1)
    }

    pub fn list_models(&self, project_id: &str) -> Result<Vec<ModelRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, name, domain, architecture, created_at
             FROM models WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(ModelRow {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                domain: row.get(3)?,
                architecture: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn list_model_versions(&self, model_id: &str) -> Result<Vec<ModelVersionRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, model_id, version_number, experiment_run_id,
                    export_format, file_path, card_json, created_at
             FROM model_versions WHERE model_id = ?1 ORDER BY version_number DESC",
        )?;
        let rows = stmt.query_map(params![model_id], |row| {
            Ok(ModelVersionRow {
                id: row.get(0)?,
                model_id: row.get(1)?,
                version_number: row.get(2)?,
                experiment_run_id: row.get(3)?,
                export_format: row.get(4)?,
                file_path: row.get(5)?,
                card_json: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn latest_model_version(&self, model_id: &str) -> Result<Option<ModelVersionRow>> {
        Ok(self.list_model_versions(model_id)?.into_iter().next())
    }
}
