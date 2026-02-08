# Chaba Architecture

## Overview

Chaba is designed as a modular system with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────┐
│                     CLI Interface                        │
│  (User commands: review, debug, cleanup, config)        │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│                  Core Orchestrator                       │
│  (Coordinates worktree, sandbox, and agent operations)  │
└─────┬──────────────┬──────────────┬─────────────────────┘
      │              │              │
┌─────▼─────┐  ┌─────▼──────┐  ┌───▼──────────────┐
│ Worktree  │  │  Sandbox   │  │  Agent Manager   │
│  Manager  │  │  Manager   │  │  (AI Integration)│
└───────────┘  └────────────┘  └──────────────────┘
      │              │                    │
┌─────▼─────────────▼────────────────────▼─────────────┐
│              Git & File System Layer                  │
└───────────────────────────────────────────────────────┘
```

## Components

### 1. CLI Interface (`cli/`)

**Responsibility**: User-facing command-line interface

**Commands**:
- `chaba review --pr <number>` - Start review environment
- `chaba debug --pr <number>` - Start debug environment with testing tools
- `chaba cleanup --pr <number>` - Clean up worktree and sandbox
- `chaba config` - Manage configuration

**Implementation**:
- Argument parsing (clap for Rust / commander for TypeScript)
- Command routing to Core Orchestrator
- Output formatting (JSON, table, or plain text)

### 2. Core Orchestrator (`core/`)

**Responsibility**: Coordinate operations across all managers

**Key Operations**:
1. Parse user command and configuration
2. Validate PR/branch existence
3. Coordinate worktree creation
4. Set up sandbox environment
5. Launch AI agents if requested
6. Monitor and report status

### 3. Worktree Manager (`core/worktree.rs` or `core/worktree.ts`)

**Responsibility**: Manage git worktree lifecycle

**Operations**:
- `create_worktree(pr_number, branch_name)` - Create isolated worktree
- `list_worktrees()` - List active worktrees
- `remove_worktree(pr_number)` - Clean up worktree
- `sync_worktree(pr_number)` - Pull latest changes

**Implementation Details**:
```rust
// Example Rust structure
pub struct WorktreeManager {
    base_dir: PathBuf,
    repo_path: PathBuf,
}

impl WorktreeManager {
    pub fn create_worktree(&self, pr: u32, branch: &str) -> Result<Worktree> {
        // 1. Fetch branch
        // 2. Create worktree at base_dir/pr-{pr}
        // 3. Checkout branch
        // 4. Return Worktree handle
    }
}
```

### 4. Sandbox Manager (`core/sandbox.rs` or `core/sandbox.ts`)

**Responsibility**: Set up isolated development environment per worktree

**Operations**:
- `setup_sandbox(worktree_path)` - Install dependencies, configure env
- `detect_project_type(path)` - Detect Node.js, Rust, Python, etc.
- `install_dependencies(path, project_type)` - Run npm/cargo/pip install
- `assign_port(pr_number)` - Assign unique port for dev server
- `copy_env_vars(main_worktree, review_worktree)` - Copy .env files

**Project Type Detection**:
```
package.json     → Node.js   → npm install / yarn
Cargo.toml       → Rust      → cargo build
requirements.txt → Python    → pip install -r requirements.txt
pom.xml          → Java      → mvn install
```

**Port Assignment**:
- Maintain a registry of PR → Port mappings
- Default range: 3000-4000
- Avoid conflicts with main workspace

### 5. Agent Manager (`agent/`)

**Responsibility**: Integrate with AI agents for automated analysis

**Supported Agents**:
- **Claude Code**: Deep source review and architectural analysis
- **Codex**: Code quality checks and best practices
- **Gemini**: Alternative perspectives and pattern detection

**MCP Integration**:
- **Playwright MCP**: Automated E2E testing
- **API Debug Tools**: HTTP request testing

**Agent Lifecycle**:
```rust
pub struct AgentSession {
    agent_type: AgentType,
    session_id: String,
    status: SessionStatus,
}

impl AgentManager {
    pub fn launch_agent(&self, agent: AgentType, worktree: &Worktree) -> Result<AgentSession> {
        // 1. Start agent process
        // 2. Pass worktree path as context
        // 3. Send initial review task
        // 4. Return session handle
    }

    pub fn get_results(&self, session: &AgentSession) -> Result<ReviewResults> {
        // Collect JSON results from agent
    }
}
```

## Data Flow

### Review Command Flow

```
User: chaba review --pr 123
  │
  ├─> CLI parses arguments
  │
  ├─> Core Orchestrator validates PR #123 exists
  │
  ├─> Worktree Manager:
  │    ├─> Fetch origin/feature-branch
  │    ├─> Create worktree at ~/reviews/pr-123
  │    └─> Checkout feature-branch
  │
  ├─> Sandbox Manager:
  │    ├─> Detect project type (Node.js)
  │    ├─> Run npm install
  │    ├─> Copy .env from main workspace
  │    └─> Assign port 3042
  │
  ├─> Agent Manager (if --with-agent flag):
  │    ├─> Launch Claude Code session
  │    ├─> Send context: PR diff, worktree path
  │    └─> Task: "Review this PR for bugs and architectural issues"
  │
  └─> Output:
       ✓ Worktree created at ~/reviews/pr-123
       ✓ Dependencies installed
       ✓ Dev server available on port 3042
       ✓ Claude Code analyzing... (see results in ~/reviews/pr-123/review.json)
```

## Configuration

### File Locations
1. **Global**: `~/.config/chaba/chaba.yaml`
2. **Project**: `<project-root>/chaba.yaml`

### Configuration Schema
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

## Error Handling

- **Git errors**: Graceful fallback, suggest manual resolution
- **Network errors**: Retry with exponential backoff
- **Port conflicts**: Auto-increment to next available port
- **Agent failures**: Log error, continue without agent analysis

## Future Extensions

- **Parallel reviews**: Manage multiple PR reviews simultaneously
- **Review templates**: Customizable review checklists
- **Integration hooks**: Webhook notifications (Slack, Discord)
- **Metrics**: Track review time, agent accuracy

---

For implementation details, see the source code in `cli/`, `core/`, and `agent/`.
