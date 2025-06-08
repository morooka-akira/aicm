# Cursor 固有の指示

## Cursor特有の要求
- MDC（Markdown + フロントマター）形式での出力
- `.cursor/rules/` ディレクトリに配置
- 型安全なフロントマター設定（always, auto_attached, agent_requested, manual）

## 出力形式
- 統合モード: `.cursor/rules/context.mdc`
- 分割モード: 複数の `.mdc` ファイル（今後実装予定）

## 設定例
```yaml
agents:
  cursor:
    split_config:
      common:
        type: always
        description: "共通コーディング規約"
        globs: ["**/*.rs", "**/*.toml"]
      project:
        type: auto_attached
        description: "プロジェクト固有ルール"
        globs: ["src/**/*"]
    additional_instructions: |
      Cursor使用時の追加注意事項をここに記述
```