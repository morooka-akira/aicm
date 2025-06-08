# プロジェクト概要

**必ず日本語で対応すること**

## プロジェクト概要

このリポジトリは、複数のAIコーディングエージェント用のcontextファイルを統一設定から自動生成するRust製コマンドラインツールです。

### 目的
- GitHub Copilot、Cline、Cursor、Claude Code用のcontextファイルを一元管理
- 一つの設定ファイルから各ツール固有のファイル形式を自動生成
- 開発チーム間でのAIツール設定の一貫性を保つ
- Rustによる高速・安全な実装

### サポート対象ツール
1. **🎯 Cursor**: `.cursor/rules/*.mdc` ファイル（実装済み）
2. **🚧 Cline**: `.clinerules/*.md` ファイル（今後実装予定）
3. **🚧 GitHub Copilot**: `instructions.md` 階層配置（今後実装予定）
4. **🚧 Claude Code**: `CLAUDE.md`（今後実装予定）

詳細な設計概要は `docs/concept.md` を参照してください。