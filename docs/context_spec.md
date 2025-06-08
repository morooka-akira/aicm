# AIコンテキストファイル仕様書

このドキュメントでは、各AIツールのコンテキストファイルのルールと形式をまとめています。

## 概要

各AIツールは異なる方法でコンテキストファイルを管理しています：

| AIツール | ファイル形式 | 配置場所 | 分割対応 |
|----------|--------------|----------|----------|
| Claude Code | `CLAUDE.md` | プロジェクトルート | ✅ (@importで分割可能) |
| Cline | `.clinerules` / `.clinerules/*.md` | プロジェクトルート | ✅ (フォルダ分割対応) |
| GitHub Copilot | `.github/copilot-instructions.md` | プロジェクトルート | ✅ (プロンプトファイル分割可能) |
| VS Code Copilot | `.github/copilot-instructions.md` / `.instructions.md` | プロジェクトルート / `.github/instructions/` | ✅ (複数ファイル対応) |
| Cursor | `.cursorrules` / `.cursor/rules/*.mdc` | プロジェクトルート / `.cursor/rules/` | ✅ (MDCファイル分割対応) |

## 1. Claude Code

### 基本形式

Claude Codeは`CLAUDE.md`ファイルでコンテキストを管理します。

#### 単独ファイル形式
```markdown
<!-- filepath: ./CLAUDE.md -->
# プロジェクト概要

このプロジェクトについての基本的な説明...

## 開発ルール

- 必ずTypeScriptを使用する
- コンポーネントは関数型で実装する
- テストは必須

## アーキテクチャ

...
```

#### 分割ファイル形式
```markdown
<!-- filepath: ./CLAUDE.md -->
# プロジェクト概要

@README で概要を確認し、@package.json で利用可能なnpmコマンドを確認してください。

# 追加の指示
- git ワークフロー @docs/git-instructions.md
- コーディング規約 @docs/coding-standards.md
- アーキテクチャ @docs/architecture.md

# 個人設定
- @~/.claude/my-project-instructions.md
```

### 特徴
- **インポート構文**: `@path/to/file` でファイルをインポート
- **再帰インポート**: 最大5階層まで可能
- **階層探索**: cwd から / まで再帰的に CLAUDE.md を探索
- **自動読み込み**: Claude Code 起動時に自動的に読み込まれる

### メモリ種別
- **プロジェクトメモリ**: `./CLAUDE.md` (チーム共有)
- **ユーザーメモリ**: `~/.claude/CLAUDE.md` (個人設定)
- **ローカルメモリ**: `./CLAUDE.local.md` (非推奨、@importを推奨)

## 2. Cline

### 基本形式

Clineは`.clinerules`ファイルまたは`.clinerules/`フォルダでルールを管理します。

#### 単独ファイル形式
```markdown
<!-- filepath: ./.clinerules -->
# プロジェクトガイドライン

## ドキュメント要件

- 機能を変更する際は /docs の関連ドキュメントを更新する
- README.md を新機能に合わせて更新する
- CHANGELOG.md にエントリを維持する

## アーキテクチャ決定記録

以下について ADR を /docs/adr に作成する：
- 主要な依存関係の変更
- アーキテクチャパターンの変更
- 新しい統合パターン
- データベーススキーマの変更

テンプレートは /docs/adr/template.md に従う

## コードスタイル & パターン

- OpenAPI Generator を使用してAPIクライアントを生成
- TypeScript axios テンプレートを使用
- 生成されたコードは /src/generated に配置
- 継承よりもコンポジションを優先
- データアクセスにはリポジトリパターンを使用
- /src/utils/errors.ts のエラーハンドリングパターンに従う

## テスト基準

- ビジネスロジックには単体テストが必要
- APIエンドポイントには統合テストが必要
- 重要なユーザーフローにはE2Eテストが必要
```

#### 分割ファイル形式
```
your-project/
├── .clinerules/              # アクティブなルール - 自動適用
│   ├── 01-coding.md
│   ├── 02-documentation.md
│   └── current-sprint.md
│
├── clinerules-bank/          # 利用可能だが非アクティブなルール
│   ├── clients/              # クライアント固有のルール
│   │   ├── client-a.md
│   │   └── client-b.md
│   ├── frameworks/           # フレームワーク固有のルール
│   │   ├── react.md
│   │   └── vue.md
│   └── project-types/        # プロジェクト種別基準
│       ├── api-service.md
│       └── frontend-app.md
└── ...
```

