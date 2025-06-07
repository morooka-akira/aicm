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