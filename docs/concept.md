# AI Context Management Tool - 設計概要

## 概要

AI Context Management Tool (AICM) は、複数の AI コーディングエージェント用の context ファイルを統一設定から自動生成する Rust 製コマンドラインツールです。

## 設計思想

### 統一管理

- 一つの設定ファイル（`aicm-config.yml`）から各ツール固有のファイル形式を自動生成
- 開発チーム間での AI ツール設定の一貫性を保つ
- 設定の重複を排除し、メンテナンス性を向上

### 柔軟性

- **グローバル設定**: 全エージェント共通の出力モード設定
- **エージェント個別設定**: 各エージェントごとの出力モード上書き
- 後方互換性を保ちながら段階的な機能拡張

### 型安全性

- Rust の型システムによるコンパイル時エラー検出
- `serde(untagged)` による柔軟な設定形式サポート
- 設定の妥当性検証

## サポート対象ツール

### 🎯 Cursor

- **出力先**: `.cursor/rules/*.mdc`
- **形式**: YAML frontmatter + Markdown
- **モード**: merged（単一ファイル）/ split（複数ファイル）

### 🚧 Cline

- **出力先**: `.clinerules` (merged) / `.clinerules/*.md` (split)
- **形式**: 純粋な Markdown
- **モード**: merged（単一ファイル）/ split（複数ファイル）

### 🚧 GitHub Copilot

- **出力先**: `.github/copilot-instructions.md` (merged) / `.github/instructions/*.md` (split)
- **形式**: 純粋な Markdown
- **モード**: merged（単一ファイル）/ split（複数ファイル）

### 🚧 Claude Code

- **出力先**: `CLAUDE.md`
- **形式**: 純粋な Markdown
- **モード**: merged のみ（Claude の仕様）

### 🚧 OpenAI Codex

- **出力先**: `AGENTS.md`
- **形式**: 純粋な Markdown
- **モード**: merged のみ（Codex の仕様）

## 設定ファイル形式

### 基本構造

```yaml
version: "1.0"
output_mode: split # グローバル設定（オプショナル、デフォルト：merged）
base_docs_dir: ./ai-context
agents:
  cursor: true # シンプル設定（後方互換性）
  cline:
    enabled: true
    output_mode: merged # エージェント個別設定
  github:
    output_mode: split # enabled のデフォルトは true
  claude: false # 無効化
  codex: true # OpenAI Codex エージェント
```

### 設定の優先順位

1. **エージェント個別設定** (`agents.{agent}.output_mode`)
2. **グローバル設定** (`output_mode`)
3. **デフォルト値** (`merged`)

### 後方互換性

既存の設定形式も引き続きサポート：

```yaml
version: "1.0"
output_mode: split
base_docs_dir: ./ai-context
agents:
  cursor: true # boolean 形式（従来通り）
  cline: false
  github: true
  claude: false
  codex: false
```

## 出力モード

### Merged モード

- 全ての Markdown ファイルを 1 つのファイルに結合
- ファイル名をヘッダーとして挿入
- 各エージェントの単一ファイル形式に対応

### Split モード

- Markdown ファイルごとに個別のファイルを生成
- ファイル名の安全化（パス区切り文字の変換）
- 各エージェントの複数ファイル形式に対応

## アーキテクチャ

### コア機能

- **MarkdownMerger**: ファイル結合・分割ロジック
- **ConfigLoader**: 設定ファイル読み込み・検証
- **AgentTrait**: エージェント共通インターフェース

### エージェント実装

- 各エージェントは独立したモジュール
- 共通の `generate()` メソッドで統一されたインターフェース
- エージェント固有の出力形式に対応

### 型システム

- `serde(untagged)` による柔軟な設定パース
- `AgentConfigTrait` による統一されたエージェント操作
- `Option<OutputMode>` による段階的な設定上書き

## 使用例

### 基本的な使用方法

```bash
# プロジェクト初期化
aicm init

# 全エージェントのファイル生成
aicm generate

# 特定エージェントのみ生成
aicm generate --agent cursor

# 設定ファイル検証
aicm validate
```

### 設定例

#### 例 1: 混在パターン

```yaml
version: "1.0"
output_mode: split # グローバル設定
base_docs_dir: ./ai-context
agents:
  cursor: true # グローバル設定（split）を使用
  cline:
    output_mode: merged # 個別設定でグローバルを上書き
  github:
    enabled: true # グローバル設定（split）を使用
  claude: false # 無効
  codex: false # 無効
```

#### 例 2: 全て個別設定

```yaml
version: "1.0"
# output_mode なし（デフォルト：merged）
base_docs_dir: ./ai-context
agents:
  cursor:
    output_mode: split
  cline:
    output_mode: merged
  github:
    output_mode: split
  claude: false
  codex:
    output_mode: merged
```

## 拡張性

### 新しいエージェントの追加

1. `src/agents/` に新しいエージェントモジュールを作成
2. `AgentTrait` を実装
3. `main.rs` の `generate_agent_files()` に追加
4. 設定型に新しいエージェント設定を追加

### 新しい設定項目の追加

1. エージェント詳細設定構造体に新しいフィールドを追加
2. `serde(default)` でデフォルト値を設定
3. 必要に応じて `AgentConfigTrait` を拡張

## パフォーマンス特徴

- **高速起動**: ネイティブバイナリによる瞬時起動（100ms 以内）
- **低メモリ**: 効率的なメモリ管理（10MB 以下）
- **並列処理**: 非同期 I/O による高速ファイル処理
- **ゼロコピー**: 不要な文字列コピーの回避

## 今後の拡張予定

### Phase 2

- ウォッチモード（ファイル変更時の自動生成）
- 設定継承機能
- カスタムテンプレート対応

### Phase 3

- プラグインシステム（WASM 対応）
- Web UI
- クラウド同期
