//! Model Registry Panel — lists registered model versions with export actions.

use gpui::{Context, IntoElement, Render, Window};
use ui::prelude::*;

#[derive(Debug, Clone)]
pub struct ModelVersionRow {
    pub version_id: String,
    pub model_name: String,
    pub format: String,
    pub metrics: serde_json::Value,
    pub registered_at: String,
}

pub struct ModelPanel {
    pub versions: Vec<ModelVersionRow>,
}

impl ModelPanel {
    pub fn new(versions: Vec<ModelVersionRow>) -> Self {
        Self { versions }
    }
}

impl Render for ModelPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .id("model-panel")
            .p_4()
            .gap_3()
            .child(
                h_flex()
                    .gap_2()
                    .child(Label::new("🤖 Models").size(LabelSize::Large))
                    .child(Label::new(format!("{} versions", self.versions.len())).size(LabelSize::Small)),
            )
            .child(if self.versions.is_empty() {
                Label::new("No models registered yet.")
                    .size(LabelSize::Small)
                    .into_any_element()
            } else {
                v_flex()
                    .gap_2()
                    .children(self.versions.iter().map(|ver| {
                        h_flex()
                            .id(ElementId::from(ver.version_id.clone()))
                            .gap_3()
                            .py_1()
                            .child(
                                v_flex()
                                    .gap_0()
                                    .child(Label::new(ver.model_name.clone()))
                                    .child(
                                        Label::new(format!("Format: {}  |  {}", ver.format, ver.registered_at))
                                            .size(LabelSize::Small),
                                    ),
                            )
                            .child(
                                Button::new(
                                    ElementId::from(format!("export-{}", ver.version_id)),
                                    "Export",
                                )
                                .style(ButtonStyle::Subtle)
                                .size(ButtonSize::Compact),
                            )
                    }))
                    .into_any_element()
            })
    }
}
