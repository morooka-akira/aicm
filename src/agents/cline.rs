/*!
 * AI Context Management Tool - Cline Agent
 *
 * Cline agent implementation
 * Specification: https://docs.cline.bot/features/cline-rules
 *
 * Split mode: Multiple .md files in .clinerules/ folder
 * Merged mode: Single .clinerules file (no extension)
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, GeneratedFile, OutputMode};
use anyhow::Result;
use tokio::fs;

/// Cline agent
pub struct ClineAgent {
    config: AIContextConfig,
    base_dir: Option<String>,
}

impl ClineAgent {
    /// Create a new Cline agent
    pub fn new(config: AIContextConfig) -> Self {
        Self {
            config,
            base_dir: None,
        }
    }

    /// For testing: create Cline agent with specified base_dir
    pub fn new_with_base_dir(config: AIContextConfig, base_dir: String) -> Self {
        Self {
            config,
            base_dir: Some(base_dir),
        }
    }

    /// Generate files for Cline
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let base_docs_dir = self.config.get_effective_base_docs_dir("cline").to_string();
        let merger = MarkdownMerger::new_with_base_dir(self.config.clone(), base_docs_dir);

        match self.config.get_effective_output_mode("cline") {
            OutputMode::Merged => self.generate_merged(&merger).await,
            OutputMode::Split => self.generate_split(&merger).await,
        }
    }

    /// Merged mode: Single .clinerules file (no extension)
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let content = merger.merge_all_with_options(Some("cline")).await?;
        let output_path = self.get_merged_output_path();

        // Delete existing .clinerules directory (for split mode) if it exists
        if let Ok(metadata) = fs::metadata(&output_path).await {
            if metadata.is_dir() {
                fs::remove_dir_all(&output_path).await?;
            }
        }

        Ok(vec![GeneratedFile::new(output_path, content)])
    }

    /// Split mode: Multiple .md files in .clinerules/ folder
    async fn generate_split(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let files = merger.get_individual_files().await?;
        let mut generated_files = Vec::new();

        // Prepare .clinerules/ directory
        let rules_dir = self.get_split_rules_dir();
        self.prepare_rules_directory(&rules_dir).await?;

        for (file_name, content) in files {
            // Create md filename by removing extension from filename
            let base_name = file_name.trim_end_matches(".md");
            let safe_name = base_name.replace(['/', '\\'], "_"); // Convert path separators to underscores

            // Use original filename (no number prefix)
            let output_filename = format!("{}.md", safe_name);

            generated_files.push(GeneratedFile::new(
                format!("{}/{}", rules_dir, output_filename),
                content,
            ));
        }

        Ok(generated_files)
    }

    /// Get output path for merged mode
    fn get_merged_output_path(&self) -> String {
        if let Some(base_dir) = &self.base_dir {
            format!("{}/.clinerules", base_dir)
        } else {
            ".clinerules".to_string() // No extension
        }
    }

    /// Get rules directory path for split mode
    fn get_split_rules_dir(&self) -> String {
        if let Some(base_dir) = &self.base_dir {
            format!("{}/.clinerules", base_dir)
        } else {
            ".clinerules".to_string() // Folder
        }
    }

    /// Prepare .clinerules/ directory (delete existing files)
    async fn prepare_rules_directory(&self, rules_dir: &str) -> Result<()> {
        // Delete existing .clinerules file (for merged mode) if it exists
        if let Ok(metadata) = fs::metadata(rules_dir).await {
            if metadata.is_file() {
                fs::remove_file(rules_dir).await?;
            } else if metadata.is_dir() {
                // If directory exists, delete its contents
                let mut entries = fs::read_dir(rules_dir).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                        fs::remove_file(path).await?;
                    }
                }
            }
        }

        // Create directory (if it doesn't exist)
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
            ClineAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        let expected_path = format!("{}/.clinerules", temp_dir.path().to_string_lossy());
        assert_eq!(files[0].path, expected_path); // No extension
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
            ClineAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        let expected_path = format!("{}/.clinerules", temp_dir.path().to_string_lossy());
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
        fs::write(docs_path.join("file1.md"), "Content 1")
            .await
            .unwrap();
        fs::write(docs_path.join("file2.md"), "Content 2")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy(), OutputMode::Split);
        let agent =
            ClineAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 2);

        // Check filenames and paths
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();
        let expected_path1 = format!("{}/.clinerules/file1.md", temp_dir.path().to_string_lossy());
        let expected_path2 = format!("{}/.clinerules/file2.md", temp_dir.path().to_string_lossy());
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
            ClineAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);

        // Confirm path separators are converted to underscores
        let expected_path = format!(
            "{}/.clinerules/subdir_nested.md",
            temp_dir.path().to_string_lossy()
        );
        assert_eq!(files[0].path, expected_path);
        assert!(files[0].content.contains("Nested content"));
    }

    #[tokio::test]
    async fn test_get_merged_output_path() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config("./docs", OutputMode::Merged);
        let agent =
            ClineAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let output_path = agent.get_merged_output_path();
        let expected_path = format!("{}/.clinerules", temp_dir.path().to_string_lossy());
        assert_eq!(output_path, expected_path); // No extension
    }

    #[tokio::test]
    async fn test_get_split_rules_dir() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config("./docs", OutputMode::Split);
        let agent =
            ClineAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let rules_dir = agent.get_split_rules_dir();
        let expected_path = format!("{}/.clinerules", temp_dir.path().to_string_lossy());
        assert_eq!(rules_dir, expected_path); // Folder
    }

    #[tokio::test]
    async fn test_prepare_rules_directory() {
        let temp_dir = tempdir().unwrap();
        let rules_dir = temp_dir.path().join(".clinerules");
        let config = create_test_config("./docs", OutputMode::Split);
        let agent =
            ClineAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        // Create directory
        fs::create_dir_all(&rules_dir).await.unwrap();

        // Create existing md file
        let existing_md = rules_dir.join("old_file.md");
        let other_file = rules_dir.join("keep_me.txt");
        fs::write(&existing_md, "old content").await.unwrap();
        fs::write(&other_file, "keep this").await.unwrap();

        // Confirm files exist
        assert!(existing_md.exists());
        assert!(other_file.exists());

        // Execute prepare_rules_directory
        agent
            .prepare_rules_directory(&rules_dir.to_string_lossy())
            .await
            .unwrap();

        // Confirm md file is deleted and other file remains
        assert!(!existing_md.exists());
        assert!(other_file.exists());
    }

    #[tokio::test]
    async fn test_simple_filename_generation() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create multiple test files
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
        let agent =
            ClineAgent::new_with_base_dir(config, temp_dir.path().to_string_lossy().to_string());

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 3);

        // Confirm simple filenames (no number prefix)
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();

        // Confirm original filenames are preserved
        assert!(paths.iter().any(|p| p.contains("apple.md")));
        assert!(paths.iter().any(|p| p.contains("banana.md")));
        assert!(paths.iter().any(|p| p.contains("cherry.md")));
    }
}
