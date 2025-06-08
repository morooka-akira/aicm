/*!
 * AI Context Management Tool - Cursor Agent
 * 
 * このファイルはCursorエージェントの実装を提供します。
 * Cursor用の.mdcファイル生成ロジックを実装しています。
 */

use crate::agents::base::BaseAgentUtils;
use crate::types::{
    BaseAgent, AIContextConfig, CursorConfig, CursorRuleConfig, 
    GeneratedFile, SplitContent, ValidationResult, AgentInfo, CursorRuleType
};
use anyhow::Result;
use async_trait::async_trait;
use serde_yaml;
use std::collections::HashMap;

/// Cursorエージェントクラス
/// 
/// Cursor用の.mdcファイルを生成します。
/// 分割モードでは複数のルールファイルを、統合モードでは単一ファイルを生成します。
pub struct CursorAgent {
    config: AIContextConfig,
    cursor_config: CursorConfig,
}

impl CursorAgent {
    /// 新しいCursorエージェントを作成
    /// 
    /// # Arguments
    /// * `config` - 全体設定
    /// * `cursor_config` - Cursor固有設定
    pub fn new(config: AIContextConfig, cursor_config: CursorConfig) -> Self {
        Self {
            config,
            cursor_config,
        }
    }

    /// 分割ファイルを生成
    async fn generate_split_files(&self, split_content: &SplitContent) -> Result<Vec<GeneratedFile>> {
        let mut files = Vec::new();

        let split_config = self.cursor_config.split_config.as_ref()
            .ok_or_else(|| anyhow::anyhow!("split_configが設定されていません"))?;

        for (name, rule_config) in split_config {
            let content = self.select_content_for_rule(name, split_content);
            let frontmatter = self.create_frontmatter(rule_config);
            let file_content = self.create_mdc_file(&frontmatter, &content)?;

            files.push(GeneratedFile {
                path: BaseAgentUtils::normalize_path(format!(".cursor/rules/{}.mdc", name)),
                content: BaseAgentUtils::sanitize_content(&file_content),
                encoding: "utf8".to_string(),
            });
        }

        Ok(files)
    }

    /// 統合ファイルを生成
    async fn generate_merged_file(&self, merged_content: &str) -> Result<Vec<GeneratedFile>> {
        // デフォルトのfrontmatter設定
        let default_frontmatter = HashMap::from([
            ("description".to_string(), serde_yaml::Value::String("AI Context Management generated rules".to_string())),
            ("alwaysApply".to_string(), serde_yaml::Value::Bool(true)),
        ]);

        let file_content = self.create_mdc_file(&default_frontmatter, merged_content)?;

        Ok(vec![GeneratedFile {
            path: BaseAgentUtils::normalize_path(".cursor/rules/context.mdc"),
            content: BaseAgentUtils::sanitize_content(&file_content),
            encoding: "utf8".to_string(),
        }])
    }

    /// ルール設定に応じてコンテンツを選択
    fn select_content_for_rule(&self, rule_name: &str, split_content: &SplitContent) -> String {
        // ルール名に基づいてコンテンツを選択
        // common, project, agent などのキーワードでコンテンツを振り分け
        if rule_name.contains("common") {
            split_content.common.clone()
        } else if rule_name.contains("project") {
            split_content.project_specific.clone()
        } else if rule_name.contains("agent") || rule_name.contains("cursor") {
            split_content.agent_specific.clone()
        } else {
            // デフォルトは共通コンテンツ
            split_content.common.clone()
        }
    }

    /// frontmatterオブジェクトを作成
    fn create_frontmatter(&self, rule_config: &CursorRuleConfig) -> HashMap<String, serde_yaml::Value> {
        let mut frontmatter = HashMap::new();
        
        frontmatter.insert(
            "description".to_string(),
            serde_yaml::Value::String(rule_config.description.clone()),
        );
        
        let always_apply = matches!(rule_config.rule_type, CursorRuleType::Always) 
            || rule_config.always_apply.unwrap_or(false);
        frontmatter.insert(
            "alwaysApply".to_string(),
            serde_yaml::Value::Bool(always_apply),
        );

        // globsが設定されている場合は追加
        if let Some(globs) = &rule_config.globs {
            if !globs.is_empty() {
                let globs_value: Vec<serde_yaml::Value> = globs
                    .iter()
                    .map(|g| serde_yaml::Value::String(g.clone()))
                    .collect();
                frontmatter.insert(
                    "globs".to_string(),
                    serde_yaml::Value::Sequence(globs_value),
                );
            }
        }

        frontmatter
    }

