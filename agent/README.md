# Agent Module

AIエージェントとの連携機能。

## 主な責務

### Claude Code Integration
- Claude Codeサブエージェントの起動
- ソースレビュータスクの自動実行
- レビューコメントの生成

### Codex Integration
- OpenAI Codex CLIとの連携
- コード品質チェック
- セカンドオピニオンの取得

### Gemini Integration
- Google Gemini CLIとの連携
- 多角的な分析の実施

### MCP Integration
- Model Context Protocol対応
- Playwright MCPによる自動テスト実行
- API debugツールの統合

## 設計方針

- 各AIエージェントは独立したプロセスとして起動
- 標準入出力またはMCPプロトコルで通信
- レビュー結果はJSON形式で取得
- 複数エージェントの並列実行をサポート

## 開発予定

- Claude Code SDK活用
- MCPサーバーとの通信実装
- エージェント実行結果の集約と整形
