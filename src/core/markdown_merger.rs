/*!
 * AI Context Management Tool - Markdown Merger (Simplified)
 *
 * シンプル化されたMarkdownファイル結合機能
 */

use crate::types::AIContextConfig;
use anyhow::Result;
use std::path::Path;
use tokio::fs;

/// Markdownファイルを自動検出・結合するクラス
pub struct MarkdownMerger {
    config: AIContextConfig,
}

impl MarkdownMerger {
    /// 新しいMarkdownマージャーを作成
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    /// docs配下の全Markdownファイルを結合
    pub async fn merge_all(&self) -> Result<String> {
        let docs_dir = Path::new(&self.config.base_docs_dir);

        // ディレクトリが存在しない場合は空文字を返す
        if !docs_dir.exists() {
            return Ok(String::new());
        }

        let markdown_files = self.find_markdown_files(docs_dir).await?;
        let mut merged_content = String::new();

        for file_path in markdown_files {
            if let Ok(content) = fs::read_to_string(&file_path).await {
                // ファイル名をヘッダーとして追加
                let relative_path = file_path
                    .strip_prefix(&self.config.base_docs_dir)
                    .unwrap_or(&file_path)
                    .to_string_lossy();

                merged_content.push_str(&format!("# {}\n\n{}\n\n", relative_path, content.trim()));
            }
        }

        Ok(merged_content.trim().to_string())
    }

    /// split用：個別ファイルの内容を取得
    pub async fn get_individual_files(&self) -> Result<Vec<(String, String)>> {
        let docs_dir = Path::new(&self.config.base_docs_dir);

        if !docs_dir.exists() {
            return Ok(Vec::new());
        }

        let markdown_files = self.find_markdown_files(docs_dir).await?;
        let mut files = Vec::new();

        for file_path in markdown_files {
            if let Ok(content) = fs::read_to_string(&file_path).await {
                let relative_path = file_path
                    .strip_prefix(&self.config.base_docs_dir)
                    .unwrap_or(&file_path)
                    .to_string_lossy()
                    .to_string();

                files.push((relative_path, content));
            }
        }

        Ok(files)
    }

    /// 指定ディレクトリから再帰的に.mdファイルを検索
    async fn find_markdown_files(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        use std::collections::VecDeque;

        let mut files = Vec::new();
        let mut dirs_to_process = VecDeque::new();
        dirs_to_process.push_back(dir.to_path_buf());

        while let Some(current_dir) = dirs_to_process.pop_front() {
            let mut entries = fs::read_dir(&current_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    // ディレクトリの場合は処理キューに追加
                    dirs_to_process.push_back(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    // .mdファイルの場合はリストに追加
                    files.push(path);
                }
            }
        }

        // ファイル名でソート（一貫した順序で処理）
        files.sort();
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentConfig, OutputMode};
    use tempfile::tempdir;
    use tokio::fs;

    fn create_test_config(base_dir: &str) -> AIContextConfig {
        AIContextConfig {
            version: "1.0".to_string(),
            output_mode: Some(OutputMode::Merged),
            base_docs_dir: base_dir.to_string(),
            agents: AgentConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_merge_all_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_merge_all_nonexistent_directory() {
        let config = create_test_config("/nonexistent/path");
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_merge_all_single_file() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用markdownファイルを作成
        fs::write(
            docs_path.join("test.md"),
            "# Test Content\n\nThis is a test.",
        )
        .await
        .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.contains("# test.md"));
        assert!(result.contains("# Test Content"));
        assert!(result.contains("This is a test."));
    }

    #[tokio::test]
    async fn test_merge_all_multiple_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // 複数のテスト用ファイルを作成
        fs::write(docs_path.join("file1.md"), "# File 1\nContent 1")
            .await
            .unwrap();
        fs::write(docs_path.join("file2.md"), "# File 2\nContent 2")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.contains("# file1.md"));
        assert!(result.contains("# File 1"));
        assert!(result.contains("Content 1"));
        assert!(result.contains("# file2.md"));
        assert!(result.contains("# File 2"));
        assert!(result.contains("Content 2"));
    }

    #[tokio::test]
    async fn test_merge_all_recursive_directories() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // サブディレクトリを作成
        let sub_dir = docs_path.join("subdir");
        fs::create_dir(&sub_dir).await.unwrap();

        // 各ディレクトリにファイルを作成
        fs::write(docs_path.join("root.md"), "Root content")
            .await
            .unwrap();
        fs::write(sub_dir.join("sub.md"), "Sub content")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.contains("# root.md"));
        assert!(result.contains("Root content"));
        assert!(result.contains("# subdir/sub.md") || result.contains("# subdir\\sub.md")); // Windows対応
        assert!(result.contains("Sub content"));
    }

    #[tokio::test]
    async fn test_get_individual_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("file1.md"), "Content 1")
            .await
            .unwrap();
        fs::write(docs_path.join("file2.md"), "Content 2")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let files = merger.get_individual_files().await.unwrap();
        assert_eq!(files.len(), 2);

        // ファイル名でソートされているかチェック
        assert_eq!(files[0].0, "file1.md");
        assert_eq!(files[0].1, "Content 1");
        assert_eq!(files[1].0, "file2.md");
        assert_eq!(files[1].1, "Content 2");
    }

    #[tokio::test]
    async fn test_ignore_non_markdown_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // 様々な拡張子のファイルを作成
        fs::write(docs_path.join("test.md"), "Markdown content")
            .await
            .unwrap();
        fs::write(docs_path.join("test.txt"), "Text file")
            .await
            .unwrap();
        fs::write(docs_path.join("test.json"), "{}").await.unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.contains("Markdown content"));
        assert!(!result.contains("Text file"));
        assert!(!result.contains("{}"));

        let files = merger.get_individual_files().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "test.md");
    }
}
