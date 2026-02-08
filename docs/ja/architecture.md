# Chabaアーキテクチャ

## 概要

Chabaはモジュール式システムとして設計されており、明確な責務分離があります:

```
┌─────────────────────────────────────────────────────────┐
│                  CLIインターフェース                      │
│  (ユーザーコマンド: review, debug, cleanup, config)      │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                コアオーケストレータ                       │
│  (worktree、sandbox、agentの操作を統括)                 │
└─────┬──────────────┬──────────────┬─────────────────────┘
      │              │              │
┌─────▼─────┐  ┌─────▼──────┐  ┌───▼──────────────┐
│ Worktree  │  │  Sandbox   │  │  Agent Manager   │
│  Manager  │  │  Manager   │  │  (AI統合)        │
└───────────┘  └────────────┘  └──────────────────┘
      │              │                    │
┌─────▼─────────────▼────────────────────▼─────────────┐
│              Git & ファイルシステム層                  │
└───────────────────────────────────────────────────────┘
```

## コンポーネント

### 1. CLIインターフェース (`cli/`)

**責務**: ユーザー向けコマンドラインインターフェース

**コマンド**:
- `chaba review --pr <number>` - レビュー環境を起動
- `chaba debug --pr <number>` - テストツール付きデバッグ環境を起動
- `chaba cleanup --pr <number>` - worktreeとsandboxをクリーンアップ
- `chaba config` - 設定管理

**実装**:
- 引数解析 (Rustの場合clap / TypeScriptの場合commander)
- コアオーケストレータへのコマンドルーティング
- 出力フォーマット (JSON、テーブル、またはプレーンテキスト)

### 2. コアオーケストレータ (`core/`)

**責務**: すべてのマネージャー間の操作を調整

**主な操作**:
1. ユーザーコマンドと設定の解析
2. PR/ブランチの存在を検証
3. worktree作成の調整
4. sandbox環境のセットアップ
5. 要求された場合AIエージェントを起動
6. ステータスの監視と報告

### 3. Worktree Manager (`core/worktree.rs` または `core/worktree.ts`)

**責務**: git worktreeのライフサイクル管理

**操作**:
- `create_worktree(pr_number, branch_name)` - 独立したworktreeを作成
- `list_worktrees()` - アクティブなworktreeをリスト表示
- `remove_worktree(pr_number)` - worktreeをクリーンアップ
- `sync_worktree(pr_number)` - 最新の変更をpull

**実装詳細**:
```rust
// Rustの構造例
pub struct WorktreeManager {
    base_dir: PathBuf,
    repo_path: PathBuf,
}

impl WorktreeManager {
    pub fn create_worktree(&self, pr: u32, branch: &str) -> Result<Worktree> {
        // 1. ブランチをfetch
        // 2. base_dir/pr-{pr} にworktreeを作成
        // 3. ブランチをcheckout
        // 4. Worktreeハンドルを返す
    }
}
```

### 4. Sandbox Manager (`core/sandbox.rs` または `core/sandbox.ts`)

**責務**: worktreeごとに独立した開発環境をセットアップ

**操作**:
- `setup_sandbox(worktree_path)` - 依存関係のインストール、環境設定
- `detect_project_type(path)` - Node.js、Rust、Pythonなどを検出
- `install_dependencies(path, project_type)` - npm/cargo/pip installを実行
- `assign_port(pr_number)` - 開発サーバー用の一意なポートを割り当て
- `copy_env_vars(main_worktree, review_worktree)` - .envファイルをコピー

**プロジェクトタイプ検出**:
```
package.json     → Node.js   → npm install / yarn
Cargo.toml       → Rust      → cargo build
requirements.txt → Python    → pip install -r requirements.txt
pom.xml          → Java      → mvn install
```

**ポート割り当て**:
- PR → ポートマッピングのレジストリを維持
- デフォルト範囲: 3000-4000
- メインワークスペースとの競合を回避

### 5. Agent Manager (`agent/`)

**責務**: 自動分析のためのAIエージェントとの統合

**サポートするエージェント**:
- **Claude Code**: 深いソースレビューとアーキテクチャ分析
- **Codex**: コード品質チェックとベストプラクティス
- **Gemini**: 代替的視点とパターン検出

**MCP統合**:
- **Playwright MCP**: 自動E2Eテスト
- **API Debugツール**: HTTPリクエストテスト

**エージェントライフサイクル**:
```rust
pub struct AgentSession {
    agent_type: AgentType,
    session_id: String,
    status: SessionStatus,
}

impl AgentManager {
    pub fn launch_agent(&self, agent: AgentType, worktree: &Worktree) -> Result<AgentSession> {
        // 1. エージェントプロセスを起動
        // 2. worktreeパスをコンテキストとして渡す
        // 3. 初期レビュータスクを送信
        // 4. セッションハンドルを返す
    }

    pub fn get_results(&self, session: &AgentSession) -> Result<ReviewResults> {
        // エージェントからJSON結果を収集
    }
}
```

## データフロー

### Reviewコマンドのフロー

```
ユーザー: chaba review --pr 123
  │
  ├─> CLIが引数を解析
  │
  ├─> コアオーケストレータがPR #123の存在を検証
  │
  ├─> Worktree Manager:
  │    ├─> origin/feature-branchをfetch
  │    ├─> ~/reviews/pr-123 にworktreeを作成
  │    └─> feature-branchをcheckout
  │
  ├─> Sandbox Manager:
  │    ├─> プロジェクトタイプを検出 (Node.js)
  │    ├─> npm installを実行
  │    ├─> メインワークスペースから.envをコピー
  │    └─> ポート3042を割り当て
  │
  ├─> Agent Manager (--with-agentフラグの場合):
  │    ├─> Claude Codeセッションを起動
  │    ├─> コンテキスト送信: PR diff、worktreeパス
  │    └─> タスク: "このPRをバグとアーキテクチャの問題についてレビュー"
  │
  └─> 出力:
       ✓ Worktreeを ~/reviews/pr-123 に作成
       ✓ 依存関係をインストール完了
       ✓ 開発サーバーがポート3042で利用可能
       ✓ Claude Codeが分析中... (結果は ~/reviews/pr-123/review.json)
```

## 設定

### ファイルの場所
1. **グローバル**: `~/.config/chaba/chaba.yaml`
2. **プロジェクト**: `<プロジェクトルート>/chaba.yaml`

### 設定スキーマ
```yaml
worktree:
  base_dir: "~/reviews"
  naming_template: "pr-{pr}"

sandbox:
  auto_install_deps: true
  port_range_start: 3000

agents:
  default_review_agents: ["claude"]
  thorough_review_agents: ["claude", "codex", "gemini"]
```

## エラーハンドリング

- **Gitエラー**: グレースフルフォールバック、手動解決を提案
- **ネットワークエラー**: 指数バックオフでリトライ
- **ポート競合**: 次の利用可能なポートに自動インクリメント
- **エージェント失敗**: エラーをログ、エージェント分析なしで継続

## 将来の拡張

- **並行レビュー**: 複数のPRレビューを同時に管理
- **レビューテンプレート**: カスタマイズ可能なレビューチェックリスト
- **統合フック**: Webhook通知 (Slack、Discord)
- **メトリクス**: レビュー時間、エージェント精度の追跡

---

実装の詳細については、`cli/`、`core/`、`agent/` のソースコードを参照してください。
