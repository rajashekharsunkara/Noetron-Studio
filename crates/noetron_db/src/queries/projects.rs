//! Project queries

use anyhow::Result;
use rusqlite::params;
use uuid::Uuid;
use crate::Db;

#[derive(Debug, Clone)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Db {
    pub fn insert_project(&self, name: &str, domain: &str, description: Option<&str>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        self.conn.execute(
            "INSERT INTO projects (id, name, domain, description) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, domain, description],
        )?;
        Ok(id)
    }

    pub fn get_project(&self, id: &str) -> Result<Option<ProjectRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, domain, description, created_at, updated_at FROM projects WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(ProjectRow {
                id: row.get(0)?,
                name: row.get(1)?,
                domain: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, domain, description, created_at, updated_at FROM projects ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProjectRow {
                id: row.get(0)?,
                name: row.get(1)?,
                domain: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}
