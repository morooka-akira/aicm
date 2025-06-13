# 無効化されたエージェントの出力ファイル削除機能実装

## 作業内容

現在のaicmでは、設定ファイルでエージェントを無効化（例：`codex: false`）しても、既に生成された出力ファイルが削除されない問題がある。この問題を解決するため、無効化されたエージェントの出力ファイルを自動的に削除する機能を実装する。

## 設計方針

### 1. 問題の詳細

- `aicm generate`実行時に、有効なエージェントのファイルのみ生成される
- 無効化されたエージェントの既存出力ファイルはそのまま残存する
- これにより、古い設定ファイルが残って混乱を招く可能性がある

### 2. 解決方針

- `aicm generate`実行時に、無効化されたエージェントの出力ファイルを検出・削除する
- 削除前にユーザーに確認を求める（オプションで自動削除も可能）
- 削除対象ファイルのログ出力を行う

### 3. 実装アプローチ

1. **ファイル検出機能**: 各エージェントの出力パスを取得する機能
2. **削除判定機能**: エージェントが無効化されているかを判定する機能
3. **ファイル削除機能**: 安全にファイル・ディレクトリを削除する機能
4. **ユーザー確認機能**: 削除前の確認プロンプト（オプション）

### 4. 技術的考慮事項

- エージェントごとに異なる出力パス形式への対応
- Split/Mergedモードでの出力パス差異の処理
- ディレクトリが空になった場合の削除判定
- エラーハンドリング（ファイルが存在しない、権限エラーなど）

## 完了要件

1. 無効化されたエージェントの出力ファイルが自動削除される
2. 削除前にユーザーに確認を求める機能
3. 削除されたファイルのログ出力
4. 全エージェント（Cursor, Cline, GitHub, Claude, Codex）に対応
5. Split/Mergedモード両方に対応
6. 適切なエラーハンドリング
7. 包括的なテストケース
8. 既存機能に影響しない

## コードベース調査結果

### 各エージェントの出力ファイル構造

| エージェント | Mergedモード | Splitモード | 削除対象 |
|-------------|-------------|------------|----------|
| Cursor | `.cursor/rules/context.mdc` | `.cursor/rules/*.mdc` | `.cursor/rules/`内の全`.mdc`ファイル |
| Cline | `.clinerules` (ファイル) | `.clinerules/*.md` | モード切替時の相互削除 |
| GitHub | `.github/copilot-instructions.md` | `.github/instructions/*.instructions.md` | モード切替時の相互削除 |
| Claude | `CLAUDE.md` | N/A | 単一ファイル（削除不要） |
| Codex | `AGENTS.md` | N/A | 単一ファイル（削除不要） |

### 現在の削除処理

- 各エージェントは生成前に既存ファイルの削除処理を実装済み
- `prepare_rules_directory()`等のメソッドで削除処理を実行
- ただし、これは有効なエージェントのみが実行される

## 詳細設計

### 1. アーキテクチャ

新しいモジュール `src/cleanup.rs` を作成し、エージェント削除処理を統一管理する。

```rust
// src/cleanup.rs
pub struct AgentCleaner;

impl AgentCleaner {
    pub async fn cleanup_disabled_agents(
        config: &AicmConfig,
        base_dir: Option<&Path>
    ) -> Result<Vec<String>, anyhow::Error>;
    
    fn get_agent_output_paths(agent: &str, config: &AicmConfig, base_dir: Option<&Path>) -> Vec<PathBuf>;
    fn cleanup_agent_files(paths: &[PathBuf]) -> Result<Vec<String>, anyhow::Error>;
}
```

### 2. 実装方針

1. **generate コマンド拡張**: `main.rs` の `generate_agent_files()` に削除処理を追加
2. **エージェント判定**: 設定で無効化されたエージェントを特定
3. **パス生成**: 各エージェントの出力パスを生成
4. **ファイル削除**: 存在するファイル/ディレクトリを安全に削除
5. **ログ出力**: 削除されたファイルをユーザーに通知

### 3. 各エージェントの削除ロジック

#### Cursor
```rust
fn cleanup_cursor_files(base_dir: Option<&Path>) -> Result<Vec<String>, anyhow::Error> {
    let rules_dir = base_dir.unwrap_or_else(|| Path::new(".")).join(".cursor/rules");
    // .mdcファイルを削除、ディレクトリが空なら削除
}
```

#### Cline
```rust
fn cleanup_cline_files(base_dir: Option<&Path>) -> Result<Vec<String>, anyhow::Error> {
    // .clinerules ファイルと .clinerules/ ディレクトリの両方をチェック
    // 存在するものを削除
}
```

#### GitHub
```rust
fn cleanup_github_files(base_dir: Option<&Path>) -> Result<Vec<String>, anyhow::Error> {
    // .github/copilot-instructions.md と .github/instructions/ の両方をチェック
    // 存在するものを削除
}
```

### 4. エラーハンドリング

- ファイル削除エラーは警告として扱い、処理を継続
- 権限エラーや存在しないファイルは無視
- 削除成功・失敗を分けてログ出力

## 実装予定

- [x] 現在のコードベース調査
- [ ] 設計詳細の決定
- [ ] cleanup.rs モジュール実装
- [ ] main.rs への統合
- [ ] テスト作成・実行
- [ ] コードレビュー対応
- [ ] PR作成