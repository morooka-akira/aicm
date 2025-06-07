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