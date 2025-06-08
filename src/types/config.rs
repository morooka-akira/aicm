/*!
 * AI Context Management Tool - Configuration Types (Simplified)
 *
 * シンプル化された設定ファイル（ai-context.yaml）の型定義
 */

use serde::{Deserialize, Serialize};

/// メインの設定ファイル構造（シンプル版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContextConfig {
    /// 設定ファイルのバージョン
    pub version: String,
    /// 出力モード: 統合 or 分割（オプショナル、デフォルト：merged）
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
    /// ベースとなるドキュメントディレクトリ
    pub base_docs_dir: String,
    /// エージェント有効/無効設定
    pub agents: AgentConfig,
}

/// 出力モードの種類
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// 全ファイルを1つに結合
    Merged,
    /// ファイルごとに分割
    Split,
}

/// エージェント有効/無効設定（拡張版）
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AgentConfig {
    /// Cursor エージェント
    #[serde(default)]
    pub cursor: CursorConfig,
    /// Cline エージェント
    #[serde(default)]
    pub cline: ClineConfig,
    /// GitHub Copilot エージェント
    #[serde(default)]
    pub github: GitHubConfig,
    /// Claude Code エージェント
    #[serde(default)]
    pub claude: ClaudeConfig,
}

/// Cursor エージェント設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CursorConfig {
    /// シンプル設定（後方互換性）
    Simple(bool),
    /// 詳細設定
    Advanced(CursorAgentConfig),
}

/// Cline エージェント設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ClineConfig {
    /// シンプル設定（後方互換性）
    Simple(bool),
    /// 詳細設定
    Advanced(ClineAgentConfig),
}

/// GitHub エージェント設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GitHubConfig {
    /// シンプル設定（後方互換性）
    Simple(bool),
    /// 詳細設定
    Advanced(GitHubAgentConfig),
}

/// Claude エージェント設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ClaudeConfig {
    /// シンプル設定（後方互換性）
    Simple(bool),
    /// 詳細設定
    Advanced(ClaudeAgentConfig),
}

/// Cursor エージェント詳細設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CursorAgentConfig {
    /// エージェント有効/無効（デフォルト：true）
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 出力モード（オプショナル、グローバル設定を上書き）
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
    /// splitモード時の詳細設定（オプショナル）
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub split_config: Option<CursorSplitConfig>,
}

/// Cursor splitモード設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CursorSplitConfig {
    /// ルール配列
    #[serde(default)]
    pub rules: Vec<CursorSplitRule>,
}

/// Cursor splitモード時のルール設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CursorSplitRule {
    /// 対象となるMarkdownファイル名パターン
    pub file_patterns: Vec<String>,
    /// Always ルール用（alwaysApply: true）
    #[serde(default, rename = "alwaysApply")]
    pub always_apply: Option<bool>,
    /// Auto Attached ルール用（globs設定）
    #[serde(default)]
    pub globs: Option<Vec<String>>,
    /// Agent Requested ルール用（description設定）
    #[serde(default)]
    pub description: Option<String>,
    /// Manual ルール用（manual: true）
    #[serde(default)]
    pub manual: Option<bool>,
}


/// Cline エージェント詳細設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClineAgentConfig {
    /// エージェント有効/無効（デフォルト：true）
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 出力モード（オプショナル、グローバル設定を上書き）
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
}

/// GitHub エージェント詳細設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubAgentConfig {
    /// エージェント有効/無効（デフォルト：true）
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 出力モード（オプショナル、グローバル設定を上書き）
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
}

/// Claude エージェント詳細設定
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaudeAgentConfig {
    /// エージェント有効/無効（デフォルト：true）
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 出力モード（オプショナル、Claude は常に merged）
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
}

/// デフォルト値: true
fn default_true() -> bool {
    true
}

/// デフォルトのエージェント設定
impl Default for CursorConfig {
    fn default() -> Self {
        Self::Simple(false)
    }
}

impl Default for ClineConfig {
    fn default() -> Self {
        Self::Simple(false)
    }
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self::Simple(false)
    }
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self::Simple(false)
    }
}

