# 2025-06-13 エージェント固有base_docs_dir設定追加作業

## 作業内容
設定ファイルにエージェント固有の`base_docs_dir`オプションを追加し、各エージェントが独自のドキュメントディレクトリを参照できるようにする。

## 設計方針
- エージェント設定に`base_docs_dir`フィールドを追加
- 優先順位: エージェント固有設定 > グローバル設定
- 既存のコードとの後方互換性を保持
- 型安全な実装を行う

## 設定例
```yaml
version: "1.0"
output_mode: split
base_docs_dir: ./ai-context
agents:
  cursor:
    output_mode: split
    base_docs_dir: ./cursor-context
  claude: true  # グローバル設定を使用
```

## 完了要件
1. エージェント設定構造体に`base_docs_dir`フィールドを追加
2. `AgentConfigTrait`に`effective_base_docs_dir`メソッドを追加
3. 各エージェント実装でエージェント固有のディレクトリを使用
4. テストケースの追加
5. ドキュメントの更新（README.md、README.ja.md）
6. -v オプションの説明も追記
7. 全テスト通過
8. cargo fmt、cargo clippy通過
9. PRを作成