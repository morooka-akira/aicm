/*!
 * AI Context Management Tool - Markdown Merger (Simplified)
 *
 * Simplified Markdown file merging functionality
 */

use crate::types::AIContextConfig;
use anyhow::Result;
use std::path::Path;
use tokio::fs;

/// Class for automatic Markdown file detection and merging
pub struct MarkdownMerger {
    config: AIContextConfig,
    base_docs_dir: Option<String>,
}

impl MarkdownMerger {
    /// Create a new Markdown merger
    pub fn new(config: AIContextConfig) -> Self {
        Self {
            config,
            base_docs_dir: None,
        }
    }

    /// Create a new Markdown merger with specific base directory
    pub fn new_with_base_dir(config: AIContextConfig, base_docs_dir: String) -> Self {
        Self {
            config,
            base_docs_dir: Some(base_docs_dir),
        }
    }

    /// Get effective base docs directory
    fn get_effective_base_docs_dir(&self) -> &str {
        self.base_docs_dir
            .as_deref()
            .unwrap_or(&self.config.base_docs_dir)
    }

    /// Merge all Markdown files under docs (includes filename headers for backward compatibility)
    pub async fn merge_all(&self) -> Result<String> {
        let base_dir = self.get_effective_base_docs_dir();
        let docs_dir = Path::new(base_dir);

        // Return empty string if directory doesn't exist
        if !docs_dir.exists() {
            return Ok(String::new());
        }

        let markdown_files = self.find_markdown_files(docs_dir).await?;
        let mut merged_content = String::new();

        // Always include filename headers for backward compatibility
        for file_path in markdown_files {
            if let Ok(content) = fs::read_to_string(&file_path).await {
                // Add filename as header
                let base_dir = self.get_effective_base_docs_dir();
                let relative_path = file_path
                    .strip_prefix(base_dir)
                    .unwrap_or(&file_path)
                    .to_string_lossy()
                    .replace('\\', "/"); // Normalize path separators for cross-platform compatibility

                merged_content.push_str(&format!("# {}\n\n{}\n\n", relative_path, content.trim()));
            }
        }

        Ok(merged_content.trim().to_string())
    }

    /// Merge all Markdown files under docs (agent name specified version)
    pub async fn merge_all_with_options(&self, agent: Option<&str>) -> Result<String> {
        let base_dir = self.get_effective_base_docs_dir();
        let docs_dir = Path::new(base_dir);

        // Return empty string if directory doesn't exist
        if !docs_dir.exists() {
            return Ok(String::new());
        }

        let markdown_files = self.find_markdown_files(docs_dir).await?;
        let mut merged_content = String::new();

        // Get include_filenames setting
        let include_filenames = if let Some(agent_name) = agent {
            self.config.get_effective_include_filenames(agent_name)
        } else {
            self.config.include_filenames.unwrap_or(false)
        };

        for file_path in markdown_files {
            if let Ok(content) = fs::read_to_string(&file_path).await {
                if include_filenames {
                    // Add filename as header
                    let relative_path = file_path
                        .strip_prefix(&self.config.base_docs_dir)
                        .unwrap_or(&file_path)
                        .to_string_lossy()
                        .replace('\\', "/"); // Normalize path separators for cross-platform compatibility

                    merged_content.push_str(&format!(
                        "# {}\n\n{}\n\n",
                        relative_path,
                        content.trim()
                    ));
                } else {
                    // Add content only without filename header
                    merged_content.push_str(&format!("{}\n\n", content.trim()));
                }
            }
        }

        Ok(merged_content.trim().to_string())
    }

