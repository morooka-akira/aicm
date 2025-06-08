/*!
 * AI Context Management Tool - Base Agent Implementation
 * 
 * このファイルはベースエージェントの共通機能を提供します。
 * 全てのエージェント実装で使用できるユーティリティ関数を定義しています。
 */

use crate::types::{AIContextConfig, ValidationResult, OutputMode};
use path_clean::PathClean;
use std::path::Path;

/// ベースエージェントの共通機能
pub struct BaseAgentUtils;

impl BaseAgentUtils {
    /// 出力モードが分割モードかどうかを判定
    pub fn is_split_mode(config: &AIContextConfig) -> bool {
        matches!(config.output_mode, OutputMode::Split)
    }

    /// ファイル内容の安全な生成
    /// 改行コードを統一し、末尾の空白を除去します
    /// 
    /// # Arguments
    /// * `content` - 元のコンテンツ
    /// 
    /// # Returns
    /// 整形されたコンテンツ
    pub fn sanitize_content(content: &str) -> String {
        let normalized = content
            .replace("\r\n", "\n") // Windows改行をUnix改行に統一
            .replace('\r', "\n")   // 古いMac改行をUnix改行に統一
            .trim_end()            // 末尾の空白を除去
            .to_string();
        
        format!("{}\n", normalized) // 最後に改行を追加
    }

