# AI Context Management Tool (aicm) 🦀

AI コーディングエージェント用の context ファイルを統一設定から自動生成する Rust 製 CLI ツール

## ✨ 概要

複数の AI ツール（GitHub Copilot、Cline、Cursor、Claude Code、OpenAI Codex）用の context ファイルを一元管理し、統一設定から各ツール固有のファイル形式を自動生成します。

## 🎯 サポート対象ツール

- **✅ Cursor**: `.cursor/rules/*.mdc` ファイル（split_config対応）
- **✅ Cline**: `.clinerules/*.md` ファイル
- **✅ GitHub Copilot**: `.github/instructions/*.instructions.md` または `.github/copilot-instructions.md`（applyTo オプション対応）
- **✅ Claude Code**: `CLAUDE.md`
- **✅ OpenAI Codex**: `AGENTS.md`

## 🚀 インストール

### Cargo からインストール（推奨）

```bash
# crates.ioからインストール（今後公開予定）
cargo install aicm

# Gitリポジトリから直接インストール
cargo install --git https://github.com/morooka-akira/aicm

# ローカルビルド・インストール
git clone https://github.com/morooka-akira/aicm
cd aicm
cargo install --path .
```

### 必要な環境

- Rust 1.70.0 以上
- Cargo（Rust と一緒にインストールされます）

## 📖 使用方法

### 基本的な使い方

```bash
# プロジェクトディレクトリで設定ファイルを初期化
aicm init

# 設定ファイルを編集
vim ai-context.yaml

# コンテキストファイルを生成
aicm generate

# 特定のエージェントのみ生成
aicm generate --agent cursor

# 外部設定ファイルを指定
aicm generate --config /path/to/custom-config.yaml
aicm generate -c ./configs/production.yaml

# 特定のエージェントと外部設定の組み合わせ
aicm generate --agent cursor --config custom.yaml

# 設定ファイルを検証
aicm validate

```

## ⚙️ 設定ファイル仕様

### 外部設定ファイルの使用

`--config` / `-c` オプションを使用して、デフォルトの `ai-context.yaml` 以外の設定ファイルを指定できます。

```bash
# カスタム設定ファイルを使用
aicm generate --config production.yaml
aicm generate -c ./configs/staging.yaml

# 絶対パスも使用可能
aicm generate --config /etc/aicm/production.yaml
```

この機能により、以下のような使い方が可能です：

- **環境別設定**: 開発・ステージング・本番環境ごとに異なる設定
- **チーム別設定**: チームごとに最適化された設定ファイル
- **プロジェクト別設定**: 複数プロジェクトでの設定ファイル共有

### 基本設定（ai-context.yaml）

```yaml
# ai-context.yaml
version: "1.0"
output_mode: split         # merged | split
include_filenames: false   # merged モード時にファイル名ヘッダーを含めるか（デフォルト: false）
base_docs_dir: ./ai-context

# エージェント設定
agents:
  # シンプル設定（有効/無効のみ）
  cursor: true
  cline: false
  github: true
  claude: true
  codex: false
```

### 詳細設定

```yaml
# ai-context.yaml
version: "1.0" 
output_mode: split
include_filenames: false   # グローバル設定（デフォルト: false）
base_docs_dir: ./ai-context

agents:
  # 詳細設定
  cursor:
    enabled: true
    output_mode: split        # エージェント個別の出力モード
    include_filenames: true   # エージェント個別のファイル名ヘッダー設定
    split_config:             # Cursor split_config機能
      rules:
        - file_patterns: ["*project*", "*overview*"]
          alwaysApply: true
        - file_patterns: ["*architecture*", "*design*"]
          globs: ["**/*.rs", "**/*.ts"]
        - file_patterns: ["*development*", "*rules*"]
          description: "開発ルール関連のエージェント要求"
        - file_patterns: ["*setup*", "*install*"]
          manual: true

  cline:
    enabled: true
    output_mode: merged
    include_filenames: false  # Clineではファイル名ヘッダーを無効化

  github:
    enabled: true
    output_mode: split
    split_config:             # GitHub applyTo オプション対応
      rules:
        - file_patterns: ["*architecture*", "*design*"]
          apply_to: ["**/*.rs", "**/*.toml"]
        - file_patterns: ["*frontend*", "*ui*"]
          apply_to: ["**/*.ts", "**/*.tsx"]

  claude:
    enabled: true
    include_filenames: true   # Claudeではファイル名ヘッダーを有効化
    # Claude は常に merged モード

  codex:
    enabled: false
    # Codex は常に merged モード
```

