/*!
 * AI Context Management Tool - Integration Tests (Simplified)
 *
 * シンプル化されたCLIコマンドの統合テスト
 */

use std::process::Command;

#[test]
fn test_cli_help_command() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
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
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("aicm"));
}
