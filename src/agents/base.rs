/*!
 * AI Context Management Tool - Base Agent Utilities (Simplified)
 *
 * Simplified base agent common functions
 */

use std::path::Path;

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
}
