/*!
 * AI Context Management Tool - Codex Agent
 *
 * OpenAI Codex agent implementation
 * Outputs AGENTS.md file (supports merged mode only)
 */

use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, GeneratedFile};
use anyhow::Result;

/// Codex agent
pub struct CodexAgent {
    config: AIContextConfig,
}

impl CodexAgent {
    /// Create a new Codex agent
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    /// Generate files for Codex (merged mode only)
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>> {
        let merger = MarkdownMerger::new(self.config.clone());
        self.generate_merged(&merger).await
    }

    /// Merged mode: merge into one file and output as AGENTS.md
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>> {
        let content = merger.merge_all_with_options(Some("codex")).await?;
        let output_path = self.get_output_path();

        Ok(vec![GeneratedFile::new(output_path, content)])
    }

    /// Get output path (AGENTS.md in project root)
    fn get_output_path(&self) -> String {
        "AGENTS.md".to_string()
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
            output_mode: Some(OutputMode::Merged), // Codex supports merged only
            include_filenames: Some(true),         // Enable headers for testing
            base_docs_dir: base_dir.to_string(),
            agents: AgentConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_generate_empty() {
        let temp_dir = tempdir().unwrap();
        let config = create_test_config(&temp_dir.path().to_string_lossy());
        let agent = CodexAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "AGENTS.md");
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
        let agent = CodexAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "AGENTS.md");

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
        let agent = CodexAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "AGENTS.md");

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
        let agent = CodexAgent::new(config);

        let files = agent.generate().await.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "AGENTS.md");

        // Confirm subdirectory files are also included
        assert!(files[0].content.contains("Nested content"));
        assert!(files[0].content.contains("# subdir/nested.md"));
    }

    #[tokio::test]
    async fn test_get_output_path() {
        let config = create_test_config("./docs");
        let agent = CodexAgent::new(config);

        let output_path = agent.get_output_path();
        assert_eq!(output_path, "AGENTS.md");
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
        let agent = CodexAgent::new(config);

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

        // Confirm Codex operates in merged mode even when Split mode is configured
        let mut config = create_test_config(&docs_path.to_string_lossy());
        config.output_mode = Some(OutputMode::Split);

        let agent = CodexAgent::new(config);
        let files = agent.generate().await.unwrap();

        // Only one file is generated even when Split mode is specified
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "AGENTS.md");
        assert!(files[0].content.contains("Test content"));
    }
}
