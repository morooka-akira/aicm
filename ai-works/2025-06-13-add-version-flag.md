# 2025-06-13 バージョンフラグ追加作業

## 作業内容
`aicm -v` または `aicm --version` コマンドでCargo.tomlのversionを出力する機能を追加する。

## 設計方針
- clap の derive API を使用してバージョンフラグを追加
- Cargo.tomlのversionフィールド（現在: 0.1.2）を自動取得
- `cargo::env!("CARGO_PKG_VERSION")` マクロを使用

## 完了要件
1. `aicm -v` コマンドでバージョンが出力される
2. `aicm --version` コマンドでバージョンが出力される
3. 出力内容はCargo.tomlのversionと一致する（0.1.2）
4. 既存の機能に影響を与えない
5. テストが通る
6. cargo fmt、cargo clippy が通る