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

    /// Create and save default configuration with comments
    pub async fn create_default<P: AsRef<Path>>(path: P) -> Result<AIContextConfig, ConfigError> {
        let template = Self::create_default_template();

        fs::write(path, template)
            .await
            .map_err(|e| ConfigError::IoError { source: e })?;

        // Return the default config object
        Ok(AIContextConfig::default())
    }

    /// Create default configuration template with detailed comments
    fn create_default_template() -> String {
        let template = [
            "# aicm Configuration File",
            "# AI Context Management Tool - Configuration for multiple AI coding agents",
            "# For more information: https://github.com/morooka-akira/aicm",
            "",
            "# Configuration file version",
            "version: \"1.0\"",
            "",
            "# Global output mode for all agents (default: merged)",
            "# - merged: Combine all markdown files into one file per agent",
            "# - split: Create separate files for each markdown file",
            "output_mode: merged",
            "",
            "# Global base documentation directory (default: ./ai-docs)",
            "# This directory should contain your markdown documentation files",
            "base_docs_dir: ./ai-docs",
            "",
            "# Agent configurations",
            "agents:",
            "  # Cursor IDE Agent - Generates .cursor/rules/*.mdc files",
            "  cursor:",
            "    enabled: true",
            "    output_mode: split  # Override global setting for Cursor",
            "    split_config:",
            "      rules:",
            "        # Always Apply Rule - Always loaded in Cursor",
            "        - file_patterns: [\"*overview*\", \"*common*\"]",
            "          alwaysApply: true",
            "        ",
            "        # Auto Attached Rule - Automatically attached when editing matching files",
            "        - file_patterns: [\"*rust*\", \"*backend*\"]",
            "          globs: [\"**/*.rs\", \"**/*.toml\"]",
            "        ",
            "        # Agent Requested Rule - Loaded when agent explicitly requests",
            "        - file_patterns: [\"*api*\", \"*architecture*\"]",
            "          description: \"API design and system architecture guidelines\"",
            "        ",
            "        # Manual Rule - Only loaded when manually referenced",
            "        - file_patterns: [\"*troubleshoot*\", \"*debug*\"]",
            "          manual: true",
            "",
            "  # GitHub Copilot Agent - Generates .github/instructions/*.instructions.md files",
            "  github:",
            "    enabled: true",
            "    output_mode: split",
            "    split_config:",
            "      rules:",
            "        # Backend development rules - Applied to Rust files",
            "        - file_patterns: [\"*rust*\", \"*backend*\", \"*api*\"]",
            "          apply_to: [\"**/*.rs\", \"**/*.toml\"]",
            "        ",
            "        # Frontend development rules - Applied to TypeScript files",
            "        - file_patterns: [\"*frontend*\", \"*ui*\", \"*component*\"]",
            "          apply_to: [\"**/*.ts\", \"**/*.tsx\", \"**/*.js\", \"**/*.jsx\"]",
            "",
            "  # Cline Agent - Generates .clinerules/*.md files",
            "  cline:",
            "    enabled: true",
            "    output_mode: merged  # Cline works well with merged content",
            "",
            "  # Claude Code Agent - Generates CLAUDE.md file",
            "  claude:",
            "    enabled: true",
            "",
            "  # OpenAI Codex Agent - Generates AGENTS.md file",
            "  codex:",
            "    enabled: true",
        ];

        template.join("\n")
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
