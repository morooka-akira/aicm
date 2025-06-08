# Cursor split_config機能実装計画

## 📅 作業日
2025-06-09

## 🎯 作業目標
Cursorエージェントのsplitモード時に、より詳細なルール設定を可能にするsplit_config機能を実装

## 📋 要件分析

### 現在の仕様（シンプル）
```yaml
agents:
  cursor:
    output_mode: split  # split時は全ファイルに同じ設定を適用
```

### 提案する新仕様（split_config対応）
```yaml
agents:
  cursor:
    output_mode: split
    split_config:
      common-rules:
        globs: ["**/*.rs", "**/*.ts"]
        type: always
        description: "共通のコーディングルール"
      react-rules:
        globs: ["**/*.tsx", "**/*.jsx"]
        type: auto_attached
        description: "React コンポーネント用ルール"
      manual-rules:
        type: manual
        description: "手動参照用の特殊ルール"
```

## 🔍 Cursor Rulesドキュメント分析

### ルールタイプ
1. **always**: 常にモデルコンテキストに含まれる（alwaysApply: true）
2. **auto_attached**: globパターンにマッチするファイルが参照時に含まれる
3. **agent_requested**: AIが必要と判断時に含まれる（descriptionが必要）
4. **manual**: @ruleNameで明示的に言及時のみ含まれる

### MDC形式
```markdown
---
description: RPC Service boilerplate
globs: ["**/*.ts"]
alwaysApply: false
---

ルールの内容...

@service-template.ts
```

## 🎯 設計方針

### 1. シンプル性重視
- 既存のsplitモードとの後方互換性維持
- split_configは任意設定
- デフォルトは現在の動作を維持

### 2. 段階的実装
- Phase 1: 基本的なtype/globs対応
- Phase 2: 詳細設定（description等）
- Phase 3: ファイル参照機能（@filename）

### 3. 型安全性
- Rust型システムを活用した設定検証
- serde deserializationでの型チェック

## 📝 実装計画

### Phase 1: 基本実装

#### 1.1 型定義追加
```rust
// src/types/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorSplitRule {
    pub globs: Option<Vec<String>>,
    pub rule_type: CursorRuleType,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorRuleType {
    Always,
    AutoAttached,
    AgentRequested,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorConfig {
    pub enabled: Option<bool>,
    pub output_mode: Option<OutputMode>,
    pub split_config: Option<HashMap<String, CursorSplitRule>>,
}
```

#### 1.2 設定ファイル仕様拡張
```yaml
# ai-context.yaml
agents:
  cursor:
    output_mode: split
    split_config:
      common-rules:
        globs: ["**/*.rs", "**/*.ts"]
        rule_type: always
        description: "共通のコーディングルール"
      react-specific:
        globs: ["**/*.tsx"]
        rule_type: auto_attached
        description: "React用ルール"
```

#### 1.3 Cursorエージェント実装更新
- split_config設定時の分割ファイル生成ロジック
- ルールタイプに応じたMDCメタデータ生成
- globs設定の適用

### Phase 2: 詳細機能

#### 2.1 高度な設定オプション
- ファイル参照機能（@filename）
- ネストルール対応
- カスタムメタデータ

#### 2.2 バリデーション強化
- globs パターン検証
- rule_type と description の組み合わせ検証
- 循環参照チェック

### Phase 3: 最適化

#### 3.1 パフォーマンス改善
- globs マッチング最適化
- ファイル生成の並列化

#### 3.2 エラーハンドリング
- 設定エラーの詳細メッセージ
- 部分的な設定失敗時の graceful fallback

## 🔧 実装の優先順位

### 高優先度
1. 基本的なsplit_config型定義
2. CursorRuleType enum実装
3. split時の条件分岐ロジック
4. 基本的なMDCメタデータ生成

### 中優先度
1. globs パターンマッチング
2. description設定対応
3. テストケース作成

### 低優先度
1. ファイル参照機能（@filename）
2. 高度なバリデーション
3. パフォーマンス最適化

## 📋 実装タスク

### ✅ Phase 1a: 設計・計画
- [x] 作業計画作成（このファイル）
- [ ] 現在のCursorエージェント実装確認
- [ ] 型定義設計詳細化

### 🔄 Phase 1b: 基本実装
- [ ] CursorSplitRule型定義追加
- [ ] CursorRuleType enum追加
- [ ] CursorConfig構造体更新
- [ ] split_config デシリアライゼーション実装

### 🔄 Phase 1c: エージェント更新
- [ ] Cursorエージェントのsplit_config対応
- [ ] MDCメタデータ生成ロジック
- [ ] 条件分岐による分割ルール適用

### 🔄 Phase 1d: テスト・検証
- [ ] ユニットテスト作成
- [ ] 実動作確認
- [ ] ドキュメント更新

## 🎯 期待される成果

1. **柔軟なルール設定**: プロジェクトの構造に応じた細かなルール制御
2. **Cursor仕様準拠**: 公式ドキュメントに沿った正確な実装
3. **シンプル性維持**: 複雑さを感じさせない直感的な設定
4. **後方互換性**: 既存設定への影響なし

## 📖 参考資料

- [Cursor Rules公式ドキュメント](https://docs.cursor.com/context/rules)
- [MDC形式仕様](https://docs.cursor.com/context/rules#rule-file-format)
- [Globs パターン仕様](https://docs.rs/glob/)

## 🚀 次のステップ

1. 現在のCursorエージェント実装確認
2. 型定義の詳細設計
3. 最小実装でのプロトタイプ作成
4. テスト駆動での段階的機能追加