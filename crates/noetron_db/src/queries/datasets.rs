//! Dataset and dataset version queries

use anyhow::Result;
use rusqlite::params;
use uuid::Uuid;
use crate::Db;

#[derive(Debug, Clone)]
pub struct DatasetRow {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub domain: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct DatasetVersionRow {
    pub id: String,
    pub dataset_id: String,
    pub version_number: u32,
    pub content_hash: String,
    pub storage_path: String,
    pub diff_summary: Option<String>,
    pub profile_json: Option<String>,
    pub created_at: String,
}

impl Db {
    pub fn insert_dataset(&self, project_id: &str, name: &str, domain: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO datasets (id, project_id, name, domain) VALUES (?1, ?2, ?3, ?4)",
            params![id, project_id, name, domain],
        )?;
        Ok(id)
    }

    pub fn insert_dataset_version(
        &self,
        dataset_id: &str,
        version_number: u32,
        content_hash: &str,
        storage_path: &str,
        diff_summary: Option<&str>,
        profile_json: Option<&str>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO dataset_versions
             (id, dataset_id, version_number, content_hash, storage_path, diff_summary, profile_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, dataset_id, version_number, content_hash, storage_path, diff_summary, profile_json],
        )?;
        Ok(id)
    }

    pub fn next_version_number(&self, dataset_id: &str) -> Result<u32> {
        let max: Option<u32> = self.conn.query_row(
            "SELECT MAX(version_number) FROM dataset_versions WHERE dataset_id = ?1",
            params![dataset_id],
            |row| row.get(0),
        )?;
        Ok(max.unwrap_or(0) + 1)
    }

    pub fn list_dataset_versions(&self, dataset_id: &str) -> Result<Vec<DatasetVersionRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, dataset_id, version_number, content_hash, storage_path,
                    diff_summary, profile_json, created_at
             FROM dataset_versions WHERE dataset_id = ?1
             ORDER BY version_number DESC",
        )?;
        let rows = stmt.query_map(params![dataset_id], |row| {
            Ok(DatasetVersionRow {
                id: row.get(0)?,
                dataset_id: row.get(1)?,
                version_number: row.get(2)?,
                content_hash: row.get(3)?,
                storage_path: row.get(4)?,
                diff_summary: row.get(5)?,
                profile_json: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn latest_dataset_version(&self, dataset_id: &str) -> Result<Option<DatasetVersionRow>> {
        Ok(self.list_dataset_versions(dataset_id)?.into_iter().next())
    }
}
