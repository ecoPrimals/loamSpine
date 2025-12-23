//! LoamSpine configuration.

use serde::{Deserialize, Serialize};
use sourdough_core::config::CommonConfig;

/// Configuration for LoamSpine.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoamSpineConfig {
    /// Common configuration.
    #[serde(flatten)]
    pub common: CommonConfig,
    
    // TODO: Add LoamSpine-specific configuration
}

impl Default for LoamSpineConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                name: "LoamSpine".to_string(),
                ..CommonConfig::default()
            },
        }
    }
}
