# Chaba Architecture

This document describes the architecture and design decisions of Chaba.

## Table of Contents

- [Overview](#overview)
- [Module Structure](#module-structure)
- [Phase 1: Basic CLI](#phase-1-basic-cli)
- [Phase 2: Sandbox Automation](#phase-2-sandbox-automation)
- [Phase 3: AI Agent Integration](#phase-3-ai-agent-integration)
- [Data Flow](#data-flow)
- [Configuration](#configuration)
- [State Management](#state-management)
- [Error Handling](#error-handling)
- [Testing Strategy](#testing-strategy)

## Overview

Chaba is a Rust CLI tool that integrates:
1. Git worktree management for isolated PR review environments
2. Automatic sandbox setup with dependency installation
3. AI agent orchestration for automated code review

### Design Principles

1. **Simplicity**: Single command to start a review environment
2. **Speed**: Parallel operations and async I/O
3. **Reliability**: Strong error handling and state persistence
4. **Extensibility**: Modular design for future features

## Module Structure

```
src/
├── lib.rs              # Library entry point
├── main.rs             # Binary entry point
├── cli.rs              # CLI argument parsing
├── commands/           # CLI command implementations
│   ├── agent_result.rs # View AI agent analysis
│   ├── cleanup.rs      # Clean up review environments
│   ├── config.rs       # Initialize configuration
│   ├── list.rs         # List active reviews
│   ├── review.rs       # Create review environment
│   └── status.rs       # Show review status
├── config.rs           # Configuration management
├── core/               # Core business logic
│   ├── agent.rs        # AI agent orchestration
│   ├── env.rs          # Environment file copying
│   ├── git.rs          # Git operations
│   ├── installer.rs    # Dependency installation
│   ├── port.rs         # Port management
│   ├── project.rs      # Project type detection
│   ├── review_analysis.rs # Analysis data structures
│   ├── sandbox.rs      # Sandbox orchestration
│   ├── state.rs        # State persistence
│   └── worktree.rs     # Worktree management
└── error.rs            # Error types
```

## Phase 1: Basic CLI

**Goal**: Manage git worktrees for PR review environments

### Components

#### GitOps (`core/git.rs`)
- Wraps `git2` crate for repository operations
- Provides async git command execution via `tokio::process::Command`
- Methods:
  - `fetch_branch()`: Fetch PR branch from remote
  - `add_worktree()`: Create new worktree
  - `remove_worktree()`: Delete worktree

#### WorktreeManager (`core/worktree.rs`)
- High-level worktree lifecycle management
- Integrates with State for persistence
- Supports custom paths and naming templates

#### State Management (`core/state.rs`)
- Persists review environments to `~/.chaba/state.yaml`
- ReviewState structure:
  ```rust
  pub struct ReviewState {
      pub pr_number: u32,
      pub branch: String,
      pub worktree_path: PathBuf,
      pub created_at: DateTime<Utc>,
      // Phase 2 fields
      pub port: Option<u16>,
      pub project_type: Option<String>,
      // Phase 3 fields
      pub agent_analyses: Vec<ReviewAnalysis>,
  }
  ```

#### CLI Commands
- `review`: Create review environment
- `cleanup`: Remove worktree and state
- `list`: Show active reviews
- `status`: Show detailed review status
- `config`: Generate configuration file

## Phase 2: Sandbox Automation

**Goal**: Automatically set up development environment in each worktree

### Components

#### ProjectDetector (`core/project.rs`)
- Detects project type by examining files:
  - Node.js: `package.json`
  - Rust: `Cargo.toml`
  - Python: `requirements.txt`, `pyproject.toml`
  - Go: `go.mod`

#### DependencyInstaller (`core/installer.rs`)
- Installs dependencies based on project type:
  - Node.js: npm, yarn, pnpm, bun (auto-detected)
  - Rust: `cargo build`
  - Python: `pip install -r requirements.txt`
  - Go: `go mod download`

#### PortManager (`core/port.rs`)
- Assigns unique ports from configured range (default: 3000-4000)
- Avoids conflicts with existing reviews
- Uses TCP binding to verify port availability

#### EnvManager (`core/env.rs`)
- Copies environment files from main worktree
- Default: `.env`
- Configurable additional files (e.g., `.env.local`)

#### SandboxManager (`core/sandbox.rs`)
- Orchestrates all sandbox setup steps:
  1. Detect project type
  2. Install dependencies
  3. Assign port
  4. Copy environment files

## Phase 3: AI Agent Integration

**Goal**: Automated code review with multiple AI agents

### Components

#### ReviewAnalysis (`core/review_analysis.rs`)
- Data structures for agent findings:
  ```rust
  pub struct ReviewAnalysis {
      pub agent: String,
      pub timestamp: String,
      pub score: Option<f32>,
      pub findings: Vec<Finding>,
  }

  pub struct Finding {
      pub severity: Severity,    // Critical, High, Medium, Low, Info
      pub category: Category,    // Security, Performance, etc.
      pub file: Option<String>,
      pub line: Option<u32>,
      pub title: String,
      pub description: String,
      pub suggestion: Option<String>,
  }
  ```

#### AgentManager (`core/agent.rs`)
- Orchestrates AI agent execution
- Supports three agents:
  - **Claude**: General-purpose review (fast)
  - **Codex**: Implementation details
  - **Gemini**: Strategic analysis
- Execution modes:
  - Parallel: All agents run simultaneously (`futures::future::join_all`)
  - Sequential: Agents run one after another
- Timeout management: 600 seconds default
- Output parsing: Basic pattern matching (future: JSON structured output)

#### Agent Commands
- `review --with-agent`: Quick review with default agents
- `review --thorough`: Comprehensive review with all agents
- `agent-result --pr <n>`: View analysis results

### Agent Execution Flow

```
User: chaba review --pr 123 --thorough
  │
  ├─> WorktreeManager.create()
  │   ├─> Git: fetch + create worktree
  │   ├─> SandboxManager: setup environment
  │   └─> State: save review
  │
  └─> AgentManager.run_review()
      ├─> Spawn tasks for each agent
      │   ├─> run_single_agent("claude")
      │   ├─> run_single_agent("codex")
      │   └─> run_single_agent("gemini")
      │
      ├─> futures::future::join_all()
      │
      ├─> Parse outputs → ReviewAnalysis
      │
      └─> State: save analyses
```

## Data Flow

### Review Creation

```
User Command
    ↓
CLI Argument Parsing
    ↓
review::execute()
    ↓
WorktreeManager::create()
    ├─> GitOps::fetch_branch()
    ├─> GitOps::add_worktree()
    ├─> SandboxManager::setup()
    │   ├─> ProjectDetector::detect()
    │   ├─> DependencyInstaller::install()
    │   ├─> PortManager::assign_port()
    │   └─> EnvManager::copy_env_files()
    ├─> (Optional) AgentManager::run_review()
    └─> State::add_review()
```

### Agent Analysis

```
AgentManager::run_review()
    ↓
Parallel Execution
    ├─> Claude Agent
    │   └─> Command: claude --model sonnet --yes <prompt>
    ├─> Codex Agent
    │   └─> Command: codex exec --full-auto --sandbox read-only <prompt>
    └─> Gemini Agent
        └─> Command: gemini -m gemini-2.0-flash-001 -s -y -p <prompt>
    ↓
Collect Results
    ↓
Parse Outputs → ReviewAnalysis
    ↓
Store in ReviewState
```

## Configuration

Configuration is loaded from:
1. `./chaba.yaml` (current directory)
2. `~/.config/chaba/chaba.yaml` (user config)
3. Default values

### Structure

```yaml
worktree:
  base_dir: ~/reviews
  naming_template: pr-{pr}
  auto_cleanup: true
  keep_days: 7

sandbox:
  auto_install_deps: true
  copy_env_from_main: true
  additional_env_files:
    - .env.local
  node:
    package_manager: auto
  port:
    enabled: true
    range_start: 3000
    range_end: 4000

agents:
  enabled: true
  default_agents:
    - claude
  thorough_agents:
    - claude
    - codex
    - gemini
  timeout: 600
  parallel: true
```

## State Management

State is persisted in `~/.chaba/state.yaml` with file permissions `600` (owner read/write only).

### Backward Compatibility

Fields added in Phase 2 and 3 use `#[serde(default)]` and `skip_serializing_if` to maintain compatibility:

```rust
// Phase 2 fields (optional)
#[serde(default, skip_serializing_if = "Option::is_none")]
pub port: Option<u16>,

// Phase 3 fields (optional)
#[serde(default, skip_serializing_if = "Vec::is_empty")]
pub agent_analyses: Vec<ReviewAnalysis>,
```

This allows:
- Phase 1 state files to load in Phase 2/3
- Phase 2 state files to load in Phase 3
- Empty fields are omitted from serialization

## Error Handling

### Error Types

All errors are defined in `error.rs` using `thiserror`:

```rust
pub enum ChabaError {
    GitError(git2::Error),
    GhCliNotFound,
    GhCliError(String),
    PrNotFound(u32),
    WorktreeExists(PathBuf),
    NotInGitRepo,
    ConfigError(String),
    NoAvailablePort { range_start, range_end },
    IoError(std::io::Error),
    Other(anyhow::Error),
}
```

### Error Handling Strategy

1. **Domain errors**: Use `ChabaError` variants
2. **Generic errors**: Wrap in `ChabaError::Other(anyhow::Error)`
3. **Propagation**: Use `?` operator with `Result<T>` type alias
4. **User messages**: Error display messages are user-friendly

## Testing Strategy

### Test Structure

```
Total: 56 tests
├── Unit Tests: 32
│   ├── review_analysis.rs: 13 tests
│   ├── state.rs: 12 tests
│   ├── port.rs: 3 tests
│   ├── env.rs: 2 tests
│   └── project.rs: 4 tests
│
└── Integration Tests: 24
    ├── cli_test.rs: 17 tests
    └── config_test.rs: 7 tests
```

### Test Coverage

**Covered**:
- Data structure serialization/deserialization
- State management (CRUD, backward compatibility)
- Port assignment logic
- Environment file copying
- Project type detection
- CLI argument parsing
- Configuration loading

**Not Covered** (requires mocking or E2E):
- Actual git operations
- AI agent execution
- GitHub CLI integration
- Dependency installation

### Testing Tools

- `tempfile`: Temporary directories for isolated tests
- `assert_cmd`: CLI binary testing
- `predicates`: Assertion helpers
- `tokio::test`: Async test support

## Future Architecture

### Phase 4: MCP Integration

Planned integration with Model Context Protocol:
- Playwright for browser testing
- API debugging tools
- Enhanced agent capabilities

### Phase 5: npm Distribution

Wrapper design:
- Rust binary distribution via npm
- Platform-specific binaries
- Automatic installation script

---

**Last Updated**: 2026-02-08
**Phases Completed**: 1, 2, 3
**Test Coverage**: 56 tests, 100% pass rate
