#[cfg(test)]
mod include_filenames_tests {
    use super::super::*;
    use crate::types::{AIContextConfig, AgentConfig, ClaudeAgentConfig, ClaudeConfig, OutputMode};
    use tempfile::tempdir;
    use tokio::fs;

    /// include_filenames設定用のテストconfig作成ヘルパー
    fn create_test_config_with_include_filenames(
        base_dir: &str,
        global_include_filenames: Option<bool>,
        agent_include_filenames: Option<bool>,
    ) -> AIContextConfig {
        let claude_config = if let Some(include_filenames) = agent_include_filenames {
            ClaudeConfig::Advanced(ClaudeAgentConfig {
                enabled: true,
                output_mode: Some(OutputMode::Merged),
                include_filenames: Some(include_filenames),
            })
        } else {
            ClaudeConfig::Simple(true)
        };

        AIContextConfig {
            version: "1.0".to_string(),
            output_mode: Some(OutputMode::Merged),
            include_filenames: global_include_filenames,
            base_docs_dir: base_dir.to_string(),
            agents: AgentConfig {
                claude: claude_config,
                ..AgentConfig::default()
            },
        }
    }

    #[tokio::test]
    async fn test_merge_all_with_default_include_filenames() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test1.md"), "# Test 1\nContent 1")
            .await
            .unwrap();
        fs::write(docs_path.join("test2.md"), "# Test 2\nContent 2")
            .await
            .unwrap();

        let config = create_test_config_with_include_filenames(
            &docs_path.to_string_lossy(),
            None, // グローバル設定なし（デフォルト false）
            None, // エージェント個別設定なし
        );
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all_with_options(Some("claude")).await.unwrap();

