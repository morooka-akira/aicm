# CLAUDE.md

**必ず日本語で対応すること**

このファイルは、このリポジトリでコードを扱う際にClaude Code (claude.ai/code) にガイダンスを提供します。

## プロジェクト概要

このリポジトリは、複数のAIコーディングエージェント用のcontextファイルを統一設定から自動生成するコマンドラインツールです。

### 目的
- GitHub Copilot、Cline、Cursor、Claude Code用のcontextファイルを一元管理
- 一つの設定ファイルから各ツール固有のファイル形式を自動生成
- 開発チーム間でのAIツール設定の一貫性を保つ

### サポート対象ツール
1. **GitHub Copilot**: `instructions.md` (階層配置対応)
2. **Cline**: `.clinerules/` ディレクトリ内の `.md` ファイル
3. **Cursor**: `.cursor/rules` ディレクトリ内の `.mdc` ファイル
4. **Claude Code**: `CLAUDE.md`

詳細な設計概要は `docs/concept.md` を参照してください。

## 開発環境セットアップ

プロジェクト構造が確立されたら、このセクションを以下で更新する必要があります：
- ビルドコマンド
- テストコマンド  
- リント/フォーマットコマンド
- CLI実行コマンド

## アーキテクチャノート

### 設計原則
- **抽象化**: 各AIツール固有の設定ファイル形式を抽象化
- **統一管理**: 共通の設定ファイルから各ツール用ファイルを生成
- **オーバーライド**: 設定ファイルのカスタマイズのみで完結する設計

### ファイル構造
```
ai-context.yaml          # 統一設定ファイル
docs/concept.md         # 設計概要ドキュメント
src/                    # CLIツールのソースコード（予定）
templates/              # 各ツール用テンプレート（予定）
```

## 参考リンク
- [Claude Code Memory (CLAUDE.md)](https://docs.anthropic.com/en/docs/claude-code/memory)
- [Cline Rules](https://docs.cline.bot/features/cline-rules)
- [GitHub Copilot Custom Instructions](https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot)
- [VS Code Copilot Customization](https://code.visualstudio.com/docs/copilot/copilot-customization#_use-instructionsmd-files)
- [Cursor Rules](https://docs.cursor.com/context/rules)