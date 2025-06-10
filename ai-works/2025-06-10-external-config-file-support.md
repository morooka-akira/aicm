# 外部設定ファイル指定機能の実装

## 作業概要
`generate --config <filepath>` または `generate -c <filepath>` で設定ファイルの外部パス指定に対応する。

## 要件

### 機能要件
1. `generate` コマンドに `--config` および `-c` オプションを追加
2. 指定されたパスの設定ファイルを読み込み
3. デフォルトの `ai-context.yaml` より優先して使用
4. 存在しないファイルを指定した場合は適切なエラーメッセージを表示
5. 既存の機能との互換性を保つ

### 技術要件
1. `clap` を使用したCLI引数の拡張
2. `config::loader` モジュールでの外部パス対応
3. エラーハンドリングの充実
4. 包括的なテストの追加

## 設計

### CLI構造の変更
```rust
#[derive(Parser, Debug)]
#[command(name = "aicm")]
pub enum Command {
    Generate {
        #[arg(short, long, help = "Specific agents to generate for")]
        agent: Option<Vec<String>>,
        
        #[arg(short, long, help = "Path to configuration file")]
        config: Option<String>,
    },
    // ... other commands
}
```

### 設定ファイル読み込み順序
1. `--config`/`-c` で指定されたファイル（最優先）
2. デフォルトの `ai-context.yaml`（フォールバック）

### エラーハンドリング
- 指定されたファイルが存在しない場合
- 指定されたファイルが読み取れない場合
- 無効なYAML形式の場合

## 実装計画

### Phase 1: CLI引数の拡張
- `src/main.rs` の `Command` enum に `config` フィールド追加
- `clap` の引数定義更新

### Phase 2: 設定ローダーの拡張
- `config::loader::load_config` に外部パス対応追加
- エラーハンドリングの改善

### Phase 3: テストの追加
- 外部設定ファイル指定のテスト
- 存在しないファイル指定のエラーテスト
- デフォルト動作の後方互換性テスト

### Phase 4: ドキュメント更新
- README.md に使用例追加
- ヘルプメッセージの改善

## 使用例

```bash
# 外部設定ファイルを指定
aicm generate --config /path/to/custom-config.yaml

# ショートオプション
aicm generate -c ./configs/production.yaml

# 特定のエージェントと組み合わせ
aicm generate --config custom.yaml --agent cursor github
```

## テスト観点

1. **正常系**
   - 外部設定ファイルの正常読み込み
   - デフォルト設定ファイルとの動作比較
   - エージェント指定との組み合わせ

2. **異常系**
   - 存在しないファイル指定
   - 読み取り権限のないファイル指定
   - 無効なYAML形式のファイル指定

3. **互換性**
   - 既存のコマンド動作に影響しないこと
   - `--agent` オプションとの組み合わせ

## 期待される結果

- ユーザーが任意の場所にある設定ファイルを指定可能
- 複数の環境（開発、ステージング、本番）での設定切り替えが容易
- 既存の機能に影響を与えない後方互換性の維持