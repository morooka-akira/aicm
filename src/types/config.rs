/*!
 * AI Context Management Tool - Configuration Types (Simplified)
 *
 * Simplified configuration file (ai-context.yaml) type definitions
 */

use serde::{Deserialize, Serialize};

/// Main configuration file structure (simplified version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContextConfig {
    /// Configuration file version
    pub version: String,
    /// Output mode: merged or split (optional, default: merged)
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
    /// Whether to include filename headers in merged mode (default: false)
    #[serde(default)]
    pub include_filenames: Option<bool>,
    /// Base documentation directory
    pub base_docs_dir: String,
    /// Agent enable/disable settings
    pub agents: AgentConfig,
}

/// Output mode types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    /// Merge all files into one
    Merged,
    /// Split by file
    Split,
}

/// Agent enable/disable settings (extended version)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AgentConfig {
    /// Cursor agent
    #[serde(default)]
    pub cursor: CursorConfig,
    /// Cline agent
    #[serde(default)]
    pub cline: ClineConfig,
    /// GitHub Copilot agent
    #[serde(default)]
    pub github: GitHubConfig,
    /// Claude Code agent
    #[serde(default)]
    pub claude: ClaudeConfig,
    /// OpenAI Codex agent
    #[serde(default)]
    pub codex: CodexConfig,
}

/// Cursor agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CursorConfig {
    /// Simple configuration (backward compatibility)
    Simple(bool),
    /// Detailed configuration
    Advanced(CursorAgentConfig),
}

/// Cline agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ClineConfig {
    /// Simple configuration (backward compatibility)
    Simple(bool),
    /// Detailed configuration
    Advanced(ClineAgentConfig),
}

/// GitHub agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum GitHubConfig {
    /// Simple configuration (backward compatibility)
    Simple(bool),
    /// Detailed configuration
    Advanced(GitHubAgentConfig),
}

/// Claude agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ClaudeConfig {
    /// Simple configuration (backward compatibility)
    Simple(bool),
    /// Detailed configuration
    Advanced(ClaudeAgentConfig),
}

/// Codex agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CodexConfig {
    /// Simple configuration (backward compatibility)
    Simple(bool),
    /// Detailed configuration
    Advanced(CodexAgentConfig),
}

/// Cursor agent detailed configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CursorAgentConfig {
    /// Agent enable/disable (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Output mode (optional, overrides global setting)
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
    /// Whether to include filename headers in merged mode (optional, overrides global setting)
    #[serde(default)]
    pub include_filenames: Option<bool>,
    /// Detailed settings for split mode (optional)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub split_config: Option<CursorSplitConfig>,
}

/// Cursor split mode configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CursorSplitConfig {
    /// Rule array
    #[serde(default)]
    pub rules: Vec<CursorSplitRule>,
}

/// GitHub split mode configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubSplitConfig {
    /// Rule array
    #[serde(default)]
    pub rules: Vec<GitHubSplitRule>,
}

/// Cursor split mode rule configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CursorSplitRule {
    /// Target Markdown filename patterns
    pub file_patterns: Vec<String>,
    /// For Always rule (alwaysApply: true)
    #[serde(default, rename = "alwaysApply")]
    pub always_apply: Option<bool>,
    /// For Auto Attached rule (globs setting)
    #[serde(default)]
    pub globs: Option<Vec<String>>,
    /// For Agent Requested rule (description setting)
    #[serde(default)]
    pub description: Option<String>,
    /// For Manual rule (manual: true)
    #[serde(default)]
    pub manual: Option<bool>,
}

/// GitHub split mode rule configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubSplitRule {
    /// Target Markdown filename patterns
    pub file_patterns: Vec<String>,
    /// File patterns for applyTo option (glob patterns)
    #[serde(default)]
    pub apply_to: Option<Vec<String>>,
}

/// Cline agent detailed configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClineAgentConfig {
    /// Agent enable/disable (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Output mode (optional, overrides global setting)
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
    /// Whether to include filename headers in merged mode (optional, overrides global setting)
    #[serde(default)]
    pub include_filenames: Option<bool>,
}

/// GitHub agent detailed configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubAgentConfig {
    /// Agent enable/disable (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Output mode (optional, overrides global setting)
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
    /// Whether to include filename headers in merged mode (optional, overrides global setting)
    #[serde(default)]
    pub include_filenames: Option<bool>,
    /// Detailed settings for split mode (optional)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub split_config: Option<GitHubSplitConfig>,
}

/// Claude agent detailed configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaudeAgentConfig {
    /// Agent enable/disable (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Output mode (optional, Claude is always merged)
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
    /// Whether to include filename headers in merged mode (optional, overrides global setting)
    #[serde(default)]
    pub include_filenames: Option<bool>,
}

