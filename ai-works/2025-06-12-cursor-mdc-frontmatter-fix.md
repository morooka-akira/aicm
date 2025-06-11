# Cursor MDC ファイルの frontmatter 生成修正

## 作業概要

CursorのMDCファイルのfrontmatter生成を修正し、適切なフォーマットで出力されるようにする。

## 現在の問題

現在のfrontmatter生成では以下の問題がある：

1. **Always Apply**の場合、不要な空行がある
2. **Agent Requested**で`globs`や`alwaysApply`が出力されない
3. **Auto Attached**で`description`や`alwaysApply`が出力されない  
4. **Manual**で必要なキーが出力されない

## 理想的なフォーマット

### Always Apply
```yaml
---
description:
globs:
alwaysApply: true
---
```

### Agent Requested
```yaml
---
description: Rustコードを実装する開発作業
globs:
alwaysApply: false
---
```

### Auto Attached
```yaml
---
description:
globs: ["*.rs", "*.toml"]
alwaysApply: false
---
```

### Manual
```yaml
---
description:
globs:
alwaysApply: false
---
```

## 設計方針

1. **共通フォーマット**: 全てのルールタイプで`description`, `globs`, `alwaysApply`を含める
2. **値の制御**: ルールタイプに応じて適切な値を設定
3. **空行削除**: 不要な空行を除去
4. **一貫性**: YAMLの構造を統一

## 実装手順

1. 現在のCursor MDCファイル生成処理を調査
2. frontmatter生成関数を修正
3. テストケースを更新・追加
4. テスト実行とlint/format
5. 変更をコミット・PR作成

## 完了要件

- [ ] Always Applyルールで正しいfrontmatterが生成される
- [ ] Agent Requestedルールで正しいfrontmatterが生成される
- [ ] Auto Attachedルールで正しいfrontmatterが生成される
- [ ] Manualルールで正しいfrontmatterが生成される
- [ ] 無駄な空行が除去される
- [ ] 関連するテストが全て通る
- [ ] `cargo fmt`と`cargo clippy`が正常に通る