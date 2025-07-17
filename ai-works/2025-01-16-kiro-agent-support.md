# Kiro エージェント対応実装

## 日付
2025-01-16

## 作業概要
kiro エージェント対応を追加し、base_docs_dir にあるファイルを `.kiro/steering/` 配下に MD ファイルとして配置する機能を実装する。

## 設計方針

### kiro エージェントの仕様
- **出力先**: `.kiro/steering/`
- **形式**: 純粋な Markdown ファイル
- **モード**: split のみ（kiro の仕様上、ファイルは分割配置）
- **参考URL**: https://kiro.dev/docs/steering/

### kiro の steering 機能について
- ファイルは `.kiro/steering/` ディレクトリに配置
- Markdown（`.md`）形式
- デフォルトで3つのファイルが作成される:
  1. `product.md` - プロダクト概要
  2. `tech.md` - 技術スタック
  3. `structure.md` - プロジェクト構造
- YAML front matter での設定対応
- 3つの包含モード:
  1. "Always" (デフォルト): 毎回読み込み
  2. "fileMatch": 特定ファイルタイプのみ
  3. "manual": チャットで `#filename` により手動参照

### 実装アプローチ
1. **シンプルな実装**: 他の merged モードエージェント（claude, codex, gemini）と同様のパターンを採用
2. **出力モード**: split のみサポート（merged は不要）
3. **設定**: boolean 形式とdetailed形式の両方をサポート
4. **ファイル配置**: base_docs_dir の各 MD ファイルを `.kiro/steering/` にコピー

## 完了要件

### 機能要件
- [ ] agents 設定に kiro エージェントを追加
- [ ] `aicm generate --agent kiro` でファイル生成が成功する
- [ ] 生成されたファイルが `.kiro/steering/` 配下に正しく配置される
- [ ] 既存の他エージェントの動作に影響しない

### コード品質要件
- [ ] Rust コードが `cargo fmt` と `cargo clippy` をパスする
- [ ] 既存テストが全て通る
- [ ] 新しい kiro エージェント用のテストが作成され、通る

### ドキュメント要件
- [ ] README.md に kiro エージェント情報を追加
- [ ] README.ja.md に kiro エージェント情報を追加（日本語）
- [ ] docs/context_spec.md に kiro の仕様を追加
- [ ] ai-context/07_references.md に kiro のリンクを追加

### 技術的要件
- [ ] 型安全性を保つ（設定型の追加）
- [ ] エラーハンドリングの実装
- [ ] 統合テストによる動作確認

## 実装ステップ

### Phase 1: 基盤実装
1. `types/config.rs` に kiro エージェント設定を追加
2. `agents/kiro.rs` を作成（基本的な generate 実装）
3. `main.rs` に kiro エージェントのジェネレーター呼び出しを追加

### Phase 2: テスト実装
4. kiro エージェント用のテストを作成
5. 統合テストを実行して全体動作確認

### Phase 3: ドキュメント更新
6. README.md, README.ja.md を更新
7. docs/context_spec.md を更新
8. ai-context/07_references.md を更新

### Phase 4: 品質保証
9. `cargo fmt`, `cargo clippy` でコード品質チェック
10. 全テスト実行による品質確認

### Phase 5: 統合
11. PR 作成

## 技術的詳細

### 参考実装
既存の gemini エージェント実装を参考にする（最もシンプルで類似の要件）

### ファイル構造
```
.kiro/steering/
├── 01_project-overview.md
├── 02_project-security.md 
├── 03_project-architecture.md
├── 04_development-setup.md
├── 05_development-rules.md
├── 06_rust-rule.md
└── 07_references.md
```

### 設定例
```yaml
# シンプル設定
agents:
  kiro: true

# 詳細設定
agents:
  kiro:
    enabled: true
    base_docs_dir: ./ai-context  # オプショナル
```

## 注意事項
- kiro は split モードのみサポート（merged モードは実装しない）
- 既存エージェントとの設定互換性を保つ
- エラーハンドリングを適切に実装する
- テストカバレッジを確保する

## 参考資料
- [Kiro Steering Documentation](https://kiro.dev/docs/steering/)
- 既存エージェント実装: `src/agents/gemini.rs`
- 設定型定義: `src/types/config.rs`