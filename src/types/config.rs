/*!
 * AI Context Management Tool - Configuration Types
 * 
 * このファイルは設定ファイル（ai-context.yaml）の型定義を提供します。
 * 各エージェント固有の設定と共通設定を定義しています。
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// メインの設定ファイル構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContextConfig {
    /// 設定ファイルのバージョン
    pub version: String,
    /// 出力モード: 統合 or 分割
    pub output_mode: OutputMode,
    /// ベースとなるドキュメントディレクトリ
    pub base_docs_dir: String,
    /// エージェント固有の設定
    pub agents: AgentConfigs,
    /// ファイルマッピング設定
    pub file_mapping: FileMapping,
    /// グローバル変数（テンプレート置換用）
    #[serde(default)]
    pub global_variables: HashMap<String, serde_yaml::Value>,
}

/// 出力モードの種類
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Merged,
    Split,
}

/// 各エージェントの設定をまとめる型
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfigs {
    #[serde(default)]
    pub cursor: Option<CursorConfig>,
    #[serde(default)]
    pub cline: Option<ClineConfig>,
    #[serde(default)]
    pub github: Option<GitHubConfig>,
    #[serde(default)]
    pub claude: Option<ClaudeConfig>,
}

/// Cursor固有の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorConfig {
    /// 分割モード時の設定
    #[serde(default)]
    pub split_config: Option<HashMap<String, CursorRuleConfig>>,
    /// 追加の指示文
    #[serde(default)]
    pub additional_instructions: Option<String>,
}

/// Cursorルール固有の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorRuleConfig {
    /// ルールの適用タイプ
    #[serde(rename = "type")]
    pub rule_type: CursorRuleType,
    /// ルールの説明
    pub description: String,
    /// 適用対象ファイルのglobパターン
    #[serde(default)]
    pub globs: Option<Vec<String>>,
    /// 常に適用するかどうか
    #[serde(default)]
    pub always_apply: Option<bool>,
}

/// Cursorルールの適用タイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorRuleType {
    Always,
    AutoAttached,
    AgentRequested,
    Manual,
}

/// Cline固有の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClineConfig {
    /// 分割モード時の設定
    #[serde(default)]
    pub split_config: Option<ClineSplitConfig>,
    /// 追加の指示文
    #[serde(default)]
    pub additional_instructions: Option<String>,
}

/// Cline分割設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClineSplitConfig {
    /// ファイル名のプレフィックス
    pub file_prefix: String,
    /// 最大ファイル数
    #[serde(default)]
    pub max_files: Option<u32>,
}

/// GitHub Copilot固有の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// 階層的ファイル配置の設定
    #[serde(default)]
    pub hierarchy_config: Option<HashMap<String, GitHubHierarchyConfig>>,
    /// 追加の指示文
    #[serde(default)]
    pub additional_instructions: Option<String>,
}

/// GitHub階層設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubHierarchyConfig {
    /// ファイルパス
    pub path: String,
    /// 適用スコープ
    pub scope: String,
    /// 優先度
    #[serde(default)]
    pub priority: Option<u32>,
}

/// Claude Code固有の設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// 言語設定
    #[serde(default)]
    pub language: Option<String>,
    /// 追加セクション
    #[serde(default)]
    pub additional_sections: Option<Vec<String>>,
    /// 追加の指示文
    #[serde(default)]
    pub additional_instructions: Option<String>,
}

/// ファイルマッピング設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    /// 共通ファイル
    pub common: Vec<String>,
    /// プロジェクト固有ファイル
    pub project_specific: Vec<String>,
    /// エージェント固有ファイル
    #[serde(default)]
    pub agent_specific: Option<HashMap<String, Vec<String>>>,
}