//! Experiment Panel — shows all runs for a pipeline with metrics.

use gpui::{Context, IntoElement, Render, Window};
use ui::prelude::*;

/// A single run row for display.
#[derive(Debug, Clone)]
pub struct RunRow {
    pub run_id: String,
    pub run_name: String,
    pub status: String,
    pub metrics: serde_json::Value,
    pub started_at: String,
}

pub struct ExperimentPanel {
    pub runs: Vec<RunRow>,
    pub selected_run: Option<String>,
}

impl ExperimentPanel {
    pub fn new(runs: Vec<RunRow>) -> Self {
        Self { runs, selected_run: None }
    }
}

impl Render for ExperimentPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("experiment-panel")
            .p_4()
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .child(Label::new("🧪 Experiments").size(LabelSize::Large))
                    .child(Label::new(format!("{} runs", self.runs.len())).size(LabelSize::Small)),
            )
            .child(if self.runs.is_empty() {
                Label::new("No runs yet. Click ▶ Run Pipeline to start.")
                    .size(LabelSize::Small)
                    .into_any_element()
            } else {
                // Runs table
                v_flex()
                    .gap_1()
                    .child(
                        // Table header
                        h_flex()
                            .gap_3()
                            .child(Label::new("Run").size(LabelSize::Small))
                            .child(Label::new("Status").size(LabelSize::Small))
                            .child(Label::new("Metrics").size(LabelSize::Small))
                            .child(Label::new("Started").size(LabelSize::Small)),
                    )
                    .children(self.runs.iter().map(|row| {
                        let metrics_str = metrics_summary(&row.metrics);
                        let is_selected = self.selected_run.as_deref() == Some(&row.run_id);

                        h_flex()
                            .id(ElementId::from(row.run_id.clone()))
                            .gap_3()
                            .py_1()
                            .when(is_selected, |el| el.font_weight(gpui::FontWeight::BOLD))
                            .child(Label::new(row.run_name.clone()))
                            .child(Label::new(status_badge(&row.status)).size(LabelSize::Small))
                            .child(Label::new(metrics_str).size(LabelSize::Small))
                            .child(Label::new(row.started_at.clone()).size(LabelSize::Small))
                    }))
                    .into_any_element()
            })
    }
}

fn metrics_summary(metrics: &serde_json::Value) -> String {
    if let Some(obj) = metrics.as_object() {
        obj.iter()
            .take(3)
            .map(|(k, v)| {
                let val = if let Some(f) = v.as_f64() {
                    format!("{f:.4}")
                } else {
                    v.to_string()
                };
                format!("{k}={val}")
            })
            .collect::<Vec<_>>()
            .join("  ")
    } else {
        String::new()
    }
}

fn status_badge(status: &str) -> &'static str {
    match status {
        "completed" => "✓",
        "running" => "⟳",
        "failed" => "✗",
        _ => "?",
    }
}