    /// パスの正規化
    /// プラットフォーム依存の問題を避けるため、常にUnixスタイルのパスに変換
    /// 
    /// # Arguments
    /// * `path` - 正規化するパス
    /// 
    /// # Returns
    /// 正規化されたパス
    pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
        path.as_ref()
            .clean()
            .to_string_lossy()
            .replace('\\', "/")
    }

    /// エラー情報付きの検証結果を作成
    /// 
    /// # Arguments
    /// * `errors` - エラーメッセージの配列
    /// * `warnings` - 警告メッセージの配列（省略可）
    /// 
    /// # Returns
    /// 検証結果オブジェクト
    pub fn create_validation_result(
        errors: Vec<String>,
        warnings: Option<Vec<String>>,
    ) -> ValidationResult {
        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings: warnings.unwrap_or_default(),
        }
    }

    /// 設定値の存在チェック
    /// 
    /// # Arguments
    /// * `value` - チェックする値
    /// 
    /// # Returns
    /// 値が存在するかどうか
    pub fn is_valid_string(value: &Option<String>) -> bool {
        match value {
            Some(s) => !s.trim().is_empty(),
            None => false,
        }
    }

    /// 文字列値の存在チェック
    /// 
    /// # Arguments
    /// * `value` - チェックする値
    /// 
    /// # Returns
    /// 値が存在するかどうか
    pub fn is_valid_str(value: &str) -> bool {
        !value.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OutputMode, FileMapping, AgentConfigs};
    use std::collections::HashMap;

    fn create_test_config(output_mode: OutputMode) -> AIContextConfig {
        AIContextConfig {
            version: "1.0".to_string(),
            output_mode,
            base_docs_dir: "./docs".to_string(),
            agents: AgentConfigs::default(),
            file_mapping: FileMapping {
                common: vec!["common.md".to_string()],
                project_specific: vec!["project.md".to_string()],
                agent_specific: None,
            },
            global_variables: HashMap::new(),
        }
    }

    #[test]
    fn test_is_split_mode() {
        let merged_config = create_test_config(OutputMode::Merged);
        let split_config = create_test_config(OutputMode::Split);

        assert!(!BaseAgentUtils::is_split_mode(&merged_config));
        assert!(BaseAgentUtils::is_split_mode(&split_config));
    }

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
        assert_eq!(result, "\n");

        // ホワイトスペースのみ
        let whitespace_content = "   \t  \n  ";
        let result = BaseAgentUtils::sanitize_content(whitespace_content);
        assert_eq!(result, "\n");

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

        // 相対パス（..を含む）
        let relative_path = "src/../src/agents/cursor.rs";
        let result = BaseAgentUtils::normalize_path(relative_path);
        assert_eq!(result, "src/agents/cursor.rs");

        // 現在ディレクトリ（.を含む）
        let current_dir_path = "./src/./agents/cursor.rs";
        let result = BaseAgentUtils::normalize_path(current_dir_path);
        assert_eq!(result, "src/agents/cursor.rs");

        // 複雑なパス
        let complex_path = "./src/../target/../src/agents/../agents/cursor.rs";
        let result = BaseAgentUtils::normalize_path(complex_path);
        assert_eq!(result, "src/agents/cursor.rs");

        // ルート相対パス
        let root_path = "/tmp/test/file.txt";
        let result = BaseAgentUtils::normalize_path(root_path);
        assert_eq!(result, "/tmp/test/file.txt");
    }

    #[test]
    fn test_create_validation_result() {
        // エラーなし、警告なし
        let result = BaseAgentUtils::create_validation_result(vec![], None);
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());

        // エラーあり
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let result = BaseAgentUtils::create_validation_result(errors.clone(), None);
        assert!(!result.valid);
        assert_eq!(result.errors, errors);
        assert!(result.warnings.is_empty());

        // 警告のみ
        let warnings = vec!["Warning 1".to_string(), "Warning 2".to_string()];
        let result = BaseAgentUtils::create_validation_result(vec![], Some(warnings.clone()));
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert_eq!(result.warnings, warnings);

        // エラーと警告両方
        let errors = vec!["Error".to_string()];
        let warnings = vec!["Warning".to_string()];
        let result = BaseAgentUtils::create_validation_result(errors.clone(), Some(warnings.clone()));
        assert!(!result.valid);
        assert_eq!(result.errors, errors);
        assert_eq!(result.warnings, warnings);
    }

    #[test]
    fn test_is_valid_string() {
        // None
        assert!(!BaseAgentUtils::is_valid_string(&None));

        // 空文字列
        assert!(!BaseAgentUtils::is_valid_string(&Some("".to_string())));

        // ホワイトスペースのみ
        assert!(!BaseAgentUtils::is_valid_string(&Some("   \t  \n  ".to_string())));

        // 有効な文字列
        assert!(BaseAgentUtils::is_valid_string(&Some("valid string".to_string())));

        // 前後にホワイトスペースがあるが、内容がある
        assert!(BaseAgentUtils::is_valid_string(&Some("  valid string  ".to_string())));
    }

    #[test]
    fn test_is_valid_str() {
        // 空文字列
        assert!(!BaseAgentUtils::is_valid_str(""));

        // ホワイトスペースのみ
        assert!(!BaseAgentUtils::is_valid_str("   \t  \n  "));

        // 有効な文字列
        assert!(BaseAgentUtils::is_valid_str("valid string"));

        // 前後にホワイトスペースがあるが、内容がある
        assert!(BaseAgentUtils::is_valid_str("  valid string  "));

        // 改行を含む文字列
        assert!(BaseAgentUtils::is_valid_str("line1\nline2"));

        // 単一文字
        assert!(BaseAgentUtils::is_valid_str("a"));
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
        // 空パス（.cleanは"."を返す）
        let empty_path = "";
        let result = BaseAgentUtils::normalize_path(empty_path);
        assert_eq!(result, ".");

        // ドットのみ
        let dot_path = ".";
        let result = BaseAgentUtils::normalize_path(dot_path);
        assert_eq!(result, ".");

        // スラッシュのみ
        let slash_path = "/";
        let result = BaseAgentUtils::normalize_path(slash_path);
        assert_eq!(result, "/");

        // 連続スラッシュ
        let double_slash_path = "src//agents//cursor.rs";
        let result = BaseAgentUtils::normalize_path(double_slash_path);
        assert_eq!(result, "src/agents/cursor.rs");
    }
}