    /// .mdcファイルの内容を作成
    fn create_mdc_file(
        &self,
        frontmatter: &HashMap<String, serde_yaml::Value>,
        content: &str,
    ) -> Result<String> {
        let yaml_frontmatter = serde_yaml::to_string(frontmatter)?;
        Ok(format!("---\n{}---\n\n{}", yaml_frontmatter, content))
    }

    /// 個別ルール設定を検証
    fn validate_rule_config(&self, name: &str, rule_config: &CursorRuleConfig) -> Vec<String> {
        let mut errors = Vec::new();

        if !BaseAgentUtils::is_valid_str(&rule_config.description) {
            errors.push(format!("ルール \"{}\" の description が設定されていません", name));
        }

        // typeの検証は型システムで保証されているため不要

        if let Some(globs) = &rule_config.globs {
            for glob in globs {
                if !BaseAgentUtils::is_valid_str(glob) {
                    errors.push(format!("ルール \"{}\" の globs に空の文字列が含まれています", name));
                }
            }
        }

        errors
    }
}

#[async_trait]
impl BaseAgent for CursorAgent {
    /// エージェント情報を取得
    fn get_info(&self) -> AgentInfo {
        AgentInfo {
            name: "cursor".to_string(),
            description: "Cursor AI Editor用のルールファイル生成エージェント".to_string(),
            output_patterns: vec![".cursor/rules/*.mdc".to_string()],
            supports_split: true,
        }
    }

    /// ファイルを生成
    async fn generate_files(
        &self,
        merged_content: &str,
        split_content: &SplitContent,
    ) -> Result<Vec<GeneratedFile>> {
        if BaseAgentUtils::is_split_mode(&self.config) && self.cursor_config.split_config.is_some() {
            self.generate_split_files(split_content).await
        } else {
            self.generate_merged_file(merged_content).await
        }
    }

    /// 出力予定のパスを取得
    fn get_output_paths(&self) -> Vec<String> {
        if BaseAgentUtils::is_split_mode(&self.config) {
            if let Some(split_config) = &self.cursor_config.split_config {
                return split_config
                    .keys()
                    .map(|name| BaseAgentUtils::normalize_path(format!(".cursor/rules/{}.mdc", name)))
                    .collect();
            }
        }

        vec![BaseAgentUtils::normalize_path(".cursor/rules/context.mdc")]
    }

    /// 設定を検証
    fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 分割モード時の設定チェック
        if BaseAgentUtils::is_split_mode(&self.config) {
            if self.cursor_config.split_config.is_none() {
                warnings.push(
                    "分割モードが指定されていますが、split_configが設定されていません。統合モードで出力されます。".to_string()
                );
            } else if let Some(split_config) = &self.cursor_config.split_config {
                // 各ルール設定の検証
                for (name, rule_config) in split_config {
                    let rule_errors = self.validate_rule_config(name, rule_config);
                    errors.extend(rule_errors);
                }
            }
        }

        BaseAgentUtils::create_validation_result(errors, Some(warnings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{OutputMode, FileMapping, SplitContent, AgentConfigs};
    use std::collections::HashMap;

    fn create_test_config() -> AIContextConfig {
        AIContextConfig {
            version: "1.0".to_string(),
            output_mode: OutputMode::Merged,
            base_docs_dir: "./docs".to_string(),
            agents: AgentConfigs::default(),
            file_mapping: FileMapping {
                common: vec!["common.md".to_string()],
                project_specific: vec!["project.md".to_string()],
                agent_specific: None,
            },
            global_variables: HashMap::new(),
        }
    }

    fn create_test_cursor_config() -> CursorConfig {
        let mut split_config = HashMap::new();
        split_config.insert(
            "common".to_string(),
            CursorRuleConfig {
                description: "Common rules".to_string(),
                rule_type: CursorRuleType::Always,
                globs: Some(vec!["**/*.rs".to_string(), "**/*.md".to_string()]),
                always_apply: Some(true),
            },
        );
        split_config.insert(
            "project".to_string(),
            CursorRuleConfig {
                description: "Project specific rules".to_string(),
                rule_type: CursorRuleType::AutoAttached,
                globs: Some(vec!["src/**/*".to_string()]),
                always_apply: None,
            },
        );

        CursorConfig {
            split_config: Some(split_config),
            additional_instructions: None,
        }
    }

    fn create_test_split_content() -> SplitContent {
        SplitContent {
            common: "# Common\nCommon content for all projects.".to_string(),
            project_specific: "# Project\nProject specific information.".to_string(),
            agent_specific: "# Cursor\nCursor specific rules and guidelines.".to_string(),
        }
    }

    #[test]
    fn test_cursor_agent_new() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config.clone(), cursor_config.clone());
        
        assert_eq!(agent.config.version, config.version);
        assert!(agent.cursor_config.split_config.is_some());
    }

