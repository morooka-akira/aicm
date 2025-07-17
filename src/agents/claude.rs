/*!
 * AI Context Management Tool - Claude Agent (Simplified)
 *
 * Simplified Claude agent implementation
 * Outputs CLAUDE.md for Claude Code (supports merged mode only)
 */

use crate::agents::base::BaseAgentUtils;
use crate::core::MarkdownMerger;
use crate::types::config::ClaudeConfig;
use crate::types::{AIContextConfig, GeneratedFile};
use anyhow::Result;
use std::path::Path;

/// Claude agent (simplified version)
pub struct ClaudeAgent {
    config: AIContextConfig,
}

impl ClaudeAgent {
    /// Create a new Claude agent
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    /// Generate files for Claude (merged mode only)
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let base_docs_dir = self
            .config
            .get_effective_base_docs_dir("claude")
            .to_string();
        let merger = MarkdownMerger::new_with_base_dir(self.config.clone(), base_docs_dir);
        self.generate_merged(&merger).await
    }

    /// Merged mode: merge into one file and output as CLAUDE.md
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let mut content = String::new();
        let output_path = self.get_output_path();

        // Get base docs files to check for duplicates with import files
        let base_docs_files = merger.get_individual_files().await?;

        // Process import files first to determine which base docs files to exclude
        let mut filtered_base_docs = Vec::new();
        let mut import_content = String::new();

        if let ClaudeConfig::Advanced(claude_config) = &self.config.agents.claude {
            if !claude_config.import_files.is_empty() {
                let project_root = Path::new(".");
                let claude_file_path = Path::new(&output_path);
                let base_docs_dir = Path::new(self.config.get_effective_base_docs_dir("claude"));

                // Filter out base docs files that are duplicated in import files
                for (file_path, file_content) in &base_docs_files {
                    let mut is_duplicate = false;

                    for import_file in &claude_config.import_files {
                        match BaseAgentUtils::is_import_file_duplicate(
                            &import_file.path,
                            project_root,
                            &base_docs_files,
                            base_docs_dir,
                        ) {
                            Ok(duplicate) => {
                                if duplicate {
                                    // Check if this specific base docs file matches the import file
                                    let base_file_full_path = base_docs_dir.join(file_path);
                                    let resolved_import_path =
                                        match BaseAgentUtils::resolve_import_file_path(
                                            &import_file.path,
                                            project_root,
                                        ) {
                                            Ok(path) => path,
                                            Err(_) => continue,
                                        };

                                    let absolute_import_path = if resolved_import_path.is_absolute()
                                    {
                                        resolved_import_path
                                    } else {
                                        project_root.join(&resolved_import_path)
                                    };

                                    // Compare canonical paths
                                    let canonical_base = base_file_full_path
                                        .canonicalize()
                                        .unwrap_or(base_file_full_path);
                                    let canonical_import = absolute_import_path
                                        .canonicalize()
                                        .unwrap_or(absolute_import_path);

                                    if canonical_base == canonical_import {
                                        is_duplicate = true;
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "Warning: Failed to check duplicate for import file '{}': {}",
                                    import_file.path, e
                                );
                            }
                        }
                    }

                    if !is_duplicate {
                        filtered_base_docs.push((file_path.clone(), file_content.clone()));
                    }
                }

                // Generate import files content
                for import_file in &claude_config.import_files {
                    match BaseAgentUtils::format_import_file(
                        import_file,
                        claude_file_path,
                        project_root,
                    ) {
                        Ok(formatted) => {
                            if !import_content.is_empty() {
                                import_content.push('\n');
                            }
                            import_content.push_str(&formatted);
                            import_content.push('\n');
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to process import file '{}': {}",
                                import_file.path, e
                            );
                        }
                    }
                }
            } else {
                // No import files, use all base docs files
                filtered_base_docs = base_docs_files;
            }
        } else {
            // Simple config, use all base docs files
            filtered_base_docs = base_docs_files;
        }

        // Generate content from filtered base docs files
        let include_filenames = self.config.get_effective_include_filenames("claude");
        for (file_path, file_content) in &filtered_base_docs {
            if include_filenames {
                content.push_str(&format!("# {file_path}\n\n{}\n\n", file_content.trim()));
            } else {
                content.push_str(&format!("{}\n\n", file_content.trim()));
            }
        }

        // Add import files content
        if !import_content.is_empty() {
            if !content.trim().is_empty() {
                content.push('\n');
            }
            content.push_str(&import_content);
        }

        Ok(vec![GeneratedFile::new(
            output_path,
            content.trim().to_string(),
        )])
    }

    /// Get output path (CLAUDE.md in project root)
    fn get_output_path(&self) -> String {
        "CLAUDE.md".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::config::{ClaudeAgentConfig, ClaudeConfig, ImportFile};
    use crate::types::{AgentConfig, OutputMode};
    use tempfile::tempdir;
    use tokio::fs;

    fn create_test_config(base_dir: &str) -> AIContextConfig {
        AIContextConfig {
            version: "1.0".to_string(),
            output_mode: Some(OutputMode::Merged), // Claude supports merged only
            include_filenames: Some(true),         // Enable headers for testing
            base_docs_dir: base_dir.to_string(),
            agents: AgentConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_generate_empty() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy());
        let agent = ClaudeAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "CLAUDE.md");
        // Empty directory results in empty content but is normal
        // (MarkdownMerger returns empty string for empty directory)
    }

    #[tokio::test]
    async fn test_generate_with_content() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test file
        fs::write(docs_path.join("test.md"), "# Test Content\nThis is a test.")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let agent = ClaudeAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "CLAUDE.md");

        // Confirm filename header is included
        assert!(files[0].content.contains("# test.md"));
        // Confirm original content is included
        assert!(files[0].content.contains("# Test Content"));
        assert!(files[0].content.contains("This is a test."));

        // Confirm it's pure Markdown (no frontmatter)
        assert!(!files[0].content.starts_with("---"));
    }

    #[tokio::test]
    async fn test_generate_multiple_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create multiple test files
        fs::write(docs_path.join("file1.md"), "Content 1")
            .await
            .unwrap();
        fs::write(docs_path.join("file2.md"), "Content 2")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let agent = ClaudeAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "CLAUDE.md");

        // Confirm both file contents are included
        assert!(files[0].content.contains("Content 1"));
        assert!(files[0].content.contains("Content 2"));

        // Confirm filename headers are included
        assert!(files[0].content.contains("# file1.md"));
        assert!(files[0].content.contains("# file2.md"));
    }

    #[tokio::test]
    async fn test_generate_with_subdirectory() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create subdirectory
        let sub_dir = docs_path.join("subdir");
        fs::create_dir(&sub_dir).await.unwrap();
        fs::write(sub_dir.join("nested.md"), "Nested content")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let agent = ClaudeAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "CLAUDE.md");

        // Confirm subdirectory files are also included
        assert!(files[0].content.contains("Nested content"));
        assert!(files[0].content.contains("# subdir/nested.md"));
    }

    #[tokio::test]
    async fn test_get_output_path() {
        let config = create_test_config("./docs");
        let agent = ClaudeAgent::new(config);

        let output_path = agent.get_output_path();
        assert_eq!(output_path, "CLAUDE.md");
    }

    #[tokio::test]
    async fn test_generate_creates_pure_markdown() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test file
        fs::write(docs_path.join("test.md"), "# Test\nContent here")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let agent = ClaudeAgent::new(config);

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
    async fn test_generate_output_mode_ignored() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test file
        fs::write(docs_path.join("test.md"), "Test content")
            .await
            .unwrap();

        // Confirm Claude operates in merged mode even when Split mode is configured
        let mut config = create_test_config(&docs_path.to_string_lossy());
        config.output_mode = Some(OutputMode::Split);

        let agent = ClaudeAgent::new(config);
        let files = agent.generate().await.unwrap();

        // Only one file is generated even when Split mode is specified
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "CLAUDE.md");
        assert!(files[0].content.contains("Test content"));
    }

    #[tokio::test]
    async fn test_generate_with_import_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test markdown file
        fs::write(docs_path.join("test.md"), "# Test Content\nThis is a test.")
            .await
            .unwrap();

        // Create import files for testing
        let import_file_path = docs_path.join("import_test.md");
        fs::write(&import_file_path, "Import file content")
            .await
            .unwrap();

        // Create config with import files
        let mut config = create_test_config(&docs_path.to_string_lossy());
        config.agents.claude = ClaudeConfig::Advanced(ClaudeAgentConfig {
            enabled: true,
            output_mode: None,
            include_filenames: Some(true),
            base_docs_dir: None,
            import_files: vec![
                ImportFile {
                    path: import_file_path.to_string_lossy().to_string(),
                    note: Some("Test import file".to_string()),
                },
                ImportFile {
                    path: "non_existent_file.md".to_string(),
                    note: None,
                },
            ],
        });

        let agent = ClaudeAgent::new(config);
        let files = agent.generate().await.unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "CLAUDE.md");

        // Confirm original content is included
        assert!(files[0].content.contains("# Test Content"));
        assert!(files[0].content.contains("This is a test."));

        // Confirm import file is included
        assert!(files[0].content.contains("# Test import file"));
        assert!(files[0].content.contains("@"));

        // Confirm it's pure Markdown (no frontmatter)
        assert!(!files[0].content.starts_with("---"));
    }

    #[tokio::test]
    async fn test_generate_import_files_only() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Don't create any markdown files in docs directory (empty base_docs_dir)

        // Create import files for testing
        let import_file_path = docs_path.join("import_only.md");
        fs::write(&import_file_path, "Only import content")
            .await
            .unwrap();

        // Create config with import files only
        let mut config = create_test_config(&docs_path.to_string_lossy());
        config.agents.claude = ClaudeConfig::Advanced(ClaudeAgentConfig {
            enabled: true,
            output_mode: None,
            include_filenames: Some(false),
            base_docs_dir: None,
            import_files: vec![ImportFile {
                path: import_file_path.to_string_lossy().to_string(),
                note: Some("Import only file".to_string()),
            }],
        });

        let agent = ClaudeAgent::new(config);
        let files = agent.generate().await.unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "CLAUDE.md");

        // Confirm import file is included
        assert!(files[0].content.contains("# Import only file"));
        assert!(files[0].content.contains("@"));
    }

    #[tokio::test]
    async fn test_generate_with_duplicate_files_excluded() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test markdown files in base_docs_dir
        fs::write(
            docs_path.join("project.md"),
            "# Project Overview\nProject content",
        )
        .await
        .unwrap();
        fs::write(docs_path.join("guide.md"), "# Guide\nGuide content")
            .await
            .unwrap();

        // Create duplicate file that will be used as import file
        let duplicate_file_path = docs_path.join("project.md");

        // Create config with import files that duplicates one of the base docs files
        let mut config = create_test_config(&docs_path.to_string_lossy());
        config.agents.claude = ClaudeConfig::Advanced(ClaudeAgentConfig {
            enabled: true,
            output_mode: None,
            include_filenames: Some(true),
            base_docs_dir: None,
            import_files: vec![
                ImportFile {
                    path: duplicate_file_path.to_string_lossy().to_string(),
                    note: Some("Project info".to_string()),
                },
                ImportFile {
                    path: "external/config.md".to_string(),
                    note: Some("External config".to_string()),
                },
            ],
        });

        let agent = ClaudeAgent::new(config);
        let files = agent.generate().await.unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "CLAUDE.md");

        // Confirm only guide.md content from base_docs_dir is included (project.md should be excluded)
        assert!(files[0].content.contains("# guide.md"));
        assert!(files[0].content.contains("Guide content"));

        // Confirm project.md content is NOT included from base_docs_dir (should only be via import file)
        assert!(!files[0].content.contains("# project.md"));
        assert!(!files[0].content.contains("Project content"));

        // Confirm import files are included
        assert!(files[0].content.contains("# Project info"));
        assert!(files[0].content.contains("# External config"));
        assert!(files[0].content.contains("@"));
    }

    #[tokio::test]
    async fn test_generate_no_duplicates_when_import_files_empty() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test markdown files
        fs::write(docs_path.join("file1.md"), "Content 1")
            .await
            .unwrap();
        fs::write(docs_path.join("file2.md"), "Content 2")
            .await
            .unwrap();

        // Create config with empty import files
        let mut config = create_test_config(&docs_path.to_string_lossy());
        config.agents.claude = ClaudeConfig::Advanced(ClaudeAgentConfig {
            enabled: true,
            output_mode: None,
            include_filenames: Some(true),
            base_docs_dir: None,
            import_files: vec![], // Empty import files
        });

        let agent = ClaudeAgent::new(config);
        let files = agent.generate().await.unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "CLAUDE.md");

        // Confirm all base docs files are included when no import files
        assert!(files[0].content.contains("Content 1"));
        assert!(files[0].content.contains("Content 2"));
        assert!(files[0].content.contains("# file1.md"));
        assert!(files[0].content.contains("# file2.md"));

        // Confirm no import files section
        assert!(!files[0].content.contains("@"));
    }

    #[tokio::test]
    async fn test_generate_simple_config_no_duplicates() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test markdown files
        fs::write(docs_path.join("test1.md"), "Test content 1")
            .await
            .unwrap();
        fs::write(docs_path.join("test2.md"), "Test content 2")
            .await
            .unwrap();

        // Use simple boolean config (not Advanced)
        let mut config = create_test_config(&docs_path.to_string_lossy());
        config.agents.claude = ClaudeConfig::Simple(true);

        let agent = ClaudeAgent::new(config);
        let files = agent.generate().await.unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "CLAUDE.md");

        // Confirm all base docs files are included with simple config
        assert!(files[0].content.contains("Test content 1"));
        assert!(files[0].content.contains("Test content 2"));

        // Confirm no import files section with simple config
        assert!(!files[0].content.contains("@"));
    }
}
