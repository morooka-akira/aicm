# AI Code Agent Context Management Tool 🦀

AI コーディングエージェント用の context ファイルを統一設定から自動生成する Rust 製 CLI ツール

## ✨ 概要

複数の AI ツール（GitHub Copilot、Cline、Cursor、Claude Code）用の context ファイルを一元管理し、統一設定から各ツール固有のファイル形式を自動生成します。

## 🎯 サポート対象ツール

- **🎯 Cursor**: `.cursor/rules/*.mdc` ファイル（YAML frontmatter 付き）
- **🚧 Cline**: `.clinerules/*.md` ファイル（今後実装予定）
- **🚧 GitHub Copilot**: `instructions.md` 階層配置（今後実装予定）
- **🚧 Claude Code**: `CLAUDE.md`（今後実装予定）

## 🚀 インストール

### Cargo からインストール（推奨）

```bash
# crates.ioからインストール（今後公開予定）
cargo install aicm

# Gitリポジトリから直接インストール
cargo install --git https://github.com/morooka-akira/aicm

# ローカルビルド・インストール
git clone https://github.com/morooka-akira/aicm
cd aicm
cargo install --path .
```

### 必要な環境

- Rust 1.70.0 以上
- Cargo（Rust と一緒にインストールされます）

## 📖 使用方法

### 基本的な使い方

```bash
# プロジェクトディレクトリで設定ファイルを初期化
aicm init

# 設定ファイルを編集
vim ai-context.yaml

# コンテキストファイルを生成
aicm generate

# 特定のエージェントのみ生成
aicm generate --agent cursor

# 設定ファイルを検証
aicm validate

# 利用可能なエージェント一覧を表示
aicm list-agents
```

### 設定ファイル例

```yaml
# ai-context.yaml
version: "1.0"
output_mode: merged # merged | split
base_docs_dir: ./docs

# エージェント固有設定
agents:
  cursor:
    split_config:
      common-rules:
        type: always
        description: "共通のコーディングルール"
        globs: ["**/*.rs", "**/*.ts"]
      project-rules:
        type: auto_attached
        description: "プロジェクト固有のルール"

# ファイルマッピング設定
file_mapping:
  common:
    - "README.md"
    - "docs/coding-standards.md"
  project_specific:
    - "docs/architecture.md"
    - "docs/api-spec.md"
  agent_specific:
    cursor:
      - "docs/cursor-specific.md"
```

## 🔧 開発環境

### セットアップ

```bash
# リポジトリをクローン
git clone https://github.com/morooka-akira/aicm
cd aicm

# ビルド
cargo build

# テスト実行
cargo test

# リリースビルド
cargo build --release

# 開発版での実行
cargo run -- init
cargo run -- generate
```

### 使用技術

- **言語**: Rust (Edition 2021)
- **CLI Framework**: clap v4 (derive API)
- **非同期処理**: Tokio
- **設定**: YAML (serde_yaml)
- **エラーハンドリング**: anyhow, thiserror
- **テスト**: 標準テストフレームワーク + tokio-test

## 📁 プロジェクト構造

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
├── docs/                      # 設計ドキュメント
│   ├── concept.md             # 設計概要
│   ├── design_doc.md          # 技術仕様書（Rust版）
│   └── requirements.md        # 要件定義
├── target/                    # ビルド出力
├── Cargo.toml                 # プロジェクト設定
└── Cargo.lock                 # 依存関係ロック
```

## 🧪 テスト

```bash
# 全テスト実行
cargo test

# 特定のテストモジュール実行
cargo test config

# テストカバレッジ（tarpaulin要インストール）
cargo install cargo-tarpaulin
cargo tarpaulin --out html

# 統合テスト実行
cargo test --test integration_test
```

## 🚢 配布・デプロイ

### リリースビルド

```bash
# 最適化されたバイナリビルド
cargo build --release

# バイナリは target/release/aicm に生成されます
```

### クロスコンパイル（例）

```bash
# macOS用（Apple Silicon）
cargo build --release --target aarch64-apple-darwin

# Linux用
cargo build --release --target x86_64-unknown-linux-gnu

# Windows用
cargo build --release --target x86_64-pc-windows-gnu
```

## ⚡ パフォーマンス特徴

- **高速起動**: Rust ネイティブバイナリによる瞬時起動
- **低メモリ使用量**: 効率的なメモリ管理
- **並列処理**: Tokio による非同期ファイル処理
- **ゼロコピー**: 不要な文字列コピーの回避

## 🔒 セキュリティ

- **メモリ安全**: Rust の所有権システムによる保証
- **型安全**: コンパイル時の厳密な型チェック
- **パストラバーサル防止**: 適切なパス正規化

## 🤝 コントリビューション

1. このリポジトリをフォーク
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

### 開発ガイドライン

- コードは Rustfmt でフォーマット（`cargo fmt`）
- Clippy の警告を解決（`cargo clippy`）
- テストを追加（`cargo test`）
- ドキュメントを更新

## 📝 ライセンス

MIT License - 詳細は [LICENSE](LICENSE) ファイルを参照

## 🙏 謝辞

このプロジェクトは以下の素晴らしいツールによって支えられています：

- [clap](https://github.com/clap-rs/clap) - CLI 構築フレームワーク
- [tokio](https://github.com/tokio-rs/tokio) - 非同期ランタイム
- [serde](https://github.com/serde-rs/serde) - シリアライゼーション
- [anyhow](https://github.com/dtolnay/anyhow) - エラーハンドリング

## 📞 サポート

- バグ報告: [Issues](https://github.com/morooka-akira/aicm/issues)
- 機能要求: [Issues](https://github.com/morooka-akira/aicm/issues)
- ディスカッション: [Discussions](https://github.com/morooka-akira/aicm/discussions)