/// Codex agent detailed configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodexAgentConfig {
    /// Agent enable/disable (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Output mode (optional, Codex is always merged)
    #[serde(default)]
    pub output_mode: Option<OutputMode>,
    /// Whether to include filename headers in merged mode (optional, overrides global setting)
    #[serde(default)]
    pub include_filenames: Option<bool>,
}

/// Default value: true
fn default_true() -> bool {
    true
}

/// Default agent configurations
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

impl Default for CodexConfig {
    fn default() -> Self {
        Self::Simple(false)
    }
}

impl Default for AIContextConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            output_mode: Some(OutputMode::Merged), // Default is merged
            include_filenames: Some(false),        // Default is false
            base_docs_dir: "./ai-docs".to_string(),
            agents: AgentConfig::default(),
        }
    }
}

impl AIContextConfig {
    /// Get list of enabled agents
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
        if self.agents.codex.is_enabled() {
            agents.push("codex".to_string());
        }
        agents
    }

    /// Get global output mode (default: merged)
    pub fn get_global_output_mode(&self) -> OutputMode {
        self.output_mode.clone().unwrap_or(OutputMode::Merged)
    }

    /// Get effective output mode for specified agent
    /// Priority: agent individual setting > global setting > default (merged)
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
            "claude" => OutputMode::Merged, // Claude is always merged
            "codex" => OutputMode::Merged,  // Codex is always merged
            _ => self.get_global_output_mode(),
        }
    }

    /// Get effective include_filenames setting for specified agent
    /// Priority: agent individual setting > global setting > default (false)
    pub fn get_effective_include_filenames(&self, agent: &str) -> bool {
        match agent {
            "cursor" => self
                .agents
                .cursor
                .get_include_filenames()
                .unwrap_or_else(|| self.include_filenames.unwrap_or(false)),
            "cline" => self
                .agents
                .cline
                .get_include_filenames()
                .unwrap_or_else(|| self.include_filenames.unwrap_or(false)),
            "github" => self
                .agents
                .github
                .get_include_filenames()
                .unwrap_or_else(|| self.include_filenames.unwrap_or(false)),
            "claude" => self
                .agents
                .claude
                .get_include_filenames()
                .unwrap_or_else(|| self.include_filenames.unwrap_or(false)),
            "codex" => self
                .agents
                .codex
                .get_include_filenames()
                .unwrap_or_else(|| self.include_filenames.unwrap_or(false)),
            _ => self.include_filenames.unwrap_or(false),
        }
    }
}

/// Common trait for agent configurations
pub trait AgentConfigTrait {
    /// Get whether agent is enabled
    fn is_enabled(&self) -> bool;
    /// Get agent individual output mode
    fn get_output_mode(&self) -> Option<OutputMode>;
    /// Get agent individual include_filenames setting
    fn get_include_filenames(&self) -> Option<bool>;
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

