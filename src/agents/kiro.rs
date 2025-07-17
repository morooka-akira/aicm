/*!
 * AI Context Management Tool - Kiro Agent
 *
 * Kiro agent implementation
 * Outputs files to .kiro/steering/ directory (supports split mode only)
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, GeneratedFile};
use anyhow::Result;

/// Kiro agent
pub struct KiroAgent {
    config: AIContextConfig,
}

impl KiroAgent {
    /// Create a new Kiro agent
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    /// Generate files for Kiro (split mode only)
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let base_docs_dir = self.config.get_effective_base_docs_dir("kiro").to_string();
        let merger = MarkdownMerger::new_with_base_dir(self.config.clone(), base_docs_dir);
        self.generate_split(&merger).await
    }

    /// Split mode: generate individual files in .kiro/steering/ directory
    async fn generate_split(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let files = merger.get_individual_files().await?;
        let mut generated_files = Vec::new();

        for (file_name, content) in files {
            let sanitized_name = self.sanitize_filename(&file_name);
            let output_path = self.get_split_output_path(&sanitized_name);

            generated_files.push(GeneratedFile::new(output_path, content));
        }

        Ok(generated_files)
    }

    /// Get split mode output path (.kiro/steering/{filename})
    fn get_split_output_path(&self, filename: &str) -> String {
        format!(".kiro/steering/{filename}")
    }

    /// Sanitize filename for file system safety
    fn sanitize_filename(&self, filename: &str) -> String {
        // Convert path separators to hyphens for file system safety
        filename.replace(['/', '\\'], "-")
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
            output_mode: Some(OutputMode::Split), // Kiro supports split only
            include_filenames: Some(false),       // Default is false
            base_docs_dir: base_dir.to_string(),
            agents: AgentConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_generate_empty() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy());
        let agent = KiroAgent::new(config);

        let files = agent.generate().await.unwrap();
        // Empty directory results in empty files list
        assert_eq!(files.len(), 0);
    }

    #[tokio::test]
    async fn test_generate_single_file() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create test file
        fs::write(docs_path.join("test.md"), "# Test Content\nThis is a test.")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let agent = KiroAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, ".kiro/steering/test.md");
        assert_eq!(files[0].content, "# Test Content\nThis is a test.");

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
        let agent = KiroAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 2);

        // Confirm both files are generated with correct paths
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();
        assert!(paths.contains(&&".kiro/steering/file1.md".to_string()));
        assert!(paths.contains(&&".kiro/steering/file2.md".to_string()));

        // Confirm content is preserved
        for file in &files {
            if file.path == ".kiro/steering/file1.md" {
                assert_eq!(file.content, "Content 1");
            } else if file.path == ".kiro/steering/file2.md" {
                assert_eq!(file.content, "Content 2");
            }
        }
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
        let agent = KiroAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);

        // Confirm subdirectory separator is sanitized
        assert_eq!(files[0].path, ".kiro/steering/subdir-nested.md");
        assert_eq!(files[0].content, "Nested content");
    }

    #[tokio::test]
    async fn test_sanitize_filename() {
        let config = create_test_config("./docs");
        let agent = KiroAgent::new(config);

        // Test path separator sanitization
        assert_eq!(
            agent.sanitize_filename("sub/dir/file.md"),
            "sub-dir-file.md"
        );
        assert_eq!(
            agent.sanitize_filename("sub\\dir\\file.md"),
            "sub-dir-file.md"
        );
        assert_eq!(agent.sanitize_filename("normal-file.md"), "normal-file.md");
    }

    #[tokio::test]
    async fn test_get_split_output_path() {
        let config = create_test_config("./docs");
        let agent = KiroAgent::new(config);

        let output_path = agent.get_split_output_path("test.md");
        assert_eq!(output_path, ".kiro/steering/test.md");

        let output_path = agent.get_split_output_path("subdir-nested.md");
        assert_eq!(output_path, ".kiro/steering/subdir-nested.md");
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
        let agent = KiroAgent::new(config);

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

        // Confirm Kiro operates in split mode even when Merged mode is configured
        let mut config = create_test_config(&docs_path.to_string_lossy());
        config.output_mode = Some(OutputMode::Merged);

        let agent = KiroAgent::new(config);
        let files = agent.generate().await.unwrap();

        // Multiple files are generated even when Merged mode is specified
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, ".kiro/steering/test.md");
        assert!(files[0].content.contains("Test content"));
    }

    #[tokio::test]
    async fn test_generate_complex_directory_structure() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // Create complex directory structure
        let sub1 = docs_path.join("docs");
        let sub2 = docs_path.join("guides");
        let nested = sub1.join("api");

        fs::create_dir_all(&sub1).await.unwrap();
        fs::create_dir_all(&sub2).await.unwrap();
        fs::create_dir_all(&nested).await.unwrap();

        // Create files in different directories
        fs::write(docs_path.join("readme.md"), "Root content")
            .await
            .unwrap();
        fs::write(sub1.join("overview.md"), "Docs content")
            .await
            .unwrap();
        fs::write(sub2.join("getting-started.md"), "Guide content")
            .await
            .unwrap();
        fs::write(nested.join("reference.md"), "API content")
            .await
            .unwrap();

        let config = create_test_config(&docs_path.to_string_lossy());
        let agent = KiroAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 4);

        // Confirm all files are generated with correct sanitized paths
        let paths: Vec<&String> = files.iter().map(|f| &f.path).collect();
        assert!(paths.contains(&&".kiro/steering/readme.md".to_string()));
        assert!(paths.contains(&&".kiro/steering/docs-overview.md".to_string()));
        assert!(paths.contains(&&".kiro/steering/guides-getting-started.md".to_string()));
        assert!(paths.contains(&&".kiro/steering/docs-api-reference.md".to_string()));
    }
}
