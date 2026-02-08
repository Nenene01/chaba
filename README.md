# Chaba (茶葉)

**AI Agent Friendly Source Review & Debug Environment**

[English](#english) | [日本語](#日本語)

---

## English

### What is Chaba?

**Chaba** (茶葉, *cha-ba*, meaning "tea leaves" in Japanese) is a tool that makes source code review seamless for AI agents and human developers working in teams.

Just as tea leaves grow on branches, each bearing unique value, code branches in team development carry their own valuable work. Chaba helps reviewers (the "tea masters") evaluate and appreciate each branch's quality, enabling parallel review workflows without interrupting ongoing development.

### Why "Chaba" (Tea Leaves)?

In team development:
- Git branches (`branch` = 枝) spread like tree branches
- Each branch bears fruitful coding work, like tea leaves on a branch
- Reviewers must carefully evaluate each "leaf" (contribution) for quality
- The best work is harvested and merged, just as premium tea leaves are selected

Chaba streamlines this process, allowing teams to review in parallel while maintaining focus on their own work.

### The Problem

Modern software development increasingly involves collaboration with AI agents (Claude Code, Codex, Gemini). However, human team members still work together and frequently request code reviews from each other.

The challenge:
- You're focused on implementing a feature with your AI agent
- A teammate asks you to review their PR
- Context switching disrupts your flow
- Setting up a review environment is manual and time-consuming

### The Solution

Chaba automates the entire review setup:

```bash
# Start a review environment with a single command
chaba review --pr 123

# Or specify a branch directly
chaba review --branch feature/new-api

# With AI agent analysis
chaba review --pr 123 --with-agent

# Thorough review with all agents
chaba review --pr 123 --thorough
```

**What happens automatically:**
1. ✅ Fetches and pulls the PR branch
2. ✅ Creates an isolated git worktree
3. ✅ Sets up a sandbox environment with dependencies
4. ✅ Launches AI agent sessions for automated analysis (optional)
5. ✅ You review in parallel without touching your main workspace

### Key Features

#### 1. Git Worktree Integration ✅
- ✅ Automatic worktree creation per PR
- ✅ Isolated environments with no impact on your main workspace
- ✅ Parallel branch management
- ✅ State persistence for tracking active reviews

#### 2. Automated Sandbox Environments ✅
- ✅ Project type detection (Node.js, Rust, Python, Go)
- ✅ Dependency installation per worktree
- ✅ Environment variable configuration (.env file copying)
- ✅ Automatic port assignment for development servers (3000-4000)
- ✅ Package manager auto-detection (npm, yarn, pnpm, bun, cargo)

#### 3. AI Agent Integration ✅
- ✅ **Claude Code**: Automated source review and analysis
- ✅ **Codex**: Code quality checks and second opinions
- ✅ **Gemini**: Multi-perspective analysis
- ✅ Parallel execution for faster reviews
- ✅ Structured finding reports with severity and categories
- ⬜ **MCP Integration**: Playwright tests, API debugging (Phase 4)

#### 4. Simple CLI Interface ✅
```bash
# Review a PR
chaba review --pr 123

# Review with AI agent analysis
chaba review --pr 123 --with-agent

# Thorough review with all agents
chaba review --pr 123 --thorough

# View agent analysis results
chaba agent-result --pr 123

# List active reviews
chaba list

# Check review status
chaba status --pr 123

# Cleanup after review
chaba cleanup --pr 123

# Initialize configuration
chaba config --local
```

### Use Case: Parallel Review Workflow

**Scenario:** You're refactoring authentication with Claude Code when a teammate requests a review.

**Traditional workflow:**
1. Stash or commit your work in progress
2. Checkout the PR branch
3. Install dependencies
4. Run tests
5. Review code
6. Checkout back to your branch
7. Continue your work

**With Chaba:**
```bash
chaba review --pr 234
```

Now you have:
- Your main work in `~/projects/myapp` → continue coding with Claude
- Review environment in `~/reviews/pr-234` → separate terminal for review
- AI agents automatically analyze the PR in the background
- Zero interruption to your flow

### Roadmap

- [x] **Phase 1**: Basic git worktree CLI operations ✅
  - [x] Git worktree creation and management
  - [x] State persistence
  - [x] CLI commands (review, cleanup, list, status, config)

- [x] **Phase 2**: Automated sandbox environment setup ✅
  - [x] Project type detection
  - [x] Dependency installation (Node.js, Rust, Python, Go)
  - [x] Port management
  - [x] Environment variable copying

- [x] **Phase 3**: AI agent integration ✅
  - [x] Claude Code integration
  - [x] Codex integration
  - [x] Gemini integration
  - [x] Parallel execution
  - [x] Structured analysis reports
  - [x] agent-result command

- [ ] **Phase 4**: MCP integration (Playwright, API debugging)
- [ ] **Phase 5**: Build automation & simulator launching
- [x] **Phase 6**: npm distribution ✅
  - [x] npm package structure
  - [x] CLI wrapper (bin/chaba.js)
  - [x] Platform packages
  - [x] GitHub Actions CI/CD
  - [x] Release automation
  - [ ] First npm publish (pending)

### Technology Stack

- **Language**: Rust 2021 Edition
- **Git Operations**: `git2` crate, `gh` CLI
- **AI Integration**: Command-line interfaces for Claude, Codex, Gemini
- **Async Runtime**: Tokio
- **Configuration**: YAML (serde_yaml)
- **Testing**: 56 tests (unit + integration)
- **CLI Framework**: clap v4

### Installation

#### Via npm (Coming Soon)
```bash
npm install -g @nenene01/chaba
```

Supported platforms:
- macOS (Intel & Apple Silicon)
- Linux (x64 & ARM64)
- Windows (x64)

#### From Source
```bash
# Clone the repository
git clone https://github.com/Nenene01/chaba.git
cd chaba

# Build and install
cargo install --path .

# Verify installation
chaba --version
```

#### Via Cargo (Planned)
```bash
cargo install chaba
```

#### Via Homebrew (Planned)
```bash
brew install chaba
```

### Quick Start

```bash
# Initialize configuration (optional)
chaba config --local

# Start reviewing a PR
chaba review --pr 123

# Or with AI agent analysis
chaba review --pr 123 --with-agent

# View analysis results
chaba agent-result --pr 123

# List active reviews
chaba list

# Clean up when done
chaba cleanup --pr 123
```

### Contributing

Chaba is in its early stages. We welcome:
- Ideas and feedback
- Pull requests
- Documentation improvements
- Translations

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Inspiration

Like [jj (Jujutsu)](https://github.com/jj-vcs/jj), Chaba embraces its Japanese roots while aiming to serve developers worldwide. We believe great tools transcend language barriers.

### License

MIT License - see [LICENSE](LICENSE) for details.

---

## 日本語

### Chabaとは？

**Chaba（茶葉）** は、AIエージェントと人間の開発者がチームで働く際のソースコードレビューをシームレスにするツールです。

茶葉が枝ごとに独自の価値を持つように、チーム開発におけるコードのブランチもそれぞれに価値ある作業が含まれています。Chabaは、レビュアー（茶師のような存在）が各ブランチの品質を評価し、進行中の開発を中断することなく並行してレビューワークフローを実現します。

### なぜ「Chaba（茶葉）」なのか？

チーム開発において:
- Gitのブランチ（枝）は木の枝のように広がります
- 各ブランチには、茶葉のように実りのあるコーディング作業が含まれています
- レビュアーは、各「葉」（コントリビューション）の品質を慎重に評価する必要があります
- 最高の作業が収穫されマージされる様子は、高級茶葉が選別される過程に似ています

Chabaはこのプロセスを効率化し、チームが自分の作業に集中しながら並行してレビューを行えるようにします。

### 解決する課題

現代のソフトウェア開発では、AIエージェント（Claude Code、Codex、Gemini）との協働が増えています。しかし、人間のチームメンバーは依然として協力して作業し、頻繁にコードレビューを依頼し合います。

課題:
- 自分のタスクにAIエージェントと集中している最中
- チームメイトからPRレビューを依頼される
- コンテキストスイッチで集中が途切れる
- レビュー環境のセットアップが手作業で時間がかかる

### 解決策

Chabaはレビューセットアップ全体を自動化します:

```bash
# 1つのコマンドでレビュー環境を起動
chaba review --pr 123

# またはブランチ名を直接指定
chaba review --branch feature/new-api
```

**自動的に実行されること:**
1. PRブランチのfetchとpull
2. 独立したgit worktreeの作成
3. 依存関係を含むsandbox環境のセットアップ
4. 自動分析のためのAIエージェントセッションを起動
5. メインワークスペースに影響を与えずに並行してレビュー

### 主な機能（計画）

#### 1. Git Worktree統合
- PRごとに自動worktree作成
- メインワークスペースに影響のない独立環境
- 並行ブランチ管理

#### 2. 自動Sandbox環境
- worktreeごとの依存関係インストール
- 環境変数の設定
- 開発サーバーの自動ポート割り当て

#### 3. AIエージェント統合
- **Claude Code**: 自動ソースレビューと分析
- **Codex**: コード品質チェックとセカンドオピニオン
- **Gemini**: 多角的な分析
- **MCP統合**: Playwrightテスト、APIデバッグ

#### 4. シンプルなCLIインターフェース
```bash
# PRをレビュー
chaba review --pr 123

# Playwrightを含むデバッグモード
chaba debug --pr 456 --with-playwright

# レビュー後のクリーンアップ
chaba cleanup --pr 123
```

### ユースケース: 並行レビューワークフロー

**シナリオ:** Claude Codeと認証機能のリファクタリング中に、チームメイトからレビュー依頼が来た。

**従来のワークフロー:**
1. 進行中の作業をstashまたはcommit
2. PRブランチにcheckout
3. 依存関係をインストール
4. テストを実行
5. コードをレビュー
6. 元のブランチにcheckout
7. 作業を再開

**Chabaを使用:**
```bash
chaba review --pr 234
```

これで:
- メインの作業は `~/projects/myapp` で継続 → Claudeとコーディング
- レビュー環境は `~/reviews/pr-234` → 別ターミナルでレビュー
- AIエージェントがバックグラウンドで自動分析
- フローを中断せずに作業可能

### ロードマップ

- [x] **Phase 1**: 基本的なgit worktree CLI操作 ✅
  - [x] Git worktreeの作成と管理
  - [x] 状態の永続化
  - [x] CLIコマンド（review, cleanup, list, status, config）

- [x] **Phase 2**: 自動sandbox環境セットアップ ✅
  - [x] プロジェクトタイプ検出
  - [x] 依存関係インストール（Node.js, Rust, Python, Go）
  - [x] ポート管理
  - [x] 環境変数のコピー

- [x] **Phase 3**: AIエージェント統合 ✅
  - [x] Claude Code統合
  - [x] Codex統合
  - [x] Gemini統合
  - [x] 並列実行
  - [x] 構造化分析レポート
  - [x] agent-resultコマンド

- [ ] **Phase 4**: MCP統合（Playwright、APIデバッグ）
- [ ] **Phase 5**: ビルド自動化 & シミュレータ起動
- [x] **Phase 6**: npm配布 ✅
  - [x] npmパッケージ構造
  - [x] CLIラッパー (bin/chaba.js)
  - [x] プラットフォームパッケージ
  - [x] GitHub Actions CI/CD
  - [x] リリース自動化
  - [ ] 初回npm公開（保留中）

### 技術スタック

- **言語**: Rust 2021 Edition
- **Git操作**: `git2` crate, `gh` CLI
- **AI統合**: Claude, Codex, Geminiのコマンドラインインターフェース
- **非同期ランタイム**: Tokio
- **設定**: YAML (serde_yaml)
- **テスト**: 56テスト（ユニット + 統合）
- **CLIフレームワーク**: clap v4

### インストール

#### npm経由（近日公開）
```bash
npm install -g @nenene01/chaba
```

対応プラットフォーム:
- macOS (Intel & Apple Silicon)
- Linux (x64 & ARM64)
- Windows (x64)

#### ソースから
```bash
# リポジトリをクローン
git clone https://github.com/Nenene01/chaba.git
cd chaba

# ビルドとインストール
cargo install --path .

# インストール確認
chaba --version
```

#### Cargo経由（予定）
```bash
cargo install chaba
```

#### Homebrew経由（予定）
```bash
brew install chaba
```

### クイックスタート

```bash
# 設定の初期化（オプション）
chaba config --local

# PRのレビューを開始
chaba review --pr 123

# AIエージェント分析を含む
chaba review --pr 123 --with-agent

# 分析結果を表示
chaba agent-result --pr 123

# アクティブなレビューを一覧表示
chaba list

# 完了後のクリーンアップ
chaba cleanup --pr 123
```

### コントリビューション

Chabaは開発初期段階です。以下を歓迎します:
- アイデアとフィードバック
- プルリクエスト
- ドキュメント改善
- 翻訳

詳細は [CONTRIBUTING.md](CONTRIBUTING.md) をご覧ください。

### インスピレーション

[jj (Jujutsu)](https://github.com/jj-vcs/jj) のように、Chabaは日本のルーツを大切にしながら、世界中の開発者に貢献することを目指しています。優れたツールは言語の壁を超えると信じています。

### ライセンス

MIT License - 詳細は [LICENSE](LICENSE) をご覧ください。

---

**Built for the future of AI-assisted team development**
