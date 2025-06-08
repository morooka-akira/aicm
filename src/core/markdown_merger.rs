/*!
 * AI Context Management Tool - Markdown Merger
 * 
 * このファイルはMarkdownファイルのマージ機能を提供します。
 * 複数のMarkdownファイルを読み込み、統合・分割コンテンツを生成します。
 */

use crate::types::{AIContextConfig, MergedContent, SplitContent};
use anyhow::Result;
use std::path::Path;
use tokio::fs;
use thiserror::Error;

/// Markdownマージャーエラー
#[derive(Error, Debug)]
pub enum MarkdownMergerError {
    #[error("ベースディレクトリが見つかりません: {path}")]
    BaseDirectoryNotFound { path: String },

    #[error("ファイルの読み込みに失敗しました: {path}")]
    FileReadError { path: String },

    #[error("IO エラー: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("その他のエラー: {source}")]
    Other {
        #[from]
        source: anyhow::Error,
    },
}

/// Markdownマージャー
/// 
/// 複数のMarkdownファイルを読み込み、設定に基づいて
/// 統合・分割コンテンツを生成します。
pub struct MarkdownMerger {
    config: AIContextConfig,
}

impl MarkdownMerger {
    /// 新しいMarkdownマージャーを作成
    /// 
    /// # Arguments
    /// * `config` - AI Context設定
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    /// Markdownファイルをマージして統合・分割コンテンツを生成
    /// 
    /// # Returns
    /// マージされたコンテンツ
    pub async fn merge(&self) -> Result<MergedContent, MarkdownMergerError> {
        let base_dir = Path::new(&self.config.base_docs_dir);
        
        // ベースディレクトリの存在確認
        if !base_dir.exists() {
            return Err(MarkdownMergerError::BaseDirectoryNotFound {
                path: self.config.base_docs_dir.clone(),
            });
        }

        // 各カテゴリのコンテンツを読み込み
        let common_content = self.read_files(&self.config.file_mapping.common, base_dir).await?;
        let project_content = self.read_files(&self.config.file_mapping.project_specific, base_dir).await?;
        let agent_content = self.read_agent_specific_files(base_dir).await?;

        // 分割コンテンツを作成
        let split_content = SplitContent {
            common: common_content,
            project_specific: project_content,
            agent_specific: agent_content,
        };

        // 統合コンテンツを作成
        let merged_content = format!(
            "{}\n\n{}\n\n{}",
            split_content.common,
            split_content.project_specific,
            split_content.agent_specific
        );

        Ok(MergedContent {
            merged: merged_content,
            split: split_content,
        })
    }

    /// ファイルリストを読み込んでコンテンツを結合
    async fn read_files(&self, file_paths: &[String], base_dir: &Path) -> Result<String, MarkdownMergerError> {
        let mut contents = Vec::new();

        for file_path in file_paths {
            let full_path = base_dir.join(file_path);
            
            match self.read_single_file(&full_path).await {
                Ok(content) => {
                    if !content.trim().is_empty() {
                        contents.push(format!("# {}\n\n{}", file_path, content));
                    }
                }
                Err(_) => {
                    // ファイルが見つからない場合は警告として記録（エラーにはしない）
                    eprintln!("ファイルの読み込みをスキップしました: {}", file_path);
                }
            }
        }

        Ok(contents.join("\n\n"))
    }

    /// エージェント固有ファイルを読み込み
    async fn read_agent_specific_files(&self, base_dir: &Path) -> Result<String, MarkdownMergerError> {
        if let Some(agent_specific) = &self.config.file_mapping.agent_specific {
            // 対象エージェント（ここではcursor）のファイルを読み込み
            let cursor_files = agent_specific.get("cursor").cloned().unwrap_or_default();
            
            self.read_files(&cursor_files, base_dir).await
        } else {
            Ok(String::new())
        }
    }

    /// 単一ファイルを読み込み
    async fn read_single_file(&self, path: &Path) -> Result<String, MarkdownMergerError> {
        if !path.exists() {
            return Err(MarkdownMergerError::FileReadError {
                path: path.to_string_lossy().to_string(),
            });
        }

        let content = fs::read_to_string(path)
            .await
            .map_err(|e| MarkdownMergerError::IoError { source: e })?;

        Ok(content)
    }

