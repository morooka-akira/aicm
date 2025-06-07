# Design Document - AI Context Management Tool

## 技術仕様書

### プロジェクト概要
NPMパッケージとして配布する、AIコーディングエージェント用contextファイル生成CLIツール

## アーキテクチャ設計

### 技術スタック
- **言語**: TypeScript
- **ランタイム**: Node.js (>=16)
- **CLI Framework**: Commander.js
- **設定**: YAML (js-yaml)
- **テンプレート**: Handlebars.js
- **ファイル操作**: fs-extra
- **パッケージ管理**: npm
- **ビルド**: TypeScript Compiler + esbuild

### パッケージ構造
```
ai-context-management/
├── bin/
│   └── ai-context.js           # CLI エントリーポイント
├── src/
│   ├── commands/               # CLIコマンド実装
│   │   ├── init.ts
│   │   ├── generate.ts
│   │   ├── validate.ts
│   │   └── show.ts
│   ├── core/                   # コア機能
│   │   ├── config.ts           # 設定読み込み
│   │   ├── markdown-merger.ts  # Markdownファイル結合
│   │   ├── agent-generator.ts  # エージェント固有出力
│   │   └── template-engine.ts  # テンプレート処理
│   ├── agents/                 # エージェント実装
│   │   ├── base.ts            # ベースクラス
│   │   ├── github.ts          # GitHub Copilot
│   │   ├── cline.ts           # Cline
│   │   ├── cursor.ts          # Cursor
│   │   └── claude.ts          # Claude Code
│   ├── templates/              # デフォルトテンプレート
│   │   ├── init/              # init時のファイルテンプレート
│   │   └── agents/            # エージェント固有テンプレート
│   ├── types/                  # TypeScript型定義
│   │   ├── config.ts
│   │   ├── agent.ts
│   │   └── index.ts
│   └── utils/                  # ユーティリティ
│       ├── file-system.ts
│       ├── validation.ts
│       └── logger.ts
├── templates/                  # 外部テンプレート
├── dist/                      # ビルド出力
├── tests/                     # テスト
├── package.json
├── tsconfig.json
└── README.md
```

## コア設計

### 1. 設定ファイル型定義
```typescript
// types/config.ts
export interface AIContextConfig {
  version: string;
  output_mode: 'merged' | 'split';
  base_docs_dir: string;
  agents: AgentConfigs;
  file_mapping: FileMapping;
  global_variables?: Record<string, any>;
}

export interface AgentConfigs {
  github?: GitHubConfig;
  cline?: ClineConfig;
  cursor?: CursorConfig;
  claude?: ClaudeConfig;
}

export interface CursorConfig {
  split_config?: {
    [key: string]: {
      type: 'always' | 'auto_attached' | 'agent_requested' | 'manual';
      description: string;
      globs?: string[];
    };
  };
  additional_instructions?: string;
}

export interface ClineConfig {
  split_config?: {
    file_prefix: string;
    max_files?: number;
  };
  additional_instructions?: string;
}
```

### 2. エージェントベースクラス設計
```typescript
// agents/base.ts
export abstract class BaseAgent {
  protected config: AIContextConfig;
  protected agentConfig: any;
  
  constructor(config: AIContextConfig, agentConfig: any) {
    this.config = config;
    this.agentConfig = agentConfig;
  }
  
  abstract generateFiles(mergedContent: string, splitContent: SplitContent): Promise<GeneratedFile[]>;
  abstract getOutputPaths(): string[];
  abstract validate(): ValidationResult;
}

export interface GeneratedFile {
  path: string;
  content: string;
  encoding?: 'utf8' | 'binary';
}

export interface SplitContent {
  common: string;
  projectSpecific: string;
  agentSpecific: string;
}
```

### 3. Markdownマージ機能
```typescript
// core/markdown-merger.ts
export class MarkdownMerger {
  private basePath: string;
  private fileMapping: FileMapping;
  
  async mergeFiles(): Promise<MergedContent> {
    const commonFiles = await this.loadCommonFiles();
    const agentFiles = await this.loadAgentFiles();
    
    return {
      merged: this.combineContent(commonFiles, agentFiles),
      split: {
        common: this.combineContent(commonFiles.common),
        projectSpecific: this.combineContent(commonFiles.project),
        agentSpecific: agentFiles
      }
    };
  }
  
  private async loadCommonFiles(): Promise<CommonFiles> {
    // ファイル読み込みとパース処理
  }
}
```

### 4. テンプレートエンジン
```typescript
// core/template-engine.ts
export class TemplateEngine {
  private handlebars: typeof Handlebars;
  
  constructor() {
    this.handlebars = Handlebars.create();
    this.registerHelpers();
  }
  
  async renderTemplate(templatePath: string, context: TemplateContext): Promise<string> {
    const template = await fs.readFile(templatePath, 'utf8');
    const compiled = this.handlebars.compile(template);
    return compiled(context);
  }
  
  private registerHelpers(): void {
    // カスタムヘルパー登録
    this.handlebars.registerHelper('frontmatter', (data: any) => {
      return `---\n${yaml.dump(data)}---\n`;
    });
  }
}
```

## エージェント実装詳細

### 1. Cursor エージェント
```typescript
// agents/cursor.ts
export class CursorAgent extends BaseAgent {
  async generateFiles(mergedContent: string, splitContent: SplitContent): Promise<GeneratedFile[]> {
    if (this.config.output_mode === 'split' && this.agentConfig.split_config) {
      return this.generateSplitFiles(splitContent);
    }
    
    return this.generateMergedFile(mergedContent);
  }
  
  private async generateSplitFiles(content: SplitContent): Promise<GeneratedFile[]> {
    const files: GeneratedFile[] = [];
    
    for (const [name, config] of Object.entries(this.agentConfig.split_config)) {
      const frontmatter = {
        description: config.description,
        ...(config.globs && { globs: config.globs }),
        alwaysApply: config.type === 'always'
      };
      
      const fileContent = `---\n${yaml.dump(frontmatter)}---\n\n${content[name] || content.common}`;
      
      files.push({
        path: `.cursor/rules/${name}.mdc`,
        content: fileContent
      });
    }
    
    return files;
  }
}
```

