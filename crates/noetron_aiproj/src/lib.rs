//! noetron_aiproj — `.aiproj` Project Format
//!
//! Handles everything related to Noetron Studio project identity:
//!   - Detecting `.aiproj/project.toml` when a folder is opened
//!   - Reading and writing the project manifest
//!   - Scaffolding a new project from a domain template
//!   - Exposing the active domain and feature set to the rest of the app

pub mod manifest;
pub mod detector;
pub mod scaffold;
pub mod domain;

pub use manifest::{ProjectManifest, Domain, FeatureLevel};
pub use detector::AiprojDetector;
