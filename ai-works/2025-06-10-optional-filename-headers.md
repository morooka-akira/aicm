# ファイル名ヘッダーオプション化機能

## 作業日
2025-06-10

## 作業概要
mergedオプション時にMarkdownMergerが自動挿入するファイル名ヘッダーをオプション化し、設定でon/offできるようにする。

## 要件

### 現在の動作
- `merge_all()` メソッドで全ファイルを結合する際、各ファイルの前に `# ファイル名` のヘッダーを自動挿入
- このヘッダーは常に表示されており、無効化できない

### 期待する動作
- デフォルトは false（ファイル名ヘッダーを挿入しない）
- グローバル設定で `include_filenames: true` を指定した場合のみヘッダーを挿入
- エージェント個別設定でも上書き可能

## 実装設計

### 1. 設定構造の拡張
グローバル設定とエージェント個別設定に `include_filenames` フィールドを追加：

```yaml
# グローバル設定
version: "1.0"
output_mode: merged
include_filenames: true  # 新規追加
base_docs_dir: ./ai-context

agents:
  claude:
    enabled: true
    include_filenames: false  # エージェント個別設定で上書き
```

### 2. 型定義の拡張
- `AIContextConfig` に `include_filenames: Option<bool>` を追加
- 各エージェント設定型にも `include_filenames: Option<bool>` を追加
- デフォルト値は `false`

### 3. MarkdownMerger の修正
- `merge_all()` メソッドに設定チェック機能を追加
- `include_filenames` が true の場合のみファイル名ヘッダーを挿入
- 後方互換性を維持

### 4. 設定優先順位
1. エージェント個別設定の `include_filenames`
2. グローバル設定の `include_filenames`  
3. デフォルト値（false）

## 実装タスク
1. 設定型の拡張
2. MarkdownMerger の修正
3. 各エージェントでの設定取得機能追加
4. テストケース作成
5. lint・フォーマットチェック
6. ドキュメント更新（必要に応じて）
7. PR作成

## 技術詳細

### 設定取得メソッド
各エージェントで有効な `include_filenames` 設定を取得するヘルパーメソッドを実装：

```rust
impl AIContextConfig {
    pub fn get_effective_include_filenames(&self, agent: &str) -> bool {
        // エージェント個別設定 > グローバル設定 > デフォルト(false)
    }
}
```

### MarkdownMerger の修正
`merge_all()` メソッドでファイル名ヘッダーの挿入を条件分岐：

```rust
if include_filenames {
    merged_content.push_str(&format!("# {}\n\n{}\n\n", relative_path, content.trim()));
} else {
    merged_content.push_str(&format!("{}\n\n", content.trim()));
}
```