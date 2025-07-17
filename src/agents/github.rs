/*!
 * AI Context Management Tool - GitHub Copilot Agent
 *
 * Context file generation agent for GitHub Copilot
 * Specification: https://code.visualstudio.com/docs/copilot/copilot-customization
 *
 * File naming conventions:
 * - Merged mode: .github/copilot-instructions.md
 * - Split mode: Generate md files under .github/instructions/
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, GeneratedFile, GitHubSplitRule, OutputMode};
use anyhow::Result;
use tokio::fs;

/// GitHub Copilot agent
pub struct GitHubAgent {
    config: AIContextConfig,
    base_dir: Option<String>,
}

impl GitHubAgent {
    /// Create a new GitHub Copilot agent
    pub fn new(config: AIContextConfig) -> Self {
        Self {
            config,
            base_dir: None,
        }
    }

    /// Create GitHub Copilot agent with specified base directory
    #[cfg(test)]
    pub fn new_with_base_dir(config: AIContextConfig, base_dir: String) -> Self {
        Self {
            config,
            base_dir: Some(base_dir),
        }
    }

    /// Generate files for GitHub Copilot
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let base_docs_dir = self
            .config
            .get_effective_base_docs_dir("github")
            .to_string();
        let merger = MarkdownMerger::new_with_base_dir(self.config.clone(), base_docs_dir);

        match self.config.get_effective_output_mode("github") {
            OutputMode::Merged => self.generate_merged(&merger).await,
            OutputMode::Split => self.generate_split(&merger).await,
        }
    }

    /// Merged mode: Generate .github/copilot-instructions.md file
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let content = merger.merge_all_with_options(Some("github")).await?;

        // GitHub Copilot uses regular Markdown files (no frontmatter)
        let instructions_content = self.create_instructions_content(&content);

        // Delete existing *.prompt.md files (for split mode)
        self.cleanup_split_files().await?;

        // Create .github directory
        let github_dir = self.get_github_dir();
        tokio::fs::create_dir_all(&github_dir).await?;

        let output_path = if let Some(base_dir) = &self.base_dir {
            format!("{base_dir}/.github/copilot-instructions.md")
        } else {
            ".github/copilot-instructions.md".to_string()
        };

        Ok(vec![GeneratedFile::new(output_path, instructions_content)])
    }

    /// Split mode: Generate .github/instructions/xxx.instructions.md files
    async fn generate_split(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let files = merger.get_individual_files().await?;
        let mut generated_files = Vec::new();

        // Delete existing .github/copilot-instructions.md file (for merged mode)
        self.cleanup_merged_file().await?;

        // Create .github/instructions directory
        let instructions_dir = self.get_instructions_dir();
        tokio::fs::create_dir_all(&instructions_dir).await?;

        // Delete existing .instructions.md files
        self.cleanup_split_files().await?;

        // If split_config is configured, generate according to those rules
        if let Some(github_config) = &self.config.agents.github.get_advanced_config() {
            if let Some(split_config) = &github_config.split_config {
                return self
                    .generate_split_with_config(&files, &split_config.rules)
                    .await;
            }
        }

        // If split_config is not configured, generate using traditional method
        for (file_name, content) in files {
            let instructions_content = self.create_instructions_content(&content);

            // Add .instructions.md by removing extension from filename
            let base_name = file_name.trim_end_matches(".md");
            let safe_name = base_name.replace(['/', '\\'], "_"); // Convert path separators to underscores

            let output_path = if let Some(base_dir) = &self.base_dir {
                format!("{base_dir}/.github/instructions/{safe_name}.instructions.md")
            } else {
                format!(".github/instructions/{safe_name}.instructions.md")
            };

            generated_files.push(GeneratedFile::new(output_path, instructions_content));
        }

        Ok(generated_files)
    }

    /// Generate files according to split_config rules
    async fn generate_split_with_config(
        &self,
        files: &[(String, String)],
        rules: &[GitHubSplitRule],
    ) -> Result<Vec<GeneratedFile>> {
        let mut generated_files = Vec::new();
        let mut processed_files = std::collections::HashSet::new();

        // Process matching files for each rule
        for rule in rules {
            // Collect files that match the rule
            for (file_name, content) in files {
                if processed_files.contains(file_name.as_str()) {
                    continue;
                }

                // Check if file matches the rule
                let matches = rule
                    .file_patterns
                    .iter()
                    .any(|pattern| self.file_matches_pattern(file_name, pattern));

                if matches {
                    // Mark matched file as processed
                    processed_files.insert(file_name.as_str());

                    // Add applyTo frontmatter
                    let instructions_content =
                        self.create_instructions_content_with_apply_to(content, &rule.apply_to);

                    // Preserve original filename
                    let base_name = file_name.trim_end_matches(".md");
                    let safe_name = base_name.replace(['/', '\\'], "_");

                    let output_path = if let Some(base_dir) = &self.base_dir {
                        format!("{base_dir}/.github/instructions/{safe_name}.instructions.md")
                    } else {
                        format!(".github/instructions/{safe_name}.instructions.md")
                    };

                    generated_files.push(GeneratedFile::new(output_path, instructions_content));
                }
            }
        }

        // Output unmatched files as-is by default (no apply_to)
        for (file_name, content) in files {
            if !processed_files.contains(file_name.as_str()) {
                let instructions_content = self.create_instructions_content(content);
                let base_name = file_name.trim_end_matches(".md");
                let safe_name = base_name.replace(['/', '\\'], "_");

                let output_path = if let Some(base_dir) = &self.base_dir {
                    format!("{base_dir}/.github/instructions/{safe_name}.instructions.md")
                } else {
                    format!(".github/instructions/{safe_name}.instructions.md")
                };

                generated_files.push(GeneratedFile::new(output_path, instructions_content));
            }
        }

        Ok(generated_files)
    }

    /// Check if filename matches pattern
    fn file_matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        // Simple wildcard matching
        if pattern.contains('*') {
            // Pattern like "*pattern*"
            if pattern.starts_with('*') && pattern.ends_with('*') {
                let middle = &pattern[1..pattern.len() - 1];
                return file_name.contains(middle);
            }
            // Pattern like "*pattern"
            if let Some(suffix) = pattern.strip_prefix('*') {
                return file_name.ends_with(suffix);
            }
            // Pattern like "pattern*"
            if let Some(prefix) = pattern.strip_suffix('*') {
                return file_name.starts_with(prefix);
            }
        }

        // Exact match or substring match
        file_name.contains(pattern)
    }

    /// Create GitHub Copilot content (pure Markdown, no frontmatter)
    fn create_instructions_content(&self, content: &str) -> String {
        content.to_string()
    }

    /// Create GitHub Copilot content with applyTo frontmatter
    fn create_instructions_content_with_apply_to(
        &self,
        content: &str,
        apply_to: &Option<Vec<String>>,
    ) -> String {
        match apply_to {
            Some(patterns) if !patterns.is_empty() => {
                let apply_to_value = patterns.join(",");
                format!("---\napplyTo: \"{apply_to_value}\"\n---\n\n{content}")
            }
            _ => content.to_string(),
        }
    }

    /// Delete split mode files (.github/instructions/*.instructions.md)
    async fn cleanup_split_files(&self) -> Result<()> {
        use tokio::fs;

        let instructions_dir = self.get_instructions_dir();

        // Do nothing if .github/instructions is not a directory
        let metadata = match fs::metadata(&instructions_dir).await {
            Ok(m) => m,
            Err(_) => return Ok(()),
        };

        if !metadata.is_dir() {
            return Ok(());
        }

        // Delete .instructions.md files in directory
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

    /// Delete merged mode file (.github/copilot-instructions.md)
    async fn cleanup_merged_file(&self) -> Result<()> {
        let merged_file_path = if let Some(base_dir) = &self.base_dir {
            format!("{base_dir}/.github/copilot-instructions.md")
        } else {
            ".github/copilot-instructions.md".to_string()
        };

        if fs::metadata(&merged_file_path).await.is_ok() {
            fs::remove_file(&merged_file_path).await?;
        }
        Ok(())
    }

    /// Get GitHub directory path
    fn get_github_dir(&self) -> String {
        if let Some(base_dir) = &self.base_dir {
            format!("{base_dir}/.github")
        } else {
            ".github".to_string()
        }
    }

    /// Get GitHub instructions directory path
    fn get_instructions_dir(&self) -> String {
        if let Some(base_dir) = &self.base_dir {
            format!("{base_dir}/.github/instructions")
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

        // Create test file
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

        // Confirm filename header is included
        assert!(files[0].content.contains("# test.md"));
        // Confirm original content is included
        assert!(files[0].content.contains("# Test Content"));
        assert!(files[0].content.contains("This is a test."));
    }

    #[tokio::test]
    async fn test_generate_split_multiple_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create multiple test files
        std::fs::write(docs_path.join("file1.md"), "Content 1").unwrap();
        std::fs::write(docs_path.join("file2.md"), "Content 2").unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 2);

        // Check filenames and paths
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

        // Check content
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

        // Create subdirectory
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

        // Confirm path separators are converted to underscores
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

        // Create test file
        fs::write(docs_path.join("test.md"), "# Test\nContent here")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Merged);
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        let content = &files[0].content;

        // Confirm it's pure Markdown (no YAML frontmatter)
        assert!(!content.starts_with("---"));
        assert!(!content.contains("description:"));
        assert!(!content.contains("alwaysApply:"));

        // Confirm content is included
        assert!(content.contains("# Test"));
        assert!(content.contains("Content here"));
    }

    #[tokio::test]
    async fn test_cleanup_split_files_ignores_file_path() {
        // Test that it ends without error even when .github/instructions is a file
        let temp_dir = tempdir().unwrap();

        // setup: create instructions as file
        let github_dir = temp_dir.path().join(".github");
        std::fs::create_dir_all(&github_dir).unwrap();

        // Delete existing path before creating file
        let instructions_path = github_dir.join("instructions");
        let _ = std::fs::remove_file(&instructions_path);
        let _ = std::fs::remove_dir_all(&instructions_path);
        std::fs::write(&instructions_path, "dummy").unwrap();

        let config = create_test_config(&temp_dir.path().to_string_lossy(), OutputMode::Split);
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        // Should not cause error when executed
        agent.cleanup_split_files().await.unwrap();

        // Confirm file remains as-is
        let metadata = std::fs::metadata(&instructions_path).unwrap();
        assert!(metadata.is_file());
    }

    #[tokio::test]
    async fn test_generate_split_with_apply_to() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test files
        std::fs::write(
            docs_path.join("architecture.md"),
            "# Architecture\nSystem design",
        )
        .unwrap();
        std::fs::write(docs_path.join("frontend.md"), "# Frontend\nUI components").unwrap();

        // Create configuration with split_config
        use crate::types::{GitHubAgentConfig, GitHubConfig, GitHubSplitConfig};
        let github_config = GitHubConfig::Advanced(GitHubAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            base_docs_dir: None,
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

        // Check architecture file
        let arch_file = files
            .iter()
            .find(|f| f.path.contains("architecture"))
            .unwrap();
        assert!(arch_file.content.contains("---"));
        assert!(arch_file.content.contains("applyTo: \"**/*.rs,**/*.toml\""));
        assert!(arch_file.content.contains("# Architecture"));

        // Check frontend file
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

        // "*pattern*" test
        assert!(agent.file_matches_pattern("test-architecture-doc.md", "*architecture*"));
        assert!(!agent.file_matches_pattern("frontend.md", "*architecture*"));

        // "*pattern" test
        assert!(agent.file_matches_pattern("setup.md", "*setup.md"));
        assert!(!agent.file_matches_pattern("setup-guide.md", "*setup.md"));

        // "pattern*" test
        assert!(agent.file_matches_pattern("frontend-components.md", "frontend*"));
        assert!(!agent.file_matches_pattern("my-frontend.md", "frontend*"));

        // Exact match test
        assert!(agent.file_matches_pattern("readme.md", "readme"));
        assert!(agent.file_matches_pattern("my-readme-file.md", "readme"));
    }

    #[tokio::test]
    async fn test_create_instructions_content_with_apply_to() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy(), OutputMode::Split);
        let agent =
            GitHubAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        // applyTo is set
        let content_with_apply_to = agent.create_instructions_content_with_apply_to(
            "Test content",
            &Some(vec!["**/*.ts".to_string(), "**/*.tsx".to_string()]),
        );
        assert!(content_with_apply_to.starts_with("---"));
        assert!(content_with_apply_to.contains("applyTo: \"**/*.ts,**/*.tsx\""));
        assert!(content_with_apply_to.contains("Test content"));

        // applyTo is not set
        let content_without_apply_to =
            agent.create_instructions_content_with_apply_to("Test content", &None);
        assert_eq!(content_without_apply_to, "Test content");

        // applyTo is empty array
        let content_empty_apply_to =
            agent.create_instructions_content_with_apply_to("Test content", &Some(vec![]));
        assert_eq!(content_empty_apply_to, "Test content");
    }

    #[tokio::test]
    async fn test_generate_split_with_apply_to_and_unmatched_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test files
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

        // Create configuration with split_config (security file does not match)
        use crate::types::{GitHubAgentConfig, GitHubConfig, GitHubSplitConfig};
        let github_config = GitHubConfig::Advanced(GitHubAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            base_docs_dir: None,
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

        // Check architecture file (applyTo, original filename)
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

        // Check frontend file (applyTo, original filename)
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

        // Check security file (default, no apply_to, original filename)
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
