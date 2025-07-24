# Kiro Inclusion Mode 対応 TODO リスト

## 進捗状況

### 完了タスク
- [x] ai-works/plans ディレクトリに設計計画ドキュメントを作成
- [x] ai-works/todos ディレクトリにTODOリストを作成

### 実装タスク

#### 1. 設定構造体の拡張
- [x] `src/types/config.rs` で `KiroConfig` 構造体に `split_config` フィールドを追加
- [x] `InclusionRule` 構造体を定義
  - `inclusion: InclusionMode` (enum: Always, FileMatch, Manual)
  - `file_patterns: Vec<String>`
  - `match_pattern: Option<String>` (fileMatch の場合のみ)
- [x] `InclusionMode` enum を定義

#### 2. 設定スキーマの更新
- [x] `KiroConfig` の serde デシリアライズ設定を追加
- [x] デフォルト値の設定（後方互換性のため）
- [x] バリデーションロジックの追加

#### 3. Kiro エージェント実装の更新
- [x] `src/agents/kiro.rs` の `generate` メソッドを更新
- [x] ファイルパターンマッチングロジックの実装
- [x] YAML frontmatter 生成ロジックの実装
- [x] 生成されたヘッダーとコンテンツの結合

#### 4. ユーティリティ関数
- [x] ファイルパターンマッチング関数の実装
- [x] YAML ヘッダー生成関数の実装
- [x] ルール適用優先順位の判定ロジック

#### 5. テスト
- [x] `KiroConfig` のデシリアライズテスト
- [x] ファイルパターンマッチングのユニットテスト
- [x] 各 inclusion モードの統合テスト
  - always モードのテスト
  - fileMatch モードのテスト
  - manual モードのテスト
- [x] 複数ルールの優先順位テスト

#### 6. 品質保証
- [x] cargo fmt の実行
- [x] cargo clippy の実行（warning もすべて修正）
- [x] cargo test の実行

#### 7. ドキュメント更新
- [x] README.md に Kiro inclusion mode の使用例を追加
- [ ] docs/concept.md に Kiro エージェントの詳細を追加
- [x] CLAUDE.md への変更確認

## 注意事項

- 既存の Kiro 設定との後方互換性を保つ
- ファイルパターンは glob パターンを使用
- 複数のルールがマッチする場合は配列の順序で最初にマッチしたものを適用