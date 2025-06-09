# GitHub Copilot applyTo オプション対応

## 作業日
2025-06-10

## 作業概要
GitHub Copilot の applyTo オプションに対応して、特定のファイルパターンにのみ適用される instructions ファイルを生成できるようにする。

## 要件

### GitHub Copilot applyTo オプションとは
- VS Code の GitHub Copilot カスタマイゼーション機能の一部
- instructions.md ファイルで YAML frontmatter を使用してファイルパターンを指定
- 指定されたファイルパターンでのみ該当の instruction が適用される

### 実装仕様
参考URL: https://code.visualstudio.com/docs/copilot/copilot-customization#_use-instructionsmd-files

```yaml
---
applyTo: "**/*.ts,**/*.tsx"
---
```

### 設定方法（提案）
Cursorの split_config と同様の構成を採用：

```yaml
agents:
  github:
    enabled: true
    output_mode: split
    split_config:
      rules:
        - file_patterns: ["*architecture*"]
          apply_to: ["**/*.rs"]
        - file_patterns: ["*frontend*"] 
          apply_to: ["**/*.ts", "**/*.tsx"]
        - file_patterns: ["*backend*"]
          apply_to: ["**/*.py"]
```

## 技術的詳細

### 1. 設定型の拡張
- `GitHubAgentConfig` に `split_config` を追加
- `GitHubSplitConfig` と `GitHubSplitRule` を定義
- `apply_to` フィールドでファイルパターン配列を指定

### 2. GitHub エージェント実装の拡張
- split モード時に `apply_to` 設定に基づいて frontmatter を生成
- 複数の apply_to パターンをカンマ区切りで結合

### 3. 出力形式
```yaml
---
applyTo: "**/*.rs,**/*.toml"
---

# Architecture

アーキテクチャに関する指示...
```

## 実装タスク
1. 作業要件をまとめる
2. GitHub Copilot applyTo 仕様を調査
3. 設定型に apply_to 設定を追加
4. GitHub エージェントで applyTo 対応を実装
5. テストケースを作成
6. lint・フォーマットチェック
7. ドキュメント更新
8. PR作成

## 参考リンク
- [VS Code Copilot Customization](https://code.visualstudio.com/docs/copilot/copilot-customization#_use-instructionsmd-files)
- 既存のCursor実装: src/agents/cursor.rs