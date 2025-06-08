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
