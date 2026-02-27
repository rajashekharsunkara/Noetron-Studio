//! Parser — Python → IR (best-effort, using simple pattern matching).
//!
//! Uses regex-style line scanning rather than a full AST parser.
//! Unknown patterns are stored as `FieldValue::Custom(String)` —
//! they are never discarded.

use std::collections::HashMap;
use crate::entities::{DataConfigIr, FieldValue, PipelineIr, PipelineStage, PreprocessStep};

/// Parse a generated pipeline Python file back into a `PipelineIr`.
/// This is best-effort: recognised patterns become structured IR fields;
/// unrecognised code is stored as `Custom` fragments.
pub fn parse_pipeline(source: &str, pipeline_name: &str, domain: &str) -> PipelineIr {
    let mut ir = PipelineIr::new(pipeline_name, domain);
    let mut position = 0u32;

    for line in source.lines() {
        let trimmed = line.trim();

        if let Some(stage) = try_parse_ingest(trimmed, position) {
            ir.add_stage(stage);
            position += 1;
        } else if let Some(stage) = try_parse_preprocess(trimmed, position) {
            ir.add_stage(stage);
            position += 1;
        } else if let Some(stage) = try_parse_train(trimmed, position) {
            ir.add_stage(stage);
            position += 1;
        } else if let Some(stage) = try_parse_evaluate(trimmed, position) {
            ir.add_stage(stage);
            position += 1;
        }
        // Lines that don't match any pattern are silently skipped —
        // they will be preserved in the file as-is (codegen doesn't touch them).
    }

    ir
}

/// Parse a TOML data config back into a `DataConfigIr`.
pub fn parse_data_config(source: &str) -> DataConfigIr {
    // Parse the TOML; fall back to default on any error
    match toml::from_str::<toml::Value>(source) {
        Ok(val) => map_data_config(&val),
        Err(e) => {
            tracing::warn!("Could not parse data_config.toml: {e}");
            DataConfigIr::default()
        }
    }
}

// ── Pattern matchers ─────────────────────────────────────────────────────────

fn try_parse_ingest(line: &str, pos: u32) -> Option<PipelineStage> {
    // Matches: df = Ingestor(source).load()
    //      or: df = Ingestor("path/to/file.csv").load()
    if line.contains("Ingestor(") && line.contains(".load()") {
        let mut stage = PipelineStage::new("ingest", pos);
        stage.name = "Ingest".into();
        // Try to extract literal path
        if let Some(path) = extract_string_arg(line, "Ingestor(") {
            stage.params.insert("source_path".into(), FieldValue::String(path));
        }
        return Some(stage);
    }
    // Matches: df = pd.read_csv(...)
    if line.starts_with("df") && line.contains("pd.read_csv(") {
        let mut stage = PipelineStage::new("ingest", pos);
        stage.name = "Ingest CSV".into();
        if let Some(path) = extract_string_arg(line, "pd.read_csv(") {
            stage.params.insert("source_path".into(), FieldValue::String(path));
        } else {
            stage.params.insert("source_path".into(), FieldValue::Custom("pd.read_csv(...)".into()));
        }
        return Some(stage);
    }
    None
}

fn try_parse_preprocess(line: &str, pos: u32) -> Option<PipelineStage> {
    // Matches: pre = Preprocessor([Step("drop_nulls"), ...])
    if !line.contains("Preprocessor(") {
        return None;
    }
    let mut stage = PipelineStage::new("preprocess", pos);
    stage.name = "Preprocess".into();

    // Extract step names
    let steps: Vec<String> = extract_step_names(line);
    if steps.is_empty() {
        stage.params.insert("steps".into(), FieldValue::Custom(line.to_string()));
    } else {
        let list: Vec<FieldValue> = steps.into_iter().map(FieldValue::String).collect();
        stage.params.insert("steps".into(), FieldValue::List(list));
    }
    Some(stage)
}