    #[test]
    fn test_get_info() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        
        let info = agent.get_info();
        assert_eq!(info.name, "cursor");
        assert!(info.description.contains("Cursor"));
        assert!(info.supports_split);
        assert!(!info.output_patterns.is_empty());
    }

    #[tokio::test]
    async fn test_generate_merged_file() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        
        let merged_content = "# Test\nThis is test content.";
        let result = agent.generate_merged_file(merged_content).await;
        
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        
        let file = &files[0];
        assert!(file.path.contains(".cursor/rules/context.mdc"));
        assert!(file.content.contains("---"));  // YAML frontmatter
        assert!(file.content.contains("This is test content"));
        assert_eq!(file.encoding, "utf8");
    }

    #[tokio::test]
    async fn test_generate_split_files() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        
        let split_content = create_test_split_content();
        let result = agent.generate_split_files(&split_content).await;
        
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 2);  // common and project rules
        
        // ファイルパスの確認
        let file_paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();
        assert!(file_paths.iter().any(|p| p.contains("common.mdc")));
        assert!(file_paths.iter().any(|p| p.contains("project.mdc")));
        
        // コンテンツの確認
        for file in &files {
            assert!(file.content.contains("---"));  // YAML frontmatter
            assert!(!file.content.trim().is_empty());
        }
    }

    #[tokio::test]
    async fn test_generate_files_merged_mode() {
        let config = create_test_config();
        let cursor_config = CursorConfig { 
            split_config: None,
            additional_instructions: None,
        };
        let agent = CursorAgent::new(config, cursor_config);
        
        let merged_content = "# Test\nMerged content";
        let split_content = create_test_split_content();
        
        let result = agent.generate_files(merged_content, &split_content).await;
        assert!(result.is_ok());
        
        let files = result.unwrap();
        assert_eq!(files.len(), 1);  // 統合モードなので1つのファイル
        assert!(files[0].path.contains("context.mdc"));
    }

    #[tokio::test]
    async fn test_generate_files_split_mode() {
        let mut config = create_test_config();
        config.output_mode = OutputMode::Split;
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        
        let merged_content = "# Test\nMerged content";
        let split_content = create_test_split_content();
        
        let result = agent.generate_files(merged_content, &split_content).await;
        assert!(result.is_ok());
        
        let files = result.unwrap();
        assert!(files.len() > 1);  // 分割モードなので複数ファイル
    }

    #[test]
    fn test_select_content_for_rule() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        let split_content = create_test_split_content();
        
        // common関連のルール
        let common_content = agent.select_content_for_rule("common", &split_content);
        assert!(common_content.contains("Common content"));
        
        // project関連のルール
        let project_content = agent.select_content_for_rule("project", &split_content);
        assert!(project_content.contains("Project specific"));
        
        // agent/cursor関連のルール
        let agent_content = agent.select_content_for_rule("cursor", &split_content);
        assert!(agent_content.contains("Cursor specific"));
        
        // その他（デフォルト）
        let default_content = agent.select_content_for_rule("unknown", &split_content);
        assert_eq!(default_content, split_content.common);
    }

    #[test]
    fn test_create_frontmatter() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        
        let rule_config = CursorRuleConfig {
            description: "Test rule".to_string(),
            rule_type: CursorRuleType::Always,
            globs: Some(vec!["*.rs".to_string()]),
            always_apply: Some(true),
        };
        
        let frontmatter = agent.create_frontmatter(&rule_config);
        
        // 必須フィールドの確認
        assert!(frontmatter.contains_key("description"));
        assert!(frontmatter.contains_key("alwaysApply"));
        assert!(frontmatter.contains_key("globs"));
        
        // 値の確認
        if let Some(serde_yaml::Value::String(desc)) = frontmatter.get("description") {
            assert_eq!(desc, "Test rule");
        } else {
            panic!("description field should be a string");
        }
        
        if let Some(serde_yaml::Value::Bool(always_apply)) = frontmatter.get("alwaysApply") {
            assert!(always_apply);
        } else {
            panic!("alwaysApply field should be a boolean");
        }
    }

    #[test]
    fn test_create_mdc_file() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        
        let mut frontmatter = HashMap::new();
        frontmatter.insert(
            "description".to_string(),
            serde_yaml::Value::String("Test description".to_string()),
        );
        frontmatter.insert(
            "alwaysApply".to_string(),
            serde_yaml::Value::Bool(true),
        );
        
        let content = "# Test Content\nThis is test content.";
        let result = agent.create_mdc_file(&frontmatter, content);
        
        assert!(result.is_ok());
        let mdc_content = result.unwrap();
        
        // YAML frontmatterの存在確認
        assert!(mdc_content.starts_with("---\n"));
        assert!(mdc_content.contains("---\n\n"));  // frontmatterの終了
        assert!(mdc_content.contains("This is test content"));
        assert!(mdc_content.contains("description: Test description"));
        assert!(mdc_content.contains("alwaysApply: true"));
    }

    #[test]
    fn test_get_output_paths_merged_mode() {
        let config = create_test_config();
        let cursor_config = CursorConfig { 
            split_config: None,
            additional_instructions: None,
        };
        let agent = CursorAgent::new(config, cursor_config);
        
        let paths = agent.get_output_paths();
        assert_eq!(paths.len(), 1);
        assert!(paths[0].contains("context.mdc"));
    }

    #[test]
    fn test_get_output_paths_split_mode() {
        let mut config = create_test_config();
        config.output_mode = OutputMode::Split;
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        
        let paths = agent.get_output_paths();
        assert!(paths.len() > 1);  // 複数のパス
        assert!(paths.iter().any(|p| p.contains("common.mdc")));
        assert!(paths.iter().any(|p| p.contains("project.mdc")));
    }

    #[test]
    fn test_validate_successful() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        
        let result = agent.validate();
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_split_mode_without_config() {
        let mut config = create_test_config();
        config.output_mode = OutputMode::Split;
        let cursor_config = CursorConfig { 
            split_config: None,
            additional_instructions: None,
        };
        let agent = CursorAgent::new(config, cursor_config);
        
        let result = agent.validate();
        assert!(result.valid);  // 警告はあるがエラーではない
        assert!(!result.warnings.is_empty());
        assert!(result.warnings[0].contains("split_configが設定されていません"));
    }

    #[test]
    fn test_validate_rule_config() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        
        // 有効な設定
        let valid_rule = CursorRuleConfig {
            description: "Valid rule".to_string(),
            rule_type: CursorRuleType::Always,
            globs: Some(vec!["*.rs".to_string()]),
            always_apply: Some(true),
        };
        let errors = agent.validate_rule_config("test", &valid_rule);
        assert!(errors.is_empty());
        
        // 無効な設定（空の説明）
        let invalid_rule = CursorRuleConfig {
            description: "".to_string(),
            rule_type: CursorRuleType::Always,
            globs: Some(vec!["".to_string()]),  // 空のglob
            always_apply: Some(true),
        };
        let errors = agent.validate_rule_config("test", &invalid_rule);
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("description が設定されていません")));
        assert!(errors.iter().any(|e| e.contains("空の文字列が含まれています")));
    }

    #[test]
    fn test_frontmatter_with_auto_attached_type() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);
        
        let rule_config = CursorRuleConfig {
            description: "Auto attached rule".to_string(),
            rule_type: CursorRuleType::AutoAttached,
            globs: None,
            always_apply: None,
        };
        
        let frontmatter = agent.create_frontmatter(&rule_config);
        
        // AutoAttachedの場合、alwaysApplyはfalseになるべき
        if let Some(serde_yaml::Value::Bool(always_apply)) = frontmatter.get("alwaysApply") {
            assert!(!always_apply);
        } else {
            panic!("alwaysApply field should be a boolean");
        }
        
        // globsが設定されていない場合は含まれない
        assert!(!frontmatter.contains_key("globs"));
    }

    #[tokio::test]
    async fn test_generate_split_files_missing_config() {
        let config = create_test_config();
        let cursor_config = CursorConfig { 
            split_config: None,
            additional_instructions: None,
        };
        let agent = CursorAgent::new(config, cursor_config);
        
        let split_content = create_test_split_content();
        let result = agent.generate_split_files(&split_content).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("split_configが設定されていません"));
    }
}