    /// ファイルの存在チェック
    pub async fn validate_files(&self) -> Result<Vec<String>, MarkdownMergerError> {
        let mut missing_files = Vec::new();
        let base_dir = Path::new(&self.config.base_docs_dir);

        // ベースディレクトリの確認
        if !base_dir.exists() {
            return Err(MarkdownMergerError::BaseDirectoryNotFound {
                path: self.config.base_docs_dir.clone(),
            });
        }

        // 共通ファイルの確認
        for file_path in &self.config.file_mapping.common {
            let full_path = base_dir.join(file_path);
            if !full_path.exists() {
                missing_files.push(format!("common: {}", file_path));
            }
        }

        // プロジェクト固有ファイルの確認
        for file_path in &self.config.file_mapping.project_specific {
            let full_path = base_dir.join(file_path);
            if !full_path.exists() {
                missing_files.push(format!("project_specific: {}", file_path));
            }
        }

        // エージェント固有ファイルの確認
        if let Some(agent_specific) = &self.config.file_mapping.agent_specific {
            for (agent, files) in agent_specific {
                for file_path in files {
                    let full_path = base_dir.join(file_path);
                    if !full_path.exists() {
                        missing_files.push(format!("{}: {}", agent, file_path));
                    }
                }
            }
        }

        Ok(missing_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OutputMode, FileMapping, AgentConfigs};
    use std::collections::HashMap;
    use tempfile::tempdir;
    use tokio::fs;

    fn create_test_config() -> AIContextConfig {
        AIContextConfig {
            version: "1.0".to_string(),
            output_mode: OutputMode::Merged,
            base_docs_dir: "./test_docs".to_string(),
            agents: AgentConfigs::default(),
            file_mapping: FileMapping {
                common: vec![
                    "common/overview.md".to_string(),
                    "common/guidelines.md".to_string(),
                ],
                project_specific: vec![
                    "project/architecture.md".to_string(),
                ],
                agent_specific: Some(HashMap::from([
                    ("cursor".to_string(), vec!["agents/cursor.md".to_string()]),
                ])),
            },
            global_variables: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_new_markdown_merger() {
        let config = create_test_config();
        let merger = MarkdownMerger::new(config.clone());
        assert_eq!(merger.config.version, config.version);
        assert_eq!(merger.config.base_docs_dir, config.base_docs_dir);
    }

    #[tokio::test]
    async fn test_merge_base_directory_not_found() {
        let config = create_test_config();
        let merger = MarkdownMerger::new(config);
        
        let result = merger.merge().await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            MarkdownMergerError::BaseDirectoryNotFound { path } => {
                assert_eq!(path, "./test_docs");
            }
            _ => panic!("Expected BaseDirectoryNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_merge_with_valid_files() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();
        
        // テスト用ディレクトリ構造を作成
        fs::create_dir_all(base_dir.join("common")).await.unwrap();
        fs::create_dir_all(base_dir.join("project")).await.unwrap();
        fs::create_dir_all(base_dir.join("agents")).await.unwrap();
        
        // テスト用ファイルを作成
        fs::write(base_dir.join("common/overview.md"), "# Overview\nThis is overview content.").await.unwrap();
        fs::write(base_dir.join("common/guidelines.md"), "# Guidelines\nThese are guidelines.").await.unwrap();
        fs::write(base_dir.join("project/architecture.md"), "# Architecture\nSystem architecture.").await.unwrap();
        fs::write(base_dir.join("agents/cursor.md"), "# Cursor Rules\nCursor specific rules.").await.unwrap();
        
        let mut config = create_test_config();
        config.base_docs_dir = base_dir.to_string_lossy().to_string();
        
        let merger = MarkdownMerger::new(config);
        let result = merger.merge().await;
        
        assert!(result.is_ok());
        let merged_content = result.unwrap();
        
        // 各セクションのコンテンツが含まれていることを確認
        assert!(merged_content.split.common.contains("Overview"));
        assert!(merged_content.split.common.contains("Guidelines"));
        assert!(merged_content.split.project_specific.contains("Architecture"));
        assert!(merged_content.split.agent_specific.contains("Cursor Rules"));
        
        // 統合コンテンツが正しく生成されていることを確認
        assert!(merged_content.merged.contains("Overview"));
        assert!(merged_content.merged.contains("Architecture"));
        assert!(merged_content.merged.contains("Cursor Rules"));
    }

    #[tokio::test]
    async fn test_read_files_with_missing_files() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();
        
        // 一部のファイルのみ作成
        fs::create_dir_all(base_dir.join("common")).await.unwrap();
        fs::write(base_dir.join("common/overview.md"), "# Overview\nContent").await.unwrap();
        
        let mut config = create_test_config();
        config.base_docs_dir = base_dir.to_string_lossy().to_string();
        
        let merger = MarkdownMerger::new(config);
        let file_paths = vec![
            "common/overview.md".to_string(),
            "common/missing.md".to_string(),
        ];
        
        let result = merger.read_files(&file_paths, base_dir).await;
        assert!(result.is_ok());
        
        let content = result.unwrap();
        assert!(content.contains("Overview"));
        // 見つからないファイルは無視されるべき
        assert!(!content.contains("missing"));
    }

    #[tokio::test]
    async fn test_read_single_file_not_found() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();
        let non_existent_file = base_dir.join("non_existent.md");
        
        let config = create_test_config();
        let merger = MarkdownMerger::new(config);
        
        let result = merger.read_single_file(&non_existent_file).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            MarkdownMergerError::FileReadError { path } => {
                assert!(path.contains("non_existent.md"));
            }
            _ => panic!("Expected FileReadError"),
        }
    }

    #[tokio::test]
    async fn test_read_single_file_success() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();
        let test_file = base_dir.join("test.md");
        let test_content = "# Test\nThis is test content.";
        
        fs::write(&test_file, test_content).await.unwrap();
        
        let config = create_test_config();
        let merger = MarkdownMerger::new(config);
        
        let result = merger.read_single_file(&test_file).await;
        assert!(result.is_ok());
        
        let content = result.unwrap();
        assert_eq!(content, test_content);
    }