### include_filenames オプション

`include_filenames` オプションは、merged モード時にファイル名ヘッダー（`# filename.md`）を含めるかどうかを制御します。

#### 設定階層

設定は以下の優先順位で適用されます：
1. **エージェント個別設定** > **グローバル設定** > **デフォルト（false）**

```yaml
# グローバル設定
include_filenames: true   # すべてのエージェントのデフォルト

agents:
  claude:
    include_filenames: false  # Claudeのみオーバーライド（グローバル設定より優先）
  
  cursor:
    # include_filenamesの指定なし → グローバル設定（true）を継承
```

#### 動作例

**include_filenames: true の場合**
```markdown
# 01_project-overview.md

# プロジェクト概要
このプロジェクトは...

# 02_architecture.md

# アーキテクチャ
システム設計について...
```

**include_filenames: false の場合**
```markdown
# プロジェクト概要
このプロジェクトは...

# アーキテクチャ
システム設計について...
```

### Cursor split_config詳細

Cursor の split_config 機能では、ファイルパターンに応じて異なるルールタイプを設定できます：

#### ルールタイプ

1. **Always（常時適用）**
   ```yaml
   - file_patterns: ["*common*", "*global*"]
     alwaysApply: true
   ```
   生成結果：
   ```yaml
   ---
   alwaysApply: true
   ---
   ```

2. **Auto Attached（自動添付）**
   ```yaml
   - file_patterns: ["*rust*", "*backend*"]
     globs: ["**/*.rs", "**/*.toml"]
   ```
   生成結果：
   ```yaml
   ---
   description: ''
   globs: ["**/*.rs", "**/*.toml"]
   alwaysApply: false
   ---
   ```

3. **Agent Requested（エージェント要求）**
   ```yaml
   - file_patterns: ["*api*", "*spec*"]
     description: "API仕様書関連のルール"
   ```
   生成結果：
   ```yaml
   ---
   description: API仕様書関連のルール
   ---
   ```

4. **Manual（手動参照）**
   ```yaml
   - file_patterns: ["*troubleshoot*", "*debug*"]
     manual: true
   ```
   生成結果：
   ```yaml
   ---
   manual: true
   ---
   ```

#### ファイルパターン

- `*project*`: "project"を含むファイル名
- `config*`: "config"で始まるファイル名  
- `*setup`: "setup"で終わるファイル名
- `exact.md`: 完全一致

#### 優先順位

複数の設定が同じルールに含まれる場合、以下の優先順位で適用されます：
1. `manual: true`
2. `alwaysApply: true`
3. `globs` 設定
4. `description` 設定
5. デフォルト（alwaysApply: true）

## 🔧 開発環境

### セットアップ

```bash
# リポジトリをクローン
git clone https://github.com/morooka-akira/aicm
cd aicm

# ビルド
cargo build

# テスト実行
cargo test

# リリースビルド
cargo build --release

# 開発版での実行
cargo run -- init
cargo run -- generate
```

### 使用技術

- **言語**: Rust (Edition 2021)
- **CLI Framework**: clap v4 (derive API)
- **非同期処理**: Tokio
- **設定**: YAML (serde_yaml)
- **エラーハンドリング**: anyhow, thiserror
- **テスト**: 標準テストフレームワーク + tokio-test

## 📁 プロジェクト構造

