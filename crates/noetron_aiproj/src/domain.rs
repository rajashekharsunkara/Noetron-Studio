//! Domain capability matrix — what features each domain supports.

use crate::manifest::{Domain, FeatureLevel};

pub struct DomainCapabilities {
    pub data_management: FeatureLevel,
    pub pipeline_management: FeatureLevel,
    pub experiment_tracking: FeatureLevel,
    pub model_management: FeatureLevel,
    pub labeling: FeatureLevel,
}

impl Domain {
    /// Returns the default capabilities for this domain.
    pub fn capabilities(&self) -> DomainCapabilities {
        match self {
            Domain::MachineLearning | Domain::DeepLearning => DomainCapabilities {
                data_management: FeatureLevel::Full,
                pipeline_management: FeatureLevel::Full,
                experiment_tracking: FeatureLevel::Full,
                model_management: FeatureLevel::Full,
                labeling: FeatureLevel::Disabled,
            },
            Domain::NaturalLanguageProcessing | Domain::ComputerVision => DomainCapabilities {
                data_management: FeatureLevel::Full,
                pipeline_management: FeatureLevel::Full,
                experiment_tracking: FeatureLevel::Full,
                model_management: FeatureLevel::Full,
                labeling: FeatureLevel::Full, // NLP/CV need labeling workbench
            },
            Domain::GenerativeAi => DomainCapabilities {
                data_management: FeatureLevel::Full,
                pipeline_management: FeatureLevel::Full,
                experiment_tracking: FeatureLevel::Full,
                model_management: FeatureLevel::Full,
                labeling: FeatureLevel::Minimal,
            },
            _ => DomainCapabilities {
                data_management: FeatureLevel::Minimal,
                pipeline_management: FeatureLevel::Full,
                experiment_tracking: FeatureLevel::Full,
                model_management: FeatureLevel::Minimal,
                labeling: FeatureLevel::Disabled,
            },
        }
    }
}
