/*!
 * AI Context Management Tool - Agent Types
 * 
 * このファイルはエージェント関連の型定義を提供します。
 * ベースエージェントトレイトと生成ファイル、コンテンツ構造を定義しています。
 */

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 生成されるファイルの情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// ファイルパス（プロジェクトルートからの相対パス）
    pub path: String,
    /// ファイルの内容
    pub content: String,
    /// 文字エンコーディング（デフォルトはutf8）
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

fn default_encoding() -> String {
    "utf8".to_string()
}

/// 分割されたコンテンツ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitContent {
    /// 共通コンテンツ
    pub common: String,
    /// プロジェクト固有コンテンツ
    pub project_specific: String,
    /// エージェント固有コンテンツ
    pub agent_specific: String,
}

/// マージされたコンテンツ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedContent {
    /// 全コンテンツを結合したもの
    pub merged: String,
    /// 分割されたコンテンツ
    pub split: SplitContent,
}

/// 検証結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 検証が成功したか
    pub valid: bool,
    /// エラーメッセージ（検証失敗時）
    pub errors: Vec<String>,
    /// 警告メッセージ
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// 成功の検証結果を作成
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// エラー付きの検証結果を作成
    pub fn with_errors(errors: Vec<String>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// 警告付きの検証結果を作成
    pub fn with_warnings(warnings: Vec<String>) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings,
        }
    }

    /// エラーと警告両方を含む検証結果を作成
    pub fn with_errors_and_warnings(errors: Vec<String>, warnings: Vec<String>) -> Self {
        Self {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_file_default_encoding() {
        let file = GeneratedFile {
            path: "test.txt".to_string(),
            content: "test content".to_string(),
            encoding: default_encoding(),
        };
        
        assert_eq!(file.encoding, "utf8");
    }

    #[test]
    fn test_generated_file_serialization() {
        let file = GeneratedFile {
            path: ".cursor/rules/test.mdc".to_string(),
            content: "---\ndescription: Test\n---\n\n# Test Content".to_string(),
            encoding: "utf8".to_string(),
        };

        let yaml = serde_yaml::to_string(&file).unwrap();
        let deserialized: GeneratedFile = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.path, file.path);
        assert_eq!(deserialized.content, file.content);
        assert_eq!(deserialized.encoding, file.encoding);
    }

    #[test]
    fn test_split_content() {
        let split = SplitContent {
            common: "# Common\nCommon content".to_string(),
            project_specific: "# Project\nProject content".to_string(),
            agent_specific: "# Agent\nAgent content".to_string(),
        };

        let yaml = serde_yaml::to_string(&split).unwrap();
        let deserialized: SplitContent = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.common, split.common);
        assert_eq!(deserialized.project_specific, split.project_specific);
        assert_eq!(deserialized.agent_specific, split.agent_specific);
    }

    #[test]
    fn test_merged_content() {
        let split = SplitContent {
            common: "Common".to_string(),
            project_specific: "Project".to_string(),
            agent_specific: "Agent".to_string(),
        };

        let merged = MergedContent {
            merged: "Common\nProject\nAgent".to_string(),
            split: split.clone(),
        };

        let yaml = serde_yaml::to_string(&merged).unwrap();
        let deserialized: MergedContent = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.merged, merged.merged);
        assert_eq!(deserialized.split.common, split.common);
    }

    #[test]
    fn test_validation_result_success() {
        let result = ValidationResult::success();
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_with_errors() {
        let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
        let result = ValidationResult::with_errors(errors.clone());
        
        assert!(!result.valid);
        assert_eq!(result.errors, errors);
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_result_with_warnings() {
        let warnings = vec!["Warning 1".to_string(), "Warning 2".to_string()];
        let result = ValidationResult::with_warnings(warnings.clone());
        
        assert!(result.valid);
        assert!(result.errors.is_empty());
        assert_eq!(result.warnings, warnings);
    }

    #[test]
    fn test_validation_result_with_errors_and_warnings() {
        let errors = vec!["Error".to_string()];
        let warnings = vec!["Warning".to_string()];
        let result = ValidationResult::with_errors_and_warnings(errors.clone(), warnings.clone());
        
        assert!(!result.valid);  // エラーがあるので無効
        assert_eq!(result.errors, errors);
        assert_eq!(result.warnings, warnings);

        // エラーがない場合は有効
        let no_errors_result = ValidationResult::with_errors_and_warnings(vec![], warnings.clone());
        assert!(no_errors_result.valid);
        assert!(no_errors_result.errors.is_empty());
        assert_eq!(no_errors_result.warnings, warnings);
    }

    #[test]
    fn test_agent_info() {
        let info = AgentInfo {
            name: "test_agent".to_string(),
            description: "Test Agent for unit testing".to_string(),
            output_patterns: vec!["*.test".to_string(), "test/**/*".to_string()],
            supports_split: true,
        };

        let yaml = serde_yaml::to_string(&info).unwrap();
        let deserialized: AgentInfo = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.name, info.name);
        assert_eq!(deserialized.description, info.description);
        assert_eq!(deserialized.output_patterns, info.output_patterns);
        assert_eq!(deserialized.supports_split, info.supports_split);
    }

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            valid: false,
            errors: vec!["Configuration error".to_string()],
            warnings: vec!["Deprecated option".to_string()],
        };

        let yaml = serde_yaml::to_string(&result).unwrap();
        let deserialized: ValidationResult = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.valid, result.valid);
        assert_eq!(deserialized.errors, result.errors);
        assert_eq!(deserialized.warnings, result.warnings);
    }

    #[test]
    fn test_generated_file_empty_content() {
        let file = GeneratedFile {
            path: "empty.txt".to_string(),
            content: "".to_string(),
            encoding: "utf8".to_string(),
        };

        assert!(file.content.is_empty());
        assert_eq!(file.path, "empty.txt");
    }

    #[test]
    fn test_split_content_empty_sections() {
        let split = SplitContent {
            common: "".to_string(),
            project_specific: "".to_string(),
            agent_specific: "".to_string(),
        };

        assert!(split.common.is_empty());
        assert!(split.project_specific.is_empty());
        assert!(split.agent_specific.is_empty());
    }

    #[test]
    fn test_agent_info_no_patterns() {
        let info = AgentInfo {
            name: "minimal_agent".to_string(),
            description: "Minimal agent".to_string(),
            output_patterns: vec![],
            supports_split: false,
        };

        assert!(info.output_patterns.is_empty());
        assert!(!info.supports_split);
    }

    #[test]
    fn test_validation_result_edge_cases() {
        // 空のエラーと警告
        let empty_result = ValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec![],
        };
        assert!(empty_result.valid);
        assert!(empty_result.errors.is_empty());
        assert!(empty_result.warnings.is_empty());

        // エラーのみ
        let error_only = ValidationResult {
            valid: false,
            errors: vec!["Single error".to_string()],
            warnings: vec![],
        };
        assert!(!error_only.valid);
        assert!(!error_only.errors.is_empty());
        assert!(error_only.warnings.is_empty());

        // 警告のみ
        let warning_only = ValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec!["Single warning".to_string()],
        };
        assert!(warning_only.valid);
        assert!(warning_only.errors.is_empty());
        assert!(!warning_only.warnings.is_empty());
    }
}

/// エージェントの基本情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// エージェント名
    pub name: String,
    /// 説明
    pub description: String,
    /// 対応する出力ファイルパターン
    pub output_patterns: Vec<String>,
    /// 分割モードに対応しているか
    pub supports_split: bool,
}

/// ベースエージェントトレイト
#[async_trait]
pub trait BaseAgent: Send + Sync {
    /// エージェント情報を取得
    fn get_info(&self) -> AgentInfo;

    /// ファイルを生成する
    /// 
    /// # Arguments
    /// * `merged_content` - マージされたコンテンツ
    /// * `split_content` - 分割されたコンテンツ
    /// 
    /// # Returns
    /// 生成されるファイルの配列
    async fn generate_files(
        &self,
        merged_content: &str,
        split_content: &SplitContent,
    ) -> Result<Vec<GeneratedFile>>;

    /// 出力予定のパスを取得する
    /// 
    /// # Returns
    /// 出力ファイルパスの配列
    fn get_output_paths(&self) -> Vec<String>;

    /// 設定を検証する
    /// 
    /// # Returns
    /// 検証結果
    fn validate(&self) -> ValidationResult;
}