```
aicm/
├── src/
│   ├── main.rs                 # CLI エントリーポイント
│   ├── lib.rs                  # ライブラリエントリーポイント
│   ├── config/                 # 設定管理
│   │   ├── mod.rs
│   │   ├── loader.rs           # 設定読み込み
│   │   └── error.rs            # 設定エラー型
│   ├── core/                   # コア機能
│   │   ├── mod.rs
│   │   └── markdown_merger.rs  # Markdownファイル結合
│   ├── agents/                 # エージェント実装
│   │   ├── mod.rs
│   │   ├── base.rs            # ベースユーティリティ
│   │   ├── cursor.rs          # Cursor実装（split_config対応）
│   │   ├── cline.rs           # Cline実装
│   │   ├── github.rs          # GitHub Copilot実装
│   │   ├── claude.rs          # Claude Code実装
│   │   └── codex.rs           # OpenAI Codex実装
│   └── types/                  # 型定義
│       ├── mod.rs
│       ├── config.rs          # 設定型（CursorSplitConfig含む）
│       └── agent.rs           # エージェント型
├── docs/                      # 設計ドキュメント
│   ├── concept.md             # 設計概要
│   ├── design_doc.md          # 技術仕様書
│   └── requirements.md        # 要件定義
├── ai-works/                  # 開発作業記録
├── target/                    # ビルド出力
├── Cargo.toml                 # プロジェクト設定
├── Cargo.lock                 # 依存関係ロック
└── ai-context.yaml            # 設定ファイル例
```

## 📤 生成される出力

### Cursor エージェント

**Split モード（split_config なし）**
```
.cursor/rules/
├── 01_project-overview.mdc
├── 02_architecture.mdc
├── 03_development-rules.mdc
└── ...
```

**Split モード（split_config あり）**
```
.cursor/rules/
├── project-overview.mdc      # alwaysApply: true
├── architecture.mdc          # globs: ["**/*.rs"], alwaysApply: false
├── development-rules.mdc     # description: "...", 
└── setup.mdc                 # manual: true
```

**Merged モード**
```
.cursor/rules/
└── context.mdc               # 全コンテンツを統合
```

### その他のエージェント

**Cline**
```
.clinerules/
├── 01-project-overview.md
├── 02-architecture.md
└── ...
```

**GitHub Copilot**
```
.github/
├── instructions/
│   ├── architecture.instructions.md   # applyTo: "**/*.rs,**/*.toml"
│   ├── frontend.instructions.md       # applyTo: "**/*.ts,**/*.tsx"
│   └── ...
└── copilot-instructions.md            # merged モード時
```

**Claude Code**
```
CLAUDE.md                     # 常に merged モード
```

**OpenAI Codex**
```
AGENTS.md                     # 常に merged モード
```

## 💡 使用例

### 実際の設定例

プロジェクトルートに `ai-context.yaml` を作成：

```yaml
version: "1.0"
output_mode: split
include_filenames: false    # デフォルトではファイル名ヘッダーを含めない
base_docs_dir: ./ai-context

agents:
  cursor:
    enabled: true
    output_mode: split
    include_filenames: true  # Cursorではファイル名ヘッダーを有効化
    split_config:
      rules:
        # プロジェクト概要は常に適用
        - file_patterns: ["*overview*", "*readme*"]
          alwaysApply: true
          
        # Rustファイル編集時にアーキテクチャ情報を自動添付
        - file_patterns: ["*architecture*", "*design*"]
          globs: ["**/*.rs", "**/*.toml"]
          
        # API開発時にエージェントが判断して適用
        - file_patterns: ["*api*", "*endpoint*"]
          description: "API設計とエンドポイント仕様"
          
        # トラブルシューティングは手動参照のみ
        - file_patterns: ["*troubleshoot*", "*debug*"]
          manual: true

  cline:
    enabled: true
    output_mode: merged
    include_filenames: false  # Clineではファイル名ヘッダーを無効化

  github:
    enabled: true
    output_mode: split
    # include_filenamesの指定なし → グローバル設定（false）を継承

  claude: true  # シンプル設定（デフォルト有効、グローバル設定を継承）
  
  codex: false  # シンプル設定（デフォルト無効）
```

