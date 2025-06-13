# AI コーディングエージェントの設定ファイルを管理するツールを作りました

こんにちは、タイムラボのエンジニアです。

最近、Cursor や Cline、GitHub Copilot など、AI を活用したコーディングエージェントがたくさん登場していますね。これらのツールは開発効率を大幅に向上させてくれるのですが、それぞれ異なる設定ファイル形式を必要とするため、プロジェクトごとに複数の設定ファイルを管理する必要があります。

そこで今回、これらの設定ファイルを統一的に管理できるツール「aicm」（AI Context Management Tool）を Rust で開発しました。

## なぜこのツールを作ろうと思ったか

### 設定ファイルの管理が煩雑になった

私たちの開発チームでは、複数の AI コーディングエージェントを使い分けています：

- **Cursor**: プロジェクトのルールを `.cursor/rules/*.mdc` ファイルで管理
- **Cline**: コンテキスト情報を `.clinerules/*.md` ファイルで管理
- **GitHub Copilot**: インストラクションを `.github/instructions/*.instructions.md` ファイルで管理
- **Claude Code**: プロジェクト情報を `CLAUDE.md` ファイルで管理

各ツールで同じような内容（プロジェクトの概要、アーキテクチャ、開発ルールなど）を記述しているにも関わらず、それぞれ独自の形式で管理する必要がありました。

### メンテナンスの手間とミスが発生

プロジェクトの仕様変更や開発ルールの更新があるたびに、すべての設定ファイルを手動で更新する必要があり、以下のような問題が発生していました：

- 更新漏れによる設定ファイル間の不整合
- 各ツールの形式に合わせた変換作業の手間
- 新しいエージェントを導入する際の初期設定の複雑さ

### チーム間での設定共有が困難

複数の開発チームが存在する場合、各チームで AI エージェントの設定が異なっていると、知見の共有やベストプラクティスの統一が困難になります。

これらの課題を解決するため、一つの設定ファイルから各 AI エージェント用のファイルを自動生成できるツールが必要だと感じ、aicm の開発に着手しました。

## aicm でできること

### 基本的な使い方

aicm を使うと、一つの設定ファイル（`aicm-config.yml`）から複数の AI エージェント用設定ファイルを自動生成できます。

```bash
# プロジェクトの初期化
aicm init

# 設定ファイルの編集
vim aicm-config.yml

# 全エージェント用ファイルの生成
aicm generate

# 特定のエージェントのみ生成
aicm generate --agent cursor
```

基本的な設定ファイルは以下のような形式です：

```yaml
# aicm-config.yml
version: "1.0"
output_mode: split
base_docs_dir: ./ai-context

agents:
  cursor: true
  cline: true
  github: true
  claude: true
  codex: false
```

### Cursor の高度な設定

Cursor では、ルールファイルに対して様々なオプションを設定できます。aicm では、これらのオプションを詳細に制御できます：

```yaml
agents:
  cursor:
    enabled: true
    output_mode: split
    split_config:
      rules:
        # 常に適用されるプロジェクト概要
        - file_patterns: ["*project*", "*overview*"]
          alwaysApply: true
        
        # Rust ファイルにのみ適用されるアーキテクチャルール
        - file_patterns: ["*architecture*", "*design*"]
          globs: ["**/*.rs", "**/*.ts"]
        
        # 開発ガイドライン（説明付き）
        - file_patterns: ["*development*", "*rules*"]
          description: "Development guidelines and coding standards"
        
        # マニュアル参照のみのトラブルシューティング
        - file_patterns: ["*troubleshoot*", "*debug*"]
          manual: true
```

これにより、以下のような `.cursor/rules/` ファイルが生成されます：

```markdown
// project-overview.mdc
---
alwaysApply: true
---

# プロジェクト概要
...
```

```markdown
// architecture.mdc
---
globs: ["**/*.rs", "**/*.ts"]
---

# アーキテクチャ設計
...
```

### GitHub Copilot の applyTo 設定

GitHub Copilot では、特定のファイルパターンにのみ適用されるインストラクションを作成できます：

```yaml
agents:
  github:
    enabled: true
    output_mode: split
    split_config:
      rules:
        # バックエンド関連のファイルにのみ適用
        - file_patterns: ["*backend*", "*api*"]
          apply_to: ["**/*.rs", "**/*.toml"]
        
        # フロントエンド関連のファイルにのみ適用
        - file_patterns: ["*frontend*", "*ui*"]
          apply_to: ["**/*.ts", "**/*.tsx"]
```

生成される `.github/instructions/` ファイルは以下のようになります：

```markdown
<!-- backend.instructions.md -->
---
applyTo: "**/*.rs,**/*.toml"
---

# バックエンド開発ガイドライン
...
```

### 柔軟な出力モード

aicm では、以下の出力モードをサポートしています：

1. **Split モード**: 各ドキュメントを個別ファイルとして出力
2. **Merged モード**: 全ドキュメントを一つのファイルに結合

エージェントごとに個別の設定も可能です：

```yaml
output_mode: split  # グローバル設定

agents:
  cursor:
    output_mode: split  # 個別ファイルで管理
  claude:
    output_mode: merged  # 一つのファイルに結合
```

### 実際の開発での効果

aicm を導入してから、以下のような効果を実感しています：

- **設定更新の時間が 80% 短縮**: 一箇所の更新で全エージェントの設定が同期
- **設定ミスの削減**: 手動コピーによる転記ミスがゼロに
- **新しいエージェントの導入が簡単**: 設定ファイルに一行追加するだけで対応
- **チーム間での設定統一**: 共通の設定テンプレートを使用

## 終わりに

AI コーディングエージェントは日々進化しており、新しいツールも続々と登場しています。それぞれのツールが独自の設定形式を持つ中で、統一的な管理方法があることで、開発チームの生産性向上に大きく貢献できると考えています。

aicm は MIT ライセンスのオープンソースとして公開しており、GitHub で開発を進めています。Rust で実装されているため、高速で安全な動作を実現しています。

```bash
# Homebrew での簡単インストール
brew tap morooka-akira/aicm
brew install aicm

# Cargo での直接インストール
cargo install --git https://github.com/morooka-akira/aicm
```

現在サポートしているエージェントは Cursor、Cline、GitHub Copilot、Claude Code、OpenAI Codex ですが、今後も新しいエージェントのサポートを追加していく予定です。

もし皆さんの開発チームでも複数の AI エージェントを使用されている場合は、ぜひ aicm を試してみてください。フィードバックや機能要望もお待ちしています！

**プロジェクトリポジトリ**: https://github.com/morooka-akira/aicm

---

タイムラボでは、このような開発効率化ツールの開発を通じて、より良い開発体験の実現を目指しています。今後も有用なツールを開発・公開していきますので、ぜひご注目ください。