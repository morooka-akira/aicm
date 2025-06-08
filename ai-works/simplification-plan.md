# AI Context Management Tool - シンプル化実行計画

## 🎯 目標

**複雑な実装を削除し、シンプルで理解しやすい仕様に変更する**

## 🚨 重要原則

- **YAGNI**: 今必要でない機能は実装しない
- **シンプル第一**: 1つの機能は1つの責務のみ
- **理解しやすさ**: コードを読めば仕様が分かる
- **余計な抽象化禁止**: 過度な一般化は行わない

## 📋 新仕様（シンプル版）

### 1. `aicm init`

```bash
aicm init
```

**動作:**
1. `ai-context.yaml` が存在するかチェック
2. 存在しない場合、デフォルト設定ファイルを生成
3. `base_docs_dir` で指定されたディレクトリを作成（存在する場合は何もしない）

**生成される ai-context.yaml:**
```yaml
version: "1.0"
output_mode: merged  # merged | split
base_docs_dir: "./docs"
agents:
  cursor: true      # 有効/無効のみ
  cline: false
  github: false
  claude: false
```

**作成されるディレクトリ:**
```
docs/
├── README.md     # 使い方説明
```

### 2. ドキュメント管理

**`docs/` 配下に自由にMarkdownファイルを配置**
- ファイル名は任意
- ディレクトリ構造も任意
- 全ての `.md` ファイルが自動的に読み込まれる

**例:**
```
docs/
├── coding-rules.md
├── project-info.md
├── security/
│   ├── auth.md
│   └── data-protection.md
└── development/
    ├── setup.md
    └── testing.md
```

### 3. `aicm generate`

```bash
aicm generate
```

**merged モード:**
- `docs/` 内の全 `.md` ファイルを読み込み
- 1つのファイルに結合
- 有効なエージェント用のファイルを生成

**split モード:**
- `docs/` 内の各 `.md` ファイルを個別に処理
- 各ファイルごとに分割して出力

**出力先:**
- **Cursor**: `.cursor/rules/context.mdc` (merged) / `.cursor/rules/*.mdc` (split)
- **Cline**: `.clinerules/rules.md` (merged) / `.clinerules/*.md` (split)
- **GitHub**: `instructions.md` (merged) / `**/instructions.md` (split)
- **Claude**: `CLAUDE.md` (merged only)

## 🗑️ 削除する複雑な機能

### 削除対象
1. **file_mapping** - 自動検出に変更
2. **project_specific** - 単純な全ファイル結合に変更
3. **agent_specific** - エージェント個別ファイルは不要
4. **split_config** - シンプルなファイル分割のみ
5. **global_variables** - テンプレート機能は削除
6. **CursorRuleConfig** - 複雑なルール設定は削除
7. **validation機能** - 基本的なファイル存在チェックのみ

### 残す機能
1. **基本的なファイル読み込み**
2. **Markdownファイル結合**
3. **エージェント別出力**
4. **merged/split モード**

## 📝 実装タスク

### Phase 1: 型定義の簡素化

**ファイル: `src/types/config.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIContextConfig {
    pub version: String,
    pub output_mode: OutputMode,
    pub base_docs_dir: String,
    pub agents: AgentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputMode {
    Merged,
    Split,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub cursor: bool,
    pub cline: bool,
    pub github: bool,
    pub claude: bool,
}
```

### Phase 2: Markdown処理の簡素化

**ファイル: `src/core/markdown_merger.rs`**

- **削除**: file_mapping機能
- **追加**: 自動ファイル検出（`**/*.md`でglob検索）
- **簡素化**: 単純な文字列結合のみ

### Phase 3: エージェント実装の簡素化

**ファイル: `src/agents/cursor.rs`**

- **削除**: split_config、CursorRuleConfig
- **簡素化**: merged/splitの2パターンのみ
- **統一**: 全エージェントで同じインターフェース

### Phase 4: CLI実装の簡素化

**ファイル: `src/main.rs`**

- **簡素化**: init、generate、validateコマンドのみ
- **削除**: list-agents（設定ファイルを見れば分かる）

### Phase 5: テストの簡素化

**ファイル: `tests/`**

- **焦点**: 実際のユースケースのみをテスト
- **削除**: 過度に細かいユニットテスト
- **強化**: 統合テスト（実際のファイル生成）

### Phase 6: ドキュメント更新

**更新対象:**
- `docs/concept.md` - 新仕様に合わせて全面書き直し
- `docs/design_doc.md` - シンプルな設計に更新
- `docs/requirements.md` - 要件を簡素化
- `docs/guide.md` - 使い方ガイドを新仕様に対応
- `README.md` - プロジェクト説明を更新

## 🎯 期待される効果

### 開発者体験の向上
- **理解しやすい**: 5分で全体が把握できる
- **デバッグしやすい**: 問題の原因が特定しやすい
- **拡張しやすい**: 新機能追加が簡単

### ユーザー体験の向上
- **設定が簡単**: 最小限の設定で動作
- **使いやすい**: 直感的な操作
- **予測可能**: 動作結果が予想できる

### 保守性の向上
- **少ないコード**: バグが入りにくい
- **明確な責務**: 各モジュールの役割が明確
- **テストしやすい**: シンプルなテストで十分なカバレッジ

## ⚠️ 実装時の注意事項

1. **段階的削除**: 一度に全てを削除せず、機能ごとに段階的に
2. **テスト維持**: 削除前に既存テストを通すことを確認
3. **後方互換性**: 既存の設定ファイルでも動作するよう配慮
4. **ドキュメント同期**: コード変更と同時にドキュメントも更新

## 🚀 実行順序

1. **ai-works/simplification-plan.md** 作成 ✅
2. **新ブランチ作成** (`feature/simplification`)
3. **型定義の簡素化** (config.rs)
4. **Markdown処理の簡素化** (markdown_merger.rs)
5. **エージェント実装の簡素化** (agents/)
6. **CLI実装の簡素化** (main.rs)
7. **テストの更新** (tests/)
8. **ドキュメントの更新** (docs/)
9. **統合テスト実行**
10. **PR作成**

---

**この計画に従って、シンプルで理解しやすい AI Context Management Tool を構築します！**