//! Codegen — deterministic IR → Python code generation.
//!
//! The same IR always produces the same Python file.
//! Generated files include a header comment instructing users to
//! edit via the No-Code view OR use Full Code mode.

use crate::entities::{DataConfigIr, FieldValue, PipelineIr, PipelineStage};

/// Generate a Python pipeline script from a `PipelineIr`.
pub fn generate_pipeline(ir: &PipelineIr) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "# Noetron Studio — generated pipeline: {}\n\
         # Domain: {}\n\
         # Edit in No-Code view (toggle top-left) or edit here directly in Full Code mode.\n\
         # Manual edits are preserved when switching back to No-Code view.\n\n",
        ir.name, ir.domain
    ));

    out.push_str("from __future__ import annotations\n\n");
    emit_imports(ir, &mut out);
    out.push('\n');
    out.push_str(&format!(
        "def run_pipeline({}):\n",
        pipeline_signature(ir)
    ));

    if ir.stages.is_empty() {
        out.push_str("    pass\n");
    } else {
        for stage in &ir.stages {
            emit_stage(stage, &mut out);
        }
        out.push_str("\n    return locals()\n");
    }

    out
}

/// Generate a TOML data config from a `DataConfigIr`.
pub fn generate_data_config(ir: &DataConfigIr) -> String {
    let mut out = String::new();
    out.push_str("# Noetron Studio — generated data config\n\
                  # Edit in No-Code view (Data Management panel) or directly here.\n\n");

    out.push_str(&format!("[source]\ntype = {:?}\npath = {:?}\n\n", ir.source_type, ir.source_path));

    out.push_str("[preprocessing]\nsteps = [\n");
    for step in &ir.preprocess_steps {
        let params: Vec<String> = step
            .params
            .iter()
            .map(|(k, v)| format!("{k} = {}", v.to_python_literal()))
            .collect();
        if params.is_empty() {
            out.push_str(&format!("    {{ name = {:?} }},\n", step.name));
        } else {
            out.push_str(&format!(
                "    {{ name = {:?}, {} }},\n",
                step.name,
                params.join(", ")
            ));
        }
    }
    out.push_str("]\n\n");

    out.push_str(&format!(
        "[split]\nval_size = {}\ntest_size = {}\nrandom_state = {}\n",
        ir.val_size, ir.test_size, ir.random_state
    ));

    out
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn pipeline_signature(ir: &PipelineIr) -> String {
    // Infer signature from first ingest stage if present
    if ir.stages.iter().any(|s| s.stage_type.starts_with("ingest")) {
        "source: str".to_string()
    } else {
        String::new()
    }
}

fn emit_imports(ir: &PipelineIr, out: &mut String) {
    let has_ingest = ir.stages.iter().any(|s| s.stage_type.starts_with("ingest"));
    let has_preprocess = ir.stages.iter().any(|s| {
        matches!(s.stage_type.as_str(), "drop_nulls" | "fill_nulls" | "encode_labels" | "normalize" | "drop_columns")
    });
    let has_train = ir.stages.iter().any(|s| s.stage_type.starts_with("train"));

    if has_ingest {
        out.push_str("from noetron_runtime.data import Ingestor\n");
    }
    if has_preprocess {
        out.push_str("from noetron_runtime.data import Preprocessor, Step\n");
    }
    if has_train {
        out.push_str("from noetron_runtime.model import Trainer\n");
        // Emit sklearn imports based on architecture param
        for stage in &ir.stages {
            if stage.stage_type.starts_with("train") {
                if let Some(FieldValue::String(arch)) = stage.params.get("architecture") {
                    if let Some(import) = sklearn_import(arch) {
                        out.push_str(&format!("{import}\n"));
                    }
                }
            }
        }
    }
}

fn emit_stage(stage: &PipelineStage, out: &mut String) {
    out.push_str(&format!("\n    # Stage {}: {}\n", stage.position + 1, stage.name));

    match stage.stage_type.as_str() {
        "ingest_csv" | "ingest" => {
            out.push_str("    df = Ingestor(source).load()\n");
        }
        "drop_nulls" | "fill_nulls" | "encode_labels" | "normalize" | "drop_columns" => {
            emit_preprocess_stage(stage, out);
        }
        "train_val_test_split" => {
            let target = stage.params.get("target_column")
                .map(|v| v.to_python_literal())
                .unwrap_or_else(|| r#""target""#.into());
            out.push_str(&format!(
                "    X_train, X_val, X_test, y_train, y_val, y_test = \\\n\
                 \x20\x20\x20\x20    pre.train_val_test_split(df, target_col={target})\n"
            ));
        }
        s if s.starts_with("train_") => {
            emit_train_stage(stage, out);
        }
        "evaluate" => {
            out.push_str("    metrics = trainer.fit_classify(X_train, y_train, X_val, y_val)\n");
        }
        _ => {
            // Unknown stage type — emit as a comment with raw params
            out.push_str(&format!("    # TODO: stage '{}' — edit in Full Code mode\n", stage.stage_type));
            out.push_str("    pass\n");
        }
    }
}

fn emit_preprocess_stage(stage: &PipelineStage, out: &mut String) {
    // Collect all preprocess steps to group them into one Preprocessor block
    let mut params_str = String::new();
    for (k, v) in &stage.params {
        params_str.push_str(&format!(", {k}={}", v.to_python_literal()));
    }
    out.push_str(&format!(
        "    pre = Preprocessor([Step({:?}{params_str})])\n    df = pre.run(df)\n",
        stage.stage_type
    ));
}

fn emit_train_stage(stage: &PipelineStage, out: &mut String) {
    let arch = stage.params.get("architecture")
        .map(|v| v.to_python_literal())
        .unwrap_or_else(|| r#""RandomForestClassifier""#.into());

    let arch_str = match stage.params.get("architecture") {
        Some(FieldValue::String(s)) => s.clone(),
        _ => "RandomForestClassifier".into(),
    };

    // Build constructor call from remaining params
    let ctor_params: Vec<String> = stage.params.iter()
        .filter(|(k, _)| *k != "architecture")
        .map(|(k, v)| format!("{k}={}", v.to_python_literal()))
        .collect();

    let ctor = format!("{arch_str}({})", ctor_params.join(", "));
    out.push_str(&format!(
        "    trainer = Trainer({ctor})\n\
         \x20\x20\x20\x20metrics = trainer.fit_classify(X_train, y_train, X_val, y_val)\n"
    ));
}

fn sklearn_import(arch: &str) -> Option<&'static str> {
    match arch {
        "RandomForestClassifier" | "RandomForestRegressor" =>
            Some("from sklearn.ensemble import RandomForestClassifier, RandomForestRegressor"),
        "LogisticRegression" =>
            Some("from sklearn.linear_model import LogisticRegression"),
        "SVC" | "SVR" =>
            Some("from sklearn.svm import SVC, SVR"),
        "XGBClassifier" | "XGBRegressor" =>
            Some("from xgboost import XGBClassifier, XGBRegressor"),
        "GradientBoostingClassifier" | "GradientBoostingRegressor" =>
            Some("from sklearn.ensemble import GradientBoostingClassifier, GradientBoostingRegressor"),
        _ => None,
    }
}
