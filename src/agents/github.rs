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
use crate::types::{AIContextConfig, GeneratedFile, GitHubSplitRule, OutputMode};
use anyhow::Result;
use tokio::fs;

/// GitHub Copilotエージェント
pub struct GitHubAgent {
    config: AIContextConfig,
    base_dir: Option<String>,
}

impl GitHubAgent {
    /// 新しいGitHub Copilotエージェントを作成
    pub fn new(config: AIContextConfig) -> Self {
        Self {
            config,
            base_dir: None,
        }
    }

    /// ベースディレクトリ指定でGitHub Copilotエージェントを作成
    #[cfg(test)]
    pub fn new_with_base_dir(config: AIContextConfig, base_dir: String) -> Self {
        Self {
            config,
            base_dir: Some(base_dir),
        }
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
        let content = merger.merge_all_with_options(Some("github")).await?;

        // GitHub Copilotは通常のMarkdownファイル（フロントマターなし）
        let instructions_content = self.create_instructions_content(&content);

        // 既存の *.prompt.md ファイルを削除（split モード用）
        self.cleanup_split_files().await?;

        // .githubディレクトリを作成
        let github_dir = self.get_github_dir();
        tokio::fs::create_dir_all(&github_dir).await?;

        let output_path = if let Some(base_dir) = &self.base_dir {
            format!("{}/.github/copilot-instructions.md", base_dir)
        } else {
            ".github/copilot-instructions.md".to_string()
        };

        Ok(vec![GeneratedFile::new(output_path, instructions_content)])
    }

    /// 分割モード：.github/instructions/xxx.instructions.md ファイルを生成
    async fn generate_split(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let files = merger.get_individual_files().await?;
        let mut generated_files = Vec::new();

        // 既存の .github/copilot-instructions.md ファイルを削除（merged モード用）
        self.cleanup_merged_file().await?;

        // .github/instructions ディレクトリを作成
        let instructions_dir = self.get_instructions_dir();
        tokio::fs::create_dir_all(&instructions_dir).await?;

        // 既存の .instructions.md ファイルを削除
        self.cleanup_split_files().await?;

        // split_config が設定されている場合は、そのルールに従って生成
        if let Some(github_config) = &self.config.agents.github.get_advanced_config() {
            if let Some(split_config) = &github_config.split_config {
                return self
                    .generate_split_with_config(&files, &split_config.rules)
                    .await;
            }
        }

        // split_config が設定されていない場合は、従来通りの生成
        for (file_name, content) in files {
            let instructions_content = self.create_instructions_content(&content);

            // ファイル名から拡張子を除去して .instructions.md を追加
            let base_name = file_name.trim_end_matches(".md");
            let safe_name = base_name.replace(['/', '\\'], "_"); // パス区切り文字をアンダースコアに変換

            let output_path = if let Some(base_dir) = &self.base_dir {
                format!(
                    "{}/.github/instructions/{}.instructions.md",
                    base_dir, safe_name
                )
            } else {
                format!(".github/instructions/{}.instructions.md", safe_name)
            };

            generated_files.push(GeneratedFile::new(output_path, instructions_content));
        }

        Ok(generated_files)
    }

