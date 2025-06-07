/*!
 * AI Context Management Tool - Main Entry Point
 * 
 * このファイルはCLIツールのメインエントリーポイントです。
 * 各AI編集エージェント用のコンテキストファイルを生成します。
 */

mod config;
mod agents;
mod core;
mod types;

use crate::config::{ConfigLoader, ConfigError};
use crate::agents::CursorAgent;
use crate::core::MarkdownMerger;
use crate::types::{AIContextConfig, CursorConfig, BaseAgent};
use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::path::Path;
use tokio::fs;

/// AI Context Management CLI Tool
#[derive(Parser)]
#[command(name = "ai-context")]
#[command(about = "AI Context Management CLI tool for generating context files for multiple AI coding agents")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new AI context configuration
    Init {
        /// Configuration file path
        #[arg(short, long, default_value = "ai-context.yaml")]
        config: String,
    },
    /// Generate context files for configured agents
    Generate {
        /// Configuration file path
        #[arg(short, long, default_value = "ai-context.yaml")]
        config: String,
        /// Target agent (cursor, cline, github, claude)
        #[arg(short, long)]
        agent: Option<String>,
    },
    /// Validate configuration file
    Validate {
        /// Configuration file path
        #[arg(short, long, default_value = "ai-context.yaml")]
        config: String,
    },
    /// List available agents
    ListAgents,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { config } => {
            handle_init(&config).await
        }
        Commands::Generate { config, agent } => {
            handle_generate(&config, agent.as_deref()).await
        }
        Commands::Validate { config } => {
            handle_validate(&config).await
        }
        Commands::ListAgents => {
            handle_list_agents().await
        }
    }
}

/// 初期化コマンドの処理
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

/// 生成コマンドの処理
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

/// Cursorファイル生成
async fn generate_cursor_files(
    config: &AIContextConfig,
    cursor_config: &CursorConfig,
    merged_content: &crate::types::MergedContent,
) -> Result<()> {
    let agent = CursorAgent::new(config.clone(), cursor_config.clone());
    
    // 検証
    let validation = agent.validate();
    if !validation.valid {
        eprintln!("❌ Cursor設定の検証に失敗しました:");
        for error in &validation.errors {
            eprintln!("  - {}", error);
        }
        return Ok(());
    }

    // 警告表示
    for warning in &validation.warnings {
        eprintln!("⚠️  {}", warning);
    }

    // ファイル生成
    let files = agent.generate_files(&merged_content.merged, &merged_content.split)
        .await
        .context("Cursorファイルの生成に失敗しました")?;

    // ファイル出力
    for file in &files {
        let file_path = Path::new(&file.path);
        
        // ディレクトリ作成
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context(format!("ディレクトリの作成に失敗しました: {:?}", parent))?;
        }

        // ファイル書き込み
        fs::write(file_path, &file.content)
            .await
            .context(format!("ファイルの書き込みに失敗しました: {:?}", file_path))?;

        println!("📄 {}", file.path);
    }

    Ok(())
}

/// 検証コマンドの処理
async fn handle_validate(config_path: &str) -> Result<()> {
    println!("設定ファイルを検証します: {}", config_path);

    match ConfigLoader::load(config_path).await {
        Ok(config) => {
            println!("✅ 設定ファイルは有効です");
            
            // ファイル存在チェック
            let merger = MarkdownMerger::new(config);
            match merger.validate_files().await {
                Ok(missing_files) => {
                    if missing_files.is_empty() {
                        println!("✅ 全てのファイルが存在します");
                    } else {
                        println!("⚠️  以下のファイルが見つかりません:");
                        for file in missing_files {
                            println!("  - {}", file);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ ファイル検証エラー: {}", e);
                }
            }
        }
        Err(ConfigError::FileNotFound { path }) => {
            eprintln!("❌ 設定ファイルが見つかりません: {}", path);
        }
        Err(e) => {
            eprintln!("❌ 設定ファイルの検証に失敗しました: {}", e);
        }
    }

    Ok(())
}

/// エージェント一覧コマンドの処理
async fn handle_list_agents() -> Result<()> {
    println!("利用可能なエージェント:");
    println!("  🎯 cursor: Cursor AI Editor用のルールファイル (.cursor/rules/*.mdc)");
    println!("  🚧 cline: Cline AI Assistant用のコンテキスト (今後実装予定)");
    println!("  🚧 github: GitHub Copilot用のナレッジ (今後実装予定)");
    println!("  🚧 claude: Claude Code用のコンテキスト (今後実装予定)");

    Ok(())
}