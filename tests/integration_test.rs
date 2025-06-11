/*!
 * AI Context Management Tool - Integration Tests (Simplified)
 *
 * シンプル化されたCLIコマンドの統合テスト
 */

use std::process::Command;
use tempfile::tempdir;

// テストヘルパー：バイナリをビルドして実行するための共通関数
fn run_aicm_command(args: &[&str], working_dir: Option<&std::path::Path>) -> std::process::Output {
    let current_dir = std::env::current_dir().unwrap();

    // バイナリをビルド
    let build_output = Command::new("cargo")
        .args(["build"])
        .current_dir(&current_dir)
        .output()
        .expect("Failed to build binary");

    assert!(build_output.status.success(), "Failed to build project");

    let binary_path = current_dir.join("target/debug/aicm");
    let mut command = Command::new(&binary_path);
    command.args(args);

    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }

    command.output().expect("Failed to execute command")
}

#[test]
fn test_cli_help_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("AI Context Management Tool"));
    assert!(stdout.contains("init"));
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("validate"));
}

#[test]
fn test_cli_version_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("aicm"));
}

#[test]
fn test_cli_generate_help_includes_config_option() {
    let output = Command::new("cargo")
        .args(["run", "--", "generate", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--config"));
    assert!(stdout.contains("-c"));
    assert!(stdout.contains("設定ファイルのパス"));
}

#[test]
fn test_cli_generate_with_nonexistent_config() {
    let temp_dir = tempdir().unwrap();
    let output = run_aicm_command(
        &["generate", "--config", "/nonexistent/config.yaml"],
        Some(temp_dir.path()),
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    // エラーメッセージにファイルが見つからない旨が含まれることを確認
    assert!(stderr.contains("nonexistent") || stderr.contains("FileNotFound"));
}

#[test]
fn test_cli_generate_with_custom_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("custom.yaml");
    let docs_path = temp_dir.path().join("docs");

    // カスタム設定ファイルを作成（claudeのみ有効でclineを無効にしてファイル削除を回避）
    let config_content = format!(
        r#"
version: "1.0"
output_mode: merged
base_docs_dir: "{}"
agents:
  claude: true
  cline: false
  cursor: false
  github: false
  codex: false
"#,
        docs_path.to_string_lossy()
    );

    std::fs::write(&config_path, config_content).unwrap();

    // docsディレクトリを作成
    std::fs::create_dir_all(&docs_path).unwrap();
    std::fs::write(docs_path.join("test.md"), "# Test content").unwrap();

    // 現在の作業ディレクトリを取得（プロジェクトルート） - ヘルパー関数内でのみ使用
    let _current_dir = std::env::current_dir().unwrap();

    let output = run_aicm_command(
        &["generate", "--config", &config_path.to_string_lossy()],
        Some(temp_dir.path()), // 一時ディレクトリで実行して出力ファイルを隔離
    );

    // 成功することを確認
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    if !output.status.success() {
        println!("STDOUT: {}", stdout);
        println!("STDERR: {}", stderr);
    }
    assert!(output.status.success());

    assert!(stdout.contains("コンテキストファイルを生成します"));
    assert!(stdout.contains("custom.yaml"));

    // 出力ファイルが一時ディレクトリに作成されることを確認
    let claude_md_path = temp_dir.path().join("CLAUDE.md");
    assert!(
        claude_md_path.exists(),
        "CLAUDE.md should be created in temp directory"
    );
}

#[test]
fn test_cli_validate_help_includes_config_option() {
    let output = Command::new("cargo")
        .args(["run", "--", "validate", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--config"));
    assert!(stdout.contains("-c"));
    assert!(stdout.contains("設定ファイルのパス"));
}

#[test]
fn test_cli_validate_with_nonexistent_config() {
    let temp_dir = tempdir().unwrap();
    let output = run_aicm_command(
        &["validate", "--config", "/nonexistent/config.yaml"],
        Some(temp_dir.path()),
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    // エラーメッセージにファイルが見つからない旨が含まれることを確認
    assert!(stderr.contains("設定ファイルの検証でエラーが発生しました"));
}

#[test]
fn test_cli_validate_with_custom_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("validate-custom.yaml");
    let docs_path = temp_dir.path().join("docs");

    // docsディレクトリを作成
    std::fs::create_dir_all(&docs_path).unwrap();

    // 有効な設定ファイルを作成（clineを無効にしてファイル削除を回避）
    let config_content = format!(
        r#"
version: "1.0"
output_mode: split
base_docs_dir: "{}"
agents:
  cursor: false
  claude: true
  cline: false
  github: false
  codex: false
"#,
        docs_path.to_string_lossy()
    );

    std::fs::write(&config_path, config_content).unwrap();

    let output = run_aicm_command(
        &["validate", "--config", &config_path.to_string_lossy()],
        Some(temp_dir.path()),
    );

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("設定ファイルを検証します"));
    assert!(stdout.contains("validate-custom.yaml"));
    assert!(stdout.contains("設定ファイルは有効です"));
}

#[test]
fn test_cli_generate_with_nonexistent_docs_dir() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("bad-config.yaml");
    let nonexistent_docs = temp_dir.path().join("nonexistent-docs");

    // 存在しないdocsディレクトリを指定した設定ファイルを作成
    let config_content = format!(
        r#"
version: "1.0"
output_mode: merged
base_docs_dir: "{}"
agents:
  claude: true
  cline: false
  cursor: false
  github: false
  codex: false
"#,
        nonexistent_docs.to_string_lossy()
    );

    std::fs::write(&config_path, config_content).unwrap();

    let output = run_aicm_command(
        &["generate", "--config", &config_path.to_string_lossy()],
        Some(temp_dir.path()),
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("ドキュメントディレクトリが存在しません"));
    assert!(stderr.contains("nonexistent-docs"));
}

#[test]
fn test_cli_validate_with_nonexistent_docs_dir() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("bad-config.yaml");
    let nonexistent_docs = temp_dir.path().join("nonexistent-docs");

    // 存在しないdocsディレクトリを指定した設定ファイルを作成
    let config_content = format!(
        r#"
version: "1.0"
output_mode: split
base_docs_dir: "{}"
agents:
  claude: true
  cline: false
  cursor: false
  github: false
  codex: false
"#,
        nonexistent_docs.to_string_lossy()
    );

    std::fs::write(&config_path, config_content).unwrap();

    let output = run_aicm_command(
        &["validate", "--config", &config_path.to_string_lossy()],
        Some(temp_dir.path()),
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("ドキュメントディレクトリが存在しません"));
    assert!(stderr.contains("nonexistent-docs"));
}
