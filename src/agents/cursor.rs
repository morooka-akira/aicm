/*!
 * AI Context Management Tool - Cursor Agent (Simplified)
 *
 * シンプル化されたCursorエージェントの実装
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, CursorConfig, GeneratedFile, OutputMode};
use anyhow::Result;
use tokio::fs;

/// Cursorエージェント（シンプル版）
pub struct CursorAgent {
    config: AIContextConfig,
    base_dir: Option<String>,
}

impl CursorAgent {
    /// 新しいCursorエージェントを作成
    pub fn new(config: AIContextConfig) -> Self {
        Self {
            config,
            base_dir: None,
        }
    }

    /// ベースディレクトリ指定でCursorエージェントを作成
    #[cfg(test)]
    pub fn new_with_base_dir(config: AIContextConfig, base_dir: String) -> Self {
        Self {
            config,
            base_dir: Some(base_dir),
        }
    }

    /// Cursor用ファイルを生成
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let merger = MarkdownMerger::new(self.config.clone());

        match self.config.get_effective_output_mode("cursor") {
            OutputMode::Merged => self.generate_merged(&merger).await,
            OutputMode::Split => self.generate_split(&merger).await,
        }
    }

    /// 統合モード：1つのファイルに結合
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let content = merger.merge_all_with_options(Some("cursor")).await?;
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

        // split_config設定の確認
        let split_config = self.get_split_config();

        if let Some(config) = split_config {
            // split_config設定がある場合：ルールベースでファイルを処理
            let mut processed_files = std::collections::HashSet::new();

            // 各ルールに対してマッチするファイルを処理
            for rule in &config.rules {
                for (file_name, content) in &files {
                    if processed_files.contains(file_name) {
                        continue;
                    }
                    if self.file_matches_patterns(file_name, &rule.file_patterns) {
                        let mdc_content = self.create_mdc_content_with_rule(content, rule);
                        let base_name = file_name.trim_end_matches(".md");
                        let safe_name = base_name.replace(['/', '\\'], "_");

                        generated_files.push(GeneratedFile::new(
                            format!("{}/{}.mdc", rules_dir, safe_name),
                            mdc_content,
                        ));
                        processed_files.insert(file_name.clone());
                    }
                }
            }

            // マッチしなかったファイルはデフォルトのalwaysルールで処理
            for (file_name, content) in &files {
                if !processed_files.contains(file_name) {
                    let mdc_content = self.create_mdc_content(content);
                    let base_name = file_name.trim_end_matches(".md");
                    let safe_name = base_name.replace(['/', '\\'], "_");

                    generated_files.push(GeneratedFile::new(
                        format!("{}/{}.mdc", rules_dir, safe_name),
                        mdc_content,
                    ));
                }
            }
        } else {
            // 従来通りの動作：全ファイルを個別に変換（alwaysApply: trueのデフォルト）
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
        }

        Ok(generated_files)
    }

    /// rulesディレクトリのパスを取得
    fn get_rules_dir(&self) -> String {
        if let Some(base_dir) = &self.base_dir {
            format!("{}/.cursor/rules", base_dir)
        } else {
            ".cursor/rules".to_string()
        }
    }

    /// .cursor/rules/ ディレクトリを準備（既存ファイルを削除）
    async fn prepare_rules_directory(&self, rules_dir: &str) -> Result<()> {
        // ディレクトリが存在する場合、中身を削除
        if fs::metadata(rules_dir).await.is_ok() {
            let mut entries = fs::read_dir(rules_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "mdc") {
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

    /// YAML frontmatterを作成（デフォルト: alwaysApply: trueのみ）
    fn create_frontmatter(&self) -> String {
        let mut frontmatter = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
        frontmatter["alwaysApply"] = serde_yaml::Value::Bool(true);
        serde_yaml::to_string(&frontmatter).unwrap_or_default()
    }

    /// split_config設定を取得
    fn get_split_config(&self) -> Option<&crate::types::CursorSplitConfig> {
        match &self.config.agents.cursor {
            CursorConfig::Advanced(config) => config.split_config.as_ref(),
            _ => None,
        }
    }

    /// ファイル名がパターンにマッチするかチェック
    fn file_matches_patterns(&self, file_name: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if self.simple_pattern_match(file_name, pattern) {
                return true;
            }
        }
        false
    }

    /// 簡単なパターンマッチング（ワイルドカード対応）
    fn simple_pattern_match(&self, file_name: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // ワイルドカードを含む場合
            if pattern.starts_with('*') && pattern.ends_with('*') {
                // "*rust*" のようなパターン
                let middle = pattern.trim_start_matches('*').trim_end_matches('*');
                return file_name.contains(middle);
            } else if pattern.starts_with('*') {
                // "*rust" のようなパターン
                let suffix = pattern.trim_start_matches('*');
                return file_name.ends_with(suffix);
            } else if pattern.ends_with('*') {
                // "rust*" のようなパターン
                let prefix = pattern.trim_end_matches('*');
                return file_name.starts_with(prefix);
            }
        }

        // 完全一致
        file_name == pattern
    }

    /// ルール設定を含むMDC形式のコンテンツを作成
    fn create_mdc_content_with_rule(
        &self,
        markdown_content: &str,
        rule: &crate::types::CursorSplitRule,
    ) -> String {
        let frontmatter = self.create_frontmatter_with_rule(rule);
        format!("---\n{}\n---\n\n{}", frontmatter, markdown_content)
    }

    /// ルール設定を含むYAML frontmatterを作成
    fn create_frontmatter_with_rule(&self, rule: &crate::types::CursorSplitRule) -> String {
        let mut frontmatter = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());

        // 優先順位: manual > alwaysApply > globs > description
        if rule.manual == Some(true) {
            // Manual: manual: true
            frontmatter["manual"] = serde_yaml::Value::Bool(true);
        } else if rule.always_apply == Some(true) {
            // Always: alwaysApply: true
            frontmatter["alwaysApply"] = serde_yaml::Value::Bool(true);
        } else if let Some(globs) = &rule.globs {
            // Auto Attached: description（空）、globs、alwaysApply: false
            frontmatter["description"] = serde_yaml::Value::String("".to_string());
            match globs.len() {
                1 => {
                    frontmatter["globs"] = serde_yaml::Value::String(globs[0].clone());
                }
                len if len > 1 => {
                    let globs_values: Vec<serde_yaml::Value> = globs
                        .iter()
                        .map(|g| serde_yaml::Value::String(g.clone()))
                        .collect();
                    frontmatter["globs"] = serde_yaml::Value::Sequence(globs_values);
                }
                _ => {
                    // 0の場合は何もしない
                }
            }
            frontmatter["alwaysApply"] = serde_yaml::Value::Bool(false);
        } else if let Some(desc) = &rule.description {
            // Agent Requested: descriptionのみ
            frontmatter["description"] = serde_yaml::Value::String(desc.clone());
        } else {
            // デフォルト: Always
            frontmatter["alwaysApply"] = serde_yaml::Value::Bool(true);
        }

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
            output_mode: Some(output_mode),
            include_filenames: Some(true), // テスト用にヘッダーを有効化
            base_docs_dir: base_dir.to_string(),
            agents: AgentConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_generate_merged_empty() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy(), OutputMode::Merged);
        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        let expected_path = format!(
            "{}/.cursor/rules/context.mdc",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(files[0].path, expected_path);
        assert!(files[0].content.contains("---"));
        assert!(files[0].content.contains("alwaysApply: true"));
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
        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        let expected_path = format!(
            "{}/.cursor/rules/context.mdc",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(files[0].path, expected_path);
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
        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 2);

        // ファイル名とパスをチェック
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();
        let expected_path1 = format!(
            "{}/.cursor/rules/file1.mdc",
            temp_dir.path().to_string_lossy()
        );
        let expected_path2 = format!(
            "{}/.cursor/rules/file2.mdc",
            temp_dir.path().to_string_lossy()
        );
        assert!(paths.contains(&&expected_path1));
        assert!(paths.contains(&&expected_path2));

        // 内容をチェック
        for file in &files {
            assert!(file.content.contains("---"));
            assert!(file.content.contains("alwaysApply: true"));

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
        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);

        // パス区切り文字がアンダースコアに変換されているかチェック
        let expected_path = format!(
            "{}/.cursor/rules/subdir_nested.mdc",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(files[0].path, expected_path);
        assert!(files[0].content.contains("Nested content"));
    }

    #[tokio::test]
    async fn test_create_mdc_content() {
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent = CursorAgent::new(config);

        let mdc_content = agent.create_mdc_content("# Test\nContent here");

        assert!(mdc_content.starts_with("---"));
        assert!(mdc_content.contains("alwaysApply: true"));
        assert!(mdc_content.contains("alwaysApply:"));
        assert!(mdc_content.contains("---\n\n# Test\nContent here"));
    }

    #[tokio::test]
    async fn test_frontmatter_format() {
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent = CursorAgent::new(config);

        let frontmatter = agent.create_frontmatter();

        // YAML形式であることを確認
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
        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

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
        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

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
        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();

        // 正しいパスが生成されることを確認
        assert_eq!(files.len(), 1);
        let expected_path = format!(
            "{}/.cursor/rules/context.mdc",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(files[0].path, expected_path);
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
        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();

        // 正しいパスが生成されることを確認
        assert_eq!(files.len(), 2);
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();
        let expected_path1 = format!(
            "{}/.cursor/rules/file1.mdc",
            temp_dir.path().to_string_lossy()
        );
        let expected_path2 = format!(
            "{}/.cursor/rules/file2.mdc",
            temp_dir.path().to_string_lossy()
        );
        assert!(paths.contains(&&expected_path1));
        assert!(paths.contains(&&expected_path2));
    }

    // === split_config機能のテスト ===

    #[tokio::test]
    async fn test_split_config_manual_rule() {
        use crate::types::{CursorAgentConfig, CursorSplitConfig, CursorSplitRule};

        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        std::fs::write(docs_path.join("manual.md"), "Manual rule content").unwrap();

        let mut config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: Some(CursorSplitConfig {
                rules: vec![CursorSplitRule {
                    file_patterns: vec!["*manual*".to_string()],
                    manual: Some(true),
                    always_apply: None,
                    globs: None,
                    description: None,
                }],
            }),
        });

        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());
        let files = agent.generate().await.unwrap();

        // manual.mdcファイルが生成されることを確認
        let manual_file = files.iter().find(|f| f.path.contains("manual")).unwrap();
        assert!(manual_file.content.contains("manual: true"));
        assert!(!manual_file.content.contains("alwaysApply:"));
        assert!(!manual_file.content.contains("globs:"));
        assert!(!manual_file.content.contains("description:"));
    }

    #[tokio::test]
    async fn test_split_config_always_rule() {
        use crate::types::{CursorAgentConfig, CursorSplitConfig, CursorSplitRule};

        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        fs::write(docs_path.join("always.md"), "Always rule content")
            .await
            .unwrap();

        let mut config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: Some(CursorSplitConfig {
                rules: vec![CursorSplitRule {
                    file_patterns: vec!["*always*".to_string()],
                    manual: None,
                    always_apply: Some(true),
                    globs: None,
                    description: None,
                }],
            }),
        });

        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());
        let files = agent.generate().await.unwrap();

        let always_file = files.iter().find(|f| f.path.contains("always")).unwrap();
        assert!(always_file.content.contains("alwaysApply: true"));
        assert!(!always_file.content.contains("manual:"));
        assert!(!always_file.content.contains("globs:"));
        assert!(!always_file.content.contains("description:"));
    }

    #[tokio::test]
    async fn test_split_config_auto_attached_rule() {
        use crate::types::{CursorAgentConfig, CursorSplitConfig, CursorSplitRule};

        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        fs::write(docs_path.join("rust.md"), "Rust content")
            .await
            .unwrap();

        let mut config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: Some(CursorSplitConfig {
                rules: vec![CursorSplitRule {
                    file_patterns: vec!["*rust*".to_string()],
                    manual: None,
                    always_apply: None,
                    globs: Some(vec!["**/*.rs".to_string()]),
                    description: None,
                }],
            }),
        });

        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());
        let files = agent.generate().await.unwrap();

        let rust_file = files.iter().find(|f| f.path.contains("rust")).unwrap();
        assert!(rust_file.content.contains("description: ''"));
        assert!(rust_file.content.contains("globs: '**/*.rs'"));
        assert!(rust_file.content.contains("alwaysApply: false"));
        assert!(!rust_file.content.contains("manual:"));
    }

    #[tokio::test]
    async fn test_split_config_agent_requested_rule() {
        use crate::types::{CursorAgentConfig, CursorSplitConfig, CursorSplitRule};

        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        fs::write(docs_path.join("agent.md"), "Agent requested content")
            .await
            .unwrap();

        let mut config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: Some(CursorSplitConfig {
                rules: vec![CursorSplitRule {
                    file_patterns: vec!["*agent*".to_string()],
                    manual: None,
                    always_apply: None,
                    globs: None,
                    description: Some("Agent requested rule".to_string()),
                }],
            }),
        });

        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());
        let files = agent.generate().await.unwrap();

        let agent_file = files.iter().find(|f| f.path.contains("agent")).unwrap();
        assert!(agent_file
            .content
            .contains("description: Agent requested rule"));
        assert!(!agent_file.content.contains("manual:"));
        assert!(!agent_file.content.contains("alwaysApply:"));
        assert!(!agent_file.content.contains("globs:"));
    }

    #[tokio::test]
    async fn test_split_config_multiple_globs() {
        use crate::types::{CursorAgentConfig, CursorSplitConfig, CursorSplitRule};

        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        fs::write(docs_path.join("multi.md"), "Multi-glob content")
            .await
            .unwrap();

        let mut config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: Some(CursorSplitConfig {
                rules: vec![CursorSplitRule {
                    file_patterns: vec!["*multi*".to_string()],
                    manual: None,
                    always_apply: None,
                    globs: Some(vec!["**/*.rs".to_string(), "**/*.toml".to_string()]),
                    description: None,
                }],
            }),
        });

        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());
        let files = agent.generate().await.unwrap();

        let multi_file = files.iter().find(|f| f.path.contains("multi")).unwrap();
        assert!(multi_file.content.contains("description: ''"));
        assert!(multi_file.content.contains("globs:"));
        assert!(multi_file.content.contains("**/*.rs"));
        assert!(multi_file.content.contains("**/*.toml"));
        assert!(multi_file.content.contains("alwaysApply: false"));
    }

    #[tokio::test]
    async fn test_split_config_unmatched_files_default() {
        use crate::types::{CursorAgentConfig, CursorSplitConfig, CursorSplitRule};

        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        std::fs::write(docs_path.join("matched.md"), "Matched content").unwrap();
        std::fs::write(docs_path.join("unmatched.md"), "Unmatched content").unwrap();

        let mut config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: Some(CursorSplitConfig {
                rules: vec![CursorSplitRule {
                    file_patterns: vec!["matched.md".to_string()],
                    manual: Some(true),
                    always_apply: None,
                    globs: None,
                    description: None,
                }],
            }),
        });

        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());
        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 2);

        let matched_file = files.iter().find(|f| f.path.contains("matched")).unwrap();
        assert!(matched_file.content.contains("manual: true"));

        let unmatched_file = files.iter().find(|f| f.path.contains("unmatched")).unwrap();
        assert!(unmatched_file.content.contains("alwaysApply: true"));
        assert!(!unmatched_file.content.contains("manual:"));
    }

    #[tokio::test]
    async fn test_file_pattern_matching() {
        let config = create_test_config("./docs", OutputMode::Split);
        let agent = CursorAgent::new(config);

        // ワイルドカード前後のテスト
        assert!(agent.simple_pattern_match("architecture.md", "*architecture*"));
        assert!(agent.simple_pattern_match("rust-guide.md", "*rust*"));

        // ワイルドカード前のみ
        assert!(agent.simple_pattern_match("test.md", "*test.md"));
        assert!(!agent.simple_pattern_match("test-file.md", "*test.md"));

        // ワイルドカード後のみ
        assert!(agent.simple_pattern_match("config.md", "config*"));
        assert!(!agent.simple_pattern_match("my-config.md", "config*"));

        // 完全一致
        assert!(agent.simple_pattern_match("exact.md", "exact.md"));
        assert!(!agent.simple_pattern_match("exact-file.md", "exact.md"));
    }

    #[tokio::test]
    async fn test_rule_priority() {
        use crate::types::{CursorAgentConfig, CursorSplitConfig, CursorSplitRule};

        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        fs::write(docs_path.join("priority.md"), "Priority test content")
            .await
            .unwrap();

        let mut config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: Some(CursorSplitConfig {
                rules: vec![CursorSplitRule {
                    file_patterns: vec!["*priority*".to_string()],
                    manual: Some(true),
                    always_apply: Some(true),
                    globs: Some(vec!["**/*.rs".to_string()]),
                    description: Some("Test description".to_string()),
                }],
            }),
        });

        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());
        let files = agent.generate().await.unwrap();

        // manualが最優先なので、manual: trueのみ含まれるべき
        let priority_file = files.iter().find(|f| f.path.contains("priority")).unwrap();
        assert!(priority_file.content.contains("manual: true"));
        assert!(!priority_file.content.contains("alwaysApply:"));
        assert!(!priority_file.content.contains("globs:"));
        assert!(!priority_file.content.contains("description:"));
    }

    #[tokio::test]
    async fn test_split_config_no_duplicate_generation() {
        use crate::types::{CursorAgentConfig, CursorSplitConfig, CursorSplitRule};

        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        std::fs::write(docs_path.join("dup.md"), "Duplicate content").unwrap();

        let mut config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: Some(CursorSplitConfig {
                rules: vec![
                    CursorSplitRule {
                        file_patterns: vec!["*dup*".to_string()],
                        manual: Some(true),
                        always_apply: None,
                        globs: None,
                        description: None,
                    },
                    CursorSplitRule {
                        file_patterns: vec!["dup.md".to_string()],
                        manual: Some(true),
                        always_apply: None,
                        globs: None,
                        description: None,
                    },
                ],
            }),
        });

        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());
        let files = agent.generate().await.unwrap();

        assert_eq!(files.len(), 1);
    }
}
