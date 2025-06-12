/*!
 * AI Context Management Tool - Agent Types (Simplified)
 *
 * Simplified agent-related type definitions
 */

use serde::{Deserialize, Serialize};

/// Information about generated files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// File path (relative path from project root)
    pub path: String,
    /// File content
    pub content: String,
}

impl GeneratedFile {
    /// Create a new GeneratedFile
    pub fn new(path: String, content: String) -> Self {
        Self { path, content }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_file_creation() {
        let file = GeneratedFile::new(
            ".cursor/rules/context.mdc".to_string(),
            "# Test Content".to_string(),
        );

        assert_eq!(file.path, ".cursor/rules/context.mdc");
        assert_eq!(file.content, "# Test Content");
    }

    #[test]
    fn test_generated_file_serialization() {
        let file = GeneratedFile {
            path: ".cursor/rules/test.mdc".to_string(),
            content: "---\ndescription: Test\n---\n\n# Test Content".to_string(),
        };

        let yaml = serde_yaml::to_string(&file).unwrap();
        let deserialized: GeneratedFile = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.path, file.path);
        assert_eq!(deserialized.content, file.content);
    }

    #[test]
    fn test_generated_file_empty_content() {
        let file = GeneratedFile::new("empty.txt".to_string(), "".to_string());
        assert!(file.content.is_empty());
        assert_eq!(file.path, "empty.txt");
    }
}
