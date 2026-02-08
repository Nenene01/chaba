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
```

**What happens automatically:**
1. Fetches and pulls the PR branch
2. Creates an isolated git worktree
3. Sets up a sandbox environment with dependencies
4. Launches AI agent sessions for automated analysis
5. You review in parallel without touching your main workspace

### Key Features (Planned)

#### 1. Git Worktree Integration
- Automatic worktree creation per PR
- Isolated environments with no impact on your main workspace
- Parallel branch management

#### 2. Automated Sandbox Environments
- Dependency installation per worktree
- Environment variable configuration
- Automatic port assignment for development servers

#### 3. AI Agent Integration
- **Claude Code**: Automated source review and analysis
- **Codex**: Code quality checks and second opinions
- **Gemini**: Multi-perspective analysis
- **MCP Integration**: Playwright tests, API debugging

#### 4. Simple CLI Interface
```bash
# Review a PR
chaba review --pr 123

# Debug mode with Playwright
chaba debug --pr 456 --with-playwright

# Cleanup after review
chaba cleanup --pr 123
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

- [ ] **Phase 1**: Basic git worktree CLI operations
- [ ] **Phase 2**: Automated sandbox environment setup
- [ ] **Phase 3**: AI agent integration (Claude Code, Codex, Gemini)
- [ ] **Phase 4**: MCP integration (Playwright, API debugging)
- [ ] **Phase 5**: Build automation & simulator launching

### Technology Stack (Planned)

- **Language**: Rust or TypeScript
- **Git Operations**: `git worktree`, `git branch`, `gh` CLI
- **AI Integration**: Claude Code SDK, MCP (Model Context Protocol)
- **Testing**: Playwright MCP integration
- **Configuration**: YAML / TOML

### Installation (Coming Soon)

```bash
# Via Homebrew (planned)
brew install chaba

# Via Cargo (planned)
cargo install chaba

# Via npm (planned)
npm install -g chaba
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

- [ ] **Phase 1**: 基本的なgit worktree CLI操作
- [ ] **Phase 2**: 自動sandbox環境セットアップ
- [ ] **Phase 3**: AIエージェント統合（Claude Code、Codex、Gemini）
- [ ] **Phase 4**: MCP統合（Playwright、APIデバッグ）
- [ ] **Phase 5**: ビルド自動化 & シミュレータ起動

### 技術スタック（予定）

- **言語**: Rust または TypeScript
- **Git操作**: `git worktree`, `git branch`, `gh` CLI
- **AI統合**: Claude Code SDK, MCP (Model Context Protocol)
- **テスト**: Playwright MCP統合
- **設定**: YAML / TOML

### インストール（近日公開）

```bash
# Homebrew経由（予定）
brew install chaba

# Cargo経由（予定）
cargo install chaba

# npm経由（予定）
npm install -g chaba
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
