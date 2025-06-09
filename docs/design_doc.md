# Design Document - AI Code Agent Context Management Tool (Rust Edition)

## 技術仕様書

### プロジェクト概要

Cargo パッケージとして配布する、AI コーディングエージェント用 context ファイル生成 CLI ツール

## アーキテクチャ設計

### 技術スタック

- **言語**: Rust (Edition 2021)
- **CLI Framework**: clap v4 (derive API)
- **設定**: YAML (serde_yaml)
- **非同期処理**: Tokio
- **エラーハンドリング**: anyhow, thiserror
- **ファイル操作**: tokio::fs
- **パッケージ管理**: Cargo
- **テスト**: Built-in test framework + tokio-test

### プロジェクト構造

```
aicm/
├── src/
│   ├── main.rs                 # CLI エントリーポイント
│   ├── lib.rs                  # ライブラリエントリーポイント
│   ├── config/                 # 設定管理
│   │   ├── mod.rs
│   │   ├── loader.rs           # 設定読み込み
│   │   └── error.rs            # 設定エラー型
│   ├── core/                   # コア機能
│   │   ├── mod.rs
│   │   └── markdown_merger.rs  # Markdownファイル結合
│   ├── agents/                 # エージェント実装
│   │   ├── mod.rs
│   │   ├── base.rs            # ベースユーティリティ
│   │   └── cursor.rs          # Cursor実装
│   └── types/                  # 型定義
│       ├── mod.rs
│       ├── config.rs          # 設定型
│       └── agent.rs           # エージェント型
├── target/                    # ビルド出力
├── Cargo.toml                 # プロジェクト設定
├── Cargo.lock                 # 依存関係ロック
└── README.md
```

## コア設計

### 1. 設定ファイル型定義

```rust
// types/config.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContextConfig {
    pub version: String,
    pub output_mode: OutputMode,
    pub base_docs_dir: String,
    pub agents: AgentConfigs,
    pub file_mapping: FileMapping,
    #[serde(default)]
    pub global_variables: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Merged,
    Split,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfigs {
    #[serde(default)]
    pub cursor: Option<CursorConfig>,
    #[serde(default)]
    pub cline: Option<ClineConfig>,
    #[serde(default)]
    pub github: Option<GitHubConfig>,
    #[serde(default)]
    pub claude: Option<ClaudeConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorConfig {
    #[serde(default)]
    pub split_config: Option<HashMap<String, CursorRuleConfig>>,
    #[serde(default)]
    pub additional_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorRuleConfig {
    #[serde(rename = "type")]
    pub rule_type: CursorRuleType,
    pub description: String,
    #[serde(default)]
    pub globs: Option<Vec<String>>,
    #[serde(default)]
    pub always_apply: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorRuleType {
    Always,
    AutoAttached,
    AgentRequested,
    Manual,
}
```

### 2. エージェントトレイト設計

```rust
// types/agent.rs
use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait BaseAgent: Send + Sync {
    fn get_info(&self) -> AgentInfo;

    async fn generate_files(
        &self,
        merged_content: &str,
        split_content: &SplitContent,
    ) -> Result<Vec<GeneratedFile>>;

    fn get_output_paths(&self) -> Vec<String>;
    fn validate(&self) -> ValidationResult;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

fn default_encoding() -> String {
    "utf8".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitContent {
    pub common: String,
    pub project_specific: String,
    pub agent_specific: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

### 3. Markdown マージ機能

```rust
// core/markdown_merger.rs
use crate::types::{AIContextConfig, MergedContent, SplitContent};
use anyhow::Result;
use std::path::Path;
use tokio::fs;

pub struct MarkdownMerger {
    config: AIContextConfig,
}

impl MarkdownMerger {
    pub fn new(config: AIContextConfig) -> Self {
        Self { config }
    }

    pub async fn merge(&self) -> Result<MergedContent, MarkdownMergerError> {
        let base_dir = Path::new(&self.config.base_docs_dir);

        // ベースディレクトリの存在確認
        if !base_dir.exists() {
            return Err(MarkdownMergerError::BaseDirectoryNotFound {
                path: self.config.base_docs_dir.clone(),
            });
        }

        // 各カテゴリのコンテンツを読み込み
        let common_content = self.read_files(&self.config.file_mapping.common, base_dir).await?;
        let project_content = self.read_files(&self.config.file_mapping.project_specific, base_dir).await?;
        let agent_content = self.read_agent_specific_files(base_dir).await?;

        // 分割コンテンツを作成
        let split_content = SplitContent {
            common: common_content,
            project_specific: project_content,
            agent_specific: agent_content,
        };

        // 統合コンテンツを作成
        let merged_content = format!(
            "{}\n\n{}\n\n{}",
            split_content.common,
            split_content.project_specific,
            split_content.agent_specific
        );

        Ok(MergedContent {
            merged: merged_content,
            split: split_content,
        })
    }
}
```

### 4. 設定ローダー

```rust
// config/loader.rs
use crate::config::error::ConfigError;
use crate::types::AIContextConfig;
use anyhow::Result;
use std::path::Path;
use tokio::fs;