### 2. Cline エージェント
```typescript
// agents/cline.ts
export class ClineAgent extends BaseAgent {
  async generateFiles(mergedContent: string, splitContent: SplitContent): Promise<GeneratedFile[]> {
    if (this.config.output_mode === 'split' && this.agentConfig.split_config) {
      return this.generateSplitFiles(splitContent);
    }
    
    return [{
      path: '.clinerules/rules.md',
      content: mergedContent
    }];
  }
  
  private async generateSplitFiles(content: SplitContent): Promise<GeneratedFile[]> {
    const files: GeneratedFile[] = [];
    const prefix = this.agentConfig.split_config.file_prefix || '';
    
    const contentMap = [
      { name: 'common', content: content.common },
      { name: 'project', content: content.projectSpecific },
      { name: 'agent', content: content.agentSpecific }
    ];
    
    contentMap.forEach((item, index) => {
      if (item.content.trim()) {
        files.push({
          path: `.clinerules/${prefix}${String(index + 1).padStart(2, '0')}-${item.name}.md`,
          content: item.content
        });
      }
    });
    
    return files;
  }
}
```

## CLIコマンド実装

### 1. Init コマンド
```typescript
// commands/init.ts
export async function initCommand(options: InitOptions): Promise<void> {
  const targetDir = process.cwd();
  
  // ディレクトリ構造作成
  await createDirectoryStructure(targetDir);
  
  // テンプレートファイル生成
  await generateTemplateFiles(targetDir);
  
  // ai-context.yaml 生成
  await generateConfigFile(targetDir, options);
  
  console.log('✅ AI Context Management initialized successfully!');
}

async function createDirectoryStructure(baseDir: string): Promise<void> {
  const dirs = [
    'docs/common',
    'docs/agents'
  ];
  
  for (const dir of dirs) {
    await fs.ensureDir(path.join(baseDir, dir));
  }
}
```

### 2. Generate コマンド
```typescript
// commands/generate.ts
export async function generateCommand(options: GenerateOptions): Promise<void> {
  const config = await loadConfig();
  const merger = new MarkdownMerger(config);
  const content = await merger.mergeFiles();
  
  const agents = getTargetAgents(options.agents, config);
  
  for (const agentName of agents) {
    const agent = createAgent(agentName, config);
    const files = await agent.generateFiles(content.merged, content.split);
    
    for (const file of files) {
      if (options.dryRun) {
        console.log(`Would generate: ${file.path}`);
        continue;
      }
      
      await fs.ensureDir(path.dirname(file.path));
      await fs.writeFile(file.path, file.content, 'utf8');
      console.log(`✅ Generated: ${file.path}`);
    }
  }
}
```

## NPMパッケージ設定

### package.json
```json
{
  "name": "ai-context-management",
  "version": "1.0.0",
  "description": "AI coding agents context file management CLI tool",
  "main": "dist/index.js",
  "bin": {
    "ai-context": "bin/ai-context.js"
  },
  "scripts": {
    "build": "tsc && esbuild src/cli.ts --bundle --platform=node --outfile=bin/ai-context.js",
    "dev": "tsx src/cli.ts",
    "test": "jest",
    "lint": "eslint src/**/*.ts",
    "prepare": "npm run build"
  },
  "keywords": ["ai", "context", "cli", "github-copilot", "cursor", "cline", "claude"],
  "engines": {
    "node": ">=16.0.0"
  },
  "dependencies": {
    "commander": "^11.0.0",
    "js-yaml": "^4.1.0",
    "handlebars": "^4.7.8",
    "fs-extra": "^11.1.1",
    "chalk": "^5.3.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "@types/js-yaml": "^4.0.5",
    "@types/fs-extra": "^11.0.1",
    "typescript": "^5.0.0",
    "esbuild": "^0.19.0",
    "tsx": "^4.0.0",
    "jest": "^29.0.0",
    "@types/jest": "^29.0.0",
    "eslint": "^8.0.0"
  }
}
```

## 配布・インストール

### グローバルインストール
```bash
npm install -g ai-context-management
ai-context init
```

### プロジェクトローカルインストール
```bash
npm install --save-dev ai-context-management
npx ai-context init
```

## テスト戦略

### 1. ユニットテスト
- 各エージェントクラスの出力テスト
- Markdownマージ機能のテスト
- 設定ファイル読み込みテスト

### 2. 統合テスト
- CLIコマンドの実行テスト
- ファイル生成の完全性テスト

### 3. E2Eテスト
- 実際のプロジェクトでの動作確認
- 各AIエージェントでの動作検証

## パフォーマンス考慮

### 1. ファイル読み込み最適化
- 必要なファイルのみ読み込み
- 非同期処理による並列化

### 2. メモリ使用量
- ストリーミング処理によるメモリ効率化
- 大きなファイルの分割処理

### 3. キャッシュ機能
- 設定ファイルのキャッシュ
- テンプレートコンパイル結果のキャッシュ

## セキュリティ考慮

### 1. ファイルアクセス
- プロジェクトディレクトリ外へのアクセス制限
- パストラバーサル攻撃の防止

### 2. テンプレート実行
- Handlebarsのサンドボックス化
- 危険な関数の無効化

### 3. 設定ファイル検証
- YAML爆弾攻撃の防止
- スキーマ検証による不正な設定の排除