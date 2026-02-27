//! Pipeline Panel — stage-lane drag-and-drop pipeline editor.
//!
//! Each stage is a card showing the stage type, name, and a compact param list.
//! Stages with `FieldValue::Custom` params show a "⚠ custom — edit in Full Code" badge.

use gpui::{Context, IntoElement, Render, Window};
use noetron_ir::entities::{FieldValue, PipelineIr};
use ui::prelude::*;

pub struct PipelinePanel {
    pub pipeline: PipelineIr,
}

impl PipelinePanel {
    pub fn new(pipeline: PipelineIr) -> Self {
        Self { pipeline }
    }
}

impl Render for PipelinePanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("pipeline-panel")
            .p_4()
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .child(Label::new("⚙ Pipeline").size(LabelSize::Large))
                    .child(Label::new(self.pipeline.name.clone()))
                    .child(
                        Button::new("add-stage", "+ Stage")
                            .style(ButtonStyle::Subtle)
                            .size(ButtonSize::Compact),
                    ),
            )
            // Stage lane — horizontal scroll
            .child(
                h_flex()
                    .id("stage-lane")
                    .gap_3()
                    .overflow_x_scroll()
                    .py_2()
                    .children(self.pipeline.stages.iter().map(|stage| {
                        let has_custom = stage.params.values().any(|v| v.is_custom());

                        v_flex()
                            .id(ElementId::from(stage.id.to_string()))
                            .w_40()
                            .p_3()
                            .gap_1()
                            .rounded_lg()
                            .border_1()
                            // Stage header
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Label::new(stage_icon(&stage.stage_type))
                                            .size(LabelSize::Small),
                                    )
                                    .child(Label::new(stage.name.clone())),
                            )
                            // Params list (compact)
                            .children(stage.params.iter().map(|(key, val)| {
                                let display = match val {
                                    FieldValue::Custom(_) => "⚠ custom".to_string(),
                                    other => other.to_python_literal(),
                                };
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Label::new(format!("{key}:"))
                                            .size(LabelSize::Small),
                                    )
                                    .child(
                                        Label::new(display).size(LabelSize::Small),
                                    )
                            }))
                            // Custom warning badge
                            .when(has_custom, |el| {
                                el.child(
                                    Label::new("edit in Full Code →")
                                        .size(LabelSize::Small),
                                )
                            })
                    })),
            )
            // Run button
            .child(
                h_flex()
                    .gap_2()
                    .child(
                        Button::new("run-pipeline", "▶ Run Pipeline")
                            .style(ButtonStyle::Filled)
                            .size(ButtonSize::Compact),
                    )
                    .child(
                        Button::new("view-code", "</> View Code")
                            .style(ButtonStyle::Subtle)
                            .size(ButtonSize::Compact),
                    ),
            )
    }
}

fn stage_icon(stage_type: &str) -> &'static str {
    match stage_type {
        "ingest" => "📥",
        "preprocess" => "🔧",
        "train" => "🧠",
        "evaluate" => "📊",
        "export" => "📤",
        _ => "▪",
    }
}
