/*!
 * AI Context Management Tool - Base Agent Utilities (Simplified)
 *
 * シンプル化されたベースエージェント共通機能
 */

use std::path::Path;

/// ベースエージェントの共通機能（シンプル版）
pub struct BaseAgentUtils;

impl BaseAgentUtils {
    /// ファイル内容の安全な生成
    /// 改行コードを統一し、末尾の空白を除去
    pub fn sanitize_content(content: &str) -> String {
        let normalized = content
            .replace("\r\n", "\n") // Windows改行をUnix改行に統一
            .replace('\r', "\n") // 古いMac改行をUnix改行に統一
            .trim_end() // 末尾の空白を除去
            .to_string();

        if normalized.is_empty() {
            String::new()
        } else {
            format!("{}\n", normalized) // 最後に改行を追加
        }
    }

    /// パスの正規化
    /// プラットフォーム依存の問題を避けるため、常にUnixスタイルのパスに変換
    pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
        path.as_ref().to_string_lossy().replace('\\', "/")
    }

    /// ファイル名からエージェント用の安全な名前を生成
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
        // Windows改行の正規化
        let windows_content = "line1\r\nline2\r\nline3";
        let result = BaseAgentUtils::sanitize_content(windows_content);
        assert_eq!(result, "line1\nline2\nline3\n");

        // Mac改行の正規化
        let mac_content = "line1\rline2\rline3";
        let result = BaseAgentUtils::sanitize_content(mac_content);
        assert_eq!(result, "line1\nline2\nline3\n");

        // 末尾空白の除去
        let trailing_space_content = "line1\nline2\nline3   \t  ";
        let result = BaseAgentUtils::sanitize_content(trailing_space_content);
        assert_eq!(result, "line1\nline2\nline3\n");

        // 空コンテンツ
        let empty_content = "";
        let result = BaseAgentUtils::sanitize_content(empty_content);
        assert_eq!(result, "");

        // ホワイトスペースのみ
        let whitespace_content = "   \t  \n  ";
        let result = BaseAgentUtils::sanitize_content(whitespace_content);
        assert_eq!(result, "");

        // 通常のコンテンツ
        let normal_content = "# Title\n\nContent here.\n";
        let result = BaseAgentUtils::sanitize_content(normal_content);
        assert_eq!(result, "# Title\n\nContent here.\n");
    }

    #[test]
    fn test_normalize_path() {
        // Unix形式のパス
        let unix_path = "src/agents/cursor.rs";
        let result = BaseAgentUtils::normalize_path(unix_path);
        assert_eq!(result, "src/agents/cursor.rs");

        // Windows形式のパス（バックスラッシュ）
        let windows_path = "src\\agents\\cursor.rs";
        let result = BaseAgentUtils::normalize_path(windows_path);
        assert_eq!(result, "src/agents/cursor.rs");

        // 混在パス
        let mixed_path = "src\\agents/cursor.rs";
        let result = BaseAgentUtils::normalize_path(mixed_path);
        assert_eq!(result, "src/agents/cursor.rs");

        // 絶対パス
        let absolute_path = "/home/user/project";
        let result = BaseAgentUtils::normalize_path(absolute_path);
        assert_eq!(result, "/home/user/project");
    }

    #[test]
    fn test_sanitize_filename() {
        // 通常のファイル名
        let normal = "normal_filename";
        assert_eq!(BaseAgentUtils::sanitize_filename(normal), "normal_filename");

        // パス区切り文字を含む
        let with_separators = "path/to/file";
        assert_eq!(
            BaseAgentUtils::sanitize_filename(with_separators),
            "path_to_file"
        );

        // Windowsの無効文字を含む
        let with_invalid = "file<name>with*invalid?chars";
        assert_eq!(
            BaseAgentUtils::sanitize_filename(with_invalid),
            "file_name_with_invalid_chars"
        );

        // スペースを含む
        let with_spaces = "file name with spaces";
        assert_eq!(
            BaseAgentUtils::sanitize_filename(with_spaces),
            "file_name_with_spaces"
        );

        // 複合パターン
        let complex = "complex/path\\with:spaces*and?invalid<chars>";
        assert_eq!(
            BaseAgentUtils::sanitize_filename(complex),
            "complex_path_with_spaces_and_invalid_chars_"
        );
    }

    #[test]
    fn test_sanitize_content_edge_cases() {
        // 複数の改行タイプが混在
        let mixed_content = "line1\r\nline2\nline3\rline4";
        let result = BaseAgentUtils::sanitize_content(mixed_content);
        assert_eq!(result, "line1\nline2\nline3\nline4\n");

        // 既に正規化されたコンテンツ（変更されないことを確認）
        let normalized_content = "line1\nline2\nline3\n";
        let result = BaseAgentUtils::sanitize_content(normalized_content);
        assert_eq!(result, "line1\nline2\nline3\n");

        // タブ文字を含むコンテンツ
        let tab_content = "line1\n\tindented line\nline3";
        let result = BaseAgentUtils::sanitize_content(tab_content);
        assert_eq!(result, "line1\n\tindented line\nline3\n");
    }

    #[test]
    fn test_normalize_path_edge_cases() {
        // 空パス
        let empty_path = "";
        let result = BaseAgentUtils::normalize_path(empty_path);
        assert_eq!(result, "");

        // 単一のバックスラッシュ
        let single_backslash = "\\";
        let result = BaseAgentUtils::normalize_path(single_backslash);
        assert_eq!(result, "/");

        // 連続するバックスラッシュ
        let multiple_backslashes = "path\\\\to\\\\file";
        let result = BaseAgentUtils::normalize_path(multiple_backslashes);
        assert_eq!(result, "path//to//file");
    }
}
