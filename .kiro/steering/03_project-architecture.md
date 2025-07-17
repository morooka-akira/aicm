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
