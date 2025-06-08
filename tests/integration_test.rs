/*!
 * AI Context Management Tool - Integration Tests
 * 
 * このファイルはエンドツーエンド統合テストを提供します。
 * 実際のCLIコマンドの動作を検証します。
 * 
 * NOTE: 現在はCLI実装が部分的なため、基本的なテストのみ実行します。
 */

use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_cli_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("AI Code Agent Context Management"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("validate"));
    assert!(stdout.contains("list-agents"));
}

#[test]
fn test_cli_version_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("aicm"));
}

#[test]
fn test_list_agents_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "list-agents"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("cursor"));
    assert!(stdout.contains("Cursor"));
}

#[test]
#[ignore = "CLI implementation incomplete - skipping until init command is fully implemented"]
fn test_init_command_creates_directory_structure() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    let output = Command::new("cargo")
        .args(&["run", "--", "init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());

    // ディレクトリ構造の確認
    assert!(temp_path.join("docs").exists());
    assert!(temp_path.join("docs/common").exists());
    assert!(temp_path.join("docs/agents").exists());
    assert!(temp_path.join("ai-context.yaml").exists());

    // ファイル内容の確認
    let config_content = fs::read_to_string(temp_path.join("ai-context.yaml")).unwrap();
    assert!(config_content.contains("version:"));
    assert!(config_content.contains("output_mode:"));
    assert!(config_content.contains("base_docs_dir:"));
}

#[test]
#[ignore = "CLI implementation incomplete - validate command not fully implemented"]
fn test_validate_command_with_missing_config() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    let output = Command::new("cargo")
        .args(&["run", "--", "validate"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute command");

    // 設定ファイルが存在しない場合はエラーになるべき
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("見つかりません") || stderr.contains("not found"));
}

#[test]
#[ignore = "CLI implementation incomplete - generate command not fully implemented"]
fn test_generate_command_with_missing_config() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    let output = Command::new("cargo")
        .args(&["run", "--", "generate"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute command");

    // 設定ファイルが存在しない場合はエラーになるべき
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("見つかりません") || stderr.contains("not found"));
}

#[test]
#[ignore = "CLI implementation incomplete - full workflow not implemented"]
fn test_full_workflow_init_validate_generate() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // 1. 初期化
    let init_output = Command::new("cargo")
        .args(&["run", "--", "init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute init command");
    assert!(init_output.status.success());

    // 2. テスト用ファイルを作成
    fs::write(
        temp_path.join("docs/common/overview.md"),
        "# Project Overview\nThis is a test project.",
    ).unwrap();

    fs::write(
        temp_path.join("docs/agents/cursor.md"),
        "# Cursor Rules\nSpecific rules for Cursor.",
    ).unwrap();

    // 3. 検証
    let validate_output = Command::new("cargo")
        .args(&["run", "--", "validate"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute validate command");
    assert!(validate_output.status.success());

    // 4. 生成
    let generate_output = Command::new("cargo")
        .args(&["run", "--", "generate"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute generate command");
    assert!(generate_output.status.success());

    // 5. 生成されたファイルの確認
    assert!(temp_path.join(".cursor").exists());
    assert!(temp_path.join(".cursor/rules").exists());
    assert!(temp_path.join(".cursor/rules/context.mdc").exists());

    let generated_content = fs::read_to_string(temp_path.join(".cursor/rules/context.mdc")).unwrap();
    assert!(generated_content.contains("---"));  // YAML frontmatter
    assert!(generated_content.contains("Project Overview"));
    assert!(generated_content.contains("Cursor Rules"));
}

#[test]
#[ignore = "CLI implementation incomplete - agent selection not implemented"]
fn test_generate_with_specific_agent() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // 初期化
    let init_output = Command::new("cargo")
        .args(&["run", "--", "init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute init command");
    assert!(init_output.status.success());

    // テスト用ファイルを作成
    fs::write(
        temp_path.join("docs/common/overview.md"),
        "# Test Content",
    ).unwrap();

    // 特定エージェントでの生成
    let generate_output = Command::new("cargo")
        .args(&["run", "--", "generate", "--agent", "cursor"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute generate command");
    assert!(generate_output.status.success());

    // Cursorファイルのみ生成されることを確認
    assert!(temp_path.join(".cursor/rules/context.mdc").exists());
}

#[test]
#[ignore = "CLI implementation incomplete - dry-run option not implemented"]
fn test_generate_dry_run() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // 初期化
    let init_output = Command::new("cargo")
        .args(&["run", "--", "init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute init command");
    assert!(init_output.status.success());

    // テスト用ファイルを作成
    fs::write(
        temp_path.join("docs/common/overview.md"),
        "# Test Content",
    ).unwrap();

    // ドライラン実行
    let dry_run_output = Command::new("cargo")
        .args(&["run", "--", "generate", "--dry-run"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute dry-run command");
    assert!(dry_run_output.status.success());

    // ファイルが実際には生成されていないことを確認
    assert!(!temp_path.join(".cursor/rules/context.mdc").exists());

    // でも出力にはプレビューが表示されているはず
    let stdout = String::from_utf8(dry_run_output.stdout).unwrap();
    assert!(stdout.contains("cursor") || stdout.contains("Preview") || stdout.contains("dry"));
}

#[test]
#[ignore = "CLI implementation incomplete - agent validation not implemented"]
fn test_invalid_agent_name() {
    let temp_dir = tempdir().unwrap();
    let temp_path = temp_dir.path();

    // 初期化
    let init_output = Command::new("cargo")
        .args(&["run", "--", "init"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute init command");
    assert!(init_output.status.success());

    // 無効なエージェント名での生成
    let generate_output = Command::new("cargo")
        .args(&["run", "--", "generate", "--agent", "invalid_agent"])
        .current_dir(temp_path)
        .output()
        .expect("Failed to execute generate command");

    // エラーになるべき
    assert!(!generate_output.status.success());
    let stderr = String::from_utf8(generate_output.stderr).unwrap();
    assert!(stderr.contains("invalid") || stderr.contains("Unknown") || stderr.contains("サポートされていない"));
}