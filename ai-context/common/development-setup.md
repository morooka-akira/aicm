# 開発環境セットアップ

## 必要な環境
- Rust 1.70.0 以上
- Cargo（Rustと一緒にインストール）

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
cargo run -- list-agents

# リント・フォーマット
cargo fmt     # コードフォーマット
cargo clippy  # リント実行

# ドキュメント生成
cargo doc --open
```