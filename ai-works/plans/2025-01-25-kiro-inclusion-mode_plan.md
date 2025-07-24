# Kiro Inclusion Mode 対応実装計画

## 概要
Kiro エージェントの steering 機能における inclusion modes に対応する。
ファイルヘッダーに YAML frontmatter 形式で inclusion 設定を追加できるようにする。

## 実装内容

### 1. Inclusion Modes
Kiro は以下の3つの inclusion モードをサポート：

1. **always** (デフォルト)
   - すべての Kiro インタラクションで自動的にロード
   - プロジェクト全体の標準、技術選好、セキュリティポリシーに適用

2. **fileMatch**  
   - 指定されたパターンにマッチするファイルでのみ自動的に含まれる
   - `fileMatchPattern` で対象ファイルパターンを指定

3. **manual**
   - `#filename` で参照することで手動で含まれる
   - 特殊なワークフローやトラブルシューティングガイドに適用

### 2. 設定フォーマット

#### aicm-config.yml での設定例：
```yaml
kiro:
  split_config:
    rules:
      - inclusion: always
        file_patterns: ["*project*"]
      - inclusion: fileMatch
        file_patterns: ["*project*"]
        match_pattern: "**/*.md"
      - inclusion: manual
        file_patterns: ["*project*"]
```

#### 生成される YAML ヘッダー例：
```yaml
---
inclusion: always
---
```

```yaml
---
inclusion: fileMatch
fileMatchPattern: "**/*.md"
---
```

```yaml
---
inclusion: manual
---
```

## 完了要件

1. **設定スキーマの拡張**
   - `KiroConfig` 構造体に `split_config` フィールドを追加
   - `InclusionRule` 構造体の定義
   - 設定のバリデーション

2. **ファイル生成ロジックの実装**
   - 各ファイルに対して適用されるルールの判定
   - YAML frontmatter の生成
   - 既存の内容との結合

3. **テスト**
   - 各 inclusion モードのテストケース
   - ファイルパターンマッチングのテスト
   - YAML ヘッダー生成のテスト

4. **ドキュメント更新**
   - README.md への使用例追加
   - CLAUDE.md への実装詳細追加

## 技術的考慮事項

- ファイルパターンは glob パターンを使用
- 複数のルールがマッチする場合は最初にマッチしたルールを適用
- YAML ヘッダーは各ファイルの先頭に追加
- 既存の Kiro エージェント実装を拡張する形で実装

## リスク

- 既存の Kiro エージェント設定との後方互換性を保つ必要がある
- ファイルパターンのマッチングロジックが複雑になる可能性