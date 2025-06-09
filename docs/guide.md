# AI Context Management Tool 使い方ガイド

## このツールは何をするものですか？

複数の AI ツール（Cursor、GitHub Copilot、Cline など）用の設定ファイルを、**一つの設定から自動で作成**するツールです。

各 AI ツールに同じことを何度も設定する手間がなくなります。

## 何ができるの？

✅ **一度書けば、全 AI ツールに適用**  
✅ **チーム全体で同じ AI 設定を共有**  
✅ **プロジェクトのルールを AI に教える**  
✅ **AI ツールごとの面倒な設定作業を自動化**

## 基本的な使い方

### 1. 初期セットアップ

```bash
# ツールをダウンロード・ビルド
cargo build --release

# プロジェクトを初期化
./target/release/aicm init
```

すると、以下のディレクトリが作成されます：

```
ai-context/
├── common/          # 全AIツール共通のルール
│   ├── project-overview.md
│   └── coding-rules.md
├── agents/          # 特定AIツール用のルール
│   ├── cursor.md
│   └── github.md
└── ai-context.yaml  # 設定ファイル
```

### 2. ルールを書く

#### `ai-context/common/` にプロジェクト共通のルールを書く

**例：`coding-rules.md`**

```markdown
# コーディングルール

## 基本方針

- 変数名は分かりやすく書く
- 関数は 1 つの機能だけを持つ
- エラーハンドリングを必ず行う

## プロジェクト固有ルール

- API キーは環境変数で管理
- ログは構造化形式で出力
```

#### `ai-context/agents/` に AI ツール固有のルールを書く

**例：`cursor.md`**

```markdown
# Cursor 用の追加ルール

- コード補完時は型安全性を重視
- リファクタリング提案を積極的に行う
```

### 3. AI 用設定ファイルを生成

```bash
# 全AIツール用のファイルを生成
./target/release/aicm generate

# 特定のAIツールだけ生成
./target/release/aicm generate --agent cursor
```

### 4. 結果を確認

以下のファイルが自動生成されます：

```
.cursor/rules/context.mdc     # Cursor用
.clinerules/rules.md          # Cline用（今後対応）
instructions.md               # GitHub Copilot用（今後対応）
CLAUDE.md                     # Claude Code用（今後対応）
```

## 設定ファイル（ai-context.yaml）の基本

```yaml
version: "1.0"
output_mode: merged # ファイルを1つにまとめる
base_docs_dir: ./ai-context # ルールファイルの場所

# AIツールの設定
agents:
  cursor: {} # Cursorを有効にする
  cline: null # Clineは無効
  github: null # GitHub Copilotは無効
  claude: null # Claude Codeは無効

# どのファイルを使うかの設定
file_mapping:
  common: # 全AIツール共通
    - common/project-overview.md
    - common/coding-rules.md
  project_specific: # プロジェクト固有
    - agents/cursor.md
```

## 高度な使い方

### よくある使い方パターン

#### チーム開発での活用

1. **チームリーダーが初期設定**
   ```bash
   aicm init
   # common/ にチームのコーディングルールを記載
   ```

2. **メンバーがAI設定を生成**
   ```bash
   git pull                    # 最新のルールを取得
   aicm generate              # 各自のAIツール設定を更新
   ```

3. **新しいルールを追加**
   ```bash
   # common/ にルールを追加
   git add . && git commit -m "Add new coding rules"
   git push
   ```

#### 役割別ルール設定

**フロントエンド・バックエンド分離**
```yaml
agents:
  cursor:
    split_config:
      frontend:
        type: auto_attached
        description: "フロントエンド用ルール"
        globs: ["src/components/**/*", "src/pages/**/*", "*.tsx", "*.css"]
      backend:
        type: auto_attached
        description: "バックエンド用ルール"
        globs: ["src/api/**/*", "src/models/**/*", "src/services/**/*"]
      common:
        type: always
        description: "共通ルール"
        globs: ["**/*.js", "**/*.ts"]
```

**セキュリティ重視プロジェクト**
```yaml
agents:
  cursor:
    split_config:
      security:
        type: always
        description: "セキュリティ必須ルール"
        globs: ["src/auth/**/*", "src/api/**/*"]
        always_apply: true
      general:
        type: auto_attached
        description: "一般的なルール"
        globs: ["src/**/*"]
```

### 統合モード vs 分割モード

