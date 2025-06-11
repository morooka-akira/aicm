# Cargo publishの除外対象追加

## 作業概要

Cargo.tomlにcargo publish時の除外対象ディレクトリ・ファイルを追加する。

## 除外対象

- `ai-works/` - 開発作業記録（開発用のみ）
- `ai-context/` - プロジェクト用ドキュメント（開発用のみ）
- `docs/` - 設計ドキュメント（開発用のみ）
- `ai-context.yaml` - サンプル設定ファイル（開発用のみ）

## 設計方針

これらのファイルは開発・保守用のものであり、エンドユーザーには不要なため、パッケージから除外してパッケージサイズを削減する。

## 実装手順

1. 作業記録を作成
2. Cargo.tomlにexcludeセクションを追加
3. cargo fmt, cargo clippyを実行
4. 変更をコミット・PR作成

## 完了要件

- [ ] Cargo.tomlにexcludeが追加される
- [ ] 指定されたディレクトリ・ファイルがすべて除外対象になる
- [ ] cargo fmt, cargo clippyが正常に通る