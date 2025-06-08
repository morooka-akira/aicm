/*!
 * AI Context Management Tool - Configuration Loader
 * 
 * このファイルは設定ファイル読み込み機能を提供します。
 * ai-context.yamlファイルの読み込みと検証を行います。
 */

use crate::config::error::ConfigError;
use crate::types::{AIContextConfig, OutputMode, FileMapping};
use anyhow::Result;
use std::path::Path;
use tokio::fs;

/// 設定ファイルのデフォルト名
pub const DEFAULT_CONFIG_FILE: &str = "ai-context.yaml";

/// 設定ローダー
pub struct ConfigLoader;

impl ConfigLoader {
    /// 設定ファイルを読み込む
    /// 
    /// # Arguments
    /// * `config_path` - 設定ファイルのパス
    /// 
    /// # Returns
    /// 読み込まれた設定
    pub async fn load<P: AsRef<Path>>(config_path: P) -> Result<AIContextConfig, ConfigError> {
        let path = config_path.as_ref();
        
        // ファイルの存在確認
        if !path.exists() {
            return Err(ConfigError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        // ファイル読み込み
        let content = fs::read_to_string(path).await?;
        
        // YAML解析
        let mut config: AIContextConfig = serde_yaml::from_str(&content)?;
        
        // 検証
        Self::validate_config(&mut config)?;
        
        Ok(config)
    }

    /// デフォルト設定ファイルを読み込む
    pub async fn load_default() -> Result<AIContextConfig, ConfigError> {
        Self::load(DEFAULT_CONFIG_FILE).await
    }

    /// 設定を検証し、必要に応じて補正する
    fn validate_config(config: &mut AIContextConfig) -> Result<(), ConfigError> {
        let mut errors = Vec::new();

        // 必須フィールドの検証
        if config.version.is_empty() {
            errors.push("version フィールドが空です".to_string());
        }

        if config.base_docs_dir.is_empty() {
            errors.push("base_docs_dir フィールドが空です".to_string());
        }

        // ファイルマッピングの検証
        Self::validate_file_mapping(&config.file_mapping, &mut errors);

        // エラーがある場合は失敗
        if !errors.is_empty() {
            return Err(ConfigError::ValidationError { errors });
        }

        Ok(())
    }

    /// ファイルマッピング設定を検証
    fn validate_file_mapping(mapping: &FileMapping, errors: &mut Vec<String>) {
        // common配列の検証
        if mapping.common.is_empty() {
            errors.push("file_mapping.common が空です".to_string());
        }

        // project_specific配列の検証
        if mapping.project_specific.is_empty() {
            errors.push("file_mapping.project_specific が空です".to_string());
        }

        // 各ファイルパスの検証
        for file_path in &mapping.common {
            if file_path.trim().is_empty() {
                errors.push("file_mapping.common に空の文字列が含まれています".to_string());
            }
        }

        for file_path in &mapping.project_specific {
            if file_path.trim().is_empty() {
                errors.push("file_mapping.project_specific に空の文字列が含まれています".to_string());
            }
        }
    }

    /// デフォルト設定を生成
    pub fn create_default_config() -> AIContextConfig {
        AIContextConfig {
            version: "1.0".to_string(),
            output_mode: OutputMode::Merged,
            base_docs_dir: "./docs".to_string(),
            agents: Default::default(),
            file_mapping: FileMapping {
                common: vec![
                    "README.md".to_string(),
                    "docs/overview.md".to_string(),
                ],
                project_specific: vec![
                    "docs/architecture.md".to_string(),
                    "docs/api.md".to_string(),
                ],
                agent_specific: None,
            },
            global_variables: Default::default(),
        }
    }

    /// 設定ファイルを生成して保存
    pub async fn save_config<P: AsRef<Path>>(
        config: &AIContextConfig,
        path: P,
    ) -> Result<(), ConfigError> {
        let yaml_content = serde_yaml::to_string(config)?;
        fs::write(path, yaml_content).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, tempdir};

    #[tokio::test]
    async fn test_load_valid_config() {
        let valid_yaml = r#"
version: "1.0"
output_mode: "merged"
base_docs_dir: "./docs"
agents: {}
file_mapping:
  common:
    - "README.md"
    - "docs/overview.md"
  project_specific:
    - "docs/architecture.md"
global_variables: {}
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", valid_yaml).unwrap();

        let result = ConfigLoader::load(temp_file.path()).await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.base_docs_dir, "./docs");
        assert!(!config.file_mapping.common.is_empty());
    }

    #[tokio::test]
    async fn test_load_file_not_found() {
        let result = ConfigLoader::load("non_existent_file.yaml").await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ConfigError::FileNotFound { path } => {
                assert!(path.contains("non_existent_file.yaml"));
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_load_invalid_yaml() {
        let invalid_yaml = r#"
version: "1.0"
output_mode: invalid_enum_value
invalid_structure: [
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", invalid_yaml).unwrap();

        let result = ConfigLoader::load(temp_file.path()).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ConfigError::YamlParseError { .. } => {
                // Expected YAML parsing error
            }
            _ => panic!("Expected YamlParseError"),
        }
    }

    #[tokio::test]
    async fn test_validation_missing_version() {
        let yaml_without_version = r#"
version: ""
output_mode: "merged"
base_docs_dir: "./docs"
agents: {}
file_mapping:
  common:
    - "README.md"
  project_specific:
    - "docs/architecture.md"
global_variables: {}
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", yaml_without_version).unwrap();

        let result = ConfigLoader::load(temp_file.path()).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ConfigError::ValidationError { errors } => {
                assert!(errors.iter().any(|e| e.contains("version")));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_validation_empty_file_mapping() {
        let yaml_empty_mapping = r#"
version: "1.0"
output_mode: "merged"
base_docs_dir: "./docs"
agents: {}
file_mapping:
  common: []
  project_specific: []
global_variables: {}
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", yaml_empty_mapping).unwrap();

        let result = ConfigLoader::load(temp_file.path()).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ConfigError::ValidationError { errors } => {
                assert!(errors.iter().any(|e| e.contains("common が空です")));
                assert!(errors.iter().any(|e| e.contains("project_specific が空です")));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_create_default_config() {
        let config = ConfigLoader::create_default_config();
        
        assert_eq!(config.version, "1.0");
        assert_eq!(config.base_docs_dir, "./docs");
        assert!(!config.file_mapping.common.is_empty());
        assert!(!config.file_mapping.project_specific.is_empty());
        assert!(matches!(config.output_mode, OutputMode::Merged));
    }

    #[tokio::test]
    async fn test_save_and_load_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");
        
        let original_config = ConfigLoader::create_default_config();
        
        // 保存
        let save_result = ConfigLoader::save_config(&original_config, &config_path).await;
        assert!(save_result.is_ok());
        
        // 読み込み
        let loaded_config = ConfigLoader::load(&config_path).await;
        assert!(loaded_config.is_ok());
        
        let loaded = loaded_config.unwrap();
        assert_eq!(loaded.version, original_config.version);
        assert_eq!(loaded.base_docs_dir, original_config.base_docs_dir);
    }

    #[test]
    fn test_validate_file_mapping() {
        let mut valid_mapping = FileMapping {
            common: vec!["file1.md".to_string()],
            project_specific: vec!["file2.md".to_string()],
            agent_specific: None,
        };
        
        let mut errors = Vec::new();
        ConfigLoader::validate_file_mapping(&valid_mapping, &mut errors);
        assert!(errors.is_empty());
        
        // 空の文字列を含むマッピング
        valid_mapping.common.push("".to_string());
        errors.clear();
        ConfigLoader::validate_file_mapping(&valid_mapping, &mut errors);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("空の文字列が含まれています")));
    }

    #[tokio::test]
    async fn test_load_nonexistent_config_file() {
        // 存在しないファイルの場合のテスト
        let nonexistent_file = "nonexistent-config.yaml";
        let result = ConfigLoader::load(nonexistent_file).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ConfigError::FileNotFound { path } => {
                assert!(path.contains(nonexistent_file));
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }
}