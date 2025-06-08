/*!
 * AI Context Management Tool - GitHub Agent (Simplified)
 *
 * シンプル化された GitHub エージェントの実装
 * GitHub 用の GITHUB.md を出力（merged モードのみ対応）
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, GeneratedFile};
use anyhow::Result;

/// GitHub エージェント（シンプル版）
pub struct GitHubAgent {
    config: AIContextConfig,
}

impl GitHubAgent {
    /// 新しい GitHub エージェントを作成
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    /// GitHub 用ファイルを生成（merged モードのみ）
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let merger = MarkdownMerger::new(self.config.clone());
        self.generate_merged(&merger).await
    }

    /// 統合モード：1つのファイルに結合して GITHUB.md として出力
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let content = merger.merge_all().await?;
        let output_path = self.get_output_path();

        Ok(vec![GeneratedFile::new(output_path, content)])
    }

    /// 出力パスを取得（プロジェクトルートの GITHUB.md）
    fn get_output_path(&self) -> String {
        "GITHUB.md".to_string()
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
            output_mode: OutputMode::Merged, // GitHub は merged のみ
            base_docs_dir: base_dir.to_string(),
            agents: AgentConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_generate_empty() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy());
        let agent = GitHubAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "GITHUB.md");
        // 空のディレクトリの場合は空のコンテンツでも正常
        // （MarkdownMerger が空のディレクトリに対して空文字列を返すため）
    }

    #[tokio::test]
    async fn test_generate_with_content() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test.md"), "# Test Content\nThis is a test.")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let agent = GitHubAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "GITHUB.md");

        // ファイル名のヘッダーが含まれることを確認
        assert!(files[0].content.contains("# test.md"));
        // 元のコンテンツが含まれることを確認
        assert!(files[0].content.contains("# Test Content"));
        assert!(files[0].content.contains("This is a test."));

        // 純粋な Markdown（frontmatter なし）であることを確認
        assert!(!files[0].content.starts_with("---"));
    }

    #[tokio::test]
    async fn test_generate_multiple_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // 複数のテスト用ファイルを作成
        fs::write(docs_path.join("file1.md"), "Content 1")
            .await
            .unwrap();
        fs::write(docs_path.join("file2.md"), "Content 2")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let agent = GitHubAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "GITHUB.md");

        // 両方のファイルの内容が含まれることを確認
        assert!(files[0].content.contains("Content 1"));
        assert!(files[0].content.contains("Content 2"));

        // ファイル名のヘッダーが含まれることを確認
        assert!(files[0].content.contains("# file1.md"));
        assert!(files[0].content.contains("# file2.md"));
    }

    #[tokio::test]
    async fn test_generate_with_subdirectory() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // サブディレクトリを作成
        let sub_dir = docs_path.join("subdir");
        fs::create_dir(&sub_dir).await.unwrap();
        fs::write(sub_dir.join("nested.md"), "Nested content")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let agent = GitHubAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "GITHUB.md");

        // サブディレクトリのファイルも含まれることを確認
        assert!(files[0].content.contains("Nested content"));
        assert!(files[0].content.contains("# subdir/nested.md"));
    }

    #[tokio::test]
    async fn test_get_output_path() {
        let config = create_test_config("./docs");
        let agent = GitHubAgent::new(config);

        let output_path = agent.get_output_path();
        assert_eq!(output_path, "GITHUB.md");
    }

    #[tokio::test]
    async fn test_generate_creates_pure_markdown() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test.md"), "# Test\nContent here")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let agent = GitHubAgent::new(config);

        let files = agent.generate().await.unwrap();
        let content = &files[0].content;

        // 純粋な Markdown であることを確認（YAML frontmatter なし）
        assert!(!content.starts_with("---"));
        assert!(!content.contains("description:"));
        assert!(!content.contains("alwaysApply:"));

        // 内容は含まれていることを確認
        assert!(content.contains("# Test"));
        assert!(content.contains("Content here"));
    }

    #[tokio::test]
    async fn test_generate_output_mode_ignored() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test.md"), "Test content")
            .await
            .unwrap();

        // Split モードで設定しても GitHub は merged で動作することを確認
        let mut config = create_test_config(&docs_path.to_string_lossy());
        config.output_mode = OutputMode::Split;

        let agent = GitHubAgent::new(config);
        let files = agent.generate().await.unwrap();

        // Split モードを指定しても 1 つのファイルのみ生成される
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "GITHUB.md");
        assert!(files[0].content.contains("Test content"));
    }
}
