# 開発ルール

## タスクの進め方

- 1. 作業開始時に、`ai-works` ディレクトリに `yyyy-mm-dd-<work name>.md` を作成し、作業内容、設計方針、完了要件をまとめてください
  - 中断を指示された場合は、それに従ってください
- 2. ルールに従って、作業を実行
- 3. rust コードに修正がある場合は、`cargo fmt`, `cargo clippy` を実行してください
  - `cargo clippy` は warning でも修正可能であれば修正してください
- 4. 修正内容が、 @README.md @docs/ の内容と乖離がある場合は、ドキュメントの更新を行ってください
- 5. ルールに従って、PR を作成

## テスト要件

- **必須**: 各モジュールは Rust 標準テストフレームワークでテストを作成すること
- **カバレッジ**: 主要な機能とエラーパスのテストを含めること
- **作業完了**: 作業終了時は必ずテストが通ることを確認すること

## テスト実行例

```bash
# 全テスト実行
cargo test

# 特定のテストモジュール実行
cargo test config
cargo test agents::cursor

# テストカバレッジ（tarpaulin要インストール）
cargo install cargo-tarpaulin
cargo tarpaulin --out html

# 統合テスト実行
cargo test --test integration_test
```

## Git 運用

- デフォルトブランチは `main`
- 開発作業については、必ずデフォルトブランチからブランチを分けて作業すること
- 作業完了後は、gh コマンドで PR を作成すること

## エラーハンドリング

- anyhow・thiserror による適切なエラー処理を行うこと