        // デフォルト（false）の場合、ファイル名ヘッダーは含まれない
        assert!(!result.contains("# test1.md"));
        assert!(!result.contains("# test2.md"));
        assert!(result.contains("# Test 1"));
        assert!(result.contains("# Test 2"));
        assert!(result.contains("Content 1"));
        assert!(result.contains("Content 2"));
    }

    #[tokio::test]
    async fn test_merge_all_with_global_include_filenames_true() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test1.md"), "# Test 1\nContent 1")
            .await
            .unwrap();
        fs::write(docs_path.join("test2.md"), "# Test 2\nContent 2")
            .await
            .unwrap();

        let config = create_test_config_with_include_filenames(
            &docs_path.to_string_lossy(),
            Some(true), // グローバル設定: true
            None,       // エージェント個別設定なし
        );
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all_with_options(Some("claude")).await.unwrap();

        // include_filenames=trueの場合、ファイル名ヘッダーが含まれる
        assert!(result.contains("# test1.md"));
        assert!(result.contains("# test2.md"));
        assert!(result.contains("# Test 1"));
        assert!(result.contains("# Test 2"));
        assert!(result.contains("Content 1"));
        assert!(result.contains("Content 2"));
    }

    #[tokio::test]
    async fn test_merge_all_with_global_include_filenames_false() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test1.md"), "# Test 1\nContent 1")
            .await
            .unwrap();
        fs::write(docs_path.join("test2.md"), "# Test 2\nContent 2")
            .await
            .unwrap();

        let config = create_test_config_with_include_filenames(
            &docs_path.to_string_lossy(),
            Some(false), // グローバル設定: false
            None,        // エージェント個別設定なし
        );
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all_with_options(Some("claude")).await.unwrap();

        // include_filenames=falseの場合、ファイル名ヘッダーは含まれない
        assert!(!result.contains("# test1.md"));
        assert!(!result.contains("# test2.md"));
        assert!(result.contains("# Test 1"));
        assert!(result.contains("# Test 2"));
        assert!(result.contains("Content 1"));
        assert!(result.contains("Content 2"));
    }

    #[tokio::test]
    async fn test_merge_all_with_agent_override_include_filenames_true() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test1.md"), "# Test 1\nContent 1")
            .await
            .unwrap();
        fs::write(docs_path.join("test2.md"), "# Test 2\nContent 2")
            .await
            .unwrap();

        let config = create_test_config_with_include_filenames(
            &docs_path.to_string_lossy(),
            Some(false), // グローバル設定: false
            Some(true),  // エージェント個別設定: true（グローバル設定を上書き）
        );
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all_with_options(Some("claude")).await.unwrap();

        // エージェント個別設定が優先され、ファイル名ヘッダーが含まれる
        assert!(result.contains("# test1.md"));
        assert!(result.contains("# test2.md"));
        assert!(result.contains("# Test 1"));
        assert!(result.contains("# Test 2"));
        assert!(result.contains("Content 1"));
        assert!(result.contains("Content 2"));
    }

    #[tokio::test]
    async fn test_merge_all_with_agent_override_include_filenames_false() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test1.md"), "# Test 1\nContent 1")
            .await
            .unwrap();
        fs::write(docs_path.join("test2.md"), "# Test 2\nContent 2")
            .await
            .unwrap();

        let config = create_test_config_with_include_filenames(
            &docs_path.to_string_lossy(),
            Some(true),  // グローバル設定: true
            Some(false), // エージェント個別設定: false（グローバル設定を上書き）
        );
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all_with_options(Some("claude")).await.unwrap();

        // エージェント個別設定が優先され、ファイル名ヘッダーは含まれない
        assert!(!result.contains("# test1.md"));
        assert!(!result.contains("# test2.md"));
        assert!(result.contains("# Test 1"));
        assert!(result.contains("# Test 2"));
        assert!(result.contains("Content 1"));
        assert!(result.contains("Content 2"));
    }

    #[tokio::test]
    async fn test_merge_all_with_options_no_agent() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test.md"), "# Test\nContent")
            .await
            .unwrap();

        let config = create_test_config_with_include_filenames(
            &docs_path.to_string_lossy(),
            Some(true), // グローバル設定: true
            None,
        );
        let merger = MarkdownMerger::new(config);

        // エージェント名を指定しない場合はグローバル設定を使用
        let result = merger.merge_all_with_options(None).await.unwrap();

        // グローバル設定（true）が適用される
        assert!(result.contains("# test.md"));
        assert!(result.contains("# Test"));
        assert!(result.contains("Content"));
    }

    #[tokio::test]
    async fn test_merge_all_backward_compatibility() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // テスト用ファイルを作成
        fs::write(docs_path.join("test.md"), "# Test\nContent")
            .await
            .unwrap();

        let config = create_test_config_with_include_filenames(
            &docs_path.to_string_lossy(),
            Some(true), // グローバル設定: true
            None,
        );
        let merger = MarkdownMerger::new(config);

        // 従来のmerge_all()メソッドはデフォルト（include_filenames=trueの動作）
        let result = merger.merge_all().await.unwrap();

        // 下位互換性: 従来通りファイル名ヘッダーが含まれる
        assert!(result.contains("# test.md"));
        assert!(result.contains("# Test"));
        assert!(result.contains("Content"));
    }

    #[tokio::test]
    async fn test_merge_all_with_subdirectory_include_filenames() {
        let temp_dir = tempdir().unwrap();
        let docs_path = temp_dir.path();

        // サブディレクトリを作成
        let sub_dir = docs_path.join("subdir");
        fs::create_dir(&sub_dir).await.unwrap();
        fs::write(sub_dir.join("nested.md"), "# Nested\nNested content")
            .await
            .unwrap();

        let config = create_test_config_with_include_filenames(
            &docs_path.to_string_lossy(),
            Some(true), // グローバル設定: true
            None,
        );
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all_with_options(Some("claude")).await.unwrap();

        // サブディレクトリのファイル名も含まれることを確認
        assert!(result.contains("# subdir/nested.md") || result.contains("# subdir\\nested.md")); // Windows対応
        assert!(result.contains("# Nested"));
        assert!(result.contains("Nested content"));
    }

    #[tokio::test]
    async fn test_merge_all_empty_directory_include_filenames() {
        let temp_dir = tempdir().unwrap();

        let config = create_test_config_with_include_filenames(
            &temp_dir.path().to_string_lossy(),
            Some(true), // グローバル設定: true
            None,
        );
        let merger = MarkdownMerger::new(config);

        let result = merger.merge_all_with_options(Some("claude")).await.unwrap();

        // 空のディレクトリの場合は空文字列
        assert!(result.is_empty());
    }
}