    /// For split: get individual file contents
    pub async fn get_individual_files(&self) -> Result<Vec<(String, String)>> {
        let base_dir = self.get_effective_base_docs_dir();
        let docs_dir = Path::new(base_dir);

        if !docs_dir.exists() {
            return Ok(Vec::new());
        }

        let markdown_files = self.find_markdown_files(docs_dir).await?;
        let mut files = Vec::new();

        for file_path in markdown_files {
            if let Ok(content) = fs::read_to_string(&file_path).await {
                let base_dir = self.get_effective_base_docs_dir();
                let relative_path = file_path
                    .strip_prefix(base_dir)
                    .unwrap_or(&file_path)
                    .to_string_lossy()
                    .replace('\\', "/"); // Normalize path separators for cross-platform compatibility

                files.push((relative_path, content));
            }
        }

        Ok(files)
    }

    /// Recursively search for .md files from specified directory
    async fn find_markdown_files(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        use std::collections::VecDeque;

        let mut files = Vec::new();
        let mut dirs_to_process = VecDeque::new();
        dirs_to_process.push_back(dir.to_path_buf());

        while let Some(current_dir) = dirs_to_process.pop_front() {
            let mut entries = fs::read_dir(&current_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.is_dir() {
                    // Add to processing queue if it's a directory
                    dirs_to_process.push_back(path);
                } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    // Add to list if it's a .md file
                    files.push(path);
                }
            }
        }

        // Sort by filename (process in consistent order)
        files.sort();
        Ok(files)
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
            output_mode: Some(OutputMode::Merged),
            include_filenames: None,
            base_docs_dir: base_dir.to_string(),
            agents: AgentConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_merge_all_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_merge_all_nonexistent_directory() {
        let config = create_test_config("/nonexistent/path");
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_merge_all_single_file() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test markdown file
        fs::write(
            docs_path.join("test.md"),
            "# Test Content\n\nThis is a test.",
        )
        .await
        .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.contains("# test.md"));
        assert!(result.contains("# Test Content"));
        assert!(result.contains("This is a test."));
    }

    #[tokio::test]
    async fn test_merge_all_multiple_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create multiple test files
        fs::write(docs_path.join("file1.md"), "# File 1\nContent 1")
            .await
            .unwrap();
        fs::write(docs_path.join("file2.md"), "# File 2\nContent 2")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.contains("# file1.md"));
        assert!(result.contains("# File 1"));
        assert!(result.contains("Content 1"));
        assert!(result.contains("# file2.md"));
        assert!(result.contains("# File 2"));
        assert!(result.contains("Content 2"));
    }

    #[tokio::test]
    async fn test_merge_all_recursive_directories() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create subdirectory
        let sub_dir = docs_path.join("subdir");
        fs::create_dir(&sub_dir).await.unwrap();

        // Create files in each directory
        fs::write(docs_path.join("root.md"), "Root content")
            .await
            .unwrap();
        fs::write(sub_dir.join("sub.md"), "Sub content")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.contains("# root.md"));
        assert!(result.contains("Root content"));
        assert!(result.contains("# subdir/sub.md") || result.contains("# subdir\\sub.md")); // Windows support
        assert!(result.contains("Sub content"));
    }

    #[tokio::test]
    async fn test_get_individual_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test files
        fs::write(docs_path.join("file1.md"), "Content 1")
            .await
            .unwrap();
        fs::write(docs_path.join("file2.md"), "Content 2")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let files = merger.get_individual_files().await.unwrap();
        assert_eq!(files.len(), 2);

        // Check if sorted by filename
        assert_eq!(files[0].0, "file1.md");
        assert_eq!(files[0].1, "Content 1");
        assert_eq!(files[1].0, "file2.md");
        assert_eq!(files[1].1, "Content 2");
    }

    #[tokio::test]
    async fn test_ignore_non_markdown_files() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create files with various extensions
        fs::write(docs_path.join("test.md"), "Markdown content")
            .await
            .unwrap();
        fs::write(docs_path.join("test.txt"), "Text file")
            .await
            .unwrap();
        fs::write(docs_path.join("test.json"), "{}").await.unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all().await.unwrap();
        assert!(result.contains("Markdown content"));
        assert!(!result.contains("Text file"));
        assert!(!result.contains("{}"));

        let files = merger.get_individual_files().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "test.md");
    }
}
