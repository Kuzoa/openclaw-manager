// Prevent Windows system from displaying console window (only for binary, lib keeps this for consistency)
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub mod commands;
pub mod models;
pub mod utils;

// Re-export commonly used types for integration tests
pub use commands::{config, diagnostics, installer, process, service, skills};
pub use models::{status::ServiceStatus, config::*};
pub use utils::{cache::EnvironmentCache, file, log_sanitizer, platform, shell};
