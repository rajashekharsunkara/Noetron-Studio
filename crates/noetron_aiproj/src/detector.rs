//! Domain detection — checks if an opened folder is an `.aiproj` project.

use std::path::Path;
use crate::manifest::ProjectManifest;

pub struct AiprojDetector;

impl AiprojDetector {
    /// Returns `Some(manifest)` if the folder contains `.aiproj/project.toml`,
    /// `None` if it is a plain folder (show standard Zed editor experience).
    pub fn detect(folder: &Path) -> Option<ProjectManifest> {
        ProjectManifest::load(folder).ok()
    }

    /// Returns true if this folder has been initialized as an `.aiproj` project.
    pub fn is_aiproj(folder: &Path) -> bool {
        folder
            .join(crate::manifest::AIPROJ_DIR)
            .join(crate::manifest::MANIFEST_FILE)
            .exists()
    }
}
