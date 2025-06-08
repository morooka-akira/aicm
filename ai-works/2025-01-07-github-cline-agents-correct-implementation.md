# GitHub・Cline エージェント正しい仕様実装作業記録

## 📅 作業日

2025-01-07

## 🎯 作業目標

GitHub と Cline エージェント用の設定ファイル出力機能を**正しい仕様**で実装する

## 📋 修正前の問題

初回実装では仕様を誤解していました：

- **誤った実装**: 両エージェントとも merged モードのみで、プロジェクトルートに `.md` ファイルを出力
- **正しい仕様**: 両方とも split/merged モードをサポートし、それぞれ専用のディレクトリ構造を使用

## 🔍 正しい仕様の確認

### Cline Rules の仕様

[Cline Rules Documentation](https://docs.cline.bot/features/cline-rules):

**Split モード（推奨）**: `.clinerules/` フォルダシステム

```
your-project/
├── .clinerules/              # フォルダ containing active rules
│   ├── 01-coding.md          # Core coding standards
│   ├── 02-documentation.md   # Documentation requirements
│   └── current-sprint.md     # Rules specific to current work
```

**Merged モード**: 単一の `.clinerules` ファイル（拡張子なし）

```
your-project/
├── .clinerules              # 単一ファイル
```

### GitHub Copilot の仕様

[VS Code Copilot Customization](https://code.visualstudio.com/docs/copilot/copilot-customization#_use-instructionsmd-files):

**Split モード**: `.github/prompts/` フォルダに複数のファイル

```
your-project/
├── .github/
│   └── prompts/
│       ├── file1.md
│       └── file2.md
```

**Merged モード**: `.github/copilot-instructions.md` 単一ファイル

```
your-project/
├── .github/
│   └── copilot-instructions.md
```

## 📝 実装タスク

### ✅ Phase 1: ブランチとクリーンアップ

- [x] 新しいブランチ作成（fix/correct-agent-implementation）
- [x] 間違った実装ファイルの修正

### ✅ Phase 2: Cline エージェント正しい実装

- [x] `src/agents/cline.rs` の大幅修正
  - [x] Split モード: `.clinerules/` フォルダに複数ファイル出力
  - [x] Merged モード: `.clinerules` 単一ファイル出力（拡張子なし）
  - [x] ファイル命名規則（数字プレフィックス `01-`, `02-` など）
  - [x] 既存の `.clinerules` ファイル/ディレクトリの適切な処理
- [x] Cline エージェントのテスト修正（8 つのテストケース、全て通過）

### ✅ Phase 3: GitHub エージェント正しい実装

- [x] `src/agents/github.rs` の大幅修正
  - [x] Split モード: `.github/prompts/` フォルダに複数ファイル出力
  - [x] Merged モード: `.github/copilot-instructions.md` 単一ファイル出力
  - [x] ディレクトリの自動作成
- [x] GitHub エージェントのテスト修正（9 つのテストケース、全て通過）

### ✅ Phase 4: 統合とテスト

- [x] 両エージェントの split/merged モード動作確認
- [x] 全テスト通過確認
- [x] lint & format 実行
- [x] 統合テスト（実際のファイル生成確認）

## 📊 テスト結果

### Cline エージェント（8 テスト）

```
test agents::cline::tests::test_get_merged_output_path ... ok
test agents::cline::tests::test_get_split_rules_dir ... ok
test agents::cline::tests::test_generate_merged_empty ... ok
test agents::cline::tests::test_generate_merged_with_content ... ok
test agents::cline::tests::test_generate_split_with_subdirectory ... ok
test agents::cline::tests::test_generate_split_multiple_files ... ok
test agents::cline::tests::test_prepare_rules_directory ... ok
test agents::cline::tests::test_numbered_filename_generation ... ok

test result: ok. 8 passed; 0 failed
```

### GitHub エージェント（9 テスト）

```
test agents::github::tests::test_get_split_prompts_dir ... ok
test agents::github::tests::test_get_merged_output_path ... ok
test agents::github::tests::test_split_vs_merged_output_paths ... ok
test agents::github::tests::test_generate_merged_empty ... ok
test agents::github::tests::test_generate_merged_with_content ... ok
test agents::github::tests::test_generate_creates_pure_markdown ... ok
test agents::github::tests::test_prepare_prompts_directory ... ok
test agents::github::tests::test_generate_split_multiple_files ... ok
test agents::github::tests::test_generate_split_with_subdirectory ... ok

test result: ok. 9 passed; 0 failed
```

## 🔧 動作確認

### Split モード

```bash
# ai-context.yaml で output_mode: split
aicm generate --agent cline
# ✅ .clinerules/01-filename.md, 02-filename.md, ... 生成

aicm generate --agent github
# ✅ .github/prompts/filename.md, ... 生成
```

### Merged モード

```bash
# ai-context.yaml で output_mode: merged
aicm generate --agent cline
# ✅ .clinerules (拡張子なし) 生成

aicm generate --agent github
# ✅ .github/copilot-instructions.md 生成
```

### 全エージェント統合テスト

```bash
aicm generate
# ✅ 4つのエージェント全て正常動作：
# - Cursor: .cursor/rules/*.mdc (split/merged 対応)
# - Cline: .clinerules/ または .clinerules (split/merged)
# - GitHub: .github/prompts/ または .github/copilot-instructions.md (split/merged)
# - Claude: CLAUDE.md (merged のみ)
```

## 🚀 重要な修正点

### 1. 正しい出力パス

- **Cline**:
  - Split: `.clinerules/01-filename.md`, `02-filename.md`
  - Merged: `.clinerules` (拡張子なし)
- **GitHub**:
  - Split: `.github/prompts/filename.md`
  - Merged: `.github/copilot-instructions.md`

### 2. ファイル命名規則

- **Cline**: 数字プレフィックス付き（推奨仕様）
- **GitHub**: 元のファイル名を保持

### 3. コンフリクト解決

- Cline エージェントに既存ファイル/ディレクトリの適切な処理を追加
- merged → split または split → merged の切り替えに対応

## ✅ 作業完了

### 🎯 実装成果

- **GitHub エージェント**: 正しい仕様で完全に再実装
- **Cline エージェント**: 正しい仕様で完全に再実装
- **両モード対応**: split/merged モードを正しくサポート
- **仕様準拠**: 公式ドキュメントに完全準拠
- **包括的テスト**: 合計 17 つのテストケース、全て通過
- **統合成功**: 4 つのエージェント全てが正常動作

### 🎉 ミッション完了

GitHub と Cline エージェントが**正しい仕様**で実装され、公式ドキュメントに完全準拠した設計で統合されました！

**最終成果物**:

- 4 つの AI エージェント（Cursor、Claude、GitHub、Cline）が正しい仕様でサポート
- split/merged モードの適切な使い分け
- 各エージェントの公式ドキュメント準拠
- 完璧なテストカバレッジと品質保証