### ディレクトリ構造例

```
your-project/
├── ai-context/                    # base_docs_dir
│   ├── 01-project-overview.md
│   ├── 02-architecture.md
│   ├── 03-api-design.md
│   ├── 04-troubleshooting.md
│   └── 05-coding-standards.md
├── ai-context.yaml               # 設定ファイル
├── src/
│   └── main.rs
└── Cargo.toml
```

### 実行例

```bash
# 設定ファイルを初期化
aicm init

# 全エージェント向けファイル生成
aicm generate

# Cursor専用ファイルのみ生成
aicm generate --agent cursor

# 外部設定ファイルを使用
aicm generate --config production.yaml

# 特定エージェント + 外部設定ファイル
aicm generate --agent github --config ./configs/github-only.yaml

# 設定ファイルの妥当性確認
aicm validate
```

### 生成結果

```
your-project/
├── .cursor/rules/
│   ├── project-overview.mdc     # alwaysApply: true
│   ├── architecture.mdc         # globs: ["**/*.rs"]
│   ├── api-design.mdc          # description: "API設計..."
│   ├── troubleshooting.mdc     # manual: true
│   └── coding-standards.mdc    # alwaysApply: true (デフォルト)
├── .clinerules/
│   └── context.md              # 全コンテンツ統合
├── .github/
│   └── instructions/
│       ├── architecture.instructions.md   # applyTo frontmatter付き
│       ├── frontend.instructions.md       # applyTo frontmatter付き
│       └── ...
├── CLAUDE.md                   # Claude用（全コンテンツ統合）
└── AGENTS.md                   # Codex用（全コンテンツ統合）
```

## 🧪 テスト

```bash
# 全テスト実行
cargo test

# 特定のテストモジュール実行
cargo test config

# テストカバレッジ（tarpaulin要インストール）
cargo install cargo-tarpaulin
cargo tarpaulin --out html

# 統合テスト実行
cargo test --test integration_test
```

## 🚢 配布・デプロイ

### リリースビルド

```bash
# 最適化されたバイナリビルド
cargo build --release

# バイナリは target/release/aicm に生成されます
```

### クロスコンパイル（例）

```bash
# macOS用（Apple Silicon）
cargo build --release --target aarch64-apple-darwin

# Linux用
cargo build --release --target x86_64-unknown-linux-gnu

# Windows用
cargo build --release --target x86_64-pc-windows-gnu
```

## ⚡ パフォーマンス特徴

- **高速起動**: Rust ネイティブバイナリによる瞬時起動
- **低メモリ使用量**: 効率的なメモリ管理
- **並列処理**: Tokio による非同期ファイル処理
- **ゼロコピー**: 不要な文字列コピーの回避

## 🔒 セキュリティ

- **メモリ安全**: Rust の所有権システムによる保証
- **型安全**: コンパイル時の厳密な型チェック
- **パストラバーサル防止**: 適切なパス正規化

## 🤝 コントリビューション

1. このリポジトリをフォーク
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

### 開発ガイドライン

- コードは Rustfmt でフォーマット（`cargo fmt`）
- Clippy の警告を解決（`cargo clippy`）
- テストを追加（`cargo test`）
- ドキュメントを更新

## 📝 ライセンス

MIT License - 詳細は [LICENSE](LICENSE) ファイルを参照

## 🙏 謝辞

このプロジェクトは以下の素晴らしいツールによって支えられています：

- [clap](https://github.com/clap-rs/clap) - CLI 構築フレームワーク
- [tokio](https://github.com/tokio-rs/tokio) - 非同期ランタイム
- [serde](https://github.com/serde-rs/serde) - シリアライゼーション
- [anyhow](https://github.com/dtolnay/anyhow) - エラーハンドリング

## 📞 サポート

- バグ報告: [Issues](https://github.com/morooka-akira/aicm/issues)
- 機能要求: [Issues](https://github.com/morooka-akira/aicm/issues)
- ディスカッション: [Discussions](https://github.com/morooka-akira/aicm/discussions)