pub struct ConfigLoader;

impl ConfigLoader {
    pub async fn load<P: AsRef<Path>>(config_path: P) -> Result<AIContextConfig, ConfigError> {
        let path = config_path.as_ref();

        // ファイルの存在確認
        if !path.exists() {
            return Err(ConfigError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        // ファイル読み込み
        let content = fs::read_to_string(path).await?;

        // YAML解析
        let mut config: AIContextConfig = serde_yaml::from_str(&content)?;

        // 検証
        Self::validate_config(&mut config)?;

        Ok(config)
    }

    fn validate_config(config: &mut AIContextConfig) -> Result<(), ConfigError> {
        let mut errors = Vec::new();

        // 必須フィールドの検証
        if config.version.is_empty() {
            errors.push("version フィールドが空です".to_string());
        }

        if config.base_docs_dir.is_empty() {
            errors.push("base_docs_dir フィールドが空です".to_string());
        }

        // ファイルマッピングの検証
        Self::validate_file_mapping(&config.file_mapping, &mut errors);

        // エラーがある場合は失敗
        if !errors.is_empty() {
            return Err(ConfigError::ValidationError { errors });
        }

        Ok(())
    }
}
```

## エージェント実装詳細

### 1. Cursor エージェント

```rust
// agents/cursor.rs
use crate::agents::base::BaseAgentUtils;
use crate::types::{BaseAgent, AIContextConfig, CursorConfig, GeneratedFile, SplitContent, ValidationResult, AgentInfo};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct CursorAgent {
    config: AIContextConfig,
    cursor_config: CursorConfig,
}

impl CursorAgent {
    pub fn new(config: AIContextConfig, cursor_config: CursorConfig) -> Self {
        Self { config, cursor_config }
    }

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

    async fn generate_merged_file(&self, merged_content: &str) -> Result<Vec<GeneratedFile>> {
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
}

#[async_trait]
impl BaseAgent for CursorAgent {
    fn get_info(&self) -> AgentInfo {
        AgentInfo {
            name: "cursor".to_string(),
            description: "Cursor AI Editor用のルールファイル生成エージェント".to_string(),
            output_patterns: vec![".cursor/rules/*.mdc".to_string()],
            supports_split: true,
        }
    }

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
```

## CLI コマンド実装

### 1. メインエントリーポイント

```rust
// main.rs
use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "aicm")]
#[command(about = "AI Code Agent Context Management CLI tool for generating context files for multiple AI coding agents")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new AI context configuration
    Init {
        #[arg(short, long, default_value = "ai-context.yaml")]
        config: String,
    },
    /// Generate context files for configured agents
    Generate {
        #[arg(short, long, default_value = "ai-context.yaml")]
        config: String,
        #[arg(short, long)]
        agent: Option<String>,
    },
    /// Validate configuration file
    Validate {
        #[arg(short, long, default_value = "ai-context.yaml")]
        config: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { config } => handle_init(&config).await,
        Commands::Generate { config, agent } => handle_generate(&config, agent.as_deref()).await,
        Commands::Validate { config } => handle_validate(&config).await,
    }
}
```

### 2. Init コマンド

```rust
async fn handle_init(config_path: &str) -> Result<()> {
    println!("AI Context Management設定ファイルを初期化します: {}", config_path);

    // 既存ファイルの確認
    if Path::new(config_path).exists() {
        eprintln!("設定ファイルは既に存在します: {}", config_path);
        return Ok(());
    }

    // デフォルト設定を生成
    let default_config = ConfigLoader::create_default_config();

    // 設定ファイルを保存
    ConfigLoader::save_config(&default_config, config_path)
        .await
        .context("設定ファイルの保存に失敗しました")?;

    println!("✅ 設定ファイルを作成しました: {}", config_path);
    println!("📝 設定ファイルを編集してプロジェクトに合わせてカスタマイズしてください");

    Ok(())
}
```

### 3. Generate コマンド

```rust
async fn handle_generate(config_path: &str, target_agent: Option<&str>) -> Result<()> {
    println!("コンテキストファイルを生成します: {}", config_path);

    // 設定読み込み
    let config = ConfigLoader::load(config_path)
        .await
        .context("設定ファイルの読み込みに失敗しました")?;

    // Markdownマージ
    let merger = MarkdownMerger::new(config.clone());
    let merged_content = merger.merge()
        .await
        .context("Markdownファイルのマージに失敗しました")?;

    // エージェント別生成
    match target_agent {
        Some("cursor") | None => {
            if let Some(cursor_config) = &config.agents.cursor {
                generate_cursor_files(&config, cursor_config, &merged_content).await?;
            } else if target_agent.is_some() {
                eprintln!("⚠️  Cursor設定が見つかりません");
            }
        }
        Some(agent) => {
            eprintln!("❌ 未対応のエージェント: {}", agent);
            eprintln!("サポートされているエージェント: cursor");
            return Ok(());
        }
    }

    println!("✅ コンテキストファイルの生成が完了しました");
    Ok(())
}
```

## Cargo パッケージ設定

### Cargo.toml

```toml
[package]
name = "aicm"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "AI Code Agent Context Management CLI tool for generating context files for multiple AI coding agents"
license = "MIT"
repository = "https://github.com/morooka-akira/aicm"
keywords = ["ai", "context", "cli", "agents", "tools"]
categories = ["command-line-utilities", "development-tools"]

[dependencies]
# CLI framework
clap = { version = "4.4", features = ["derive"] }

# YAML parsing
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# File system operations
tokio = { version = "1.0", features = ["full"] }

# Pattern matching for glob patterns
glob = "0.3"

# Path manipulation
path-clean = "1.0"

# Async traits
async-trait = "0.1"

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.8"

[[bin]]
name = "aicm"
path = "src/main.rs"

[lib]
name = "ai_code_agent_context_management"
path = "src/lib.rs"

[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
```

## 配布・インストール

### Cargo での配布

```bash
# crates.ioからインストール
cargo install aicm

# Gitリポジトリから直接インストール
cargo install --git https://github.com/morooka-akira/aicm

# ローカルビルド・インストール
cargo install --path .
```

### 使用方法

```bash
# ヘルプ表示
aicm --help

# 設定ファイル初期化
aicm init

# コンテキストファイル生成
aicm generate

# 設定ファイル検証
aicm validate
```

## テスト戦略

### 1. ユニットテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio_test;

    #[tokio::test]
    async fn test_cursor_agent_generation() {
        let config = create_test_config();
        let cursor_config = create_test_cursor_config();
        let agent = CursorAgent::new(config, cursor_config);

        let split_content = create_test_split_content();
        let files = agent.generate_files("", &split_content).await.unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].path.ends_with(".mdc"));
    }

    #[tokio::test]
    async fn test_config_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("ai-context.yaml");

        let config = create_test_config();
        ConfigLoader::save_config(&config, &config_path).await.unwrap();

        let loaded_config = ConfigLoader::load(&config_path).await.unwrap();
        assert_eq!(loaded_config.version, config.version);
    }
}
```

### 2. 統合テスト

```rust
// tests/integration_test.rs
use ai_code_agent_context_management::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_current_dir(&temp_dir).unwrap();

    // 初期化
    let result = handle_init("ai-context.yaml").await;
    assert!(result.is_ok());

    // 生成
    let result = handle_generate("ai-context.yaml", Some("cursor")).await;
    assert!(result.is_ok());

    // 出力ファイルの確認
    assert!(Path::new(".cursor/rules/context.mdc").exists());
}
```

## パフォーマンス考慮

### 1. 非同期処理

- Tokio による効率的な I/O 処理
- 並列ファイル読み込み
- ゼロコピー文字列処理

### 2. メモリ効率

- Rust の所有権システムによるメモリ安全性
- 不要なクローンの回避
- ストリーミング処理対応

### 3. バイナリサイズ最適化

- LTO（Link Time Optimization）
- コード生成ユニット最適化
- デバッグシンボル削除

## セキュリティ考慮

### 1. メモリ安全性

- Rust の型システムによるメモリ安全性保証
- データ競合の静的防止

### 2. ファイルアクセス

- パス正規化によるディレクトリトラバーサル防止
- 適切なエラーハンドリング

### 3. 設定ファイル検証

- Serde による型安全なデシリアライゼーション
- 厳密なスキーマ検証

## 今後の拡張

### 1. 新しいエージェント追加

- Cline, GitHub Copilot, Claude Code 実装
- プラグインシステムの検討

### 2. 設定機能強化

- 環境変数による設定オーバーライド
- 設定ファイルのホットリロード

### 3. パフォーマンス改善

- 並列処理の強化
- キャッシュ機能の追加
- インクリメンタル生成
