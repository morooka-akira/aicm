/*!
 * AI Context Management Tool - Markdown Merger
 * 
 * このファイルはMarkdownファイルのマージ機能を提供します。
 * 複数のMarkdownファイルを読み込み、統合・分割コンテンツを生成します。
 */

use crate::types::{AIContextConfig, MergedContent, SplitContent};
use anyhow::{Result, Context};
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