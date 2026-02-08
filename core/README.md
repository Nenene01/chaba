# Core Module

Chabaのコア機能実装。

## 主な責務

### Git Worktree Manager
- git worktreeの作成、管理、削除
- ブランチ操作の自動化
- リモートブランチのpullとcheckout

### Sandbox Manager
- worktreeごとの独立したsandbox環境構築
- 依存関係のインストール（npm、cargo、pip等）
- 環境変数の設定と管理
- ポート番号の自動割り当て

### Repository Analyzer
- PRの変更内容の解析
- 影響範囲の特定
- テストファイルの検出

## 技術スタック（予定）

- Rust / TypeScript
- `git2` (Rust) または `simple-git` (TypeScript)
- プロセス管理ライブラリ
