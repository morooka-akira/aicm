# aicm init テンプレート YAML 修正と validate エラー改善

## 作業概要

aicm init で作成されるテンプレートファイルに YAML バリデーションエラーがあったため、修正を実施。
また、validate コマンドでのエラー出力を詳細化し、ユーザビリティを向上。

## 問題

1. **aicm init テンプレートに version フィールドが抜けている**
   - ConfigLoader::create_default_template() で version フィールドが含まれていなかった
   - これにより aicm validate が失敗

2. **YAML パースエラーの詳細が不十分**
   - ConfigError::YamlError のエラーメッセージが "YAML file parsing error occurred" のみ
   - 行番号やエラーの詳細が表示されない

## 設計方針

- テンプレートファイルには必須フィールドを全て含める
- エラーメッセージは具体的で問題解決に役立つ情報を提供
- 既存のテストとの互換性を保つ

## 完了要件

- [x] aicm init で作成されるテンプレートに version フィールドを追加
- [x] YAML パースエラーで行番号と詳細を表示
- [x] 全テストが通ること
- [x] cargo fmt, cargo clippy でエラーが出ないこと

## 実装内容

### 1. テンプレート修正

**ファイル**: `src/config/loader.rs`

```rust
// 修正前
"output_mode: merged",

// 修正後  
"# Configuration file version",
"version: \"1.0\"",
"",
"# Global output mode for all agents (default: merged)",
"# - merged: Combine all markdown files into one file per agent", 
"# - split: Create separate files for each markdown file",
"output_mode: merged",
```

### 2. エラーメッセージ改善

**ファイル**: `src/config/error.rs`

```rust
// 修正前
#[error("YAML file parsing error occurred")]

// 修正後
#[error("YAML parsing error: {source}")]
```

### 3. テスト修正

エラーメッセージの変更に伴い、対応するテストも更新:

```rust
// 修正前
assert!(config_error.to_string().contains("YAML file parsing error occurred"));

// 修正後
assert!(config_error.to_string().contains("YAML parsing error"));
```

## 検証結果

### 修正前

```bash
$ cargo run -- init
$ cargo run -- validate
❌ Configuration validation error: YAML file parsing error occurred
```

### 修正後

```bash
$ cargo run -- init
✅ Created aicm-config.yml

$ cargo run -- validate  
✅ Configuration file is valid
  Version: 1.0
  Output mode: Some(Merged)
  Documentation directory: ./ai-docs (exists)
  Enabled agents: cursor, cline, github, claude, codex

# 無効なYAMLでの詳細エラー表示
$ echo "invalid: yaml: [" > test.yml && cargo run -- validate --config test.yml
❌ Configuration validation error: YAML parsing error: mapping values are not allowed in this context at line 1 column 14
```

## テスト結果

```bash
$ cargo test
test result: ok. 126 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo fmt && cargo clippy
# エラーなし
```

## 影響範囲

- テンプレート生成のみ（既存の設定ファイルには影響なし）
- エラーメッセージの改善（機能的な変更なし）
- 後方互換性を保持

## 追加改善点

今後検討できる改善:
- より具体的なバリデーションエラーメッセージ（どのフィールドが問題かなど）
- YAML スキーマバリデーション
- 設定ファイルの自動修正提案