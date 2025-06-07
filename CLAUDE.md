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

### 必要な環境
- Node.js >= 16.0.0
- pnpm >= 8.0.0

### 主要コマンド
```bash
pnpm install      # 依存関係インストール
pnpm run dev      # 開発実行 (tsx使用)
pnpm run build    # TypeScriptビルド
pnpm run test     # Vitestテスト実行
pnpm run test:coverage  # カバレッジ付きテスト
pnpm run check:fix      # Biome リント+フォーマット自動修正
```

## 開発ルール

### テスト要件
- **必須**: 各モジュールはVitestでテストを作成すること
- **カバレッジ**: C1ルート（分岐網羅）を最低限通過すること
  - 条件分岐、try-catch、関数の全パスをテスト
  - vitest.config.tsで80%以上のカバレッジを設定済み
- **作業完了**: 作業終了時は必ずテストが通ることを確認すること

### テスト実行例
```bash
# 開発中のテスト
pnpm run test:watch

# カバレッジ確認
pnpm run test:coverage

# CI用（全テスト実行）
pnpm run test:run
```

### コード品質
- **Biome**: リンター・フォーマッターで品質管理
- **Husky**: pre-commit/pre-pushでの自動チェック
- **TypeScript**: 厳格な型チェック設定

## アーキテクチャノート

### 設計原則
- **抽象化**: 各AIツール固有の設定ファイル形式を抽象化
- **統一管理**: 共通の設定ファイルから各ツール用ファイルを生成
- **オーバーライド**: 設定ファイルのカスタマイズのみで完結する設計

### ファイル構造
```
src/
├── commands/    # CLIコマンド実装
├── core/        # コア機能（マージ、設定読み込み等）
├── agents/      # エージェント実装（GitHub、Cline、Cursor、Claude）
├── types/       # TypeScript型定義
├── utils/       # ユーティリティ関数
└── templates/   # テンプレートファイル

tests/          # テストファイル（Vitest）
docs/           # 設計ドキュメント
├── concept.md       # 設計概要
├── design_doc.md    # 技術仕様書
└── requirements.md  # 要件定義
```

### 実装時の注意点
- 新しい関数・クラスを作成する際は、対応するテストファイルも同時作成
- テストファイルは `*.test.ts` の命名規則を使用
- エラーハンドリングも含めてテストケースを作成
- モックを使用して外部依存を分離してテスト

## 参考リンク
- [Claude Code Memory (CLAUDE.md)](https://docs.anthropic.com/en/docs/claude-code/memory)
- [Cline Rules](https://docs.cline.bot/features/cline-rules)
- [GitHub Copilot Custom Instructions](https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot)
- [VS Code Copilot Customization](https://code.visualstudio.com/docs/copilot/copilot-customization#_use-instructionsmd-files)
- [Cursor Rules](https://docs.cursor.com/context/rules)