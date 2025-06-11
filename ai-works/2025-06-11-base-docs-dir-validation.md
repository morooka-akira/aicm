# base_docs_dir ディレクトリ存在確認機能の追加

## 作業日
2025-06-11

## 要件
`generate`と`validate`コマンドで、設定ファイルの`base_docs_dir`ディレクトリが存在しない場合にエラーを表示する機能を追加する。

## 現在の問題
- `base_docs_dir`が存在しない場合でも`generate`コマンドがエラーにならない
- `validate`コマンドでもディレクトリの存在確認が不十分

## 技術要件

### 1. generate コマンドの改修
- `handle_generate`関数で、設定読み込み後に`base_docs_dir`の存在確認を追加
- ディレクトリが存在しない場合は適切なエラーメッセージを表示
- エラー時は処理を中断して非ゼロの終了コードで終了

### 2. validate コマンドの改修  
- `handle_validate`関数で、設定読み込み後に`base_docs_dir`の存在確認を追加
- 現在は警告メッセージのみだが、エラーとして扱う
- 検証結果にディレクトリ存在確認の結果を含める

### 3. エラーハンドリング
- 一貫性のあるエラーメッセージ形式
- 日本語でのわかりやすいメッセージ
- ConfigErrorを拡張するか、新しいエラー型を定義

### 4. テスト追加
- 存在しないディレクトリでの`generate`コマンドテスト
- 存在しないディレクトリでの`validate`コマンドテスト
- 正常ケースのテストも確認

## 実装方針

### エラーチェックの場所
- `handle_generate`: 設定読み込み後、エージェント処理前
- `handle_validate`: 設定読み込み後、検証処理中

### エラーメッセージ例
```
❌ ドキュメントディレクトリが存在しません: ./nonexistent-docs
💡 ディレクトリを作成するか、設定ファイルのbase_docs_dirを正しいパスに変更してください
```

### 実装ステップ
1. ConfigErrorにDirectoryNotFound variant追加（または新しいエラー型）
2. ディレクトリ存在確認のヘルパー関数作成
3. handle_generate関数にチェック追加
4. handle_validate関数にチェック追加
5. テスト追加
6. 統合テスト確認

## 期待される動作

### generate コマンド
```bash
# 存在しないディレクトリの場合
$ aicm generate
❌ ドキュメントディレクトリが存在しません: ./nonexistent-docs
💡 ディレクトリを作成するか、設定ファイルのbase_docs_dirを正しいパスに変更してください
$ echo $?
1

# 存在するディレクトリの場合
$ aicm generate  
コンテキストファイルを生成します: ai-context.yaml
✅ コンテキストファイルの生成が完了しました
```

### validate コマンド
```bash
# 存在しないディレクトリの場合
$ aicm validate
設定ファイルを検証します: ai-context.yaml
❌ ドキュメントディレクトリが存在しません: ./nonexistent-docs
$ echo $?
1

# 存在するディレクトリの場合  
$ aicm validate
設定ファイルを検証します: ai-context.yaml
✅ 設定ファイルは有効です
  バージョン: 1.0
  出力モード: Split
  ドキュメントディレクトリ: ./docs （存在します）
  有効なエージェント: claude
```