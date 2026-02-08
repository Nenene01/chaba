# Contributing to Chaba

[English](#english) | [日本語](#日本語)

---

## English

Thank you for your interest in contributing to Chaba! We welcome contributions from developers worldwide.

### Ways to Contribute

- **Bug Reports**: Found a bug? [Open an issue](https://github.com/Nenene01/chaba/issues)
- **Feature Requests**: Have an idea? Share it in [Discussions](https://github.com/Nenene01/chaba/discussions)
- **Code Contributions**: Submit a pull request
- **Documentation**: Improve docs or add translations
- **Testing**: Try Chaba and report your experience

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Nenene01/chaba.git
cd chaba

# For Rust development (if we choose Rust)
cargo build
cargo test

# For TypeScript development (if we choose TypeScript)
npm install
npm run build
npm test
```

### Pull Request Process

1. **Fork** the repository
2. **Create a branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make your changes** with clear, descriptive commits
4. **Add tests** if applicable
5. **Update documentation** if needed
6. **Submit a PR** with a clear description

### Commit Message Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add worktree cleanup command
fix: resolve port conflict in sandbox
docs: update installation guide
test: add integration tests for review command
refactor: simplify git operations module
```

### Code Style

- **Rust**: Follow `rustfmt` and `clippy` suggestions
- **TypeScript**: Follow ESLint and Prettier configurations
- **Comments**: Write clear comments, especially for complex logic
- **Tests**: Aim for good test coverage

### Translation Contributions

We welcome translations! See [`docs/localization.md`](docs/localization.md) for guidelines.

**Current languages:**
- English (en)
- Japanese (ja)

**Help wanted:**
- Chinese (zh)
- Korean (ko)
- Spanish (es)
- French (fr)
- German (de)

### Questions?

- Join our [Discussions](https://github.com/Nenene01/chaba/discussions)
- Open an [Issue](https://github.com/Nenene01/chaba/issues)

### Code of Conduct

Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before contributing.

---

## 日本語

Chabaへの貢献に興味を持っていただきありがとうございます！世界中の開発者からの貢献を歓迎します。

### 貢献方法

- **バグ報告**: バグを見つけましたか？ [Issueを開く](https://github.com/Nenene01/chaba/issues)
- **機能リクエスト**: アイデアがありますか？ [Discussions](https://github.com/Nenene01/chaba/discussions)でシェア
- **コード貢献**: プルリクエストを送信
- **ドキュメント**: ドキュメントの改善や翻訳の追加
- **テスト**: Chabaを試して、フィードバックを報告

### 開発環境のセットアップ

```bash
# リポジトリをクローン
git clone https://github.com/Nenene01/chaba.git
cd chaba

# Rust開発の場合（Rustを選択した場合）
cargo build
cargo test

# TypeScript開発の場合（TypeScriptを選択した場合）
npm install
npm run build
npm test
```

### プルリクエストのプロセス

1. リポジトリを **Fork**
2. `main`から **ブランチを作成**:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. 明確で説明的なコミットで **変更を加える**
4. 該当する場合は **テストを追加**
5. 必要に応じて **ドキュメントを更新**
6. 明確な説明を含む **PRを送信**

### コミットメッセージガイドライン

[Conventional Commits](https://www.conventionalcommits.org/)に従います:

```
feat: worktreeクリーンアップコマンドを追加
fix: sandbox内のポート競合を解決
docs: インストールガイドを更新
test: reviewコマンドの統合テストを追加
refactor: git操作モジュールを簡素化
```

### コードスタイル

- **Rust**: `rustfmt` と `clippy` の提案に従う
- **TypeScript**: ESLint と Prettier の設定に従う
- **コメント**: 特に複雑なロジックには明確なコメントを書く
- **テスト**: 良好なテストカバレッジを目指す

### 翻訳への貢献

翻訳を歓迎します！ガイドラインは [`docs/localization.md`](docs/localization.md) を参照してください。

**現在の言語:**
- 英語 (en)
- 日本語 (ja)

**協力求む:**
- 中国語 (zh)
- 韓国語 (ko)
- スペイン語 (es)
- フランス語 (fr)
- ドイツ語 (de)

### 質問がある場合

- [Discussions](https://github.com/Nenene01/chaba/discussions)に参加
- [Issue](https://github.com/Nenene01/chaba/issues)を開く

### 行動規範

貢献する前に、[行動規範](CODE_OF_CONDUCT.md)をお読みください。

---

**Thank you for helping make Chaba better for everyone! 🍵**