    /// split_config のルールに従ってファイルを生成
    async fn generate_split_with_config(
        &self,
        files: &[(String, String)],
        rules: &[GitHubSplitRule],
    ) -> Result<Vec<GeneratedFile>> {
        let mut generated_files = Vec::new();
        let mut processed_files = std::collections::HashSet::new();

        // 各ルールに対してマッチするファイルを処理
        for rule in rules {
            // ルールにマッチするファイルを収集
            for (file_name, content) in files {
                if processed_files.contains(file_name.as_str()) {
                    continue;
                }

                // ファイルがルールにマッチするかチェック
                let matches = rule
                    .file_patterns
                    .iter()
                    .any(|pattern| self.file_matches_pattern(file_name, pattern));

                if matches {
                    // マッチしたファイルを処理済みとしてマーク
                    processed_files.insert(file_name.as_str());

                    // applyTo フロントマターを追加
                    let instructions_content =
                        self.create_instructions_content_with_apply_to(content, &rule.apply_to);

                    // 元のファイル名を保持
                    let base_name = file_name.trim_end_matches(".md");
                    let safe_name = base_name.replace(['/', '\\'], "_");

                    let output_path = if let Some(base_dir) = &self.base_dir {
                        format!(
                            "{}/.github/instructions/{}.instructions.md",
                            base_dir, safe_name
                        )
                    } else {
                        format!(".github/instructions/{}.instructions.md", safe_name)
                    };

                    generated_files.push(GeneratedFile::new(output_path, instructions_content));
                }
            }
        }

        // マッチしなかったファイルはデフォルトでそのまま出力（apply_toなし）
        for (file_name, content) in files {
            if !processed_files.contains(file_name.as_str()) {
                let instructions_content = self.create_instructions_content(content);
                let base_name = file_name.trim_end_matches(".md");
                let safe_name = base_name.replace(['/', '\\'], "_");

                let output_path = if let Some(base_dir) = &self.base_dir {
                    format!(
                        "{}/.github/instructions/{}.instructions.md",
                        base_dir, safe_name
                    )
                } else {
                    format!(".github/instructions/{}.instructions.md", safe_name)
                };

                generated_files.push(GeneratedFile::new(output_path, instructions_content));
            }
        }

        Ok(generated_files)
    }

    /// ファイル名がパターンにマッチするかチェック
    fn file_matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        // シンプルなワイルドカードマッチング
        if pattern.contains('*') {
            // "*pattern*" のようなパターンの場合
            if pattern.starts_with('*') && pattern.ends_with('*') {
                let middle = &pattern[1..pattern.len() - 1];
                return file_name.contains(middle);
            }
            // "*pattern" のようなパターンの場合
            if let Some(suffix) = pattern.strip_prefix('*') {
                return file_name.ends_with(suffix);
            }
            // "pattern*" のようなパターンの場合
            if let Some(prefix) = pattern.strip_suffix('*') {
                return file_name.starts_with(prefix);
            }
        }

