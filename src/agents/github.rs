/*!
 * AI Context Management Tool - GitHub Agent
 *
 * GitHub エージェントの実装
 * 仕様: https://code.visualstudio.com/docs/copilot/copilot-customization#_use-instructionsmd-files
 *
 * Split モード: .github/prompts/ フォルダに複数の .md ファイル
 * Merged モード: .github/copilot-instructions.md 単一ファイル
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, GeneratedFile, OutputMode};
use anyhow::Result;
use tokio::fs;

/// GitHub エージェント
pub struct GitHubAgent {
    config: AIContextConfig,
}

impl GitHubAgent {
    /// 新しい GitHub エージェントを作成
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    /// GitHub 用ファイルを生成
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let merger = MarkdownMerger::new(self.config.clone());

        match self.config.output_mode {
            OutputMode::Merged => self.generate_merged(&merger).await,
            OutputMode::Split => self.generate_split(&merger).await,
        }
    }

    /// Merged モード：.github/copilot-instructions.md 単一ファイル
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let content = merger.merge_all().await?;
        let output_path = self.get_merged_output_path();

        // .github ディレクトリを作成
        fs::create_dir_all(".github").await?;

        // 既存の .github/prompts/ ディレクトリ内の .md ファイルを削除（split モード用）
        let prompts_dir = self.get_split_prompts_dir();
        if fs::metadata(&prompts_dir).await.is_ok() {
            let mut entries = fs::read_dir(&prompts_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                    fs::remove_file(path).await?;
                }
            }
        }

        Ok(vec![GeneratedFile::new(output_path, content)])
    }

    /// Split モード：.github/prompts/ フォルダに複数の .md ファイル
    async fn generate_split(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let files = merger.get_individual_files().await?;
        let mut generated_files = Vec::new();

        // .github/prompts/ ディレクトリを準備
        let prompts_dir = self.get_split_prompts_dir();
        self.prepare_prompts_directory(&prompts_dir).await?;

        for (file_name, content) in files {
            // ファイル名から拡張子を除去してmdファイル名を作成
            let base_name = file_name.trim_end_matches(".md");
            let safe_name = base_name.replace(['/', '\\'], "_"); // パス区切り文字をアンダースコアに変換

            generated_files.push(GeneratedFile::new(
                format!("{}/{}.md", prompts_dir, safe_name),
                content,
            ));
        }

        Ok(generated_files)
    }

    /// Merged モードの出力パスを取得
    fn get_merged_output_path(&self) -> String {
        ".github/copilot-instructions.md".to_string()
    }

    /// Split モードのプロンプトディレクトリのパスを取得
    fn get_split_prompts_dir(&self) -> String {
        ".github/prompts".to_string()
    }

    /// .github/prompts/ ディレクトリを準備（既存ファイルを削除）
    async fn prepare_prompts_directory(&self, prompts_dir: &str) -> Result<()> {
        // 既存の .github/copilot-instructions.md ファイルを削除（merged モード用）
        let merged_file = self.get_merged_output_path();
        if fs::metadata(&merged_file).await.is_ok() {
            fs::remove_file(&merged_file).await?;
        }

        // ディレクトリが存在する場合、中身を削除
        if fs::metadata(prompts_dir).await.is_ok() {
            let mut entries = fs::read_dir(prompts_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                    fs::remove_file(path).await?;
                }
            }
        }

        // ディレクトリを作成（存在しない場合）
        fs::create_dir_all(prompts_dir).await?;
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
            output_mode,
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
        assert!(paths.contains(&&".github/prompts/file1.md".to_string()));
        assert!(paths.contains(&&".github/prompts/file2.md".to_string()));

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
        assert_eq!(files[0].path, ".github/prompts/subdir_nested.md");
        assert!(files[0].content.contains("Nested content"));
    }

    #[tokio::test]
    async fn test_get_merged_output_path() {
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent = GitHubAgent::new(config);

        let output_path = agent.get_merged_output_path();
        assert_eq!(output_path, ".github/copilot-instructions.md");
    }

    #[tokio::test]
    async fn test_get_split_prompts_dir() {
        let config = create_test_config("./docs", OutputMode::Split);
        let agent = GitHubAgent::new(config);

        let prompts_dir = agent.get_split_prompts_dir();
        assert_eq!(prompts_dir, ".github/prompts");
    }

    #[tokio::test]
    async fn test_prepare_prompts_directory() {
        let temp_dir = tempdir().unwrap();
        let prompts_dir = temp_dir.path().join(".github/prompts");
        let config = create_test_config("./docs", OutputMode::Split);
        let agent = GitHubAgent::new(config);

        // ディレクトリを作成
        fs::create_dir_all(&prompts_dir).await.unwrap();

        // 既存のmdファイルを作成
        let existing_md = prompts_dir.join("old_file.md");
        let other_file = prompts_dir.join("keep_me.txt");
        fs::write(&existing_md, "old content").await.unwrap();
        fs::write(&other_file, "keep this").await.unwrap();

        // ファイルが存在することを確認
        assert!(existing_md.exists());
        assert!(other_file.exists());

        // prepare_prompts_directory を実行
        agent
            .prepare_prompts_directory(&prompts_dir.to_string_lossy())
            .await
            .unwrap();

        // mdファイルは削除され、他のファイルは残っていることを確認
        assert!(!existing_md.exists());
        assert!(other_file.exists());
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
    async fn test_split_vs_merged_output_paths() {
        let config_split = create_test_config("./docs", OutputMode::Split);
        let config_merged = create_test_config("./docs", OutputMode::Merged);

        let agent_split = GitHubAgent::new(config_split);
        let agent_merged = GitHubAgent::new(config_merged);

        // Split モードとMerged モードで異なるパスを使用することを確認
        assert_eq!(agent_split.get_split_prompts_dir(), ".github/prompts");
        assert_eq!(
            agent_merged.get_merged_output_path(),
            ".github/copilot-instructions.md"
        );
    }
}
