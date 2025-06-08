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
    /// 出力モード: 統合 or 分割
    pub output_mode: OutputMode,
    /// ベースとなるドキュメントディレクトリ
    pub base_docs_dir: String,
    /// エージェント有効/無効設定
    pub agents: AgentConfig,
}

/// 出力モードの種類
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// 全ファイルを1つに結合
    Merged,
    /// ファイルごとに分割
    Split,
}

/// エージェント有効/無効設定（シンプル版）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    /// Cursor エージェント
    #[serde(default)]
    pub cursor: bool,
    /// Cline エージェント
    #[serde(default)]
    pub cline: bool,
    /// GitHub Copilot エージェント
    #[serde(default)]
    pub github: bool,
    /// Claude Code エージェント
    #[serde(default)]
    pub claude: bool,
}

impl Default for AIContextConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            output_mode: OutputMode::Merged,
            base_docs_dir: "./docs".to_string(),
            agents: AgentConfig::default(),
        }
    }
}

impl AIContextConfig {

    /// 有効なエージェントのリストを取得
    pub fn enabled_agents(&self) -> Vec<String> {
        let mut agents = Vec::new();
        if self.agents.cursor {
            agents.push("cursor".to_string());
        }
        if self.agents.cline {
            agents.push("cline".to_string());
        }
        if self.agents.github {
            agents.push("github".to_string());
        }
        if self.agents.claude {
            agents.push("claude".to_string());
        }
        agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AIContextConfig::default();
        assert_eq!(config.version, "1.0");
        assert!(matches!(config.output_mode, OutputMode::Merged));
        assert_eq!(config.base_docs_dir, "./docs");
        assert!(!config.agents.cursor);
        assert!(!config.agents.cline);
        assert!(!config.agents.github);
        assert!(!config.agents.claude);
    }

    #[test]
    fn test_enabled_agents_empty() {
        let config = AIContextConfig::default();
        assert!(config.enabled_agents().is_empty());
    }

    #[test]
    fn test_enabled_agents_multiple() {
        let mut config = AIContextConfig::default();
        config.agents.cursor = true;
        config.agents.claude = true;

        let enabled = config.enabled_agents();
        assert_eq!(enabled.len(), 2);
        assert!(enabled.contains(&"cursor".to_string()));
        assert!(enabled.contains(&"claude".to_string()));
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
    fn test_agent_config_serialization() {
        let mut agents = AgentConfig::default();
        agents.cursor = true;
        agents.github = true;

        let yaml = serde_yaml::to_string(&agents).unwrap();
        let deserialized: AgentConfig = serde_yaml::from_str(&yaml).unwrap();

        assert!(deserialized.cursor);
        assert!(!deserialized.cline);
        assert!(deserialized.github);
        assert!(!deserialized.claude);
    }

    #[test]
    fn test_complete_config_serialization() {
        let mut config = AIContextConfig::default();
        config.output_mode = OutputMode::Split;
        config.base_docs_dir = "./custom-docs".to_string();
        config.agents.cursor = true;

        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: AIContextConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.version, "1.0");
        assert!(matches!(deserialized.output_mode, OutputMode::Split));
        assert_eq!(deserialized.base_docs_dir, "./custom-docs");
        assert!(deserialized.agents.cursor);
    }

    #[test]
    fn test_yaml_parsing_with_missing_fields() {
        let yaml = r#"
version: "1.0"
output_mode: merged
base_docs_dir: "./docs"
agents:
  cursor: true
"#;

        let config: AIContextConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.agents.cursor);
        assert!(!config.agents.cline); // default false
        assert!(!config.agents.github); // default false
        assert!(!config.agents.claude); // default false
    }
}
