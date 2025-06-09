/*!
 * AI Context Management Tool - GitHub Copilot Agent
 *
 * GitHub Copilot用のコンテキストファイル生成エージェント
 * 仕様: https://code.visualstudio.com/docs/copilot/copilot-customization
 *
 * ファイル命名規則:
 * - 統合モード: .github/copilot-instructions.md
 * - 分割モード: .github/instructions/ 配下にmdファイルを生成
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, GeneratedFile, OutputMode};
use anyhow::Result;
use tokio::fs;

/// GitHub Copilotエージェント
pub struct GitHubAgent {
    config: AIContextConfig,
}

impl GitHubAgent {
    /// 新しいGitHub Copilotエージェントを作成
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    /// GitHub Copilot用ファイルを生成
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let merger = MarkdownMerger::new(self.config.clone());

        match self.config.get_effective_output_mode("github") {
            OutputMode::Merged => self.generate_merged(&merger).await,
            OutputMode::Split => self.generate_split(&merger).await,
        }
    }

    /// 統合モード：.github/copilot-instructions.md ファイルを生成
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let content = merger.merge_all().await?;

        // GitHub Copilotは通常のMarkdownファイル（フロントマターなし）
        let instructions_content = self.create_instructions_content(&content);

        // 既存の *.prompt.md ファイルを削除（split モード用）
        self.cleanup_split_files().await?;

        // .githubディレクトリを作成
        tokio::fs::create_dir_all(".github").await?;

        Ok(vec![GeneratedFile::new(
            ".github/copilot-instructions.md".to_string(),
            instructions_content,
        )])
    }

    /// 分割モード：.github/instructions/xxx.instructions.md ファイルを生成
    async fn generate_split(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let files = merger.get_individual_files().await?;
        let mut generated_files = Vec::new();

        // 既存の .github/copilot-instructions.md ファイルを削除（merged モード用）
        self.cleanup_merged_file().await?;

        // .github/instructions ディレクトリを作成
        tokio::fs::create_dir_all(".github/instructions").await?;

        // 既存の .instructions.md ファイルを削除
        self.cleanup_split_files().await?;

        for (file_name, content) in files {
            let instructions_content = self.create_instructions_content(&content);

            // ファイル名から拡張子を除去して .instructions.md を追加
            let base_name = file_name.trim_end_matches(".md");
            let safe_name = base_name.replace(['/', '\\'], "_"); // パス区切り文字をアンダースコアに変換

            generated_files.push(GeneratedFile::new(
                format!(".github/instructions/{}.instructions.md", safe_name),
                instructions_content,
            ));
        }

        Ok(generated_files)
    }

    /// GitHub Copilot用のコンテンツを作成（純粋なMarkdown、フロントマターなし）
    fn create_instructions_content(&self, content: &str) -> String {
        content.to_string()
    }

    /// 分割モード用ファイル（.github/instructions/*.instructions.md）を削除
    async fn cleanup_split_files(&self) -> Result<()> {
        use tokio::fs;

        // .github/instructions がディレクトリでなければ何もしない
        let metadata = match fs::metadata(".github/instructions").await {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };

        if !metadata.is_dir() {
            return Ok(());
        }

        // ディレクトリ内の .instructions.md ファイルを削除
        let mut entries = fs::read_dir(".github/instructions").await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(".instructions.md") {
                        fs::remove_file(path).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 統合モード用ファイル（.github/copilot-instructions.md）を削除
    async fn cleanup_merged_file(&self) -> Result<()> {
        if fs::metadata(".github/copilot-instructions.md")
            .await
            .is_ok()
        {
            fs::remove_file(".github/copilot-instructions.md").await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentConfig, OutputMode};
    use tempfile::tempdir;
    use tokio::fs;

    fn create_test_config(base_dir: &str, output_mode: OutputMode) -> AIContextConfig {
        AIContextConfig {
            version: "1.0".to_string(),
            output_mode: Some(output_mode),
            base_docs_dir: base_dir.to_string(),
            agents: AgentConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_generate_merged_empty() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy(), OutputMode::Merged);
        let agent = GitHubAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, ".github/copilot-instructions.md");
    }

    #[tokio::test]
    async fn test_generate_merged_with_content() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test.md"), "# Test Content\nThis is a test.")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Merged);
        let agent = GitHubAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, ".github/copilot-instructions.md");

        // ファイル名のヘッダーが含まれることを確認
        assert!(files[0].content.contains("# test.md"));
        // 元のコンテンツが含まれることを確認
        assert!(files[0].content.contains("# Test Content"));
        assert!(files[0].content.contains("This is a test."));
    }

    #[tokio::test]
    async fn test_generate_split_multiple_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // 複数のテスト用ファイルを作成
        fs::write(docs_path.join("file1.md"), "Content 1")
            .await
            .unwrap();
        fs::write(docs_path.join("file2.md"), "Content 2")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        let agent = GitHubAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 2);

        // ファイル名とパスをチェック
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();
        assert!(paths.contains(&&".github/instructions/file1.instructions.md".to_string()));
        assert!(paths.contains(&&".github/instructions/file2.instructions.md".to_string()));

        // 内容をチェック
        for file in &files {
            if file.path.contains("file1") {
                assert!(file.content.contains("Content 1"));
            } else if file.path.contains("file2") {
                assert!(file.content.contains("Content 2"));
            }
        }
    }

    #[tokio::test]
    async fn test_generate_split_with_subdirectory() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // サブディレクトリを作成
        let sub_dir = docs_path.join("subdir");
        fs::create_dir(&sub_dir).await.unwrap();
        fs::write(sub_dir.join("nested.md"), "Nested content")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        let agent = GitHubAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);

        // パス区切り文字がアンダースコアに変換されていることを確認
        assert_eq!(
            files[0].path,
            ".github/instructions/subdir_nested.instructions.md"
        );
        assert!(files[0].content.contains("Nested content"));
    }

    #[tokio::test]
    async fn test_generate_creates_pure_markdown() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test.md"), "# Test\nContent here")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Merged);
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
    async fn test_cleanup_split_files_ignores_file_path() {
        // .github/instructions がファイルの場合でもエラーなく終了すること
        let temp_dir = tempdir().unwrap();

        // setup: instructions をファイルとして作成
        fs::create_dir_all(".github").await.unwrap();
        // 既存のパスを削除してからファイルを作成
        let _ = fs::remove_file(".github/instructions").await;
        let _ = fs::remove_dir_all(".github/instructions").await;
        fs::write(".github/instructions", "dummy").await.unwrap();

        let config = create_test_config(&temp_dir.path().to_string_lossy(), OutputMode::Split);
        let agent = GitHubAgent::new(config);

        // 実行してもエラーが発生しないこと
        agent.cleanup_split_files().await.unwrap();

        // ファイルはそのまま残っていることを確認
        let metadata = fs::metadata(".github/instructions").await.unwrap();
        assert!(metadata.is_file());

        // 後片付け
        fs::remove_file(".github/instructions").await.unwrap();
    }
}
