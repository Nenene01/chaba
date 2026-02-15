# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-15

### Added

#### Core Features
- Git worktree management (create, delete, list, status)
- AI agent integration with multiple orchestration support
- Sandbox environment auto-setup (dependencies, environment variables)
- Port management system for development servers
- State management with optimistic locking

#### Git Integration
- Merge and rebase commands for worktrees (#10)
- Git statistics display (file changes, commits ahead/behind) (#15)
- Automatic base directory creation (#1)
- Worktree existence validation (#3)

#### Session Management
- Claude Code session data copying between worktrees (#7)
- Session history and context preservation

#### Hooks System
- Post-create worktree hooks (#8)
- Environment variables support (CHABA_WORKTREE_PATH, CHABA_BRANCH, CHABA_PR)
- Non-blocking async execution

#### User Interface
- Terminal User Interface (TUI) with ratatui (#6)
- Interactive worktree management
- Visual status indicators

#### Developer Experience
- `--force`/`--yes` option for non-interactive cleanup (#2)
- `.gitignore` suggestion for local config (#4)
- Comprehensive error handling and user guidance
- Progress indicators for long-running operations

#### Safety & Security
- Path traversal protection
- Symlink attack prevention
- Concurrent modification detection
- Uncommitted changes validation before git operations

### Technical Details

#### Dependencies
- Rust 2021 edition
- Git2 for repository operations
- Tokio for async runtime
- Ratatui/Crossterm for TUI
- Dialoguer for interactive prompts
- Serde for configuration management

#### Supported Platforms
- macOS (x86_64, ARM64)
- Linux (x86_64, ARM64)
- Windows (x86_64)

### Configuration

Default configuration file locations:
- `./chaba.yaml` (project-specific)
- `~/.config/chaba/chaba.yaml` (global)

### Commands

- `chaba review` - Create review environment for PR/branch
- `chaba cleanup` - Remove review environment
- `chaba list` - List active review environments
- `chaba status` - Show detailed status of a review environment
- `chaba merge` - Merge branch into worktree
- `chaba rebase` - Rebase worktree onto branch
- `chaba config` - Initialize configuration
- `chaba agent-result` - View AI agent analysis results
- `chaba tui` - Launch Terminal User Interface

### Installation

Download pre-built binaries from [GitHub Releases](https://github.com/Nenene01/chaba/releases).

Or build from source:
```bash
cargo install --git https://github.com/Nenene01/chaba
```

### Contributors

- Kei Sekine Dev (@Nenene01)
- Claude Sonnet 4.5 (AI Development Partner)

[0.1.0]: https://github.com/Nenene01/chaba/releases/tag/v0.1.0
