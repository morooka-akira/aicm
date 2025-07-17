# 作業記録: uninlined_format_args clippy警告の修正

## 作業内容
clippy の `uninlined_format_args` 警告を修正し、format! マクロを新しいインライン形式に更新する。

## 設計方針
- `format!("{}", var)` 形式を `format!("{var}")` 形式に変更
- 全ての該当箇所を一括で修正
- コードの動作は変更しない（フォーマットのみの変更）

## 対象ファイル
1. src/agents/base.rs (lines 27, 117, 119)
2. src/agents/cline.rs (lines 80, 83, 94, 103)
3. src/agents/cursor.rs (lines 60, 93, 109, and others)
4. src/agents/github.rs (multiple lines)
5. src/agents/kiro.rs (line 47 - already fixed)
6. src/config/error.rs (line 81)

## 完了要件
- [x] 全ての対象ファイルで format! マクロをインライン形式に修正
- [x] cargo fmt の実行
- [x] cargo clippy の実行（uninlined_format_args警告が消えることを確認）
- [x] cargo test の実行（テストが通ることを確認）

## 結果
- uninlined_format_args 警告は完全に消去された
- 全ての単体テスト・統合テストが正常に通過
- コードフォーマットも正常に適用済み

## 修正内容
以下のファイルで `format!("{}", var)` を `format!("{var}")` 形式に修正:

1. src/agents/base.rs - 3箇所修正
2. src/agents/cline.rs - 4箇所修正  
3. src/agents/cursor.rs - 13箇所修正
4. src/agents/github.rs - 9箇所修正
5. src/agents/claude.rs - 1箇所修正
6. src/config/error.rs - 1箇所修正
7. src/core/markdown_merger.rs - 1箇所修正