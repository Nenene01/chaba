# Chaba

**AI Agent Friendly Source Review & Debug Environment**

AIエージェントがより簡単にソースレビューできるようにするツール。
git worktree、branch操作、sandbox環境を統合し、チーム開発における並行作業を強力にサポートします。

## 概要

チーム開発では、自分のタスクに集中している最中に他のメンバーからPRレビューを依頼されることがよくあります。
Chabaは、PRのIDやブランチ名を指定するだけで、以下を自動的に実行します:

- ブランチのpull
- 必要に応じてcheckout
- 予定されているディレクトリ配下でworktreeとしてclone
- レビュー・デバッグ環境の起動（Claude、Codex、Geminiなどのサブエージェントと連携）

これにより、**自分のタスクを中断せずに**、別セッションで並行してレビュー作業を進めることができます。

## 主な機能（計画）

### 1. Git Worktree統合
- PRごとに独立したworktreeを自動作成
- 既存の作業ディレクトリに影響を与えず、複数のブランチを同時に操作可能

### 2. Sandbox環境の自動構築
- 各worktreeに対応したsandbox環境を自動セットアップ
- 他のtreeと連携しながら、依存関係の解決や環境変数の設定を実施

### 3. AIエージェントとの統合
- Claude Code、Codex、Geminiなどのサブエージェントを活用
- ソースレビューの自動実行
- API debugやplaywrightテストのMCP実行
- buildやシミュレータ起動の自動化

### 4. 簡単なコマンドインターフェース
```bash
# PRレビュー環境を起動
chaba review --pr 123

# ブランチ名を指定してレビュー環境を起動
chaba review --branch feature/new-api

# デバッグモードで起動（API debugやplaywrightも含む）
chaba debug --pr 456 --with-playwright
```

## ユースケース

### シナリオ: 集中作業中のPRレビュー依頼

1. **現在の状況**
   あなたは `feature/auth-refactor` ブランチで認証機能のリファクタリングに集中しています。
   Claude Codeと協力してコーディング中です。

2. **レビュー依頼が到着**
   チームメンバーから「PR #234をレビューしてほしい」とSlackで連絡が来ました。

3. **Chabaで並行作業**
   ```bash
   chaba review --pr 234
   ```
   このコマンド一つで:
   - PR #234のブランチを自動でpull
   - `~/reviews/pr-234` などの専用worktreeを作成
   - 必要な依存関係をインストール
   - レビュー用のAIエージェントセッションを起動

4. **並行してレビュー実施**
   - メインの作業は `feature/auth-refactor` で継続
   - 別ターミナル/セッションでPR #234のレビューを実施
   - 変更内容の確認、テスト実行、コメント作成を並行して進行

5. **レビュー完了後**
   ```bash
   chaba cleanup --pr 234
   ```
   worktreeとsandbox環境を自動クリーンアップ

## 技術構成（予定）

- **言語**: Rust / TypeScript / Shell Script
- **Git操作**: `git worktree`, `git branch`, `gh` CLI
- **AIエージェント連携**: Claude Code SDK, MCP (Model Context Protocol)
- **テスト環境**: Playwright MCP integration
- **設定管理**: YAML / TOML

## プロジェクトの目標

私たちは今後、Claude、Codex、GeminiなどのAIエージェントを通してソフトウェアを構築します。
しかし、人間側はチームで活動することが多く、他のメンバーからのPR依頼で作業を中断することは避けられません。

**Chabaは、AIエージェントとの協働を前提とした新しい開発フローを実現します。**

- 自分のタスクに集中しながら、並行してレビュー作業を進められる
- AIエージェントが自動的にレビュー環境を構築し、初期分析を実施
- 人間は最終判断に集中でき、チーム全体の生産性が向上

## ロードマップ

- [ ] Phase 1: 基本的なgit worktree操作のCLI実装
- [ ] Phase 2: Sandbox環境の自動構築
- [ ] Phase 3: AIエージェント連携（Claude Code, Codex, Gemini）
- [ ] Phase 4: Playwright MCP統合とAPI debug機能
- [ ] Phase 5: Build & Simulator起動の自動化

## ライセンス

MIT License

## コントリビューション

このプロジェクトは開発初期段階です。
アイデア、フィードバック、PRを歓迎します！

---

**Built for the future of AI-assisted team development**
