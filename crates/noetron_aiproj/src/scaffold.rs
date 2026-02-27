//! Project scaffolding — creates the `.aiproj/` structure from a domain template.

use anyhow::Result;
use std::path::Path;
use crate::manifest::{Domain, ProjectManifest};

/// Scaffold a new `.aiproj` project in `project_root`.
/// Creates all required directories and writes the manifest + starter files.
pub fn scaffold(project_root: &Path, name: &str, domain: Domain) -> Result<ProjectManifest> {
    let manifest = ProjectManifest::new(name, domain.clone());

    // Create .aiproj directory tree
    let aiproj = project_root.join(".aiproj");
    for sub in &["db", "data", "experiments", "models", "pipelines"] {
        std::fs::create_dir_all(aiproj.join(sub))?;
    }

    // Write the manifest
    manifest.save(project_root)?;

    // Write domain-specific starter files
    write_domain_starters(project_root, &domain)?;

    tracing::info!(
        "Scaffolded .aiproj project '{}' ({}) in {}",
        name,
        domain.display_name(),
        project_root.display()
    );

    Ok(manifest)
}

fn write_domain_starters(root: &Path, domain: &Domain) -> Result<()> {
    let src = root.join("src");
    std::fs::create_dir_all(&src)?;

    match domain {
        Domain::MachineLearning => {
            write_file(
                &src.join("pipeline.py"),
                include_str!("../../../.aiproj_templates/ml/pipeline.py"),
            )?;
            write_file(
                &src.join("data_config.toml"),
                include_str!("../../../.aiproj_templates/ml/data_config.toml"),
            )?;
            write_file(
                &root.join("requirements.txt"),
                "scikit-learn>=1.4\npandas>=2.0\nnumpy>=1.26\n",
            )?;
        }
        Domain::DeepLearning => {
            write_file(
                &src.join("pipeline.py"),
                include_str!("../../../.aiproj_templates/dl/pipeline.py"),
            )?;
            write_file(
                &root.join("requirements.txt"),
                "torch>=2.0\nnumpy>=1.26\nonnx>=1.15\nonnxruntime>=1.17\n",
            )?;
        }
        Domain::NaturalLanguageProcessing => {
            write_file(
                &src.join("pipeline.py"),
                include_str!("../../../.aiproj_templates/nlp/pipeline.py"),
            )?;
            write_file(
                &root.join("requirements.txt"),
                "transformers>=4.38\ntokenizers>=0.15\ndatasets>=2.17\n",
            )?;
        }
        Domain::ComputerVision => {
            write_file(
                &src.join("pipeline.py"),
                include_str!("../../../.aiproj_templates/cv/pipeline.py"),
            )?;
            write_file(
                &root.join("requirements.txt"),
                "Pillow>=10.0\nonnxruntime>=1.17\nnumpy>=1.26\n",
            )?;
        }
        _ => {
            // Generic starter for other domains
            write_file(
                &src.join("pipeline.py"),
                "# Noetron Studio — pipeline entry point\n\ndef run_pipeline():\n    pass\n",
            )?;
        }
    }

    Ok(())
}

fn write_file(path: &Path, content: &str) -> Result<()> {
    if !path.exists() {
        std::fs::write(path, content)?;
    }
    Ok(())
}

// Domain re-export for convenience
pub use crate::manifest::Domain;