### 特徴
- **フォルダシステム**: `.clinerules/` フォルダ内のすべてのMarkdownファイルを処理
- **ルールバンク**: 非アクティブなルールを `clinerules-bank/` で管理
- **動的切り替え**: 必要に応じてルールをアクティブフォルダにコピー
- **UI管理**: v3.13からポップオーバーUIでルールの切り替えが可能

## 3. GitHub Copilot

### 基本形式

GitHub Copilotは`.github/copilot-instructions.md`でカスタム指示を管理します。

#### 単独ファイル形式
```markdown
<!-- filepath: ./.github/copilot-instructions.md -->
我々はJavaの依存関係管理にMavenではなくBazelを使用しているため、Javaパッケージについて話すときは、常にBazelを使用した指示とコードサンプルを提供してください。

JavaScriptは常にダブルクォートとタブでインデントして記述するため、レスポンスにJavaScriptコードが含まれる場合は、これらの規約に従ってください。

我々のチームは作業項目の追跡にJiraを使用しています。
```

#### 分割ファイル形式（プロンプトファイル）
```markdown
<!-- filepath: ./.github/prompts/New React form.prompt.md -->
あなたの目標は新しいReactフォームコンポーネントを生成することです。

提供されていない場合はフォーム名とフィールドを尋ねてください。

フォームの要件：
- フォーム設計システムコンポーネントを使用: [design-system/Form.md](../docs/design-system/Form.md)
- フォーム状態管理には `react-hook-form` を使用:
  - フォームデータには常にTypeScript型を定義
  - registerを使用した *非制御* コンポーネントを優先
  - 不要な再レンダリングを防ぐため `defaultValues` を使用
- バリデーションには `yup` を使用:
  - 別ファイルで再利用可能なバリデーションスキーマを作成
  - 型安全性を確保するためTypeScript型を使用
  - UXフレンドリーなバリデーションルールをカスタマイズ
```

### 特徴
- **自動適用**: すべてのチャット質問に自動的に追加
- **プロンプトファイル**: 再利用可能なプロンプトを `.prompt.md` で定義
- **優先順位**: 個人指示 > リポジトリ指示 > 組織指示
- **参照表示**: チャットレスポンスでファイルが参照として表示

## 4. VS Code Copilot

### 基本形式

VS Code Copilotは複数の形式でカスタム指示を管理します。

#### 単独ファイル形式（基本）
```markdown
<!-- filepath: ./.github/copilot-instructions.md -->
我々はJavaの依存関係管理にMavenではなくBazelを使用しているため、Javaパッケージについて話すときは、常にBazelを使用した指示とコードサンプルを提供してください。

JavaScriptは常にダブルクォートとタブでインデントして記述するため、レスポンスにJavaScriptコードが含まれる場合は、これらの規約に従ってください。

我々のチームは作業項目の追跡にJiraを使用しています。
```

#### 分割ファイル形式（インストラクションファイル）
```markdown
<!-- filepath: ./.github/instructions/general-coding.instructions.md -->
---
applyTo: "**"
---
# プロジェクト全般コーディング規約

## 命名規則
- コンポーネント名、インターフェース、型エイリアスにはPascalCaseを使用
- 変数、関数、メソッドにはcamelCaseを使用
- プライベートクラスメンバーにはアンダースコア（_）をプレフィックス
- 定数にはALL_CAPSを使用

## エラーハンドリング
- 非同期操作にはtry/catchブロックを使用
- Reactコンポーネントで適切なエラーバウンダリを実装
- 常にコンテキスト情報と共にエラーをログ出力
```

```markdown
<!-- filepath: ./.github/instructions/typescript-react.instructions.md -->
---
applyTo: "**/*.ts,**/*.tsx"
---
# TypeScriptとReact用プロジェクトコーディング規約

すべてのコードに[一般的なコーディングガイドライン](./general-coding.instructions.md)を適用する。

## TypeScriptガイドライン
- すべての新しいコードにTypeScriptを使用
- 可能な限り関数型プログラミング原則に従う
- データ構造と型定義にはインターフェースを使用
- 不変データを優先（const、readonly）
- オプショナルチェーン（?.）とnull合体（??）演算子を使用

## Reactガイドライン
- hooksを使用した関数型コンポーネントを使用
- Reactフックのルールに従う（条件付きフックは禁止）
- 子要素を持つコンポーネントにはReact.FC型を使用
- コンポーネントは小さく焦点を絞ったものにする
- コンポーネントスタイリングにはCSSモジュールを使用
```

