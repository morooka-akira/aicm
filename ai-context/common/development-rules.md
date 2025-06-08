# 開発ルール

## テスト要件
- **必須**: 各モジュールはRust標準テストフレームワークでテストを作成すること
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

## コード品質
- **rustfmt**: 統一されたコードフォーマット
- **clippy**: 高品質なRustコードのためのリンター
- **型安全性**: Rustの強力な型システムを活用
- **エラーハンドリング**: anyhow・thiserrorによる適切なエラー処理