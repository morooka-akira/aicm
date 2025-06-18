# Claude Import Files Support

## 作業日
2025-06-18

## 作業内容
Claude Code専用の`import_files`機能を実装。Claude Codeの@filepath記法を使用してファイルを参照し、CLAUDE.mdに出力する。

## 設計方針

### 1. 対象エージェント
- **Claude のみ**：@filepath記法はClaude Code専用機能

### 2. 設定仕様
```yaml
agents:
  claude:
    enabled: true
    import_files:
      - path: "~/.claude/my-project-instructions.md"
        note: "個人のコーディングスタイル設定"
      - path: "./docs/api-reference.md"
        note: "API仕様書"
```

### 3. 出力形式
```markdown
# 個人のコーディングスタイル設定
@../relative/path/to/file.md

# API仕様書
@./docs/api-reference.md
```

### 4. パス解決ルール
- 絶対パス: そのまま使用
- 相対パス: プロジェクトルートからの相対パス
- チルダ記法（~/）: ホームディレクトリへの展開
- **重要**: 出力パスはCLAUDE.mdからの相対パスに変換

### 5. 実装アプローチ
1. `ImportFile`構造体の定義
2. `ClaudeAgentConfig`に`import_files`フィールド追加
3. パス解決ユーティリティ関数の実装
4. Claude エージェントでの処理実装

## 完了要件
1. ✅ 設定型の定義（ImportFile構造体）
2. ✅ ClaudeAgentConfig への import_files フィールド追加
3. ✅ パス解決ユーティリティ関数の実装
4. ✅ ClaudeAgent での import_files 処理実装
5. ✅ テストの作成・実行
6. ✅ ドキュメントの更新
7. ✅ 全テスト通過の確認
8. ✅ cargo fmt, cargo clippy の実行
9. ✅ PR作成

## 実装詳細

### 1. ImportFile構造体
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImportFile {
    pub path: String,
    pub note: Option<String>,
}
```

### 2. ClaudeAgentConfig拡張
```rust
pub struct ClaudeAgentConfig {
    // ... 既存フィールド
    #[serde(default)]
    pub import_files: Vec<ImportFile>,
}
```

### 3. パス解決関数
```rust
fn resolve_import_file_path(
    file_path: &str,
    output_file_path: &Path,
) -> Result<String, Box<dyn Error>>
```

### 4. 出力フォーマット
```rust
fn format_import_file(
    import_file: &ImportFile,
    output_file_path: &Path,
) -> Result<String, Box<dyn Error>>
```

## 検証項目
- [ ] 設定ファイルの正常パース
- [ ] 絶対パス、相対パス、チルダ記法の正しい解決
- [ ] CLAUDE.mdからの相対パス計算
- [ ] Claude エージェントでの正常動作
- [ ] 存在しないファイルでのエラーハンドリング
- [ ] 全テストの通過
- [ ] cargo fmt, cargo clippy の警告なし

## 参考資料
- [Claude Code Memory Documentation](https://docs.anthropic.com/ja/docs/claude-code/memory#claude-md%E3%81%AE%E3%82%A4%E3%83%B3%E3%83%9D%E3%83%BC%E3%83%88)