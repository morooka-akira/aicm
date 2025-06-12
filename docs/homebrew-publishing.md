# Homebrew 公開手順ガイド

このドキュメントは Homebrew でツールを公開・更新する手順をまとめたものです。

## 概要

Homebrew でのツール公開は以下の流れで行われます：

1. **Formula 作成/更新** → PR 作成
2. **CI/CD による自動テスト** → bottle 自動生成
3. **承認後の自動公開** → bottle 配布開始

## 用語説明

| 用語 | 意味 |
|------|------|
| **Formula** | インストール方法を定義したファイル（`Formula/yourtool.rb`） |
| **Bottle** | 事前コンパイル済みバイナリ（高速インストール用） |
| **Tap** | Formula を管理するリポジトリ |
| **bottle DSL** | Formula 内の `bottle do...end` ブロック |

## 手順詳細

### 1. 初回公開（新しいツールの場合）

#### 1.1 Formula ファイル作成
```ruby
# Formula/aicm.rb
class Aicm < Formula
  desc "AI Code Agent Context Management CLI tool"
  homepage "https://github.com/morooka-akira/aicm"
  url "https://github.com/morooka-akira/aicm/archive/v0.1.0.tar.gz"
  sha256 "your_sha256_hash_here"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--locked", "--root", prefix, "--path", "."
  end

  test do
    system "#{bin}/aicm", "--version"
  end
end
```

#### 1.2 PR 作成
```bash
# Tap リポジトリで
git checkout -b add-aicm-v0.1.0
git add Formula/aicm.rb
git commit -m "aicm: add new formula v0.1.0"
git push origin add-aicm-v0.1.0
# GitHub で PR 作成
```

### 2. バージョンアップ（既存ツールの更新）

#### 2.1 簡単な方法（推奨）
```bash
# 公式 Homebrew の場合
brew bump-formula-pr aicm --url=https://github.com/morooka-akira/aicm/archive/v0.1.1.tar.gz

# 個人 Tap の場合
brew bump-formula-pr morooka-akira/aicm/aicm --url=https://github.com/morooka-akira/aicm/archive/v0.1.1.tar.gz
```

#### 2.2 手動の場合
```ruby
# Formula/aicm.rb を編集
- url "https://github.com/morooka-akira/aicm/archive/v0.1.0.tar.gz"
- sha256 "old_hash..."
+ url "https://github.com/morooka-akira/aicm/archive/v0.1.1.tar.gz"
+ sha256 "new_hash..."
```

### 3. CI/CD 自動処理フロー

#### 3.1 PR 作成時（`tests.yml` 発火）
```yaml
# 自動実行される処理
- Formula の検証（brew audit）
- 各プラットフォームでのビルドテスト
- bottle の自動生成
- Artifact として bottle を保存
```

**出力例：**
```
✅ macOS arm64 bottle: aicm--0.1.1.arm64_ventura.bottle.tar.gz
✅ macOS x86_64 bottle: aicm--0.1.1.x86_64_ventura.bottle.tar.gz
✅ Linux x86_64 bottle: aicm--0.1.1.x86_64_linux.bottle.tar.gz
```

#### 3.2 PR 承認後（`publish.yml` 発火）
```yaml
# pr-pull ラベル付与で自動実行
- bottle を GitHub Releases にアップロード
- Formula に bottle DSL を自動追加
- 更新されたコミットを自動マージ
```

**自動追加される bottle DSL：**
```ruby
class Aicm < Formula
  # ... 既存の内容 ...
  
  bottle do
    sha256 cellar: :any_skip_relocation, arm64_ventura:  "hash1..."
    sha256 cellar: :any_skip_relocation, x86_64_ventura: "hash2..."
    sha256 cellar: :any_skip_relocation, x86_64_linux:   "hash3..."
  end
  
  # ... 残りの内容 ...
end
```

### 4. ユーザーでの利用開始

#### 4.1 インストール
```bash
# 公式 Homebrew の場合
brew install aicm

# 個人 Tap の場合
brew install morooka-akira/aicm/aicm
```

#### 4.2 bottle による高速インストール
- bottle が利用可能な場合、ソースビルドではなく事前コンパイル済みバイナリがダウンロードされる
- インストール時間が大幅短縮（数秒～数十秒）

## 重要なポイント

### ✅ やること
- Formula のバージョンと URL のみ更新
- PR 作成
- CI が緑になるのを待つ
- PR に `pr-pull` ラベルを付与（承認後）

### ❌ やらないこと
- 手動での bottle 作成
- bottle DSL の手動追加
- Release の手動作成

## トラブルシューティング

### CI が失敗する場合
1. **Formula 構文エラー**: `brew audit` の出力を確認
2. **ビルドエラー**: 依存関係や build script を確認
3. **テストエラー**: `test do` ブロックの内容を確認

### bottle が生成されない場合
1. **アーキテクチャサポート**: 対象プラットフォームでビルド可能か確認
2. **依存関係**: 実行時依存関係が適切に設定されているか確認

## 参考リンク

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
- [Creating a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)