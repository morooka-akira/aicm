# プロジェクト概要

このファイルは、このリポジトリでコードを扱う際に Claude Code (claude.ai/code)、Cline, Corsor, Github Copilot、Codex など各種 AI コードエージェント にガイダンスを提供します。

**必ず日本語で対応すること**

## プロジェクト概要

このリポジトリは、複数の AI コーディングエージェント用の context ファイルを統一設定から自動生成する Rust 製コマンドラインツール `aicm` です。

### 目的

- GitHub Copilot、Cline、Cursor、Claude Code、OpenAI Codex 用の context ファイルを一元管理
- 一つの設定ファイルから各ツール固有のファイル形式を自動生成
- 開発チーム間での AI ツール設定の一貫性を保つ
- Rust による高速・安全な実装

### サポート対象ツール

1. **✅ Cursor**: `.cursor/rules/*.mdc` ファイル（実装済み）
2. **✅ Cline**: `.clinerules/*.md` ファイル（実装済み）
3. **✅ GitHub Copilot**: `.github/instructions/*.instructions.md` または `.github/copilot-instructions.md`（applyTo オプション対応済み）
4. **✅ Claude Code**: `CLAUDE.md`（実装済み）
5. **✅ OpenAI Codex**: `AGENTS.md`（実装済み）

詳細な設計概要は @docs/concept.md を参照してください。

# セキュリティ 🔒

## 機密ファイル

以下のファイルは読み取りや修正を行わないでください：

- .env ファイル
- \*_/config/secrets._
- \*_/_.pem
- API キー、トークン、認証情報を含むファイル

## セキュリティプラクティス

- 機密ファイルをコミットしない
- シークレットには環境変数を使用
- 認証情報をログや出力に含めない

# アーキテクチャ

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
│   ├── cursor.rs          # Cursor実装（実装済み）
│   ├── cline.rs           # Cline実装（実装済み）
│   ├── github.rs          # GitHub Copilot実装（実装済み）
│   └── claude.rs          # Claude実装（実装済み）
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

# 開発版での実行（バイナリ名: aicm）
cargo run -- init
cargo run -- generate
cargo run -- generate --agent cursor
cargo run -- validate

# リント・フォーマット
cargo fmt     # コードフォーマット
cargo clippy  # リント実行

# ドキュメント生成
cargo doc --open
```

# 開発ルール

## タスクの進め方

- 1. 作業開始時に、`ai-works` ディレクトリに `yyyy-mm-dd-<work name>.md` を作成し、作業内容、設計方針、完了要件をまとめてください
  - 中断を指示された場合は、それに従ってください
- 2. ルールに従って、作業を実行
- 3. rust コードに修正がある場合は、`cargo fmt`, `cargo clippy` を実行してください
  - `cargo clippy` は warning でも修正可能であれば修正してください
- 4. 修正内容が、 @README.md @docs/ の内容と乖離がある場合は、ドキュメントの更新を行ってください
- 5. 手順 1 でドキュメントに書いた完了要件を全て満たしているか、もう一度チェックしてください。
- 6. ルールに従って、PR を作成
  - デフォルトブランチは `main`
  - 作業開始時に、必ずデフォルトブランチからブランチを分けて作業すること
  - 作業完了後は、gh コマンドで PR を作成すること

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

## エラーハンドリング

- anyhow・thiserror による適切なエラー処理を行うこと

# rust 固有のルール

## 実装時の注意点

- 新しい機能を追加する際は、対応するテストも同時作成
- テストファイルは `#[cfg(test)]` モジュールまたは `tests/` ディレクトリを使用
- エラーハンドリング（`Result<T, E>`）も含めてテストケースを作成
- 非同期処理は `#[tokio::test]` を使用してテスト

## エラーハンドリング

- anyhow・thiserror による適切なエラー処理を行うこと

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

## 主要なクレート

- **clap**: CLI 構築フレームワーク（derive API 使用）
- **tokio**: 非同期ランタイム（full features）
- **serde + serde_yaml**: 設定ファイル処理
- **anyhow + thiserror**: エラーハンドリング
- **async-trait**: 非同期トレイト
- **path-clean**: パス正規化
- **glob**: パターンマッチング

## 開発用クレート

- **tokio-test**: 非同期テスト
- **tempfile**: テスト用一時ファイル

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
- [OpenAI Codex](https://openai.com/ja-JP/index/introducing-codex/)