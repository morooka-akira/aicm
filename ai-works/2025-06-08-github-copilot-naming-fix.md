# GitHub Copilot 正しいファイル命名規則修正

## 📅 作業日

2025-06-08

## 🎯 作業目標

GitHub Copilot エージェントのファイル命名規則を VS Code Copilot Customization の公式仕様に準拠させる

## 📋 問題点

### 現在の実装（間違い）

- **統合モード**: `.github/copilot-instructions.md` ✅ 正しい
- **分割モード**: `.github/instructions/*.instructions.md` ❌ 間違い

### 正しい仕様

VS Code Copilot Customization ドキュメント（https://code.visualstudio.com/docs/copilot/copilot-customization）によると：

1. **Instructions ファイル**:

   - 統合: `.github/copilot-instructions.md`
   - 分割: `.github/instructions/*.instructions.md`

2. **Prompt ファイル（実験的機能）**:
   - 分割: `.github/prompts/*.prompt.md`

### 現在のコードの問題

- 分割モードで `.instructions.md` ファイルを生成しているが、これは正しい
- しかし、ドキュメントコメントで「`*.prompt.md`」と間違った説明をしている

## 🔍 修正内容

### 1. ドキュメントコメント修正

`src/agents/github.rs` のコメントを正しい仕様に修正：

```rust
/*!
 * AI Context Management Tool - GitHub Copilot Agent
 *
 * GitHub Copilot用のコンテキストファイル生成エージェント
 * 仕様: https://code.visualstudio.com/docs/copilot/copilot-customization
 *
 * ファイル命名規則:
 * - 統合モード: .github/copilot-instructions.md
 * - 分割モード: .github/instructions/*.instructions.md
 */
```

### 2. 設定オプション追加検討

将来的に Prompt ファイル（`.prompt.md`）もサポートする場合の設計を検討

## 📝 修正タスク

### ✅ Phase 1: ドキュメントコメント修正

- [x] `src/agents/github.rs` のヘッダーコメント修正
- [x] 実装は既に正しいので変更不要

### ✅ Phase 2: ドキュメント更新

- [x] `docs/` 配下のドキュメント更新
- [x] `README.md` の説明修正

### ✅ Phase 3: テスト・動作確認

- [x] 既存テストが通ることを確認
- [x] 実際のファイル生成確認

## 🎯 期待される成果

1. ドキュメントと実装の一致
2. VS Code Copilot Customization 公式仕様への完全準拠
3. 将来の Prompt ファイル機能拡張への準備

## ✅ 作業完了確認

- [ ] ドキュメントコメント修正
- [ ] 関連ドキュメント更新
- [ ] テスト通過確認
- [ ] PR 作成
