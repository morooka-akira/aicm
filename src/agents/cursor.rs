/*!
 * AI Context Management Tool - Cursor Agent (Simplified)
 *
 * Simplified Cursor agent implementation
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, CursorConfig, GeneratedFile, OutputMode};
use anyhow::Result;
use tokio::fs;

/// Cursor agent (simplified version)
pub struct CursorAgent {
    config: AIContextConfig,
    base_dir: Option<String>,
}

impl CursorAgent {
    /// Create a new Cursor agent
    pub fn new(config: AIContextConfig) -> Self {
        Self {
            config,
            base_dir: None,
        }
    }

    /// Create Cursor agent with specified base directory
    #[cfg(test)]
    pub fn new_with_base_dir(config: AIContextConfig, base_dir: String) -> Self {
        Self {
            config,
            base_dir: Some(base_dir),
        }
    }

    /// Generate files for Cursor
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let merger = MarkdownMerger::new(self.config.clone());

        match self.config.get_effective_output_mode("cursor") {
            OutputMode::Merged => self.generate_merged(&merger).await,
            OutputMode::Split => self.generate_split(&merger).await,
        }
    }

    /// Merged mode: merge into one file
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let content = merger.merge_all_with_options(Some("cursor")).await?;
        let mdc_content = self.create_mdc_content(&content);

        // Create .cursor/rules/ directory and delete existing files
        let rules_dir = self.get_rules_dir();
        self.prepare_rules_directory(&rules_dir).await?;

        Ok(vec![GeneratedFile::new(
            format!("{}/context.mdc", rules_dir),
            mdc_content,
        )])
    }

    /// Split mode: split by file
    async fn generate_split(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let files = merger.get_individual_files().await?;
        let mut generated_files = Vec::new();

        // Create .cursor/rules/ directory and delete existing files
        let rules_dir = self.get_rules_dir();
        self.prepare_rules_directory(&rules_dir).await?;

        // Check split_config setting
        let split_config = self.get_split_config();

        if let Some(config) = split_config {
            // When split_config setting exists: process files based on rules
            let mut processed_files = std::collections::HashSet::new();

            // Process matching files for each rule
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

            // Process unmatched files with default always rule
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
            // Traditional behavior: convert all files individually (default alwaysApply: true)
            for (file_name, content) in files {
                let mdc_content = self.create_mdc_content(&content);

                // Create mdc filename by removing extension from filename
                let base_name = file_name.trim_end_matches(".md");
                let safe_name = base_name.replace(['/', '\\'], "_"); // Convert path separators to underscores

                generated_files.push(GeneratedFile::new(
                    format!("{}/{}.mdc", rules_dir, safe_name),
                    mdc_content,
                ));
            }
        }

        Ok(generated_files)
    }

    /// Get rules directory path
    fn get_rules_dir(&self) -> String {
        if let Some(base_dir) = &self.base_dir {
            format!("{}/.cursor/rules", base_dir)
        } else {
            ".cursor/rules".to_string()
        }
    }

    /// Prepare .cursor/rules/ directory (delete existing files)
    async fn prepare_rules_directory(&self, rules_dir: &str) -> Result<()> {
        // Delete contents if directory exists
        if fs::metadata(rules_dir).await.is_ok() {
            let mut entries = fs::read_dir(rules_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "mdc") {
                    fs::remove_file(path).await?;
                }
            }
        }

        // Create directory (if it doesn't exist)
        fs::create_dir_all(rules_dir).await?;
        Ok(())
    }

    /// Create MDC format content (YAML frontmatter + Markdown)
    fn create_mdc_content(&self, markdown_content: &str) -> String {
        let frontmatter = self.create_frontmatter();
        format!("---\n{}---\n\n{}", frontmatter, markdown_content)
    }

    /// Create YAML frontmatter (default: Always Apply format)
    fn create_frontmatter(&self) -> String {
        // Build directly as string for ideal format
        "description:\nglobs:\nalwaysApply: true\n".to_string()
    }

    /// Get split_config setting
    fn get_split_config(&self) -> Option<&crate::types::CursorSplitConfig> {
        match &self.config.agents.cursor {
            CursorConfig::Advanced(config) => config.split_config.as_ref(),
            _ => None,
        }
    }

    /// Check if filename matches patterns
    fn file_matches_patterns(&self, file_name: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if self.simple_pattern_match(file_name, pattern) {
                return true;
            }
        }
        false
    }

    /// Simple pattern matching (with wildcard support)
    fn simple_pattern_match(&self, file_name: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // When pattern contains wildcards
            if pattern.starts_with('*') && pattern.ends_with('*') {
                // Pattern like "*rust*"
                let middle = pattern.trim_start_matches('*').trim_end_matches('*');
                return file_name.contains(middle);
            } else if pattern.starts_with('*') {
                // Pattern like "*rust"
                let suffix = pattern.trim_start_matches('*');
                return file_name.ends_with(suffix);
            } else if pattern.ends_with('*') {
                // Pattern like "rust*"
                let prefix = pattern.trim_end_matches('*');
                return file_name.starts_with(prefix);
            }
        }

        // Exact match
        file_name == pattern
    }

    /// Create MDC format content with rule settings
    fn create_mdc_content_with_rule(
        &self,
        markdown_content: &str,
        rule: &crate::types::CursorSplitRule,
    ) -> String {
        let frontmatter = self.create_frontmatter_with_rule(rule);
        format!("---\n{}---\n\n{}", frontmatter, markdown_content)
    }

    /// Create YAML frontmatter with rule settings
    fn create_frontmatter_with_rule(&self, rule: &crate::types::CursorSplitRule) -> String {
        // Priority: manual > alwaysApply > globs > description
        if rule.manual == Some(true) {
            // Manual: description:, globs:, alwaysApply: false
            "description:\nglobs:\nalwaysApply: false\n".to_string()
        } else if rule.always_apply == Some(true) {
            // Always Apply: description:, globs:, alwaysApply: true
            "description:\nglobs:\nalwaysApply: true\n".to_string()
        } else if let Some(globs) = &rule.globs {
            // Auto Attached: description:, globs: value, alwaysApply: false
            let globs_value = match globs.len() {
                1 => format!(" {}", globs[0]),
                len if len > 1 => {
                    let globs_list = globs
                        .iter()
                        .map(|g| format!("  - {}", g))
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("\n{}", globs_list)
                }
                _ => "".to_string(), // Empty for 0 case
            };
            format!("description:\nglobs:{}\nalwaysApply: false\n", globs_value)
        } else if let Some(desc) = &rule.description {
            // Agent Requested: description: value, globs:, alwaysApply: false
            format!("description: {}\nglobs:\nalwaysApply: false\n", desc)
        } else {
            // Default: Always Apply
            "description:\nglobs:\nalwaysApply: true\n".to_string()
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
            include_filenames: Some(true), // Enable headers for testing
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
        assert!(files[0].content.contains("description:"));
        assert!(files[0].content.contains("globs:"));
    }

    #[tokio::test]
    async fn test_generate_merged_with_content() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test file
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

        // Create multiple test files
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

        // Check filenames and paths
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

        // Check content
        for file in &files {
            assert!(file.content.contains("---"));
            assert!(file.content.contains("alwaysApply: true"));
            assert!(file.content.contains("description:"));
            assert!(file.content.contains("globs:"));

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

        // Create subdirectory
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

        // Check if path separators are converted to underscores
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
        assert!(mdc_content.contains("description:"));
        assert!(mdc_content.contains("globs:"));
        assert!(mdc_content.contains("---\n\n# Test\nContent here"));
    }

    #[tokio::test]
    async fn test_frontmatter_format() {
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent = CursorAgent::new(config);

        let frontmatter = agent.create_frontmatter();

        // Confirm it's YAML format
        assert!(frontmatter.contains("alwaysApply:"));
        assert!(frontmatter.contains("description:"));
        assert!(frontmatter.contains("globs:"));

        // Confirm it's parseable
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

        // Start with directory not existing
        assert!(!rules_dir.exists());

        // Execute prepare_rules_directory
        agent
            .prepare_rules_directory(&rules_dir.to_string_lossy())
            .await
            .unwrap();

        // Confirm directory was created
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

        // Create directory
        fs::create_dir_all(&rules_dir).await.unwrap();

        // Create existing mdc file
        let existing_mdc = rules_dir.join("old_file.mdc");
        let other_file = rules_dir.join("keep_me.txt");
        fs::write(&existing_mdc, "old content").await.unwrap();
        fs::write(&other_file, "keep this").await.unwrap();

        // Confirm files exist
        assert!(existing_mdc.exists());
        assert!(other_file.exists());

        // Execute prepare_rules_directory
        agent
            .prepare_rules_directory(&rules_dir.to_string_lossy())
            .await
            .unwrap();

        // Confirm mdc file is deleted and other file remains
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

        // Create test file
        fs::write(docs_path.join("test.md"), "# Test Content")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Merged);
        let agent =
            CursorAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();

        // Correct path should be generated
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

        // Create multiple test files
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

        // Correct paths should be generated
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

    // === split_config functionality tests ===

    #[tokio::test]
    async fn test_split_config_manual_rule() {
        use crate::types::{CursorAgentConfig, CursorSplitConfig, CursorSplitRule};

        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test file
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

        // Confirm manual.mdc file is generated
        let manual_file = files.iter().find(|f| f.path.contains("manual")).unwrap();
        assert!(manual_file.content.contains("description:"));
        assert!(manual_file.content.contains("globs:"));
        assert!(manual_file.content.contains("alwaysApply: false"));
        assert!(!manual_file.content.contains("manual:"));
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
        assert!(always_file.content.contains("description:"));
        assert!(always_file.content.contains("globs:"));
        assert!(!always_file.content.contains("manual:"));
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
        assert!(rust_file.content.contains("description:"));
        assert!(rust_file.content.contains("globs: **/*.rs"));
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
        assert!(agent_file.content.contains("globs:"));
        assert!(agent_file.content.contains("alwaysApply: false"));
        assert!(!agent_file.content.contains("manual:"));
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
        assert!(multi_file.content.contains("description:"));
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
        assert!(matched_file.content.contains("description:"));
        assert!(matched_file.content.contains("globs:"));
        assert!(matched_file.content.contains("alwaysApply: false"));

        let unmatched_file = files.iter().find(|f| f.path.contains("unmatched")).unwrap();
        assert!(unmatched_file.content.contains("alwaysApply: true"));
        assert!(unmatched_file.content.contains("description:"));
        assert!(unmatched_file.content.contains("globs:"));
        assert!(!unmatched_file.content.contains("manual:"));
    }

    #[tokio::test]
    async fn test_file_pattern_matching() {
        let config = create_test_config("./docs", OutputMode::Split);
        let agent = CursorAgent::new(config);

        // Test before and after wildcards
        assert!(agent.simple_pattern_match("architecture.md", "*architecture*"));
        assert!(agent.simple_pattern_match("rust-guide.md", "*rust*"));

        // Test only before
        assert!(agent.simple_pattern_match("test.md", "*test.md"));
        assert!(!agent.simple_pattern_match("test-file.md", "*test.md"));

        // Test only after
        assert!(agent.simple_pattern_match("config.md", "config*"));
        assert!(!agent.simple_pattern_match("my-config.md", "config*"));

        // Exact match
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

        // manual should be highest priority, should be in Manual format
        let priority_file = files.iter().find(|f| f.path.contains("priority")).unwrap();
        assert!(priority_file.content.contains("description:"));
        assert!(priority_file.content.contains("globs:"));
        assert!(priority_file.content.contains("alwaysApply: false"));
        assert!(!priority_file.content.contains("manual:"));
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
