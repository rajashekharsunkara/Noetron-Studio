//! noetron_db — Local SQLite Project Database
//!
//! All Noetron Studio project state lives in `.aiproj/db/project.db`.
//! This crate owns the schema, migrations, and typed query helpers.

pub mod db;
pub mod migrations;
pub mod queries;

pub use db::Db;
