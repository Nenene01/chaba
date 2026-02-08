# CLI Module

Chabaのコマンドラインインターフェース実装。

## 主な機能

- `chaba review` - PRレビュー環境の起動
- `chaba debug` - デバッグ環境の起動
- `chaba cleanup` - worktreeとsandbox環境のクリーンアップ
- `chaba config` - 設定管理

## 開発予定

- Rust または TypeScript で実装
- `clap` (Rust) または `commander` (TypeScript) を使用したCLI解析
- コア機能との連携