impl Default for AIContextConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            output_mode: None, // デフォルトは None（merged として扱われる）
            base_docs_dir: "./docs".to_string(),
            agents: AgentConfig::default(),
        }
    }
}

impl AIContextConfig {
    /// 有効なエージェントのリストを取得
    pub fn enabled_agents(&self) -> Vec<String> {
        let mut agents = Vec::new();
        if self.agents.cursor.is_enabled() {
            agents.push("cursor".to_string());
        }
        if self.agents.cline.is_enabled() {
            agents.push("cline".to_string());
        }
        if self.agents.github.is_enabled() {
            agents.push("github".to_string());
        }
        if self.agents.claude.is_enabled() {
            agents.push("claude".to_string());
        }
        agents
    }

    /// グローバル出力モードを取得（デフォルト：merged）
    pub fn get_global_output_mode(&self) -> OutputMode {
        self.output_mode.clone().unwrap_or(OutputMode::Merged)
    }

    /// 指定されたエージェントの有効な出力モードを取得
    /// 優先順位: エージェント個別設定 > グローバル設定 > デフォルト（merged）
    pub fn get_effective_output_mode(&self, agent: &str) -> OutputMode {
        match agent {
            "cursor" => self
                .agents
                .cursor
                .get_output_mode()
                .unwrap_or_else(|| self.get_global_output_mode()),
            "cline" => self
                .agents
                .cline
                .get_output_mode()
                .unwrap_or_else(|| self.get_global_output_mode()),
            "github" => self
                .agents
                .github
                .get_output_mode()
                .unwrap_or_else(|| self.get_global_output_mode()),
            "claude" => OutputMode::Merged, // Claude は常に merged
            _ => self.get_global_output_mode(),
        }
    }
}

/// エージェント設定の共通トレイト
pub trait AgentConfigTrait {
    /// エージェントが有効かどうかを取得
    fn is_enabled(&self) -> bool;
    /// エージェント個別の出力モードを取得
    fn get_output_mode(&self) -> Option<OutputMode>;
}

impl AgentConfigTrait for CursorConfig {
    fn is_enabled(&self) -> bool {
        match self {
            Self::Simple(enabled) => *enabled,
            Self::Advanced(config) => config.enabled,
        }
    }

    fn get_output_mode(&self) -> Option<OutputMode> {
        match self {
            Self::Simple(_) => None,
            Self::Advanced(config) => config.output_mode.clone(),
        }
    }
}

impl AgentConfigTrait for ClineConfig {
    fn is_enabled(&self) -> bool {
        match self {
            Self::Simple(enabled) => *enabled,
            Self::Advanced(config) => config.enabled,
        }
    }

    fn get_output_mode(&self) -> Option<OutputMode> {
        match self {
            Self::Simple(_) => None,
            Self::Advanced(config) => config.output_mode.clone(),
        }
    }
}

impl AgentConfigTrait for GitHubConfig {
    fn is_enabled(&self) -> bool {
        match self {
            Self::Simple(enabled) => *enabled,
            Self::Advanced(config) => config.enabled,
        }
    }

    fn get_output_mode(&self) -> Option<OutputMode> {
        match self {
            Self::Simple(_) => None,
            Self::Advanced(config) => config.output_mode.clone(),
        }
    }
}

impl AgentConfigTrait for ClaudeConfig {
    fn is_enabled(&self) -> bool {
        match self {
            Self::Simple(enabled) => *enabled,
            Self::Advanced(config) => config.enabled,
        }
    }

