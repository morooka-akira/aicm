---
applyTo: "**/*.rs,**/*.md"
---

# 05_development-rules.md

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


# 06_development-setup.md

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