    fn get_include_filenames(&self) -> Option<bool> {
        match self {
            Self::Simple(_) => None,
            Self::Advanced(config) => config.include_filenames,
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

    fn get_include_filenames(&self) -> Option<bool> {
        match self {
            Self::Simple(_) => None,
            Self::Advanced(config) => config.include_filenames,
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

    fn get_include_filenames(&self) -> Option<bool> {
        match self {
            Self::Simple(_) => None,
            Self::Advanced(config) => config.include_filenames,
        }
    }
}

impl GitHubConfig {
    /// Get detailed configuration
    pub fn get_advanced_config(&self) -> Option<&GitHubAgentConfig> {
        match self {
            Self::Simple(_) => None,
            Self::Advanced(config) => Some(config),
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

    fn get_include_filenames(&self) -> Option<bool> {
        match self {
            Self::Simple(_) => None,
            Self::Advanced(config) => config.include_filenames,
        }
    }
}

impl AgentConfigTrait for CodexConfig {
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

    fn get_include_filenames(&self) -> Option<bool> {
        match self {
            Self::Simple(_) => None,
            Self::Advanced(config) => config.include_filenames,
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
        assert_eq!(config.output_mode, Some(OutputMode::Merged));
        assert_eq!(config.include_filenames, Some(false));
        assert_eq!(config.get_global_output_mode(), OutputMode::Merged);
        assert_eq!(config.base_docs_dir, "./ai-docs");
        assert!(!config.agents.cursor.is_enabled());
        assert!(!config.agents.cline.is_enabled());
        assert!(!config.agents.github.is_enabled());
        assert!(!config.agents.claude.is_enabled());
        assert!(!config.agents.codex.is_enabled());
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
        config.agents.codex = CodexConfig::Simple(true);

        let enabled = config.enabled_agents();
        assert_eq!(enabled.len(), 3);
        assert!(enabled.contains(&"cursor".to_string()));
        assert!(enabled.contains(&"claude".to_string()));
        assert!(enabled.contains(&"codex".to_string()));
    }

    #[test]
    fn test_enabled_agents_advanced_config() {
        let mut config = AIContextConfig::default();
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: None,
        });
        config.agents.cline = ClineConfig::Advanced(ClineAgentConfig {
            enabled: false,
            output_mode: Some(OutputMode::Merged),
            include_filenames: None,
        });

        let enabled = config.enabled_agents();
        assert_eq!(enabled.len(), 1);
        assert!(enabled.contains(&"cursor".to_string()));
    }

    #[test]
    fn test_global_output_mode() {
        let mut config = AIContextConfig::default();

        // Default (None) case uses Merged
        assert_eq!(config.get_global_output_mode(), OutputMode::Merged);

        // Explicitly set
        config.output_mode = Some(OutputMode::Split);
        assert_eq!(config.get_global_output_mode(), OutputMode::Split);
    }

    #[test]
    fn test_effective_output_mode_global_fallback() {
        let mut config = AIContextConfig {
            output_mode: Some(OutputMode::Split),
            ..Default::default()
        };
        config.agents.cursor = CursorConfig::Simple(true);

        // No agent individual settings, use global setting
        assert_eq!(
            config.get_effective_output_mode("cursor"),
            OutputMode::Split
        );
    }

    #[test]
    fn test_effective_output_mode_agent_override() {
        let mut config = AIContextConfig {
            output_mode: Some(OutputMode::Split),
            ..Default::default()
        };
        config.agents.cursor = CursorConfig::Advanced(CursorAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Merged),
            split_config: None,
        });

        // Agent individual settings override global setting
        assert_eq!(
            config.get_effective_output_mode("cursor"),
            OutputMode::Merged
        );
    }

    #[test]
    fn test_effective_output_mode_claude_always_merged() {
        let mut config = AIContextConfig {
            output_mode: Some(OutputMode::Split),
            ..Default::default()
        };
        config.agents.claude = ClaudeConfig::Advanced(ClaudeAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split), // Set but ignored
        });

        // Claude is always merged
        assert_eq!(
            config.get_effective_output_mode("claude"),
            OutputMode::Merged
        );
    }

    #[test]
    fn test_effective_output_mode_codex_always_merged() {
        let mut config = AIContextConfig {
            output_mode: Some(OutputMode::Split),
            ..Default::default()
        };
        config.agents.codex = CodexConfig::Advanced(CodexAgentConfig {
            enabled: true,
            include_filenames: None,
            output_mode: Some(OutputMode::Split), // Set but ignored
        });

        // Codex is always merged
        assert_eq!(
            config.get_effective_output_mode("codex"),
            OutputMode::Merged
        );
    }

    #[test]
    fn test_effective_output_mode_default_fallback() {
        let config = AIContextConfig::default();

        // No global setting or agent individual settings → Default (merged)
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
            include_filenames: None,
            output_mode: Some(OutputMode::Split),
            split_config: None,
        });

        let yaml = serde_yaml::to_string(&cursor_config).unwrap();
        let deserialized: CursorConfig = serde_yaml::from_str(&yaml).unwrap();

        assert!(deserialized.is_enabled());
        assert_eq!(deserialized.get_output_mode(), Some(OutputMode::Split));

        // split_config: null not output
        assert!(!yaml.contains("split_config"));
    }

    #[test]
    fn test_backward_compatibility_parsing() {
        // Parse existing configuration format
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

        // Backward compatibility: No agent individual settings, use global setting
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
        // Parse new configuration format
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
        assert!(config.agents.github.is_enabled()); // enabled default is true
        assert!(!config.agents.claude.is_enabled());

        // Effective output mode confirmation
        assert_eq!(
            config.get_effective_output_mode("cursor"),
            OutputMode::Split
        ); // Global setting
        assert_eq!(
            config.get_effective_output_mode("cline"),
            OutputMode::Merged
        ); // Individual setting
        assert_eq!(
            config.get_effective_output_mode("github"),
            OutputMode::Split
        ); // Individual setting
        assert_eq!(
            config.get_effective_output_mode("claude"),
            OutputMode::Merged
        ); // Always merged
    }

    #[test]
    fn test_mixed_format_parsing() {
        // Parse mixed format
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

        // Global output_mode not present, use default (merged)
        assert_eq!(config.get_global_output_mode(), OutputMode::Merged);

        assert!(config.agents.cursor.is_enabled());
        assert!(!config.agents.cline.is_enabled());
        assert!(config.agents.github.is_enabled());
        assert!(config.agents.claude.is_enabled());

        // Effective output mode confirmation
        assert_eq!(
            config.get_effective_output_mode("cursor"),
            OutputMode::Merged
        ); // Global default
        assert_eq!(
            config.get_effective_output_mode("github"),
            OutputMode::Split
        ); // Individual setting
        assert_eq!(
            config.get_effective_output_mode("claude"),
            OutputMode::Merged
        ); // Always merged
    }
}
