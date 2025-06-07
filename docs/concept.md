# AI Context Management Tool - 設計概要

## プロジェクト概要

このプロジェクトは、複数のAIコーディングエージェント用のcontextファイルを統一設定から自動生成するコマンドラインツールです。開発者が複数のAIツールを使い分ける際の設定管理を効率化し、一貫性を保つことを目的としています。

## サポート対象ツール

### 1. GitHub Copilot
- **ファイル**: `instructions.md` （ワークスペース内の任意の場所）
- **形式**: Markdown（複数ファイル対応、階層的適用）
- **公式ドキュメント**: 
  - [Adding repository custom instructions for GitHub Copilot](https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot)
  - [VS Code Copilot Customization](https://code.visualstudio.com/docs/copilot/copilot-customization#_use-instructionsmd-files)

### 2. Cline
- **ファイル**: `.clinerules/` ディレクトリ内の `.md` ファイル
- **形式**: Markdown（複数ファイル対応、数値プレフィックスによる順序制御）
- **公式ドキュメント**: [Cline Rules](https://docs.cline.bot/features/cline-rules)

### 3. Cursor
- **ファイル**: `.cursor/rules` ディレクトリ内の `.mdc` ファイル
- **形式**: MDC（Markdown + フロントマター）
- **公式ドキュメント**: [Cursor Rules](https://docs.cursor.com/context/rules)

### 4. Claude Code
- **ファイル**: `CLAUDE.md`
- **形式**: Markdown

## アーキテクチャ設計原則

### 1. 抽象化による統一管理
- 各AIツール固有の設定ファイル形式を抽象化
- 共通の設定ファイルから各ツール用ファイルを生成
- 設定ファイルのオーバーライドのみで完結する設計

### 2. 設定の階層構造
```
共通設定 (ベース)
├── ツール固有設定 (オーバーライド)
└── プロジェクト固有設定 (最終オーバーライド)
```

### 3. ファイル生成の仕組み
- **入力**: 統一設定ファイル（YAML/JSON形式）
- **処理**: ツール別テンプレートエンジン
- **出力**: 各AIツール用contextファイル

## ワークフロー例

### 1. プロジェクト初期化
```bash
ai-context init
# docs/ ディレクトリ構造が作成される
# ai-context.yaml のテンプレートが生成される
```

### 2. ナレッジの作成・編集
```bash
# 共通ナレッジの編集
vim docs/common/coding-standards.md
vim docs/common/project-overview.md

# エージェント固有設定
vim docs/agents/cursor.md
vim docs/agents/cline.md
```

### 3. contextファイル生成
```bash
# 統合モード：各エージェントに1ファイル出力
ai-context generate

# 分割モード：対応エージェントは複数ファイル出力
ai-context generate --split
```

### 4. 生成結果
```
統合モード:
├── instructions.md                  (all merged)
├── .clinerules/rules.md             (all merged)
├── .cursor/rules/context.mdc        (all merged)
└── CLAUDE.md                        (all merged)

分割モード:
├── instructions.md                  # ルート階層
├── src/
│   ├── instructions.md              # コンポーネント固有
│   └── components/
│       └── instructions.md          # より具体的な指示
├── .clinerules/
│   ├── 01-common.md
│   ├── 02-project.md
│   └── 03-specific.md
├── .cursor/rules/
│   ├── common.mdc          (type: always)
│   ├── project.mdc         (type: auto_attached)
│   └── specific.mdc        (type: agent_requested)
└── CLAUDE.md               (all merged - 分割非対応)
```

## 設計アーキテクチャ

### ドキュメントベースアプローチ

#### 初期化とディレクトリ構造
```bash
ai-context init
```
実行すると以下のディレクトリ構造が作成される：

```
docs/
├── common/           # 共通ナレッジ（全エージェント共通）
│   ├── coding-standards.md
│   ├── project-overview.md
│   └── team-conventions.md
├── agents/           # エージェント固有設定
│   ├── github.md     # GitHub Copilot固有
│   ├── cline.md      # Cline固有
│   ├── cursor.md     # Cursor固有
│   └── claude.md     # Claude Code固有
└── ai-context.yaml   # 生成設定ファイル
```

#### ナレッジ管理の仕組み

1. **ベースはMarkdown**: 全てのナレッジは `.md` ファイルで管理
2. **階層構造**: `common/` と `agents/` で分離
3. **マージロジック**: 
   - デフォルト: 全ファイルを結合して各エージェントに出力
   - オプション指定: エージェント対応に応じて分割出力

### 出力モード

#### 1. 統合モード（デフォルト）
```bash
ai-context generate
```
- `common/` + `agents/{tool}.md` を結合
- 各エージェントに1つのcontextファイルとして出力

#### 2. 分割モード
```bash
ai-context generate --split
```
- 分割対応エージェントのみ、複数ファイルで出力
- **Cline**: `.clinerules/01-common.md`, `.clinerules/02-project.md` など
- **Cursor**: `.cursor/rules/common.mdc`, `.cursor/rules/project.mdc` など
- **GitHub Copilot**: 階層的な `instructions.md` ファイル配置
- **Claude**: 統合モードと同じ（分割非対応）

### エージェント固有ルール対応

#### 設定ファイル例（ai-context.yaml）
```yaml
# 基本設定
output_mode: "merged"  # "merged" | "split"
base_docs_dir: "docs"

# エージェント固有設定
agents:
  cursor:
    split_config:
      common:
        type: "always"
        description: "共通コーディング規約"
        globs: ["**/*.ts", "**/*.js"]
      project:
        type: "auto_attached"
        description: "プロジェクト固有ルール"
        globs: ["src/**/*"]
  
  cline:
    split_config:
      file_prefix: "01-"  # 01-common.md, 02-project.md
      
  github:
    hierarchy_config:
      root:
        path: "instructions.md"
        scope: "workspace"
      src:
        path: "src/instructions.md"
        scope: "source_code"
      components:
        path: "src/components/instructions.md"
        scope: "ui_components"
    additional_instructions: |
      チームはJiraを使用してタスク管理を行っています。

# ファイルマッピング
file_mapping:
  common:
    - "common/coding-standards.md"
    - "common/project-overview.md"
  project_specific:
    - "common/team-conventions.md"
    - "agents/{agent}.md"
```

## CLI インターフェース設計

```bash
# プロジェクト初期化（ドキュメントディレクトリ作成）
ai-context init

# 統合モードで全エージェントのcontextファイル生成
ai-context generate

# 分割モードで生成（対応エージェントのみ）
ai-context generate --split

# 特定エージェントのみ生成
ai-context generate --agent github
ai-context generate --agent cline,cursor

# 分割 + 特定エージェント
ai-context generate --split --agent cursor

# 設定ファイルの検証
ai-context validate

# 現在の設定とマッピングを表示
ai-context show

# ドライランモード（実際には生成せずプレビュー）
ai-context generate --dry-run
```

## 実装フェーズ

### Phase 1: Core Framework & Init
1. `ai-context init` コマンド実装
2. ドキュメントディレクトリ構造の自動生成
3. 基本的な設定ファイル（ai-context.yaml）読み込み
4. Markdownファイルの結合・マージ機能

### Phase 2: エージェント対応（統合モード）
1. GitHub Copilot出力対応
2. Cline出力対応
3. Cursor出力対応（基本的な.mdcファイル生成）
4. Claude Code出力対応

### Phase 3: 分割モード対応
1. Cline分割出力（数値プレフィックス対応）
2. Cursor分割出力（frontmatter設定対応）
3. エージェント固有設定のオーバーライド機能

### Phase 4: 高度な機能
1. `--dry-run` プレビュー機能
2. 設定の検証・リンティング
3. テンプレートのカスタマイズ機能
4. ウォッチモード（ファイル変更時自動生成）

## 技術選択

- **言語**: Python または Node.js（エコシステムと開発効率を考慮）
- **設定形式**: YAML（人間が読みやすく、コメント対応）
- **テンプレート**: Jinja2 または類似のテンプレートエンジン
- **CLI**: Click（Python）または Commander.js（Node.js）

## 利点

### 従来の課題
- 各AIエージェントごとに異なる設定ファイル形式
- 同じナレッジを複数箇所で重複管理
- エージェント固有ルール（globs、type設定など）の複雑性
- チーム間での設定同期の困難

### 本ツールによる解決

1. **統一管理**: ドキュメントベースで全エージェントのナレッジを一元化
2. **柔軟な出力**: 統合/分割モードでエージェントの特性に対応
3. **保守性**: Markdownベースで可読性が高く、編集が容易
4. **チーム協働**: `docs/` ディレクトリによる構造化された共有
5. **バージョン管理**: 全設定ファイルをGitで一括管理
6. **拡張性**: 新しいエージェント対応時も設定追加のみで完結