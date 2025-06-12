/*!
 * AI Context Management Tool - Integration Tests (Simplified)
 *
 * Simplified CLI command integration tests
 */

use std::process::Command;
use tempfile::tempdir;

// Test helper: Common function to build and execute binary
fn run_aicm_command(args: &[&str], working_dir: Option<&std::path::Path>) -> std::process::Output {
    let current_dir = std::env::current_dir().unwrap();

    // Build binary
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
    assert!(stdout.contains("Path to configuration file"));
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
    // Verify that error message contains file not found indication
    assert!(stderr.contains("nonexistent") || stderr.contains("FileNotFound"));
}

#[test]
fn test_cli_generate_with_custom_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("custom.yaml");
    let docs_path = temp_dir.path().join("docs");

    // Create custom config file (only claude enabled, cline disabled to avoid file deletion)
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

    // Create docs directory
    std::fs::create_dir_all(&docs_path).unwrap();
    std::fs::write(docs_path.join("test.md"), "# Test content").unwrap();

    // Get current working directory (project root) - only used within helper function
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

    assert!(stdout.contains("Generating context files"));
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
    assert!(stdout.contains("Path to configuration file"));
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
    // Verify that error message contains file not found indication
    assert!(stderr.contains("Configuration validation error"));
}

#[test]
fn test_cli_validate_with_custom_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("validate-custom.yaml");
    let docs_path = temp_dir.path().join("docs");

    // Create docs directory
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
    assert!(stdout.contains("Validating configuration file"));
    assert!(stdout.contains("validate-custom.yaml"));
    assert!(stdout.contains("Configuration file is valid"));
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
    assert!(stderr.contains("Documentation directory does not exist"));
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
    assert!(stderr.contains("Documentation directory does not exist"));
    assert!(stderr.contains("nonexistent-docs"));
}
