//! Data Management Panel — source path picker, profile stats, version selector.

use gpui::{App, Context, IntoElement, Render, Window};
use noetron_ir::entities::DataConfigIr;
use ui::prelude::*;

pub struct DataPanel {
    pub config: DataConfigIr,
    pub profile: Option<serde_json::Value>,
}

impl DataPanel {
    pub fn new(config: DataConfigIr) -> Self {
        Self { config, profile: None }
    }
}

impl Render for DataPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let source_label = if self.config.source_path.is_empty() {
            "No dataset selected".to_string()
        } else {
            self.config.source_path.clone()
        };

        v_flex()
            .id("data-panel")
            .p_4()
            .gap_3()
            .child(
                // Header
                h_flex()
                    .gap_2()
                    .child(Label::new("📦 Data Management").size(LabelSize::Large)),
            )
            .child(
                // Source section
                v_flex()
                    .gap_1()
                    .child(Label::new("Source").size(LabelSize::Small))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Label::new(source_label))
                            .child(
                                Button::new("pick-source", "Browse…")
                                    .style(ButtonStyle::Subtle)
                                    .size(ButtonSize::Compact),
                            ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Label::new(format!("Type: {}", self.config.source_type)).size(LabelSize::Small))
                            .child(Label::new(format!("Val: {:.0}%", self.config.val_size * 100.0)).size(LabelSize::Small))
                            .child(Label::new(format!("Test: {:.0}%", self.config.test_size * 100.0)).size(LabelSize::Small)),
                    ),
            )
            .child(
                // Steps section
                v_flex()
                    .gap_1()
                    .child(Label::new("Preprocessing Steps").size(LabelSize::Small))
                    .children(self.config.preprocess_steps.iter().map(|step| {
                        h_flex()
                            .gap_1()
                            .px_2()
                            .py_1()
                            .child(Label::new("▸").size(LabelSize::Small))
                            .child(Label::new(step.name.clone()))
                    }))
                    .child(
                        Button::new("add-step", "+ Add Step")
                            .style(ButtonStyle::Subtle)
                            .size(ButtonSize::Compact),
                    ),
            )
            .child(
                // Profile section (shown if profile was computed)
                if let Some(profile) = &self.profile {
                    let shape = profile.get("shape")
                        .and_then(|v| v.as_array())
                        .map(|arr| format!("{} rows × {} cols",
                            arr.first().and_then(|v| v.as_u64()).unwrap_or(0),
                            arr.get(1).and_then(|v| v.as_u64()).unwrap_or(0)))
                        .unwrap_or_else(|| "Unknown shape".into());

                    let mem = profile.get("memory_mb")
                        .and_then(|v| v.as_f64())
                        .map(|m| format!("{m:.2} MB"))
                        .unwrap_or_default();

                    h_flex()
                        .gap_4()
                        .child(Label::new(shape).size(LabelSize::Small))
                        .child(Label::new(mem).size(LabelSize::Small))
                        .into_any_element()
                } else {
                    Button::new("profile-btn", "▶ Profile Dataset")
                        .style(ButtonStyle::Subtle)
                        .size(ButtonSize::Compact)
                        .into_any_element()
                },
            )
    }
}
