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