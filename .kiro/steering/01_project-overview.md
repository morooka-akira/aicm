# プロジェクト概要

このファイルは、このリポジトリでコードを扱う際に Claude Code (claude.ai/code)、Cline, Corsor, Github Copilot、Codex など各種 AI コードエージェント にガイダンスを提供します。

# 言語

- **エージェントのやり取りは必ず日本語で対応すること**
- **rust コード上のコメント及び、コマンドの出力は、英語で記載すること**

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
