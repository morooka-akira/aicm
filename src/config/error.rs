/*!
 * AI Context Management Tool - Configuration Error Types
 * 
 * このファイルは設定関連のエラー型を定義します。
 */

use thiserror::Error;

/// 設定読み込み時のエラー
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("設定ファイルが見つかりません: {path}")]
    FileNotFound { path: String },

    #[error("設定ファイルの読み込みに失敗しました: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("YAML解析に失敗しました: {source}")]
    YamlParseError {
        #[from]
        source: serde_yaml::Error,
    },

    #[error("設定の検証に失敗しました: {errors:?}")]
    ValidationError { errors: Vec<String> },

    #[error("必須フィールドが不足しています: {field}")]
    MissingRequiredField { field: String },

    #[error("不正な値です: {field} = {value}")]
    InvalidValue { field: String, value: String },
}