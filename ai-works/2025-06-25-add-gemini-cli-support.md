# Gemini CLI サポート追加 - 2025-06-25

## 作業内容

Gemini CLI（https://github.com/google-gemini/gemini-cli）サポートを aicm に追加する

## 設計方針

- Gemini CLI は OpenAI Codex と同様にマージモードのみサポート
- 出力ファイル名は `GEMINI.md`
- 既存の Codex エージェントの実装をベースとして実装
- split モードは存在しないため、merged モードのみ対応

## 実装要件

### 1. 新しいエージェント実装
- `src/agents/gemini.rs` を作成
- `AgentTrait` を実装
- merged モードでの `GEMINI.md` 出力

### 2. 設定追加
- `types/config.rs` に gemini 設定を追加
- YAML 設定での gemini エージェント有効化

### 3. CLI 統合
- `main.rs` での gemini エージェント対応
- `--agent gemini` オプション対応

### 4. テスト追加
- `gemini.rs` のユニットテスト
- 統合テスト対応

### 5. ドキュメント更新
- README.md の対応ツール一覧更新
- README.ja.md の対応ツール一覧更新
- `ai-context/07_references.md` にGemini CLI のリンク追加

## 完了要件

- [ ] Gemini エージェント実装完了
- [ ] 設定ファイル対応完了
- [ ] CLI コマンド対応完了
- [ ] テスト追加・実行完了
- [ ] ドキュメント更新完了
- [ ] cargo fmt, cargo clippy 実行完了
- [ ] PR 作成完了

## 技術仕様

- 出力ファイル: `GEMINI.md`
- 出力モード: merged のみ
- 基本的な動作は codex エージェントと同様
- import_files などの高度な機能は不要（シンプルな実装）