#### 統合モード（デフォルト）
全ての設定を1つのファイルにまとめます。

```yaml
version: "1.0"
output_mode: merged    # 統合モード
agents:
  cursor: {}
```

**結果**: `.cursor/rules/context.mdc` が1つ生成される

#### 分割モード
設定を複数のファイルに分けて生成します。

```yaml
version: "1.0"
output_mode: split     # 分割モード
agents:
  cursor:
    split_config:
      common:
        type: always
        description: "共通プロジェクトルール"
        globs: ["**/*.rs", "**/*.toml"]
        always_apply: true
      development:
        type: auto_attached
        description: "開発環境設定"
        globs: ["src/**/*"]
```

**結果**: 複数のファイルが生成される
```
.cursor/rules/
├── common.mdc        # always適用
└── development.mdc   # 該当ファイル編集時に適用
```

### split_config の詳細設定

#### type（適用タイミング）
- **`always`**: 常に適用される最重要ルール
- **`auto_attached`**: 該当ファイル編集時に自動適用
- **`agent_requested`**: AIが必要と判断した時に適用
- **`manual`**: 手動で選択した時のみ適用

#### globs（適用対象ファイル）
```yaml
globs:
  - "**/*.rs"           # 全Rustファイル
  - "src/**/*"          # srcディレクトリ内全て
  - "tests/**/*.rs"     # テストファイルのみ
  - "*.toml"            # ルートの設定ファイル
```

#### 完全な分割設定例
```yaml
agents:
  cursor:
    split_config:
      # 基本ルール（常に適用）
      common:
        type: always
        description: "基本コーディングルール"
        globs: ["**/*.rs", "**/*.toml"]
        always_apply: true
      
      # プロジェクト固有（自動適用）
      project:
        type: auto_attached
        description: "プロジェクト固有設定"
        globs: ["src/**/*"]
      
      # セキュリティ（必要時のみ）
      security:
        type: agent_requested
        description: "セキュリティガイドライン"
        globs: ["src/auth/**/*", "src/api/**/*"]
      
      # テスト用（手動選択）
      testing:
        type: manual
        description: "テスト記述ルール"
        globs: ["tests/**/*"]
    
    additional_instructions: |
      このプロジェクトはWebアプリケーションです。
      セキュリティを最優先にしてください。
```

### プロジェクト固有の設定

```yaml
# プロジェクト特有の情報をAIに教える
agents:
  cursor:
    additional_instructions: |
      このプロジェクトはECサイトです。
      セキュリティを最優先にしてください。
      個人情報の取り扱いに注意してください。
```

## 便利なコマンド

```bash
# 設定をチェック
aicm validate
# ヘルプを表示
aicm --help
```

## トラブルシューティング

### Q: ファイルが生成されない

**A:** `agents` の設定を確認してください

```yaml
# ❌ 無効
agents:
  cursor: null

# ✅ 有効
agents:
  cursor: {}
```

### Q: ファイルが見つからないエラー

**A:** `file_mapping` のパスが正しいか確認してください

```yaml
file_mapping:
  common:
    - common/存在するファイル.md # ファイルが実際に存在するか確認
```

### Q: 分割モードで複数ファイルを生成したい

**A:** `output_mode: split` と `split_config` を設定してください

```yaml
output_mode: split
agents:
  cursor:
    split_config:
      common:
        type: always
        description: "基本ルール"
      project:
        type: auto_attached  
        description: "プロジェクト設定"
```

### Q: 特定のファイルタイプにだけルールを適用したい

**A:** `globs` でファイルパターンを指定してください

```yaml
split_config:
  rust_only:
    type: always
    description: "Rust専用ルール"
    globs: ["**/*.rs"]  # Rustファイルのみ
  frontend:
    type: auto_attached
    description: "フロントエンド用"
    globs: ["src/components/**/*", "src/pages/**/*"]
```

### Q: チームで設定を共有したい

**A:** `ai-context/` ディレクトリと `ai-context.yaml` を Git に含めてください

```bash
git add ai-context/ ai-context.yaml
git commit -m "Add AI context configuration"
```

## まとめ

1. **`aicm init`** でプロジェクトを初期化
2. **`ai-context/`** にルールを書く
3. **`aicm generate`** で AI 用ファイルを生成
4. 各 AI ツールが最適な設定で動作！

これで、複数の AI ツールを一括管理できるようになります。
