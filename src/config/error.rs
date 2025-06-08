/*!
 * AI Context Management Tool - Configuration Error Types (Simplified)
 *
 * シンプル化されたエラー型定義
 */

use thiserror::Error;

/// 設定関連のエラー型（シンプル版）
#[derive(Error, Debug)]
pub enum ConfigError {
    /// ファイルが見つからない
    #[error("設定ファイルが見つかりません: {path}")]
    FileNotFound { path: String },

    /// ファイル読み書きエラー
    #[error("ファイルの読み書きでエラーが発生しました")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    /// YAML解析エラー
    #[error("YAMLファイルの解析でエラーが発生しました")]
    YamlError {
        #[from]
        source: serde_yaml::Error,
    },

    /// 設定値検証エラー
    #[error("設定値が無効です: {message}")]
    ValidationError { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_config_error_display() {
        let error = ConfigError::FileNotFound {
            path: "test.yaml".to_string(),
        };
        assert!(error.to_string().contains("設定ファイルが見つかりません"));
        assert!(error.to_string().contains("test.yaml"));
    }

    #[test]
    fn test_validation_error() {
        let error = ConfigError::ValidationError {
            message: "バージョンが指定されていません".to_string(),
        };
        assert!(error.to_string().contains("設定値が無効です"));
        assert!(error.to_string().contains("バージョンが指定されていません"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = IoError::new(ErrorKind::PermissionDenied, "Permission denied");
        let config_error = ConfigError::IoError { source: io_error };

        assert!(config_error
            .to_string()
            .contains("ファイルの読み書きでエラーが発生しました"));
    }

    #[test]
    fn test_yaml_error_conversion() {
        let yaml_content = "invalid: yaml: content: [";
        let yaml_error = serde_yaml::from_str::<serde_yaml::Value>(yaml_content).unwrap_err();
        let config_error = ConfigError::YamlError { source: yaml_error };

        assert!(config_error
            .to_string()
            .contains("YAMLファイルの解析でエラーが発生しました"));
    }

    #[test]
    fn test_error_debug_format() {
        let error = ConfigError::FileNotFound {
            path: "debug_test.yaml".to_string(),
        };
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("FileNotFound"));
        assert!(debug_string.contains("debug_test.yaml"));
    }

    #[test]
    fn test_error_source_chain() {
        let io_error = IoError::new(ErrorKind::NotFound, "File not found");
        let config_error = ConfigError::IoError { source: io_error };

        // エラーソースが適切に設定されていることを確認
        assert!(config_error.source().is_some());
    }
}
