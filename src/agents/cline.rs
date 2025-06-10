/*!
 * AI Context Management Tool - Cline Agent
 *
 * Cline エージェントの実装
 * 仕様: https://docs.cline.bot/features/cline-rules
 *
 * Split モード: .clinerules/ フォルダに複数の .md ファイル
 * Merged モード: .clinerules 単一ファイル（拡張子なし）
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, GeneratedFile, OutputMode};
use anyhow::Result;
use tokio::fs;

/// Cline エージェント
pub struct ClineAgent {
    config: AIContextConfig,
}

impl ClineAgent {
    /// 新しい Cline エージェントを作成
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    /// Cline 用ファイルを生成
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let merger = MarkdownMerger::new(self.config.clone());

        match self.config.get_effective_output_mode("cline") {
            OutputMode::Merged => self.generate_merged(&merger).await,
            OutputMode::Split => self.generate_split(&merger).await,
        }
    }

    /// Merged モード：.clinerules 単一ファイル（拡張子なし）
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let content = merger.merge_all_with_options(Some("cline")).await?;
        let output_path = self.get_merged_output_path();

        // 既存の .clinerules ディレクトリ（split モード用）が存在する場合は削除
        if let Ok(metadata) = fs::metadata(&output_path).await {
            if metadata.is_dir() {
                fs::remove_dir_all(&output_path).await?;
            }
        }

        Ok(vec![GeneratedFile::new(output_path, content)])
    }

    /// Split モード：.clinerules/ フォルダに複数の .md ファイル
    async fn generate_split(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let files = merger.get_individual_files().await?;
        let mut generated_files = Vec::new();

        // .clinerules/ ディレクトリを準備
        let rules_dir = self.get_split_rules_dir();
        self.prepare_rules_directory(&rules_dir).await?;

        for (file_name, content) in files {
            // ファイル名から拡張子を除去してmdファイル名を作成
            let base_name = file_name.trim_end_matches(".md");
            let safe_name = base_name.replace(['/', '\\'], "_"); // パス区切り文字をアンダースコアに変換

            // 元のファイル名を使用（数字プレフィックスなし）
            let output_filename = format!("{}.md", safe_name);

            generated_files.push(GeneratedFile::new(
                format!("{}/{}", rules_dir, output_filename),
                content,
            ));
        }

        Ok(generated_files)
    }

    /// Merged モードの出力パスを取得
    fn get_merged_output_path(&self) -> String {
        ".clinerules".to_string() // 拡張子なし
    }

    /// Split モードのルールディレクトリのパスを取得
    fn get_split_rules_dir(&self) -> String {
        ".clinerules".to_string() // フォルダ
    }

    /// .clinerules/ ディレクトリを準備（既存ファイルを削除）
    async fn prepare_rules_directory(&self, rules_dir: &str) -> Result<()> {
        // 既存の .clinerules ファイル（merged モード用）が存在する場合は削除
        if let Ok(metadata) = fs::metadata(rules_dir).await {
            if metadata.is_file() {
                fs::remove_file(rules_dir).await?;
            } else if metadata.is_dir() {
                // ディレクトリが存在する場合、中身を削除
                let mut entries = fs::read_dir(rules_dir).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                        fs::remove_file(path).await?;
                    }
                }
            }
        }

        // ディレクトリを作成（存在しない場合）
        fs::create_dir_all(rules_dir).await?;
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
            include_filenames: None,
            base_docs_dir: base_dir.to_string(),
            agents: AgentConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_generate_merged_empty() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy(), OutputMode::Merged);
        let agent = ClineAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, ".clinerules"); // 拡張子なし
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
        let agent = ClineAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, ".clinerules");

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
        let agent = ClineAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 2);

        // ファイル名とパスをチェック
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();
        assert!(paths.contains(&&".clinerules/file1.md".to_string()));
        assert!(paths.contains(&&".clinerules/file2.md".to_string()));

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
        let agent = ClineAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);

        // パス区切り文字がアンダースコアに変換されていることを確認
        assert_eq!(files[0].path, ".clinerules/subdir_nested.md");
        assert!(files[0].content.contains("Nested content"));
    }

    #[tokio::test]
    async fn test_get_merged_output_path() {
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent = ClineAgent::new(config);

        let output_path = agent.get_merged_output_path();
        assert_eq!(output_path, ".clinerules"); // 拡張子なし
    }

    #[tokio::test]
    async fn test_get_split_rules_dir() {
        let config = create_test_config("./docs", OutputMode::Split);
        let agent = ClineAgent::new(config);

        let rules_dir = agent.get_split_rules_dir();
        assert_eq!(rules_dir, ".clinerules"); // フォルダ
    }

    #[tokio::test]
    async fn test_prepare_rules_directory() {
        let temp_dir = tempdir().unwrap();
        let rules_dir = temp_dir.path().join(".clinerules");
        let config = create_test_config("./docs", OutputMode::Split);
        let agent = ClineAgent::new(config);

        // ディレクトリを作成
        fs::create_dir_all(&rules_dir).await.unwrap();

        // 既存のmdファイルを作成
        let existing_md = rules_dir.join("old_file.md");
        let other_file = rules_dir.join("keep_me.txt");
        fs::write(&existing_md, "old content").await.unwrap();
        fs::write(&other_file, "keep this").await.unwrap();

        // ファイルが存在することを確認
        assert!(existing_md.exists());
        assert!(other_file.exists());

        // prepare_rules_directory を実行
        agent
            .prepare_rules_directory(&rules_dir.to_string_lossy())
            .await
            .unwrap();

        // mdファイルは削除され、他のファイルは残っていることを確認
        assert!(!existing_md.exists());
        assert!(other_file.exists());
    }

    #[tokio::test]
    async fn test_simple_filename_generation() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // 複数のテスト用ファイルを作成
        fs::write(docs_path.join("apple.md"), "Apple content")
            .await
            .unwrap();
        fs::write(docs_path.join("banana.md"), "Banana content")
            .await
            .unwrap();
        fs::write(docs_path.join("cherry.md"), "Cherry content")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        let agent = ClineAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 3);

        // シンプルなファイル名（数字プレフィックスなし）を確認
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();

        // 元のファイル名が保持されることを確認
        assert!(paths.iter().any(|p| p.contains("apple.md")));
        assert!(paths.iter().any(|p| p.contains("banana.md")));
        assert!(paths.iter().any(|p| p.contains("cherry.md")));
    }
}
