/*!
 * AI Context Management Tool - Configuration Error Types (Simplified)
 *
 * Simplified error type definitions
 */

use thiserror::Error;

/// Configuration-related error types (simplified version)
#[derive(Error, Debug)]
pub enum ConfigError {
    /// File not found
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },

    /// File I/O error
    #[error("File I/O error occurred")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    /// YAML parsing error
    #[error("YAML parsing error: {source}")]
    YamlError {
        #[from]
        source: serde_yaml::Error,
    },

    /// Configuration validation error
    #[error("Invalid configuration value: {message}")]
    ValidationError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::FileNotFound {
            path: "test.yaml".to_string(),
        };
        assert!(error.to_string().contains("Configuration file not found"));
        assert!(error.to_string().contains("test.yaml"));
    }

    #[test]
    fn test_validation_error() {
        let error = ConfigError::ValidationError {
            message: "Version is not specified".to_string(),
        };
        assert!(error.to_string().contains("Invalid configuration value"));
        assert!(error.to_string().contains("Version is not specified"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = IoError::new(ErrorKind::PermissionDenied, "Permission denied");
        let config_error = ConfigError::IoError { source: io_error };

        assert!(config_error.to_string().contains("File I/O error occurred"));
    }

    #[test]
    fn test_yaml_error_conversion() {
        let yaml_content = "invalid: yaml: content: [";
        let yaml_error = serde_yaml::from_str::<serde_yaml::Value>(yaml_content).unwrap_err();
        let config_error = ConfigError::YamlError { source: yaml_error };

        assert!(config_error.to_string().contains("YAML parsing error"));
    }

    #[test]
    fn test_error_debug_format() {
        let error = ConfigError::FileNotFound {
            path: "debug_test.yaml".to_string(),
        };
        let debug_string = format!("{error:?}");
        assert!(debug_string.contains("FileNotFound"));
        assert!(debug_string.contains("debug_test.yaml"));
    }

    #[test]
    fn test_error_source_chain() {
        let io_error = IoError::new(ErrorKind::NotFound, "File not found");
        let config_error = ConfigError::IoError { source: io_error };

        // Confirm error source is properly set
        assert!(config_error.source().is_some());
    }
}