    fn get_output_mode(&self) -> Option<OutputMode> {
        match self {
            Self::Simple(_) => None,
            Self::Advanced(config) => config.output_mode.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AIContextConfig::default();
        assert_eq!(config.version, "1.0");
        assert!(config.output_mode.is_none());
        assert_eq!(config.get_global_output_mode(), OutputMode::Merged);
        assert_eq!(config.base_docs_dir, "./docs");
        assert!(!config.agents.cursor.is_enabled());
        assert!(!config.agents.cline.is_enabled());
        assert!(!config.agents.github.is_enabled());
        assert!(!config.agents.claude.is_enabled());
    }

    #[test]
    fn test_enabled_agents_empty() {
        let config = AIContextConfig::default();
        assert!(config.enabled_agents().is_empty());
    }

    #[test]
    fn test_enabled_agents_simple_config() {
        let mut config = AIContextConfig::default();
        config.agents.cursor = CursorConfig::Simple(true);
        config.agents.claude = ClaudeConfig::Simple(true);

        let enabled = config.enabled_agents();
        assert_eq!(enabled.len(), 2);
        assert!(enabled.contains(&"cursor".to_string()));
        assert!(enabled.contains(&"claude".to_string()));
    }

    #[test]
    fn test_enabled_agents_advanced_config() {
        let mut config = AIContextConfig::default();
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            output_mode: Some(OutputMode::Split),
            split_config: None,
        });
        config.agents.cline = ClineConfig::Advanced(ClineAgentConfig {
            enabled: false,
            output_mode: Some(OutputMode::Merged),
        });