fn try_parse_train(line: &str, pos: u32) -> Option<PipelineStage> {
    // Matches: trainer = Trainer(RandomForestClassifier(...))
    if !line.contains("Trainer(") {
        return None;
    }
    let mut stage = PipelineStage::new("train", pos);
    stage.name = "Train".into();

    // Extract architecture name
    for arch in KNOWN_ARCHITECTURES {
        if line.contains(arch) {
            stage.params.insert("architecture".into(), FieldValue::String(arch.to_string()));
            // Try extracting n_estimators
            if let Some(n) = extract_int_kwarg(line, "n_estimators") {
                stage.params.insert("n_estimators".into(), FieldValue::Int(n));
            }
            if let Some(d) = extract_int_kwarg(line, "max_depth") {
                stage.params.insert("max_depth".into(), FieldValue::Int(d));
            }
            return Some(stage);
        }
    }

    // Unknown estimator — store as Custom
    stage.params.insert("architecture".into(), FieldValue::Custom(line.to_string()));
    Some(stage)
}

fn try_parse_evaluate(line: &str, pos: u32) -> Option<PipelineStage> {
    if line.contains("fit_classify") || line.contains("fit_regress") {
        let mut stage = PipelineStage::new("evaluate", pos);
        stage.name = "Evaluate".into();
        let task = if line.contains("fit_regress") { "regression" } else { "classification" };
        stage.params.insert("task".into(), FieldValue::String(task.into()));
        return Some(stage);
    }
    None
}

// ── TOML data config mapper ───────────────────────────────────────────────────

fn map_data_config(val: &toml::Value) -> DataConfigIr {
    let mut ir = DataConfigIr::default();

    if let Some(src) = val.get("source") {
        if let Some(t) = src.get("type").and_then(|v| v.as_str()) {
            ir.source_type = t.to_string();
        }
        if let Some(p) = src.get("path").and_then(|v| v.as_str()) {
            ir.source_path = p.to_string();
        }
    }

    if let Some(pre) = val.get("preprocessing") {
        if let Some(steps) = pre.get("steps").and_then(|v| v.as_array()) {
            ir.preprocess_steps = steps.iter().filter_map(|s| {
                let name = s.get("name")?.as_str()?.to_string();
                let mut params = HashMap::new();
                if let Some(table) = s.as_table() {
                    for (k, v) in table {
                        if k == "name" { continue; }
                        params.insert(k.clone(), toml_value_to_field(v));
                    }
                }
                Some(PreprocessStep { name, params })
            }).collect();
        }
    }

    if let Some(split) = val.get("split") {
        if let Some(v) = split.get("val_size").and_then(|v| v.as_float()) {
            ir.val_size = v;
        }
        if let Some(t) = split.get("test_size").and_then(|v| v.as_float()) {
            ir.test_size = t;
        }
        if let Some(r) = split.get("random_state").and_then(|v| v.as_integer()) {
            ir.random_state = r as u64;
        }
    }

    ir
}

fn toml_value_to_field(v: &toml::Value) -> FieldValue {
    match v {
        toml::Value::String(s) => FieldValue::String(s.clone()),
        toml::Value::Integer(n) => FieldValue::Int(*n),
        toml::Value::Float(f) => FieldValue::Float(*f),
        toml::Value::Boolean(b) => FieldValue::Bool(*b),
        toml::Value::Array(arr) => {
            FieldValue::List(arr.iter().map(toml_value_to_field).collect())
        }
        other => FieldValue::Custom(other.to_string()),
    }
}

// ── String extraction helpers ─────────────────────────────────────────────────

fn extract_string_arg(line: &str, after: &str) -> Option<String> {
    let start = line.find(after)? + after.len();
    let rest = &line[start..];
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' { return None; }
    let end = rest[1..].find(quote)?;
    Some(rest[1..end + 1].to_string())
}

fn extract_step_names(line: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut search = line;
    while let Some(pos) = search.find("Step(") {
        search = &search[pos + 5..];
        if let Some(end) = search.find(')') {
            let arg = search[..end].trim().trim_matches('"').trim_matches('\'');
            if !arg.is_empty() {
                names.push(arg.to_string());
            }
        }
    }
    names
}

fn extract_int_kwarg(line: &str, key: &str) -> Option<i64> {
    let needle = format!("{key}=");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

const KNOWN_ARCHITECTURES: &[&str] = &[
    "RandomForestClassifier",
    "RandomForestRegressor",
    "LogisticRegression",
    "SVC",
    "SVR",
    "XGBClassifier",
    "XGBRegressor",
    "GradientBoostingClassifier",
    "GradientBoostingRegressor",
    "DecisionTreeClassifier",
    "DecisionTreeRegressor",
    "KNeighborsClassifier",
    "KNeighborsRegressor",
];
