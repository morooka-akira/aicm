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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_config_error_display() {
        // FileNotFound エラーのテスト
        let error = ConfigError::FileNotFound {
            path: "test.yaml".to_string(),
        };
        assert!(error.to_string().contains("設定ファイルが見つかりません"));
        assert!(error.to_string().contains("test.yaml"));

        // ValidationError エラーのテスト
        let validation_error = ConfigError::ValidationError {
            errors: vec![
                "version フィールドが空です".to_string(),
                "base_docs_dir フィールドが空です".to_string(),
            ],
        };
        let error_msg = validation_error.to_string();
        assert!(error_msg.contains("設定の検証に失敗しました"));
        assert!(error_msg.contains("version フィールドが空です"));

        // MissingRequiredField エラーのテスト
        let missing_field_error = ConfigError::MissingRequiredField {
            field: "version".to_string(),
        };
        assert!(missing_field_error.to_string().contains("必須フィールドが不足しています"));
        assert!(missing_field_error.to_string().contains("version"));

        // InvalidValue エラーのテスト
        let invalid_value_error = ConfigError::InvalidValue {
            field: "output_mode".to_string(),
            value: "invalid".to_string(),
        };
        assert!(invalid_value_error.to_string().contains("不正な値です"));
        assert!(invalid_value_error.to_string().contains("output_mode"));
        assert!(invalid_value_error.to_string().contains("invalid"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
        let config_error: ConfigError = io_error.into();
        
        match config_error {
            ConfigError::IoError { source } => {
                assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
                assert!(source.to_string().contains("Permission denied"));
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_yaml_error_conversion() {
        // 無効なYAMLを作成してパースエラーを生成
        let invalid_yaml = "invalid: yaml: content: [";
        let yaml_error = serde_yaml::from_str::<serde_yaml::Value>(invalid_yaml).unwrap_err();
        let config_error: ConfigError = yaml_error.into();
        
        match config_error {
            ConfigError::YamlParseError { source: _ } => {
                // YAML パースエラーが正しく変換されたことを確認
                assert!(config_error.to_string().contains("YAML解析に失敗しました"));
            }
            _ => panic!("Expected YamlParseError variant"),
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = ConfigError::FileNotFound {
            path: "test.yaml".to_string(),
        };
        
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("FileNotFound"));
        assert!(debug_output.contains("test.yaml"));
    }

    #[test]
    fn test_validation_error_multiple_errors() {
        let errors = vec![
            "エラー1".to_string(),
            "エラー2".to_string(),
            "エラー3".to_string(),
        ];
        
        let validation_error = ConfigError::ValidationError { 
            errors: errors.clone() 
        };
        
        let error_msg = validation_error.to_string();
        for error in &errors {
            assert!(error_msg.contains(error));
        }
    }

    #[test]
    fn test_error_source_chain() {
        // IO エラーをソースとするConfigErrorのテスト
        let original_io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let config_error = ConfigError::IoError { source: original_io_error };
        
        // エラーソースを取得できることを確認
        let source = std::error::Error::source(&config_error);
        assert!(source.is_some());
        assert!(source.unwrap().to_string().contains("File not found"));
    }
}