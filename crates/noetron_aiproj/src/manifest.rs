//! Project manifest — reads and writes `.aiproj/project.toml`

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub const AIPROJ_DIR: &str = ".aiproj";
pub const MANIFEST_FILE: &str = "project.toml";

/// The AI domain this project targets.
/// Controls which features, panels, and templates are active.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    MachineLearning,
    DeepLearning,
    NaturalLanguageProcessing,
    ComputerVision,
    GenerativeAi,
    CognitiveComputing,
    Robotics,
    ExpertSystems,
    FuzzyLogic,
    EvolutionaryComputation,
    SwarmIntelligence,
}

impl Domain {
    pub fn display_name(&self) -> &str {
        match self {
            Domain::MachineLearning => "Machine Learning",
            Domain::DeepLearning => "Deep Learning",
            Domain::NaturalLanguageProcessing => "Natural Language Processing",
            Domain::ComputerVision => "Computer Vision",
            Domain::GenerativeAi => "Generative AI",
            Domain::CognitiveComputing => "Cognitive Computing",
            Domain::Robotics => "Robotics & Control",
            Domain::ExpertSystems => "Expert Systems",
            Domain::FuzzyLogic => "Fuzzy Logic & Control",
            Domain::EvolutionaryComputation => "Evolutionary Computation",
            Domain::SwarmIntelligence => "Swarm Intelligence",
        }
    }

    pub fn short_label(&self) -> &str {
        match self {
            Domain::MachineLearning => "ML",
            Domain::DeepLearning => "DL",
            Domain::NaturalLanguageProcessing => "NLP",
            Domain::ComputerVision => "CV",
            Domain::GenerativeAi => "GenAI",
            Domain::CognitiveComputing => "Cog",
            Domain::Robotics => "Robotics",
            Domain::ExpertSystems => "ES",
            Domain::FuzzyLogic => "Fuzzy",
            Domain::EvolutionaryComputation => "Evo",
            Domain::SwarmIntelligence => "Swarm",
        }
    }
}

/// How active a feature is for this project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FeatureLevel {
    #[default]
    Full,
    Minimal,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Features {
    #[serde(default)]
    pub data_management: FeatureLevel,
    #[serde(default)]
    pub pipeline_management: FeatureLevel,
    #[serde(default)]
    pub experiment_tracking: FeatureLevel,
    #[serde(default)]
    pub model_management: FeatureLevel,
    #[serde(default)]
    pub versioning: FeatureLevel,
    #[serde(default)]
    pub logic: FeatureLevel,
    #[serde(default = "disabled")]
    pub labeling: FeatureLevel,
}

fn disabled() -> FeatureLevel {
    FeatureLevel::Disabled
}

impl Default for Features {
    fn default() -> Self {
        Self {
            data_management: FeatureLevel::Full,
            pipeline_management: FeatureLevel::Full,
            experiment_tracking: FeatureLevel::Full,
            model_management: FeatureLevel::Full,
            versioning: FeatureLevel::Full,
            logic: FeatureLevel::Full,
            labeling: FeatureLevel::Disabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlConfig {
    #[serde(default = "default_task")]
    pub task_type: String,
    pub target_column: Option<String>,
    #[serde(default = "default_framework")]
    pub framework: String,
}

fn default_task() -> String { "classification".into() }
fn default_framework() -> String { "sklearn".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningConfig {
    #[serde(default = "default_backend")]
    pub backend: String,
    #[serde(default)]
    pub remote: String,
}

fn default_backend() -> String { "built-in".into() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    #[serde(default = "default_interpreter")]
    pub interpreter: String,
    #[serde(default = "default_requirements")]
    pub requirements: String,
}

fn default_interpreter() -> String { ".venv/bin/python".into() }
fn default_requirements() -> String { "requirements.txt".into() }

/// The full `.aiproj/project.toml` manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project: ProjectMeta,
    #[serde(default)]
    pub features: Features,
    pub versioning: Option<VersioningConfig>,
    pub python: Option<PythonConfig>,
    // Domain-specific config blocks
    #[serde(rename = "domain.machine_learning")]
    pub ml_config: Option<MlConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub id: String,
    pub name: String,
    pub domain: Domain,
    pub created_at: String,
}

impl ProjectManifest {
    /// Create a new manifest for a project with the given name and domain.
    pub fn new(name: impl Into<String>, domain: Domain) -> Self {
        Self {
            project: ProjectMeta {
                id: Uuid::new_v4().to_string(),
                name: name.into(),
                domain,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
            features: Features::default(),
            versioning: Some(VersioningConfig {
                backend: "built-in".into(),
                remote: String::new(),
            }),
            python: Some(PythonConfig {
                interpreter: ".venv/bin/python".into(),
                requirements: "requirements.txt".into(),
            }),
            ml_config: None,
        }
    }

    /// Load from `.aiproj/project.toml` inside `project_root`.
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = project_root.join(AIPROJ_DIR).join(MANIFEST_FILE);
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }

    /// Write to `.aiproj/project.toml` inside `project_root`.
    pub fn save(&self, project_root: &Path) -> Result<()> {
        let dir = project_root.join(AIPROJ_DIR);
        std::fs::create_dir_all(&dir)?;
        let path = dir.join(MANIFEST_FILE);
        let text = toml::to_string_pretty(self)?;
        std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))
    }

    pub fn aiproj_dir(&self, project_root: &Path) -> PathBuf {
        project_root.join(AIPROJ_DIR)
    }
}
