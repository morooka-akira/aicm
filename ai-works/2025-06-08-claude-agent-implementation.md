# Claude エージェント実装作業記録

## 📅 作業日

2025-06-08

## 🎯 作業目標

Claude Code 用の CLAUDE.md を出力する機能を実装する

## 📋 要件

- **シンプル化原則に従う**: 余計な機能は実装しない
- **既存の Cursor エージェントと一貫性**: 同じ抽象度で実装
- **merged モードのみ**: simplification-plan.md の通り Claude は merged のみ
- **出力先**: `CLAUDE.md` (プロジェクトルート)
- **テスト作成**: 必須
- **ドキュメント更新**: docs 配下の更新
- **PR 作成**: 完了後に PR 作成

## 🔍 既存実装の分析

### Cursor エージェントの構造

- `CursorAgent::new(config)` でインスタンス作成
- `generate()` メソッドで `Vec<GeneratedFile>` を返す
- `OutputMode::Merged` と `OutputMode::Split` に対応
- `.cursor/rules/` ディレクトリに出力
- MDC 形式（YAML frontmatter + Markdown）

### Claude エージェントの設計方針

- **シンプル**: merged モードのみ対応
- **出力先**: `CLAUDE.md` (ルート)
- **フォーマット**: 純粋な Markdown（frontmatter なし）
- **一貫性**: 同じインターフェース（`new` + `generate`）

## 📝 実装タスク

### Phase 1: Claude エージェント実装

- [x] `src/agents/claude.rs` を作成
- [x] `ClaudeAgent` 構造体と実装
- [x] `generate()` メソッド（merged のみ）
- [x] テスト作成

### Phase 2: 統合

- [x] `src/agents/mod.rs` に Claude エージェント追加
- [x] `src/main.rs` の `generate_agent_files` に Claude 追加

### Phase 3: テスト

- [x] 単体テスト
- [x] 統合テスト確認

### Phase 4: ドキュメント更新

- [x] `docs/` 配下の関連ドキュメント更新

### Phase 5: PR 作成

- [x] ブランチ作成
- [x] 実装コミット
- [ ] PR 作成

## 🚨 注意事項

- **YAGNI 原則**: 今必要でない機能は実装しない
- **テスト必須**: 作業完了時にテストが通ることを確認
- **コード品質**: rustfmt と clippy を実行
- **一貫性**: 既存の Cursor エージェントと同じパターンを踏襲

## 📈 期待される動作

```bash
# ai-context.yaml で claude: true にして
aicm generate

# または特定のエージェントのみ
aicm generate --agent claude
```

**出力**: プロジェクトルートに `CLAUDE.md` が生成される
**内容**: `docs/` 配下の全 `.md` ファイルを結合した純粋な Markdown

---
