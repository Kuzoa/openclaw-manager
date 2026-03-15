use serde::{Deserialize, Serialize};

/// Result of a single detection step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DetectionResult {
    /// The target was found
    Found,
    /// The target was not found
    NotFound,
    /// An error occurred during detection
    Error,
}

/// A single step in the environment detection process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionStep {
    /// Phase name (e.g., "Phase 1: npm global prefix")
    pub phase: String,
    /// Action performed (e.g., "Checking npm prefix")
    pub action: String,
    /// Target path or item being checked
    pub target: String,
    /// Result of this detection step
    pub result: DetectionResult,
    /// Additional message (typically for errors)
    pub message: Option<String>,
}
