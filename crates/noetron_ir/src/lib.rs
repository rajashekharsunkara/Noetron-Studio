//! noetron_ir — Intermediate Representation
//!
//! The IR is the single source of truth for every Noetron entity.
//! Both the no-code form view and the full code view are renderings of the IR.
//!
//! - `entities`: typed Rust structs for all platform entities
//! - `codegen`: IR → Python (deterministic, lossless)
//! - `parser`: Python → IR (best-effort; unknown patterns → `Custom`)
//! - `sync`: background sync loop between IR and files on disk

pub mod entities;
pub mod codegen;
pub mod parser;
pub mod sync;

pub use entities::*;

