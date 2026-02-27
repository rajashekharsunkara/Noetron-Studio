//! noetron_ui — four no-code panels rendered in GPUI.
//!
//! Panels:
//! - `data_panel`   — Data Management: source, profile, version selector
//! - `pipeline_panel` — Stage-lane pipeline editor (drag-drop cards)
//! - `experiment_panel` — Experiment dashboard (runs table + metric charts)
//! - `model_panel` — Model registry (versions, export buttons)
//!
//! All panels are `Render` structs that read from `noetron_ir` entities.

pub mod data_panel;
pub mod experiment_panel;
pub mod model_panel;
pub mod pipeline_panel;

pub use data_panel::DataPanel;
pub use experiment_panel::ExperimentPanel;
pub use model_panel::ModelPanel;
pub use pipeline_panel::PipelinePanel;
