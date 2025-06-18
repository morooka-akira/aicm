/*!
 * AI Context Management Tool - Base Agent Utilities (Simplified)
 *
 * Simplified base agent common functions
 */

use crate::types::config::ImportFile;
use std::env;
use std::path::{Path, PathBuf};

/// Base agent common functions (simplified version)
pub struct BaseAgentUtils;

impl BaseAgentUtils {
    /// Safe generation of file content
    /// Normalize line breaks and remove trailing whitespace
    pub fn sanitize_content(content: &str) -> String {
        let normalized = content
            .replace("\r\n", "\n") // Normalize Windows line breaks to Unix line breaks
            .replace('\r', "\n") // Normalize old Mac line breaks to Unix line breaks
            .trim_end() // Remove trailing whitespace
            .to_string();

        if normalized.is_empty() {
            String::new()
        } else {
            format!("{}\n", normalized) // Add line break at the end
        }
    }

    /// Path normalization
    /// Convert to Unix-style paths consistently to avoid platform-dependent issues
    pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
        path.as_ref().to_string_lossy().replace('\\', "/")
    }

    /// Generate agent-safe name from filename
    pub fn sanitize_filename(filename: &str) -> String {
        filename
            .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
            .replace(' ', "_")
    }

    /// Resolve file path from various notations (absolute, relative, tilde) for Claude import files
    /// Returns canonical path string
    pub fn resolve_import_file_path<P: AsRef<Path>>(
        file_path: &str,
        base_path: P,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let path = if file_path.starts_with("~/") {
            // Tilde notation: ~/path -> $HOME/path
            if let Ok(home_dir) = env::var("HOME") {
                PathBuf::from(home_dir).join(&file_path[2..])
            } else {
                return Err("HOME environment variable not found".into());
            }
        } else if Path::new(file_path).is_absolute() {
            // Absolute path
            PathBuf::from(file_path)
        } else {
            // Relative path: resolve from base_path (project root)
            base_path.as_ref().join(file_path)
        };

        // Canonicalize path and handle errors
        match path.canonicalize() {
            Ok(canonical_path) => Ok(canonical_path),
            Err(_) => {
                // If canonicalize fails (file doesn't exist), return the constructed path
                Ok(path)
            }
        }
    }

    /// Calculate relative path from CLAUDE.md to target file
    /// Returns relative path string suitable for @filepath notation
    pub fn calculate_claude_relative_path<P1: AsRef<Path>, P2: AsRef<Path>>(
        from_file: P1,
        to_file: P2,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let from_dir = from_file
            .as_ref()
            .parent()
            .ok_or("Cannot get parent directory of CLAUDE.md")?;

        let relative_path =
            pathdiff::diff_paths(&to_file, from_dir).ok_or("Cannot calculate relative path")?;

        Ok(Self::normalize_path(relative_path))
    }

    /// Format import file for Claude output
    /// Returns formatted string with note and @filepath notation
    pub fn format_import_file(
        import_file: &ImportFile,
        claude_file_path: &Path,
        project_root: &Path,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Resolve the file path
        let resolved_path = Self::resolve_import_file_path(&import_file.path, project_root)?;

        // Calculate relative path from CLAUDE.md to target file
        let relative_path = Self::calculate_claude_relative_path(claude_file_path, &resolved_path)?;

        // Format output
        if let Some(note) = &import_file.note {
            Ok(format!("# {}\n@{}", note, relative_path))
        } else {
            Ok(format!("@{}", relative_path))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_content() {
        // Windows line break normalization
        let windows_content = "line1\r\nline2\r\nline3";
        let result = BaseAgentUtils::sanitize_content(windows_content);
        assert_eq!(result, "line1\nline2\nline3\n");

        // Mac line break normalization
        let mac_content = "line1\rline2\rline3";
        let result = BaseAgentUtils::sanitize_content(mac_content);
        assert_eq!(result, "line1\nline2\nline3\n");

        // Trailing whitespace removal
        let trailing_space_content = "line1\nline2\nline3   \t  ";
        let result = BaseAgentUtils::sanitize_content(trailing_space_content);
        assert_eq!(result, "line1\nline2\nline3\n");

        // Empty content
        let empty_content = "";
        let result = BaseAgentUtils::sanitize_content(empty_content);
        assert_eq!(result, "");

        // Whitespace only
        let whitespace_content = "   \t  \n  ";
        let result = BaseAgentUtils::sanitize_content(whitespace_content);
        assert_eq!(result, "");

        // Normal content
        let normal_content = "# Title\n\nContent here.\n";
        let result = BaseAgentUtils::sanitize_content(normal_content);
        assert_eq!(result, "# Title\n\nContent here.\n");
    }

    #[test]
    fn test_normalize_path() {
        // Unix-style path
        let unix_path = "src/agents/cursor.rs";
        let result = BaseAgentUtils::normalize_path(unix_path);
        assert_eq!(result, "src/agents/cursor.rs");

        // Windows-style path (backslash)
        let windows_path = "src\\agents\\cursor.rs";
        let result = BaseAgentUtils::normalize_path(windows_path);
        assert_eq!(result, "src/agents/cursor.rs");

        // Mixed path
        let mixed_path = "src\\agents/cursor.rs";
        let result = BaseAgentUtils::normalize_path(mixed_path);
        assert_eq!(result, "src/agents/cursor.rs");

        // Absolute path
        let absolute_path = "/home/user/project";
        let result = BaseAgentUtils::normalize_path(absolute_path);
        assert_eq!(result, "/home/user/project");
    }

    #[test]
    fn test_sanitize_filename() {
        // Normal filename
        let normal = "normal_filename";
        assert_eq!(BaseAgentUtils::sanitize_filename(normal), "normal_filename");

        // Contains path separators
        let with_separators = "path/to/file";
        assert_eq!(
            BaseAgentUtils::sanitize_filename(with_separators),
            "path_to_file"
        );

        // Contains Windows invalid characters
        let with_invalid = "file<name>with*invalid?chars";
        assert_eq!(
            BaseAgentUtils::sanitize_filename(with_invalid),
            "file_name_with_invalid_chars"
        );

        // Contains spaces
        let with_spaces = "file name with spaces";
        assert_eq!(
            BaseAgentUtils::sanitize_filename(with_spaces),
            "file_name_with_spaces"
        );

        // Complex pattern
        let complex = "complex/path\\with:spaces*and?invalid<chars>";
        assert_eq!(
            BaseAgentUtils::sanitize_filename(complex),
            "complex_path_with_spaces_and_invalid_chars_"
        );
    }

    #[test]
    fn test_sanitize_content_edge_cases() {
        // Mixed line break types
        let mixed_content = "line1\r\nline2\nline3\rline4";
        let result = BaseAgentUtils::sanitize_content(mixed_content);
        assert_eq!(result, "line1\nline2\nline3\nline4\n");

        // Already normalized content (confirm no changes)
        let normalized_content = "line1\nline2\nline3\n";
        let result = BaseAgentUtils::sanitize_content(normalized_content);
        assert_eq!(result, "line1\nline2\nline3\n");

        // Content with tab characters
        let tab_content = "line1\n\tindented line\nline3";
        let result = BaseAgentUtils::sanitize_content(tab_content);
        assert_eq!(result, "line1\n\tindented line\nline3\n");
    }

    #[test]
    fn test_normalize_path_edge_cases() {
        // Empty path
        let empty_path = "";
        let result = BaseAgentUtils::normalize_path(empty_path);
        assert_eq!(result, "");

        // Single backslash
        let single_backslash = "\\";
        let result = BaseAgentUtils::normalize_path(single_backslash);
        assert_eq!(result, "/");

        // Multiple backslashes
        let multiple_backslashes = "path\\\\to\\\\file";
        let result = BaseAgentUtils::normalize_path(multiple_backslashes);
        assert_eq!(result, "path//to//file");
    }

    #[test]
    fn test_resolve_import_file_path_absolute() {
        let base_path = Path::new("/project");
        let absolute_path = "/absolute/path/file.txt";

        let result = BaseAgentUtils::resolve_import_file_path(absolute_path, base_path);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert_eq!(resolved, PathBuf::from("/absolute/path/file.txt"));
    }

    #[test]
    fn test_resolve_import_file_path_relative() {
        let base_path = Path::new("/project");
        let relative_path = "docs/guide.md";

        let result = BaseAgentUtils::resolve_import_file_path(relative_path, base_path);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert_eq!(resolved, PathBuf::from("/project/docs/guide.md"));
    }

    #[test]
    fn test_resolve_import_file_path_tilde() {
        // Set HOME environment variable for testing
        env::set_var("HOME", "/home/testuser");

        let base_path = Path::new("/project");
        let tilde_path = "~/documents/file.txt";

        let result = BaseAgentUtils::resolve_import_file_path(tilde_path, base_path);
        assert!(result.is_ok());

        let resolved = result.unwrap();
        assert_eq!(resolved, PathBuf::from("/home/testuser/documents/file.txt"));

        // Clean up
        env::remove_var("HOME");
    }

    #[test]
    fn test_calculate_claude_relative_path() {
        let claude_file = Path::new("/project/CLAUDE.md");
        let target_file = Path::new("/project/docs/guide.md");

        let result = BaseAgentUtils::calculate_claude_relative_path(claude_file, target_file);
        assert!(result.is_ok());

        let relative = result.unwrap();
        assert_eq!(relative, "docs/guide.md");
    }

    #[test]
    fn test_calculate_claude_relative_path_parent_directory() {
        let claude_file = Path::new("/project/CLAUDE.md");
        let target_file = Path::new("/home/user/config.md");

        let result = BaseAgentUtils::calculate_claude_relative_path(claude_file, target_file);
        assert!(result.is_ok());

        let relative = result.unwrap();
        assert!(relative.contains("../"));
    }

    #[test]
    fn test_format_import_file_with_note() {
        let import_file = ImportFile {
            path: "docs/guide.md".to_string(),
            note: Some("Project guide".to_string()),
        };

        let claude_file = Path::new("/project/CLAUDE.md");
        let project_root = Path::new("/project");

        let result = BaseAgentUtils::format_import_file(&import_file, claude_file, project_root);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert!(formatted.contains("# Project guide"));
        assert!(formatted.contains("@docs/guide.md"));
    }

    #[test]
    fn test_format_import_file_without_note() {
        let import_file = ImportFile {
            path: "docs/guide.md".to_string(),
            note: None,
        };

        let claude_file = Path::new("/project/CLAUDE.md");
        let project_root = Path::new("/project");

        let result = BaseAgentUtils::format_import_file(&import_file, claude_file, project_root);
        assert!(result.is_ok());

        let formatted = result.unwrap();
        assert!(!formatted.contains("#"));
        assert!(formatted.starts_with("@docs/guide.md"));
    }
}
