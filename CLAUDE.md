# 01_project-overview.md

# プロジェクト概要

**必ず日本語で対応すること**

このファイルは、このリポジトリでコードを扱う際に Claude Code (claude.ai/code) にガイダンスを提供します。

## プロジェクト概要

このリポジトリは、複数の AI コーディングエージェント用の context ファイルを統一設定から自動生成する Rust 製コマンドラインツールです。

### 目的

- GitHub Copilot、Cline、Cursor、Claude Code 用の context ファイルを一元管理
- 一つの設定ファイルから各ツール固有のファイル形式を自動生成
- 開発チーム間での AI ツール設定の一貫性を保つ
- Rust による高速・安全な実装

### サポート対象ツール

1. **🎯 Cursor**: `.cursor/rules/*.mdc` ファイル（実装済み）
2. **🚧 Cline**: `.clinerules/*.md` ファイル（今後実装予定）
3. **🚧 GitHub Copilot**: `instructions.md` 階層配置（今後実装予定）
4. **🚧 Claude Code**: `CLAUDE.md`（今後実装予定）

詳細な設計概要は `docs/concept.md` を参照してください。

# 02_architecture.md

# アーキテクチャノート

## 設計原則

- **型安全性**: Rust の型システムによるコンパイル時エラー検出
- **メモリ安全性**: 所有権システムによる安全なメモリ管理
- **並行処理**: Tokio による効率的な非同期処理
- **抽象化**: トレイトベースのエージェント設計
- **統一管理**: 共通の設定ファイルから各ツール用ファイルを生成

## プロジェクト構造

```
src/
├── main.rs                 # CLI エントリーポイント
├── lib.rs                  # ライブラリエントリーポイント
├── config/                 # 設定管理
│   ├── mod.rs
│   ├── loader.rs           # 設定読み込み
│   └── error.rs            # 設定エラー型
├── core/                   # コア機能
│   ├── mod.rs
│   └── markdown_merger.rs  # Markdownファイル結合
├── agents/                 # エージェント実装
│   ├── mod.rs
│   ├── base.rs            # ベースユーティリティ
│   └── cursor.rs          # Cursor実装
└── types/                  # 型定義
    ├── mod.rs
    ├── config.rs          # 設定型
    └── agent.rs           # エージェント型

docs/                      # 設計ドキュメント
├── concept.md             # 設計概要
├── design_doc.md          # 技術仕様書（Rust版）
└── requirements.md        # 要件定義

target/                    # ビルド出力
├── debug/                 # デバッグビルド
└── release/               # リリースビルド

Cargo.toml                 # プロジェクト設定
Cargo.lock                 # 依存関係ロック
```

## 実装時の注意点

- 新しい機能を追加する際は、対応するテストも同時作成
- テストファイルは `#[cfg(test)]` モジュールまたは `tests/` ディレクトリを使用
- エラーハンドリング（`Result<T, E>`）も含めてテストケースを作成
- 非同期処理は `#[tokio::test]` を使用してテスト

## 型システムの活用

- `serde` による設定ファイルの型安全なデシリアライゼーション
- `async-trait` による非同期トレイトの実装
- `thiserror` による構造化されたエラー型定義
- オプション型（`Option<T>`）による明示的な Null 安全性

## パフォーマンス特徴

- **高速起動**: ネイティブバイナリによる瞬時起動（100ms 以内）
- **低メモリ**: 効率的なメモリ管理（10MB 以下）
- **並列処理**: 非同期 I/O による高速ファイル処理
- **ゼロコピー**: 不要な文字列コピーの回避

# 03_dependencies.md

# 依存関係

## 主要なクレート

- **clap**: CLI 構築フレームワーク（derive API 使用）
- **tokio**: 非同期ランタイム
- **serde + serde_yaml**: 設定ファイル処理
- **anyhow + thiserror**: エラーハンドリング
- **async-trait**: 非同期トレイト
- **path-clean**: パス正規化

## 開発用クレート

- **tokio-test**: 非同期テスト
- **tempfile**: テスト用一時ファイル

# 04_development-rules.md

# 開発ルール

## テスト要件

- **必須**: 各モジュールは Rust 標準テストフレームワークでテストを作成すること
- **カバレッジ**: 主要な機能とエラーパスのテストを含めること
- **作業完了**: 作業終了時は必ずテストが通ることを確認すること

## テスト実行例

```bash
# 全テスト実行
cargo test

# 特定のテストモジュール実行
cargo test config
cargo test agents::cursor

# テストカバレッジ（tarpaulin要インストール）
cargo install cargo-tarpaulin
cargo tarpaulin --out html

# 統合テスト実行
cargo test --test integration_test
```

## Git 運用

- 開発作業については、ブランチを分けて作業すること
- 指示された内容は、まず ai-works ディレクトリ内に作業要件を整理すること
- 作業完了後は、gh コマンドで PR を作成すること

## コード品質

- **rustfmt**: 統一されたコードフォーマット
- **clippy**: 高品質な Rust コードのためのリンター
- **型安全性**: Rust の強力な型システムを活用
- **エラーハンドリング**: anyhow・thiserror による適切なエラー処理

## Lint & Format

- 作業完了時に `cargo fmt` と `cargo clippy` を実行してください。

## 作業記録の作成

- 作業開始時に、`ai-works` ディレクトリに `yyyy-mm-dd-<work name>.md` を作成し、作業内容、要件をまとめてください
- 指示された場合は、一度作業内容を指示者に確認してもらってから作業を進めてください

# 05_development-setup.md

# 開発環境セットアップ

## 必要な環境

- Rust 1.70.0 以上
- Cargo（Rust と一緒にインストール）

## 主要コマンド

```bash
# プロジェクトクローン
git clone https://github.com/morooka-akira/aicm
cd aicm

# ビルド
cargo build

# リリースビルド
cargo build --release

# テスト実行
cargo test

# 開発版での実行
cargo run -- init
cargo run -- generate
cargo run -- validate

# リント・フォーマット
cargo fmt     # コードフォーマット
cargo clippy  # リント実行

# ドキュメント生成
cargo doc --open
```

# 06_roadmap.md

# 今後の拡張予定

## Phase 2 機能

- Cline、GitHub Copilot、Claude Code エージェント実装
- ウォッチモード（ファイル変更時の自動生成）
- 設定継承機能

## Phase 3 機能

- プラグインシステム（WASM 対応）
- Web UI
- クラウド同期

# 07_references.md

# 参考リンク

## 技術ドキュメント

- [Rust 公式ドキュメント](https://doc.rust-lang.org/)
- [Tokio 公式ドキュメント](https://tokio.rs/)
- [clap 公式ドキュメント](https://docs.rs/clap/)
- [serde 公式ドキュメント](https://serde.rs/)

## AI ツール関連

- [Claude Code Memory (CLAUDE.md)](https://docs.anthropic.com/en/docs/claude-code/memory)
- [Cline Rules](https://docs.cline.bot/features/cline-rules)
- [GitHub Copilot Custom Instructions](https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot)
- [VS Code Copilot Customization](https://code.visualstudio.com/docs/copilot/copilot-customization#_use-instructionsmd-files)
- [Cursor Rules](https://docs.cursor.com/context/rules)