        let enabled = config.enabled_agents();
        assert_eq!(enabled.len(), 1);
        assert!(enabled.contains(&"cursor".to_string()));
    }

    #[test]
    fn test_global_output_mode() {
        let mut config = AIContextConfig::default();

        // デフォルト（None）の場合は Merged
        assert_eq!(config.get_global_output_mode(), OutputMode::Merged);

        // 明示的に設定した場合
        config.output_mode = Some(OutputMode::Split);
        assert_eq!(config.get_global_output_mode(), OutputMode::Split);
    }

    #[test]
    fn test_effective_output_mode_global_fallback() {
        let mut config = AIContextConfig::default();
        config.output_mode = Some(OutputMode::Split);
        config.agents.cursor = CursorConfig::Simple(true);

        // エージェント個別設定なし → グローバル設定を使用
        assert_eq!(
            config.get_effective_output_mode("cursor"),
            OutputMode::Split
        );
    }

    #[test]
    fn test_effective_output_mode_agent_override() {
        let mut config = AIContextConfig::default();
        config.output_mode = Some(OutputMode::Split);
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            output_mode: Some(OutputMode::Merged),
            split_config: None,
        });

        // エージェント個別設定がグローバル設定を上書き
        assert_eq!(
            config.get_effective_output_mode("cursor"),
            OutputMode::Merged
        );
    }

    #[test]
    fn test_effective_output_mode_claude_always_merged() {
        let mut config = AIContextConfig::default();
        config.output_mode = Some(OutputMode::Split);
        config.agents.claude = ClaudeConfig::Advanced(ClaudeAgentConfig {
            enabled: true,
            output_mode: Some(OutputMode::Split), // 設定されていても無視される
        });

        // Claude は常に merged
        assert_eq!(
            config.get_effective_output_mode("claude"),
            OutputMode::Merged
        );
    }

    #[test]
    fn test_effective_output_mode_default_fallback() {
        let config = AIContextConfig::default();

        // グローバル設定もエージェント個別設定もなし → デフォルト（merged）
        assert_eq!(
            config.get_effective_output_mode("cursor"),
            OutputMode::Merged
        );
        assert_eq!(
            config.get_effective_output_mode("unknown"),
            OutputMode::Merged
        );
    }

    #[test]
    fn test_output_mode_serialization() {
        let merged = OutputMode::Merged;
        let yaml = serde_yaml::to_string(&merged).unwrap();
        assert_eq!(yaml.trim(), "merged");

        let split = OutputMode::Split;
        let yaml = serde_yaml::to_string(&split).unwrap();
        assert_eq!(yaml.trim(), "split");
    }

    #[test]
    fn test_simple_agent_config_serialization() {
        let cursor_config = CursorConfig::Simple(true);
        let yaml = serde_yaml::to_string(&cursor_config).unwrap();
        assert_eq!(yaml.trim(), "true");

        let cursor_config = CursorConfig::Simple(false);
        let yaml = serde_yaml::to_string(&cursor_config).unwrap();
        assert_eq!(yaml.trim(), "false");
    }

    #[test]
    fn test_advanced_agent_config_serialization() {
        let cursor_config = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            output_mode: Some(OutputMode::Split),
            split_config: None,
        });

        let yaml = serde_yaml::to_string(&cursor_config).unwrap();
        let deserialized: CursorConfig = serde_yaml::from_str(&yaml).unwrap();

        assert!(deserialized.is_enabled());
        assert_eq!(deserialized.get_output_mode(), Some(OutputMode::Split));
        
        // split_config: null が出力されないことを確認
        assert!(!yaml.contains("split_config"));
    }

    #[test]
    fn test_backward_compatibility_parsing() {
        // 既存の設定形式をパース
        let yaml = r#"
version: "1.0"
output_mode: split
base_docs_dir: "./ai-context"
agents:
  cursor: true
  cline: false
  github: true
  claude: false
"#;

        let config: AIContextConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.version, "1.0");
        assert_eq!(config.get_global_output_mode(), OutputMode::Split);
        assert_eq!(config.base_docs_dir, "./ai-context");
        assert!(config.agents.cursor.is_enabled());
        assert!(!config.agents.cline.is_enabled());
        assert!(config.agents.github.is_enabled());
        assert!(!config.agents.claude.is_enabled());

        // 後方互換性：エージェント個別設定なし → グローバル設定を使用
        assert_eq!(
            config.get_effective_output_mode("cursor"),
            OutputMode::Split
        );
        assert_eq!(
            config.get_effective_output_mode("github"),
            OutputMode::Split
        );
    }

    #[test]
    fn test_new_format_parsing() {
        // 新しい設定形式をパース
        let yaml = r#"
version: "1.0"
output_mode: split
base_docs_dir: "./ai-context"
agents:
  cursor: true
  cline:
    enabled: true
    output_mode: merged
  github:
    output_mode: split
  claude: false
"#;

        let config: AIContextConfig = serde_yaml::from_str(yaml).unwrap();

        assert!(config.agents.cursor.is_enabled());
        assert!(config.agents.cline.is_enabled());
        assert!(config.agents.github.is_enabled()); // enabled のデフォルトは true
        assert!(!config.agents.claude.is_enabled());

        // 有効な出力モードの確認
        assert_eq!(
            config.get_effective_output_mode("cursor"),
            OutputMode::Split
        ); // グローバル設定
        assert_eq!(
            config.get_effective_output_mode("cline"),
            OutputMode::Merged
        ); // 個別設定
        assert_eq!(
            config.get_effective_output_mode("github"),
            OutputMode::Split
        ); // 個別設定
        assert_eq!(
            config.get_effective_output_mode("claude"),
            OutputMode::Merged
        ); // 常に merged
    }

    #[test]
    fn test_mixed_format_parsing() {
        // 混在形式のパース
        let yaml = r#"
version: "1.0"
base_docs_dir: "./ai-context"
agents:
  cursor: true
  cline:
    enabled: false
    output_mode: merged
  github:
    output_mode: split
  claude: true
"#;

        let config: AIContextConfig = serde_yaml::from_str(yaml).unwrap();

        // グローバル output_mode がない場合はデフォルト（merged）
        assert_eq!(config.get_global_output_mode(), OutputMode::Merged);

        assert!(config.agents.cursor.is_enabled());
        assert!(!config.agents.cline.is_enabled());
        assert!(config.agents.github.is_enabled());
        assert!(config.agents.claude.is_enabled());

        // 有効な出力モードの確認
        assert_eq!(
            config.get_effective_output_mode("cursor"),
            OutputMode::Merged
        ); // グローバルデフォルト
        assert_eq!(
            config.get_effective_output_mode("github"),
            OutputMode::Split
        ); // 個別設定
        assert_eq!(
            config.get_effective_output_mode("claude"),
            OutputMode::Merged
        ); // 常に merged
    }
}
