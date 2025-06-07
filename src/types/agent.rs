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