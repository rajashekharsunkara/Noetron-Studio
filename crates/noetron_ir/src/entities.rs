//! IR entity definitions — the canonical in-memory types for all Noetron entities.
//!
//! These map 1:1 to the database schema and are the source of truth for codegen.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Field Value ───────────────────────────────────────────────────────────────

/// A value in a pipeline stage parameter or experiment hyperparameter.
/// `Custom` holds code fragments the parser couldn't map structurally —
/// they are preserved verbatim in codegen and shown as "⚠ custom" in the UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<FieldValue>),
    /// Raw Python expression — preserved verbatim, not editable in form view.
    Custom(String),
}

impl FieldValue {
    pub fn is_custom(&self) -> bool {
        matches!(self, FieldValue::Custom(_))
    }

    pub fn to_python_literal(&self) -> String {
        match self {
            FieldValue::String(s) => format!("{s:?}"),
            FieldValue::Int(n) => n.to_string(),
            FieldValue::Float(f) => {
                if f.fract() == 0.0 { format!("{f:.1}") } else { f.to_string() }
            }
            FieldValue::Bool(b) => if *b { "True".into() } else { "False".into() },
            FieldValue::List(items) => {
                let inner: Vec<_> = items.iter().map(|v| v.to_python_literal()).collect();
                format!("[{}]", inner.join(", "))
            }
            FieldValue::Custom(raw) => raw.clone(),
        }
    }
}

// ── Pipeline Stage ────────────────────────────────────────────────────────────

/// A single step in a pipeline (maps to one card in the stage-lane editor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStage {
    pub id: Uuid,
    pub name: String,
    /// e.g. "ingest_csv", "drop_nulls", "train_random_forest"
    pub stage_type: String,
    pub params: HashMap<String, FieldValue>,
    /// Position in the stage lane (0-indexed, left to right).
    pub position: u32,
}

impl PipelineStage {
    pub fn new(stage_type: impl Into<String>, position: u32) -> Self {
        let st: String = stage_type.into();
        Self {
            id: Uuid::new_v4(),
            name: st.clone(),
            stage_type: st,
            params: HashMap::new(),
            position,
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, value: FieldValue) -> Self {
        self.params.insert(key.into(), value);
        self
    }
}

// ── Pipeline IR ───────────────────────────────────────────────────────────────

/// The full pipeline definition — what the stage-lane editor and codegen work from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineIr {
    pub id: Uuid,
    pub name: String,
    pub domain: String,
    pub stages: Vec<PipelineStage>,
}

impl PipelineIr {
    pub fn new(name: impl Into<String>, domain: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            domain: domain.into(),
            stages: Vec::new(),
        }
    }

    pub fn add_stage(&mut self, stage: PipelineStage) {
        self.stages.push(stage);
        self.stages.sort_by_key(|s| s.position);
    }

    /// Returns whether every field in every stage has a known (non-Custom) value.
    pub fn is_fully_structured(&self) -> bool {
        self.stages.iter().all(|s| {
            s.params.values().all(|v| !v.is_custom())
        })
    }
}

// ── Data Config IR ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessStep {
    pub name: String,
    pub params: HashMap<String, FieldValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfigIr {
    pub source_type: String,   // "csv", "json", "parquet", "image_folder", ...
    pub source_path: String,
    pub preprocess_steps: Vec<PreprocessStep>,
    pub val_size: f64,
    pub test_size: f64,
    pub random_state: u64,
}

impl Default for DataConfigIr {
    fn default() -> Self {
        Self {
            source_type: "csv".into(),
            source_path: String::new(),
            preprocess_steps: Vec::new(),
            val_size: 0.15,
            test_size: 0.15,
            random_state: 42,
        }
    }
}

// ── Model Config IR ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigIr {
    pub architecture: String,
    pub hyperparams: HashMap<String, FieldValue>,
    pub export_format: String,
}
