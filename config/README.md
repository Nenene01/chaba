# Config Module

Chabaの設定ファイルとサンプル。

## 設定ファイル形式

`chaba.yaml` または `chaba.toml` をプロジェクトルートまたは `~/.config/chaba/` に配置。

### サンプル設定 (YAML)

```yaml
# Worktree設定
worktree:
  base_dir: "~/reviews"  # worktreeを作成するベースディレクトリ
  auto_cleanup: true      # レビュー完了後に自動削除
  keep_days: 7            # 自動削除までの日数

# Sandbox設定
sandbox:
  auto_install_deps: true  # 依存関係の自動インストール
  node_version: "lts"      # Node.jsバージョン（nvmを使用）
  port_range_start: 3000   # 開発サーバーのポート範囲開始
  port_range_end: 4000     # 開発サーバーのポート範囲終了

# AIエージェント設定
agents:
  claude:
    enabled: true
    model: "sonnet"
  codex:
    enabled: true
  gemini:
    enabled: true

  # レビュー時のデフォルト動作
  default_review_agents:
    - claude

  # 詳細レビュー時（--thoroughオプション）
  thorough_review_agents:
    - claude
    - codex
    - gemini

# MCP設定
mcp:
  playwright:
    enabled: true
    auto_run_tests: true  # PRのテストを自動実行

# GitHub設定
github:
  auto_fetch_pr: true      # PR番号から自動的にfetch
  draft_review_comment: true  # レビューコメントのドラフトを作成