    #[tokio::test]
    async fn test_validate_files_all_exist() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();
        
        // 必要なディレクトリとファイルを作成
        fs::create_dir_all(base_dir.join("common")).await.unwrap();
        fs::create_dir_all(base_dir.join("project")).await.unwrap();
        fs::create_dir_all(base_dir.join("agents")).await.unwrap();
        
        fs::write(base_dir.join("common/overview.md"), "content").await.unwrap();
        fs::write(base_dir.join("common/guidelines.md"), "content").await.unwrap();
        fs::write(base_dir.join("project/architecture.md"), "content").await.unwrap();
        fs::write(base_dir.join("agents/cursor.md"), "content").await.unwrap();
        
        let mut config = create_test_config();
        config.base_docs_dir = base_dir.to_string_lossy().to_string();
        
        let merger = MarkdownMerger::new(config);
        let result = merger.validate_files().await;
        
        assert!(result.is_ok());
        let missing_files = result.unwrap();
        assert!(missing_files.is_empty());
    }

    #[tokio::test]
    async fn test_validate_files_some_missing() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();
        
        // 一部のファイルのみ作成
        fs::create_dir_all(base_dir.join("common")).await.unwrap();
        fs::write(base_dir.join("common/overview.md"), "content").await.unwrap();
        // guidelines.md, architecture.md, cursor.md は作成しない
        
        let mut config = create_test_config();
        config.base_docs_dir = base_dir.to_string_lossy().to_string();
        
        let merger = MarkdownMerger::new(config);
        let result = merger.validate_files().await;
        
        assert!(result.is_ok());
        let missing_files = result.unwrap();
        assert!(!missing_files.is_empty());
        assert!(missing_files.iter().any(|f| f.contains("guidelines.md")));
        assert!(missing_files.iter().any(|f| f.contains("architecture.md")));
        assert!(missing_files.iter().any(|f| f.contains("cursor.md")));
    }

    #[tokio::test]
    async fn test_read_agent_specific_files_no_config() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();
        
        let mut config = create_test_config();
        config.base_docs_dir = base_dir.to_string_lossy().to_string();
        config.file_mapping.agent_specific = None;
        
        let merger = MarkdownMerger::new(config);
        let result = merger.read_agent_specific_files(base_dir).await;
        
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.is_empty());
    }

    #[tokio::test]
    async fn test_read_files_empty_content_handling() {
        let temp_dir = tempdir().unwrap();
        let base_dir = temp_dir.path();
        
        fs::create_dir_all(base_dir.join("test")).await.unwrap();
        
        // 空のファイルと通常のファイルを作成
        fs::write(base_dir.join("test/empty.md"), "").await.unwrap();
        fs::write(base_dir.join("test/content.md"), "# Content\nActual content").await.unwrap();
        fs::write(base_dir.join("test/whitespace.md"), "   \n  \t  \n").await.unwrap();
        
        let mut config = create_test_config();
        config.base_docs_dir = base_dir.to_string_lossy().to_string();
        
        let merger = MarkdownMerger::new(config);
        let file_paths = vec![
            "test/empty.md".to_string(),
            "test/content.md".to_string(),
            "test/whitespace.md".to_string(),
        ];
        
        let result = merger.read_files(&file_paths, base_dir).await;
        assert!(result.is_ok());
        
        let content = result.unwrap();
        // 空のファイルとホワイトスペースのみのファイルは除外される
        assert!(content.contains("Actual content"));
        assert!(!content.contains("# test/empty.md"));
        assert!(!content.contains("# test/whitespace.md"));
    }

    #[test]
    fn test_markdown_merger_error_display() {
        let error = MarkdownMergerError::BaseDirectoryNotFound {
            path: "/test/path".to_string(),
        };
        assert!(error.to_string().contains("ベースディレクトリが見つかりません"));
        assert!(error.to_string().contains("/test/path"));

        let file_error = MarkdownMergerError::FileReadError {
            path: "/test/file.md".to_string(),
        };
        assert!(file_error.to_string().contains("ファイルの読み込みに失敗しました"));
        assert!(file_error.to_string().contains("/test/file.md"));
    }
}