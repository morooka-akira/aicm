# Claude Code 固有の指示

## Claude Code特有の要求
- このツールは Claude Code (claude.ai/code) で使用されることを前提とする
- CLAUDE.md ファイルの自動生成機能を提供する
- 他のエージェント固有設定との整合性を保つ

## 出力形式
- Markdown形式
- 統合モードのみ対応（分割モード非対応）
- プロジェクトルートに `CLAUDE.md` として出力

## 設定例
```yaml
agents:
  claude:
    language: "ja"
    additional_sections:
      - "development"
      - "testing"
    additional_instructions: |
      Claude Code使用時の追加注意事項をここに記述
```