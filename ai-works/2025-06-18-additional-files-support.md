# Additional Files Support for AI Agents

## 作業日
2025-06-18

## 作業内容
各AI エージェント（claude、cursor、cline、github、codex）に `additional_files` 設定を追加し、base_docs_dir以外のファイルを参照・出力できるようにする機能を実装。

## 設計方針
1. **設定仕様**：
   - `additional_files` フィールドを各エージェント設定に追加
   - `path` と `note` を持つオブジェクトの配列として実装
   - パスは絶対パス、相対パス、チルダ記法（~）をサポート

2. **出力形式**：
   - ファイル参照は `@filepath` 形式で出力
   - noteがある場合は `# note\n@filepath` 形式で出力
   - 出力パスは生成されるファイルからの相対パスに変換

3. **実装アプローチ**：
   - 設定型に `AdditionalFile` 構造体を追加
   - `AgentDetailConfig` に `additional_files` フィールドを追加
   - 各エージェントの generate メソッドで additional_files を処理
   - パス解決ロジックを共通化

## 完了要件
1. 設定型の拡張（AdditionalFile、AgentDetailConfig）
2. パス解決ユーティリティ関数の実装
3. 各エージェントでの additional_files 処理実装
4. 設定バリデーション機能の追加
5. テストの作成・実行
6. ドキュメントの更新
7. 全テスト通過の確認
8. cargo fmt, cargo clippy の実行
9. PR作成

## 実装詳細

### 1. 設定型の定義
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdditionalFile {
    pub path: String,
    pub note: Option<String>,
}
```

### 2. AgentDetailConfig への追加
```rust
pub struct AgentDetailConfig {
    // ... 既存フィールド
    #[serde(default)]
    pub additional_files: Vec<AdditionalFile>,
}
```

### 3. パス解決関数
```rust
fn resolve_additional_file_path(
    file_path: &str,
    output_file_path: &Path,
) -> Result<String, Box<dyn Error>>
```

### 4. 出力フォーマット
```rust
fn format_additional_file(
    additional_file: &AdditionalFile,
    output_file_path: &Path,
) -> Result<String, Box<dyn Error>>
```

## 検証項目
- [ ] 設定ファイルの正常パース
- [ ] 絶対パス、相対パス、チルダ記法の正しい解決
- [ ] 出力ファイルからの相対パス計算
- [ ] 各エージェントでの正常動作
- [ ] 存在しないファイルでのエラーハンドリング
- [ ] 全テストの通過
- [ ] cargo fmt, cargo clippy の警告なし