        // 完全一致またはサブストリング一致
        file_name.contains(pattern)
    }

    /// GitHub Copilot用のコンテンツを作成（純粋なMarkdown、フロントマターなし）
    fn create_instructions_content(&self, content: &str) -> String {
        content.to_string()
    }

    /// applyTo フロントマターを含むGitHub Copilot用のコンテンツを作成
    fn create_instructions_content_with_apply_to(
        &self,
        content: &str,
        apply_to: &Option<Vec<String>>,
    ) -> String {
        match apply_to {
            Some(patterns) if !patterns.is_empty() => {
                let apply_to_value = patterns.join(",");
                format!("---\napplyTo: \"{}\"\n---\n\n{}", apply_to_value, content)
            }
            _ => content.to_string(),
        }
    }

    /// 分割モード用ファイル（.github/instructions/*.instructions.md）を削除
    async fn cleanup_split_files(&self) -> Result<()> {
        use tokio::fs;

        let instructions_dir = self.get_instructions_dir();

        // .github/instructions がディレクトリでなければ何もしない
        let metadata = match fs::metadata(&instructions_dir).await {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };

        if !metadata.is_dir() {
            return Ok(());
        }

        // ディレクトリ内の .instructions.md ファイルを削除
        let mut entries = fs::read_dir(&instructions_dir).await?;
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
        let merged_file_path = if let Some(base_dir) = &self.base_dir {
            format!("{}/.github/copilot-instructions.md", base_dir)
        } else {
            ".github/copilot-instructions.md".to_string()
        };

        if fs::metadata(&merged_file_path).await.is_ok() {
            fs::remove_file(&merged_file_path).await?;
        }
        Ok(())
    }

    /// GitHubディレクトリのパスを取得
    fn get_github_dir(&self) -> String {
        if let Some(base_dir) = &self.base_dir {
            format!("{}/.github", base_dir)
        } else {
            ".github".to_string()
        }
    }

    /// GitHub instructionsディレクトリのパスを取得
    fn get_instructions_dir(&self) -> String {
        if let Some(base_dir) = &self.base_dir {
            format!("{}/.github/instructions", base_dir)
        } else {
            ".github/instructions".to_string()
        }
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
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        let expected_path = format!(
            "{}/.github/copilot-instructions.md",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(files[0].path, expected_path);
    }

    #[tokio::test]
    async fn test_generate_merged_with_content() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        std::fs::write(docs_path.join("test.md"), "# Test Content\nThis is a test.").unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Merged);
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        let expected_path = format!(
            "{}/.github/copilot-instructions.md",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(files[0].path, expected_path);

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
        std::fs::write(docs_path.join("file1.md"), "Content 1").unwrap();
        std::fs::write(docs_path.join("file2.md"), "Content 2").unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 2);

        // ファイル名とパスをチェック
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();
        let expected_path1 = format!(
            "{}/.github/instructions/file1.instructions.md",
            temp_dir.path().to_string_lossy()
        );
        let expected_path2 = format!(
            "{}/.github/instructions/file2.instructions.md",
            temp_dir.path().to_string_lossy()
        );
        assert!(paths.contains(&&expected_path1));
        assert!(paths.contains(&&expected_path2));

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
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);

        // パス区切り文字がアンダースコアに変換されていることを確認
        let expected_path = format!(
            "{}/.github/instructions/subdir_nested.instructions.md",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(files[0].path, expected_path);
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
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

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
        let github_dir = temp_dir.path().join(".github");
        std::fs::create_dir_all(&github_dir).unwrap();

        // 既存のパスを削除してからファイルを作成
        let instructions_path = github_dir.join("instructions");
        let _ = std::fs::remove_file(&instructions_path);
        let _ = std::fs::remove_dir_all(&instructions_path);
        std::fs::write(&instructions_path, "dummy").unwrap();

        let config = create_test_config(&temp_dir.path().to_string_lossy(), OutputMode::Split);
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        // 実行してもエラーが発生しないこと
        agent.cleanup_split_files().await.unwrap();

        // ファイルはそのまま残っていることを確認
        let metadata = std::fs::metadata(&instructions_path).unwrap();
        assert!(metadata.is_file());
    }

    #[tokio::test]
    async fn test_generate_split_with_apply_to() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        std::fs::write(
            docs_path.join("architecture.md"),
            "# Architecture\nSystem design",
        )
        .unwrap();
        std::fs::write(docs_path.join("frontend.md"), "# Frontend\nUI components").unwrap();

        // split_config 付きの設定を作成
        use crate::types::{GitHubAgentConfig, GitHubConfig, GitHubSplitConfig};
        let github_config = GitHubConfig::Advanced(GitHubAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: Some(GitHubSplitConfig {
                rules: vec![
                    GitHubSplitRule {
                        file_patterns: vec!["*architecture*".to_string()],
                        apply_to: Some(vec!["**/*.rs".to_string(), "**/*.toml".to_string()]),
                    },
                    GitHubSplitRule {
                        file_patterns: vec!["*frontend*".to_string()],
                        apply_to: Some(vec!["**/*.ts".to_string(), "**/*.tsx".to_string()]),
                    },
                ],
            }),
        });

        let mut config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        config.agents.github = github_config;

        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());
        let files = agent.generate().await.unwrap();

        assert_eq!(files.len(), 2);

        // アーキテクチャファイルをチェック
        let arch_file = files
            .iter()
            .find(|f| f.path.contains("architecture"))
            .unwrap();
        assert!(arch_file.content.contains("---"));
        assert!(arch_file.content.contains("applyTo: \"**/*.rs,**/*.toml\""));
        assert!(arch_file.content.contains("# Architecture"));

        // フロントエンドファイルをチェック
        let frontend_file = files.iter().find(|f| f.path.contains("frontend")).unwrap();
        assert!(frontend_file.content.contains("---"));
        assert!(frontend_file
            .content
            .contains("applyTo: \"**/*.ts,**/*.tsx\""));
        assert!(frontend_file.content.contains("# Frontend"));
    }

    #[tokio::test]
    async fn test_file_matches_pattern() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy(), OutputMode::Split);
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        // "*pattern*" のテスト
        assert!(agent.file_matches_pattern("test-architecture-doc.md", "*architecture*"));
        assert!(!agent.file_matches_pattern("frontend.md", "*architecture*"));

        // "*pattern" のテスト
        assert!(agent.file_matches_pattern("setup.md", "*setup.md"));
        assert!(!agent.file_matches_pattern("setup-guide.md", "*setup.md"));

        // "pattern*" のテスト
        assert!(agent.file_matches_pattern("frontend-components.md", "frontend*"));
        assert!(!agent.file_matches_pattern("my-frontend.md", "frontend*"));

        // 完全一致のテスト
        assert!(agent.file_matches_pattern("readme.md", "readme"));
        assert!(agent.file_matches_pattern("my-readme-file.md", "readme"));
    }

    #[tokio::test]
    async fn test_create_instructions_content_with_apply_to() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy(), OutputMode::Split);
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        // applyTo が設定されている場合
        let content_with_apply_to = agent.create_instructions_content_with_apply_to(
            "Test content",
            &Some(vec!["**/*.ts".to_string(), "**/*.tsx".to_string()]),
        );
        assert!(content_with_apply_to.starts_with("---"));
        assert!(content_with_apply_to.contains("applyTo: \"**/*.ts,**/*.tsx\""));
        assert!(content_with_apply_to.contains("Test content"));

        // applyTo が設定されていない場合
        let content_without_apply_to =
            agent.create_instructions_content_with_apply_to("Test content", &None);
        assert_eq!(content_without_apply_to, "Test content");

        // applyTo が空配列の場合
        let content_empty_apply_to =
            agent.create_instructions_content_with_apply_to("Test content", &Some(vec![]));
        assert_eq!(content_empty_apply_to, "Test content");
    }

    #[tokio::test]
    async fn test_generate_split_with_apply_to_and_unmatched_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        std::fs::write(
            docs_path.join("03_architecture.md"),
            "# Architecture\nSystem design",
        )
        .unwrap();
        std::fs::write(
            docs_path.join("02_frontend.md"),
            "# Frontend\nUI components",
        )
        .unwrap();
        std::fs::write(
            docs_path.join("01_security.md"),
            "# Security\nSecurity guidelines",
        )
        .unwrap();

        // split_config 付きの設定を作成（securityファイルはマッチしない）
        use crate::types::{GitHubAgentConfig, GitHubConfig, GitHubSplitConfig};
        let github_config = GitHubConfig::Advanced(GitHubAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: Some(GitHubSplitConfig {
                rules: vec![
                    GitHubSplitRule {
                        file_patterns: vec!["*architecture*".to_string()],
                        apply_to: Some(vec!["**/*.rs".to_string(), "**/*.toml".to_string()]),
                    },
                    GitHubSplitRule {
                        file_patterns: vec!["*frontend*".to_string()],
                        apply_to: Some(vec!["**/*.ts".to_string(), "**/*.tsx".to_string()]),
                    },
                ],
            }),
        });

        let mut config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        config.agents.github = github_config;

        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());
        let files = agent.generate().await.unwrap();

        assert_eq!(files.len(), 3); // architecture, frontend, security

        // アーキテクチャファイル（applyTo付き、元のファイル名保持）をチェック
        let arch_file = files
            .iter()
            .find(|f| f.path.contains("03_architecture"))
            .unwrap();
        let expected_arch_path = format!(
            "{}/.github/instructions/03_architecture.instructions.md",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(arch_file.path, expected_arch_path);
        assert!(arch_file.content.contains("---"));
        assert!(arch_file.content.contains("applyTo: \"**/*.rs,**/*.toml\""));
        assert!(arch_file.content.contains("# Architecture"));

        // フロントエンドファイル（applyTo付き、元のファイル名保持）をチェック
        let frontend_file = files
            .iter()
            .find(|f| f.path.contains("02_frontend"))
            .unwrap();
        let expected_frontend_path = format!(
            "{}/.github/instructions/02_frontend.instructions.md",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(frontend_file.path, expected_frontend_path);
        assert!(frontend_file.content.contains("---"));
        assert!(frontend_file
            .content
            .contains("applyTo: \"**/*.ts,**/*.tsx\""));
        assert!(frontend_file.content.contains("# Frontend"));

        // セキュリティファイル（デフォルト、applyToなし、元のファイル名保持）をチェック
        let security_file = files
            .iter()
            .find(|f| f.path.contains("01_security"))
            .unwrap();
        let expected_security_path = format!(
            "{}/.github/instructions/01_security.instructions.md",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(security_file.path, expected_security_path);
        assert!(!security_file.content.contains("---"));
        assert!(!security_file.content.contains("applyTo:"));
        assert!(security_file.content.contains("# Security"));
    }
}
