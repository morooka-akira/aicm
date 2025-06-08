# このプロジェクトのビジネスルール

## プロジェクト概要
AI Context Management Tool - 複数のAIツール用設定ファイルを統一管理するCLIツール

## 重要な概念
- **エージェント**: Cursor、Cline、GitHub Copilot、Claude Codeなどの各AIツール
- **統合モード**: 全内容を1ファイルに結合
- **分割モード**: 複数ファイルに分割してより細かい制御

## このプロジェクト特有のルール
- 設定ファイルは必ずai-context.yamlという名前
- 生成されるファイルは各AIツールの公式仕様に厳密に準拠
- エラー時は日本語メッセージを表示
- ファイルが見つからない場合はスキップ（エラーにしない）

## 出力ファイルの命名規則
- Cursor: `.cursor/rules/*.mdc`
- Cline: `.clinerules/*.md`
- GitHub Copilot: `instructions.md`
- Claude Code: `CLAUDE.md`