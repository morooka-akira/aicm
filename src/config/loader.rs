/*!
 * AI Context Management Tool - Configuration Loader (Simplified)
 *
 * Simplified configuration file loading functionality
 */

use crate::config::error::ConfigError;
use crate::types::AIContextConfig;
use std::path::Path;
use tokio::fs;

/// Configuration file loader (simplified version)
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration file from specified path
    pub async fn load<P: AsRef<Path>>(path: P) -> Result<AIContextConfig, ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        let content = fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::IoError { source: e })?;

        let config: AIContextConfig =
            serde_yaml::from_str(&content).map_err(|e| ConfigError::YamlError { source: e })?;

        Self::validate_config(&config)?;
        Ok(config)
    }

    /// Create and save default configuration
    pub async fn create_default<P: AsRef<Path>>(path: P) -> Result<AIContextConfig, ConfigError> {
        let config = AIContextConfig::default();
        Self::save(path, &config).await?;
        Ok(config)
    }

    /// Save configuration file
    pub async fn save<P: AsRef<Path>>(
        path: P,
        config: &AIContextConfig,
    ) -> Result<(), ConfigError> {
        let yaml_content =
            serde_yaml::to_string(config).map_err(|e| ConfigError::YamlError { source: e })?;

        fs::write(path, yaml_content)
            .await
            .map_err(|e| ConfigError::IoError { source: e })?;

        Ok(())
    }

    /// Basic configuration validation
    fn validate_config(config: &AIContextConfig) -> Result<(), ConfigError> {
        if config.version.is_empty() {
            return Err(ConfigError::ValidationError {
                message: "Version is not specified".to_string(),
            });
        }

        if config.base_docs_dir.is_empty() {
            return Err(ConfigError::ValidationError {
                message: "base_docs_dir is not specified".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentConfigTrait, OutputMode};
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_load_valid_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("aicm-config.yml");

        let valid_yaml = r#"
version: "1.0"
output_mode: merged
base_docs_dir: "./docs"
agents:
  cursor: true
  cline: false
  github: false
  claude: false
"#;

        fs::write(&config_path, valid_yaml).await.unwrap();

        let config = ConfigLoader::load(&config_path).await.unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.get_global_output_mode(), OutputMode::Merged);
        assert_eq!(config.base_docs_dir, "./docs");
        assert!(config.agents.cursor.is_enabled());
        assert!(!config.agents.cline.is_enabled());
    }

    #[tokio::test]
    async fn test_load_file_not_found() {
        let result = ConfigLoader::load("/nonexistent/path/config.yaml").await;
        assert!(result.is_err());

        if let Err(ConfigError::FileNotFound { path }) = result {
            assert!(path.contains("nonexistent"));
        } else {
            panic!("Expected FileNotFound error");
        }
    }

    #[tokio::test]
    async fn test_load_invalid_yaml() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("invalid.yaml");

        let invalid_yaml = r#"
version: "1.0"
output_mode: invalid_mode
base_docs_dir: "./docs"
agents: not_an_object
"#;

        fs::write(&config_path, invalid_yaml).await.unwrap();

        let result = ConfigLoader::load(&config_path).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::YamlError { .. }));
    }

    #[tokio::test]
    async fn test_validate_config_missing_version() {
        let config = AIContextConfig {
            version: "".to_string(),
            ..Default::default()
        };

        let result = ConfigLoader::validate_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::ValidationError { message }) = result {
            assert!(message.contains("Version is not specified"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[tokio::test]
    async fn test_validate_config_missing_base_docs_dir() {
        let config = AIContextConfig {
            base_docs_dir: "".to_string(),
            ..Default::default()
        };

        let result = ConfigLoader::validate_config(&config);
        assert!(result.is_err());

        if let Err(ConfigError::ValidationError { message }) = result {
            assert!(message.contains("base_docs_dir is not specified"));
        } else {
            panic!("Expected ValidationError");
        }
    }

    #[tokio::test]
    async fn test_create_default() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("default.yaml");

        let config = ConfigLoader::create_default(&config_path).await.unwrap();

        // Check default values
        assert_eq!(config.version, "1.0");
        assert_eq!(config.get_global_output_mode(), OutputMode::Merged);
        assert_eq!(config.base_docs_dir, "./ai-docs");
        assert!(!config.agents.cursor.is_enabled());
        assert!(!config.agents.cline.is_enabled());
        assert!(!config.agents.github.is_enabled());
        assert!(!config.agents.claude.is_enabled());

        // Confirm file was actually created
        assert!(config_path.exists());
    }

    #[tokio::test]
    async fn test_save_and_load_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test.yaml");

        let mut original_config = AIContextConfig::default();
        original_config.agents.cursor = crate::types::CursorConfig::Simple(true);
        original_config.agents.claude = crate::types::ClaudeConfig::Simple(true);

        // Save
        ConfigLoader::save(&config_path, &original_config)
            .await
            .unwrap();

        // Load
        let loaded_config = ConfigLoader::load(&config_path).await.unwrap();

        // Confirm contents match
        assert_eq!(loaded_config.version, original_config.version);
        assert_eq!(loaded_config.base_docs_dir, original_config.base_docs_dir);
        assert_eq!(loaded_config.agents.cursor, original_config.agents.cursor);
        assert_eq!(loaded_config.agents.claude, original_config.agents.claude);
    }

    #[tokio::test]
    async fn test_load_config_with_partial_agents() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("partial.yaml");

        let partial_yaml = r#"
version: "1.0"
output_mode: split
base_docs_dir: "./custom-docs"
agents:
  cursor: true
"#;

        fs::write(&config_path, partial_yaml).await.unwrap();

        let config = ConfigLoader::load(&config_path).await.unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.get_global_output_mode(), OutputMode::Split);
        assert_eq!(config.base_docs_dir, "./custom-docs");
        assert!(config.agents.cursor.is_enabled());
        assert!(!config.agents.cline.is_enabled()); // default false
        assert!(!config.agents.github.is_enabled()); // default false
        assert!(!config.agents.claude.is_enabled()); // default false
    }
}
