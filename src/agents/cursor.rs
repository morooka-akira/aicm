/*!
 * AI Context Management Tool - Cursor Agent (Simplified)
 *
 * シンプル化されたCursorエージェントの実装
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, GeneratedFile, OutputMode};
use anyhow::Result;
use std::collections::HashMap;
use tokio::fs;

/// Cursorエージェント（シンプル版）
pub struct CursorAgent {
    config: AIContextConfig,
}

impl CursorAgent {
    /// 新しいCursorエージェントを作成
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    /// Cursor用ファイルを生成
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let merger = MarkdownMerger::new(self.config.clone());

        match self.config.output_mode {
            OutputMode::Merged => self.generate_merged(&merger).await,
            OutputMode::Split => self.generate_split(&merger).await,
        }
    }

    /// 統合モード：1つのファイルに結合
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let content = merger.merge_all().await?;
        let mdc_content = self.create_mdc_content(&content);

        // .cursor/rules/ ディレクトリを作成し、既存ファイルを削除
        let rules_dir = self.get_rules_dir();
        self.prepare_rules_directory(&rules_dir).await?;

        Ok(vec![GeneratedFile::new(
            format!("{}/context.mdc", rules_dir),
            mdc_content,
        )])
    }

    /// 分割モード：ファイルごとに分割
    async fn generate_split(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let files = merger.get_individual_files().await?;
        let mut generated_files = Vec::new();

        // .cursor/rules/ ディレクトリを作成し、既存ファイルを削除
        let rules_dir = self.get_rules_dir();
        self.prepare_rules_directory(&rules_dir).await?;

        for (file_name, content) in files {
            let mdc_content = self.create_mdc_content(&content);

            // ファイル名から拡張子を除去してmdcファイル名を作成
            let base_name = file_name.trim_end_matches(".md");
            let safe_name = base_name.replace(['/', '\\'], "_"); // パス区切り文字をアンダースコアに変換

            generated_files.push(GeneratedFile::new(
                format!("{}/{}.mdc", rules_dir, safe_name),
                mdc_content,
            ));
        }

        Ok(generated_files)
    }

    /// rulesディレクトリのパスを取得
    fn get_rules_dir(&self) -> String {
        ".cursor/rules".to_string()
    }

    /// .cursor/rules/ ディレクトリを準備（既存ファイルを削除）
    async fn prepare_rules_directory(&self, rules_dir: &str) -> Result<()> {
        // ディレクトリが存在する場合、中身を削除
        if fs::metadata(rules_dir).await.is_ok() {
            let mut entries = fs::read_dir(rules_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "mdc") {
                    fs::remove_file(path).await?;
                }
            }
        }

        // ディレクトリを作成（存在しない場合）
        fs::create_dir_all(rules_dir).await?;
        Ok(())
    }

    /// MDC形式のコンテンツを作成（YAML frontmatter + Markdown）
    fn create_mdc_content(&self, markdown_content: &str) -> String {
        let frontmatter = self.create_frontmatter();
        format!("---\n{}\n---\n\n{}", frontmatter, markdown_content)
    }

    /// YAML frontmatterを作成
    fn create_frontmatter(&self) -> String {
        let mut frontmatter = HashMap::new();
        frontmatter.insert("description", "AI Context Management generated rules");
        frontmatter.insert("alwaysApply", "true");

        serde_yaml::to_string(&frontmatter).unwrap_or_default()
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
        let agent = CursorAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, ".cursor/rules/context.mdc");
        assert!(files[0].content.contains("---"));
        assert!(files[0]
            .content
            .contains("description: AI Context Management generated rules"));
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
        let agent = CursorAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, ".cursor/rules/context.mdc");
        assert!(files[0].content.contains("# test.md"));
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
        let agent = CursorAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 2);

        // ファイル名とパスをチェック
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();
        assert!(paths.contains(&&".cursor/rules/file1.mdc".to_string()));
        assert!(paths.contains(&&".cursor/rules/file2.mdc".to_string()));

        // 内容をチェック
        for file in &files {
            assert!(file.content.contains("---"));
            assert!(file
                .content
                .contains("description: AI Context Management generated rules"));

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
        let agent = CursorAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);

        // パス区切り文字がアンダースコアに変換されているかチェック
        assert_eq!(files[0].path, ".cursor/rules/subdir_nested.mdc");
        assert!(files[0].content.contains("Nested content"));
    }

    #[tokio::test]
    async fn test_create_mdc_content() {
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent = CursorAgent::new(config);

        let mdc_content = agent.create_mdc_content("# Test\nContent here");

        assert!(mdc_content.starts_with("---"));
        assert!(mdc_content.contains("description: AI Context Management generated rules"));
        assert!(mdc_content.contains("alwaysApply:"));
        assert!(mdc_content.contains("---\n\n# Test\nContent here"));
    }

    #[tokio::test]
    async fn test_frontmatter_format() {
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent = CursorAgent::new(config);

        let frontmatter = agent.create_frontmatter();

        // YAML形式であることを確認
        assert!(frontmatter.contains("description:"));
        assert!(frontmatter.contains("alwaysApply:"));

        // パース可能であることを確認
        let parsed: serde_yaml::Value = serde_yaml::from_str(&frontmatter).unwrap();
        assert!(parsed.is_mapping());
    }

    #[tokio::test]
    async fn test_prepare_rules_directory_new() {
        let temp_dir = tempdir().unwrap();
        let rules_dir = temp_dir.path().join(".cursor/rules");
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent = CursorAgent::new(config);

        // ディレクトリが存在しない状態から開始
        assert!(!rules_dir.exists());

        // prepare_rules_directory を実行
        agent
            .prepare_rules_directory(&rules_dir.to_string_lossy())
            .await
            .unwrap();

        // ディレクトリが作成されたことを確認
        assert!(rules_dir.exists());
        assert!(rules_dir.is_dir());
    }

    #[tokio::test]
    async fn test_prepare_rules_directory_removes_existing_mdc_files() {
        let temp_dir = tempdir().unwrap();
        let rules_dir = temp_dir.path().join(".cursor/rules");
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent = CursorAgent::new(config);

        // ディレクトリを作成
        fs::create_dir_all(&rules_dir).await.unwrap();

        // 既存のmdcファイルを作成
        let existing_mdc = rules_dir.join("old_file.mdc");
        let other_file = rules_dir.join("keep_me.txt");
        fs::write(&existing_mdc, "old content").await.unwrap();
        fs::write(&other_file, "keep this").await.unwrap();

        // ファイルが存在することを確認
        assert!(existing_mdc.exists());
        assert!(other_file.exists());

        // prepare_rules_directory を実行
        agent
            .prepare_rules_directory(&rules_dir.to_string_lossy())
            .await
            .unwrap();

        // mdcファイルは削除され、他のファイルは残っていることを確認
        assert!(!existing_mdc.exists());
        assert!(other_file.exists());
    }

    #[tokio::test]
    async fn test_get_rules_dir() {
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent = CursorAgent::new(config);

        let rules_dir = agent.get_rules_dir();
        assert_eq!(rules_dir, ".cursor/rules");
    }

    #[tokio::test]
    async fn test_generate_merged_creates_correct_path() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test.md"), "# Test Content")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Merged);
        let agent = CursorAgent::new(config);

        let files = agent.generate().await.unwrap();

        // 正しいパスが生成されることを確認
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, ".cursor/rules/context.mdc");
        assert!(files[0].content.contains("# Test Content"));
    }

    #[tokio::test]
    async fn test_generate_split_creates_correct_paths() {
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
        let agent = CursorAgent::new(config);

        let files = agent.generate().await.unwrap();

        // 正しいパスが生成されることを確認
        assert_eq!(files.len(), 2);
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();
        assert!(paths.contains(&&".cursor/rules/file1.mdc".to_string()));
        assert!(paths.contains(&&".cursor/rules/file2.mdc".to_string()));
    }
}