### 特徴
- **複数形式対応**: `.github/copilot-instructions.md` と `.instructions.md` の両方をサポート
- **スコープ指定**: `applyTo` プロパティでファイルパターンを指定可能
- **設定連携**: VS Code設定でも指示を定義可能
- **自動適用**: 指定したパターンのファイルに自動適用

## 5. Cursor

### 基本形式

Cursorは`.cursorrules`（非推奨）または`.cursor/rules/*.mdc`でルールを管理します。

#### 単独ファイル形式（非推奨）
```markdown
<!-- filepath: ./.cursorrules -->
- 我々の内部RPCパターンを使用してサービスを定義する
- サービス名には常にsnake_caseを使用する
- TypeScriptの新しいコードにはすべて厳密な型付けを使用する
```

#### 分割ファイル形式（推奨）
```markdown
<!-- filepath: ./.cursor/rules/rpc-service.mdc -->
---
description: RPC Service boilerplate
globs: "**/*.ts"
alwaysApply: false
---

- 我々の内部RPCパターンを使用してサービスを定義する
- サービス名には常にsnake_caseを使用する

@service-template.ts
```

```markdown
<!-- filepath: ./.cursor/rules/typescript-strict.mdc -->
---
description: TypeScript strict typing rules
globs: "**/*.ts,**/*.tsx"
alwaysApply: true
---

- TypeScriptの新しいコードにはすべて厳密な型付けを使用する
- 型安全性を確保するため `any` の使用を避ける
- インターフェースを適切に定義する
```

### ルール種別
- **Always**: 常にモデルコンテキストに含まれる
- **Auto Attached**: globパターンにマッチするファイルが参照されたときに含まれる
- **Agent Requested**: AIが必要と判断したときに含まれる（descriptionが必要）
- **Manual**: @ruleNameで明示的に言及されたときのみ含まれる

### 特徴
- **MDC形式**: メタデータとコンテンツを単一ファイルで管理
- **ネストルール**: プロジェクト構造に応じてルールを階層化可能
- **ファイル参照**: `@filename.ts` でファイルをコンテキストに含める
- **チャット生成**: `/Generate Cursor Rules` コマンドでルールを生成可能

## ベストプラクティス

### 共通の推奨事項

1. **明確で簡潔な指示**: 曖昧さを避け、具体的な指示を記述する
2. **構造化**: 見出し、リスト、コードブロックを使用して構造化する
3. **分離とモジュール化**: 大きなルールは複数のファイルに分割する
4. **例の提供**: 具体例やテンプレートファイルを参照する
5. **バージョン管理**: プロジェクトルールはバージョン管理システムに含める

### ツール別の特徴を活かした使い方

- **Claude Code**: インポート機能を活用してプロジェクトドキュメントを体系的に整理
- **Cline**: ルールバンクシステムでコンテキストに応じた柔軟なルール適用
- **GitHub Copilot**: プロンプトファイルで再利用可能なタスクテンプレートを作成
- **VS Code Copilot**: `applyTo` プロパティでファイル種別ごとの細かな制御
- **Cursor**: MDC形式のメタデータを活用した高度なルール管理

### 避けるべき事項

1. **外部リソースへの参照**: 特定のコーディング規約ドキュメントへの言及
2. **スタイル指定**: 特定のレスポンススタイルの強制
3. **詳細レベルの指定**: 常に特定の詳細レベルでの回答を要求
4. **競合する指示**: 複数のルールファイル間での矛盾した指示

## 統合管理のアプローチ

このプロジェクト（ai-context-management）では、単一の設定ファイル（`ai-context.yaml`）から各ツール固有の形式を自動生成することで、以下を実現します：

1. **一元管理**: すべてのAIツール設定を一箇所で管理
2. **一貫性**: チーム全体で同じコンテキストを共有
3. **効率性**: 設定変更時の各ツールファイルの手動更新を不要に
4. **拡張性**: 新しいAIツールへの対応を容易に

各ツールの特徴を理解した上で、統一された管理システムを構築することで、AIツールの活用効率を大幅に向上させることができます。
