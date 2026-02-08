# Chaba (茶葉)

**AI Agent Friendly Source Review & Debug Environment**

[![npm version](https://img.shields.io/npm/v/@nenene01/chaba.svg)](https://www.npmjs.com/package/@nenene01/chaba)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

```bash
npm install -g @nenene01/chaba
```

## Quick Start

```bash
# Create review environment for PR #123
chaba review --pr 123

# Review with AI agent analysis
chaba review --pr 123 --with-agent

# Thorough review with all agents
chaba review --pr 123 --thorough

# View agent analysis results
chaba agent-result --pr 123

# List active reviews
chaba list

# Clean up when done
chaba cleanup --pr 123
```

## Features

- **Git Worktree Management**: Create isolated review environments for PRs
- **Automatic Sandbox Setup**: Detect project type and install dependencies
- **AI Agent Integration**: Automated code review with Claude, Codex, and Gemini
- **Port Management**: Automatic port assignment for parallel development
- **Environment Copying**: Copy .env files to review environments

## Requirements

- Node.js >= 14
- Git
- GitHub CLI (`gh`) for PR operations

## Supported Platforms

- macOS (Intel & Apple Silicon)
- Linux (x64 & ARM64)
- Windows (x64)

## Alternative Installation Methods

### Via Cargo (Rust)

```bash
cargo install chaba
```

### From Source

```bash
git clone https://github.com/Nenene01/chaba.git
cd chaba
cargo install --path .
```

## Documentation

- [GitHub Repository](https://github.com/Nenene01/chaba)
- [Architecture](https://github.com/Nenene01/chaba/blob/master/ARCHITECTURE.md)
- [Contributing](https://github.com/Nenene01/chaba/blob/master/CONTRIBUTING.md)

## License

MIT License - see [LICENSE](https://github.com/Nenene01/chaba/blob/master/LICENSE) for details.

## Author

Kei Sekine ([@Nenene01](https://github.com/Nenene01))

---

Built for the future of AI-assisted team development 🍵
