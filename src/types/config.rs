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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_output_mode_serialization() {
        // Merged
        let merged = OutputMode::Merged;
        let yaml = serde_yaml::to_string(&merged).unwrap();
        assert!(yaml.trim() == "merged");
        
        let deserialized: OutputMode = serde_yaml::from_str(&yaml).unwrap();
        matches!(deserialized, OutputMode::Merged);

        // Split
        let split = OutputMode::Split;
        let yaml = serde_yaml::to_string(&split).unwrap();
        assert!(yaml.trim() == "split");
        
        let deserialized: OutputMode = serde_yaml::from_str(&yaml).unwrap();
        matches!(deserialized, OutputMode::Split);
    }

    #[test]
    fn test_cursor_rule_type_serialization() {
        // Always
        let always = CursorRuleType::Always;
        let yaml = serde_yaml::to_string(&always).unwrap();
        assert!(yaml.trim() == "always");

        // AutoAttached
        let auto_attached = CursorRuleType::AutoAttached;
        let yaml = serde_yaml::to_string(&auto_attached).unwrap();
        assert!(yaml.trim() == "auto_attached");

        // AgentRequested
        let agent_requested = CursorRuleType::AgentRequested;
        let yaml = serde_yaml::to_string(&agent_requested).unwrap();
        assert!(yaml.trim() == "agent_requested");

        // Manual
        let manual = CursorRuleType::Manual;
        let yaml = serde_yaml::to_string(&manual).unwrap();
        assert!(yaml.trim() == "manual");
    }

    #[test]
    fn test_agent_configs_default() {
        let config = AgentConfigs::default();
        assert!(config.cursor.is_none());
        assert!(config.cline.is_none());
        assert!(config.github.is_none());
        assert!(config.claude.is_none());
    }

    #[test]
    fn test_cursor_config_serialization() {
        let mut split_config = HashMap::new();
        split_config.insert(
            "common".to_string(),
            CursorRuleConfig {
                rule_type: CursorRuleType::Always,
                description: "Common rules".to_string(),
                globs: Some(vec!["**/*.rs".to_string()]),
                always_apply: Some(true),
            },
        );

        let config = CursorConfig {
            split_config: Some(split_config),
            additional_instructions: Some("Additional instructions".to_string()),
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: CursorConfig = serde_yaml::from_str(&yaml).unwrap();

        assert!(deserialized.split_config.is_some());
        assert_eq!(deserialized.additional_instructions, Some("Additional instructions".to_string()));

        let split_config = deserialized.split_config.unwrap();
        assert!(split_config.contains_key("common"));
        
        let rule = &split_config["common"];
        assert_eq!(rule.description, "Common rules");
        matches!(rule.rule_type, CursorRuleType::Always);
    }

    #[test]
    fn test_cursor_rule_config_defaults() {
        let rule = CursorRuleConfig {
            rule_type: CursorRuleType::AutoAttached,
            description: "Test rule".to_string(),
            globs: None,
            always_apply: None,
        };

        let yaml = serde_yaml::to_string(&rule).unwrap();
        let deserialized: CursorRuleConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.description, "Test rule");
        matches!(deserialized.rule_type, CursorRuleType::AutoAttached);
        assert!(deserialized.globs.is_none());
        assert!(deserialized.always_apply.is_none());
    }

    #[test]
    fn test_file_mapping_serialization() {
        let mapping = FileMapping {
            common: vec!["common1.md".to_string(), "common2.md".to_string()],
            project_specific: vec!["project.md".to_string()],
            agent_specific: Some(HashMap::from([
                ("cursor".to_string(), vec!["cursor.md".to_string()]),
                ("cline".to_string(), vec!["cline1.md".to_string(), "cline2.md".to_string()]),
            ])),
        };

        let yaml = serde_yaml::to_string(&mapping).unwrap();
        let deserialized: FileMapping = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.common.len(), 2);
        assert_eq!(deserialized.project_specific.len(), 1);
        assert!(deserialized.agent_specific.is_some());

        let agent_specific = deserialized.agent_specific.unwrap();
        assert!(agent_specific.contains_key("cursor"));
        assert!(agent_specific.contains_key("cline"));
        assert_eq!(agent_specific["cursor"].len(), 1);
        assert_eq!(agent_specific["cline"].len(), 2);
    }

    #[test]
    fn test_ai_context_config_complete() {
        let config = AIContextConfig {
            version: "1.0".to_string(),
            output_mode: OutputMode::Split,
            base_docs_dir: "./docs".to_string(),
            agents: AgentConfigs {
                cursor: Some(CursorConfig {
                    split_config: None,
                    additional_instructions: None,
                }),
                cline: Some(ClineConfig {
                    split_config: Some(ClineSplitConfig {
                        file_prefix: "01-".to_string(),
                        max_files: Some(10),
                    }),
                    additional_instructions: None,
                }),
                github: Some(GitHubConfig {
                    hierarchy_config: Some(HashMap::from([
                        ("root".to_string(), GitHubHierarchyConfig {
                            path: "instructions.md".to_string(),
                            scope: "workspace".to_string(),
                            priority: Some(1),
                        }),
                    ])),
                    additional_instructions: None,
                }),
                claude: Some(ClaudeConfig {
                    language: Some("ja".to_string()),
                    additional_sections: Some(vec!["development".to_string()]),
                    additional_instructions: None,
                }),
            },
            file_mapping: FileMapping {
                common: vec!["common.md".to_string()],
                project_specific: vec!["project.md".to_string()],
                agent_specific: None,
            },
            global_variables: HashMap::from([
                ("PROJECT_NAME".to_string(), serde_yaml::Value::String("AI Context Management".to_string())),
                ("VERSION".to_string(), serde_yaml::Value::String("1.0".to_string())),
            ]),
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: AIContextConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.version, "1.0");
        matches!(deserialized.output_mode, OutputMode::Split);
        assert_eq!(deserialized.base_docs_dir, "./docs");

        // エージェント設定の確認
        assert!(deserialized.agents.cursor.is_some());
        assert!(deserialized.agents.cline.is_some());
        assert!(deserialized.agents.github.is_some());
        assert!(deserialized.agents.claude.is_some());

        // Cline設定の詳細確認
        let cline_config = deserialized.agents.cline.unwrap();
        assert!(cline_config.split_config.is_some());
        let split_config = cline_config.split_config.unwrap();
        assert_eq!(split_config.file_prefix, "01-");
        assert_eq!(split_config.max_files, Some(10));

        // グローバル変数の確認
        assert_eq!(deserialized.global_variables.len(), 2);
        assert!(deserialized.global_variables.contains_key("PROJECT_NAME"));
    }

    #[test]
    fn test_github_hierarchy_config() {
        let hierarchy = GitHubHierarchyConfig {
            path: "src/instructions.md".to_string(),
            scope: "source_code".to_string(),
            priority: Some(2),
        };

        let yaml = serde_yaml::to_string(&hierarchy).unwrap();
        let deserialized: GitHubHierarchyConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.path, "src/instructions.md");
        assert_eq!(deserialized.scope, "source_code");
        assert_eq!(deserialized.priority, Some(2));
    }

    #[test]
    fn test_claude_config_defaults() {
        let config = ClaudeConfig {
            language: None,
            additional_sections: None,
            additional_instructions: None,
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: ClaudeConfig = serde_yaml::from_str(&yaml).unwrap();

        assert!(deserialized.language.is_none());
        assert!(deserialized.additional_sections.is_none());
        assert!(deserialized.additional_instructions.is_none());
    }

    #[test]
    fn test_minimal_ai_context_config() {
        // 最小限の設定をテスト
        let config = AIContextConfig {
            version: "1.0".to_string(),
            output_mode: OutputMode::Merged,
            base_docs_dir: "./docs".to_string(),
            agents: AgentConfigs::default(),
            file_mapping: FileMapping {
                common: vec!["README.md".to_string()],
                project_specific: vec!["docs/project.md".to_string()],
                agent_specific: None,
            },
            global_variables: HashMap::new(),
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: AIContextConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(deserialized.version, "1.0");
        matches!(deserialized.output_mode, OutputMode::Merged);
        assert!(deserialized.global_variables.is_empty());
        assert!(deserialized.file_mapping.agent_specific.is_none());
    }

    #[test]
    fn test_invalid_yaml_parsing() {
        // 無効なoutput_mode
        let invalid_yaml = r#"
version: "1.0"
output_mode: "invalid"
base_docs_dir: "./docs"
agents: {}
file_mapping:
  common: ["file.md"]
  project_specific: ["project.md"]
global_variables: {}
        "#;

        let result: Result<AIContextConfig, _> = serde_yaml::from_str(invalid_yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_file_mapping_arrays() {
        let mapping = FileMapping {
            common: vec![],
            project_specific: vec![],
            agent_specific: None,
        };

        let yaml = serde_yaml::to_string(&mapping).unwrap();
        let deserialized: FileMapping = serde_yaml::from_str(&yaml).unwrap();

        assert!(deserialized.common.is_empty());
        assert!(deserialized.project_specific.is_empty());
        assert!(deserialized.agent_specific.is_none());
    }
}