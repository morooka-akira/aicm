# GitHub Copilot Agent Implementation and File Naming Fix

## 作業概要

GitHub Copilotエージェントの実装と、正しいファイル命名規則の修正を行います。

## 問題点

現在のコードでは、GitHub Copilotのコンテキストファイル命名が間違っています。

### 正しい命名規則

VS Code Copilot Customizationドキュメント（https://code.visualstudio.com/docs/copilot/copilot-customization#_use-instructionsmd-files）によると：

1. **単一ファイル**: `instructions.md`
2. **複数ファイル**: `*.prompt.md` をサフィックスとして使用

### 現在の問題

- 複数ファイルの場合に `.prompt.md` サフィックスが使用されていない
- GitHub Copilotエージェントが未実装

## 修正計画

### Phase 1: GitHub Copilotエージェントの実装

1. **`src/agents/github.rs` の作成**
   - GitHubAgentの実装
   - 統合モード: `instructions.md` ファイルの生成
   - 分割モード: `*.prompt.md` ファイルの生成

2. **型定義の更新**
   - 既存のGitHubConfig関連の型定義を確認・調整

### Phase 2: ファイル命名の修正

1. **分割モードでの正しい命名**
   ```
   統合モード: instructions.md
   分割モード: overview.prompt.md, rules.prompt.md, etc.
   ```

2. **ディレクトリ配置**
   - ワークスペースルートに配置
   - サブディレクトリ配置のサポート（階層的適用）

### Phase 3: テストの実装

1. **ユニットテスト**
   - ファイル生成のテスト
   - 命名規則のテスト
   - 統合・分割モードのテスト

2. **統合テスト**
   - 実際のファイル生成の動作確認

## 実装詳細

### GitHubAgent構造

```rust
pub struct GitHubAgent {
    config: AIContextConfig,
}

impl GitHubAgent {
    pub fn new(config: AIContextConfig) -> Self;
    pub async fn generate(&self) -> Result<Vec<GeneratedFile>>;
    
    // 統合モード: instructions.md
    async fn generate_merged(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>>;
    
    // 分割モード: *.prompt.md
    async fn generate_split(&self, merger: &MarkdownMerger) -> Result<Vec<GeneratedFile>>;
}
```

### ファイル生成パターン

1. **統合モード**
   ```
   instructions.md
   ```

2. **分割モード**
   ```
   overview.prompt.md
   rules.prompt.md
   architecture.prompt.md
   ```

## 期待される成果

1. GitHub Copilotエージェントの完全実装
2. 正しいファイル命名規則の適用
3. VS Code Copilot Customization仕様への準拠
4. テストカバレッジの追加
5. ドキュメントの更新

## 実装順序

1. ✅ 作業計画の作成（このファイル）
2. ✅ GitHub Copilotエージェントの実装
3. ✅ テストの作成
4. ✅ 統合テストでの動作確認
5. ✅ ドキュメントの更新
6. 🔄 コミット・PR作成

## 実装完了

GitHub Copilotエージェントのファイル命名修正が完了しました：

### 修正内容
- **統合モード**: `instructions.md`（ワークスペースルート）
- **分割モード**: `*.prompt.md`（ワークスペースルート）

### 実装されたメソッド
- `create_instructions_content()`: 純粋なMarkdownコンテンツ生成
- `cleanup_split_files()`: .prompt.mdファイル削除
- `cleanup_merged_file()`: instructions.mdファイル削除

### テスト結果
GitHub Copilotエージェントのテスト: ✅ 5/5 passed

## 参考資料

- [VS Code Copilot Customization](https://code.visualstudio.com/docs/copilot/copilot-customization#_use-instructionsmd-files)
- [GitHub Copilot Custom Instructions](https://docs.github.com/en/copilot/customizing-copilot/adding-repository-custom-instructions-for-github-copilot)