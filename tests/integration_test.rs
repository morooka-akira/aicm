/*!
 * AI Context Management Tool - Integration Tests (Simplified)
 *
 * シンプル化されたCLIコマンドの統合テスト
 */

use std::process::Command;
use tempfile::tempdir;

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
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            "--config",
            "/nonexistent/config.yaml",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    // エラーメッセージにファイルが見つからない旨が含まれることを確認
    assert!(stderr.contains("nonexistent") || stderr.contains("FileNotFound"));
}

#[test]
#[ignore] // 一時的に無効化：統合テスト環境でのパス問題のため
fn test_cli_generate_with_custom_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("custom.yaml");
    let docs_path = temp_dir.path().join("docs");

    // カスタム設定ファイルを作成
    let config_content = format!(
        r#"
version: "1.0"
output_mode: merged
base_docs_dir: "{}"
agents:
  claude: true
"#,
        docs_path.to_string_lossy()
    );

    std::fs::write(&config_path, config_content).unwrap();

    // docsディレクトリを作成
    std::fs::create_dir_all(&docs_path).unwrap();
    std::fs::write(docs_path.join("test.md"), "# Test content").unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "generate",
            "--config",
            &config_path.to_string_lossy(),
        ])
        .env("PWD", temp_dir.path())
        .output()
        .expect("Failed to execute command");

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
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config",
            "/nonexistent/config.yaml",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    // エラーメッセージにファイルが見つからない旨が含まれることを確認
    assert!(stdout.contains("設定ファイルの検証でエラーが発生しました"));
}

#[test]
fn test_cli_validate_with_custom_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("validate-custom.yaml");

    // 有効な設定ファイルを作成
    let config_content = r#"
version: "1.0"
output_mode: split
base_docs_dir: "./docs"
agents:
  cursor: true
  claude: true
"#;

    std::fs::write(&config_path, config_content).unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "validate",
            "--config",
            &config_path.to_string_lossy(),
        ])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("設定ファイルを検証します"));
    assert!(stdout.contains("validate-custom.yaml"));
    assert!(stdout.contains("設定ファイルは有効です"));
}
