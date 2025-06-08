# このプロジェクトの技術スタック

## 使用技術
- **言語**: Rust
- **フレームワーク**: Tokio（非同期処理）
- **CLI**: clap（derive API）
- **設定**: serde_yaml
- **エラーハンドリング**: anyhow + thiserror

## プロジェクト特有のルール
- 非同期処理は必ずTokioを使用
- エラーは構造化エラー型（thiserror）で定義
- 設定ファイルはYAML形式のみ
- テストは#[tokio::test]を使用

## 禁止事項
- unwrap()の使用禁止（テスト以外）
- std::thread::spawn()の直接使用禁止
- blocking I/Oの使用禁止