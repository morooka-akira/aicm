# OpenAI Codex エージェント実装

## 作業日
2025-06-10

## 作業概要
OpenAI Codex エージェントに対応するため、新しいエージェント実装を追加する。

## 要件
- OpenAI Codex エージェント用のファイル生成に対応
- 既存のClaude実装を参考にした実装
- AGENTS.md ファイルの生成に対応
- 設定ファイルでCodexを選択できるよう拡張

## 技術的詳細

### OpenAI Codex について
- OpenAI が開発したコード生成AI
- GitHub Copilot の基盤技術
- AGENTS.md ファイルを使用してエージェントに指示を与える

### 実装方針
1. 既存のClaude実装 (src/agents/claude.rs) を参考にする
2. Codex用のエージェント実装 (src/agents/codex.rs) を作成
3. AGENTS.md ファイルを生成する機能を実装
4. 設定ファイルでCodexを指定できるよう拡張

### ファイル生成先
- `AGENTS.md` - プロジェクトルートに生成

## 実装タスク
1. Claude実装の調査
2. Codex エージェント実装の作成
3. モジュール設定の更新
4. 設定型の拡張
5. テストケースの作成
6. lint・フォーマットチェック
7. PR作成

## 参考リンク
- [OpenAI Codex](https://openai.com/ja-JP/index/introducing-codex/)
- 既存のClaude実装: src/agents/claude.rs