/*!
 * AI Context Management Tool - Library
 *
 * This library provides context generation functions for various AI agents.
 */

pub mod agents;
pub mod config;
pub mod core;
pub mod types;

pub use agents::*;
pub use config::*;
pub use core::*;
pub use types::*;

/// Default configuration file name
pub const DEFAULT_CONFIG_FILE: &str = "aicm-config.yml";
