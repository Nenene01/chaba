# Getting Started with Chaba

## Prerequisites

- **Git**: Version 2.5 or later (for `git worktree` support)
- **GitHub CLI** (`gh`): For PR operations
- **Node.js** or **Rust**: Depending on implementation (TBD)

## Installation

> **Note**: Chaba is currently in early development. Installation methods will be available soon.

### Planned Installation Methods

```bash
# Via Homebrew (macOS/Linux)
brew install chaba

# Via Cargo (Rust)
cargo install chaba

# Via npm (Node.js)
npm install -g chaba

# From source
git clone https://github.com/Nenene01/chaba.git
cd chaba
cargo build --release  # or: npm run build
```

## Initial Setup

### 1. Authenticate with GitHub

```bash
gh auth login
```

### 2. Configure Chaba

Create a configuration file:

```bash
# Global configuration
mkdir -p ~/.config/chaba
chaba config init

# Or project-specific
cd your-project
chaba config init --local
```

Edit `~/.config/chaba/chaba.yaml`:

```yaml
worktree:
  base_dir: "~/reviews"  # Where to create review worktrees
  auto_cleanup: true
  keep_days: 7

sandbox:
  auto_install_deps: true

agents:
  default_review_agents: ["claude"]
```

## Basic Usage

### Review a Pull Request

```bash
# Navigate to your project
cd ~/projects/myapp

# Start reviewing PR #123
chaba review --pr 123
```

**What happens:**
1. Fetches the PR branch
2. Creates a worktree at `~/reviews/pr-123`
3. Installs dependencies
4. Outputs the review environment path

**Output:**
```
✓ Fetched PR #123: "Add user authentication"
✓ Created worktree at ~/reviews/pr-123
✓ Installed dependencies (npm install)
✓ Ready to review!

To start reviewing:
  cd ~/reviews/pr-123
  code .  # or your preferred editor
```

### Review by Branch Name

```bash
chaba review --branch feature/new-api
```

### Enable AI Agent Analysis

```bash
# Use default agent (Claude Code)
chaba review --pr 123 --with-agent

# Use multiple agents for thorough review
chaba review --pr 123 --thorough
```

### Debug Mode with Testing Tools

```bash
# Launch with Playwright integration
chaba debug --pr 123 --with-playwright

# Launch with API debugging tools
chaba debug --pr 123 --with-api-debug
```

## Working in the Review Environment

Once the review environment is set up:

```bash
# Navigate to the review worktree
cd ~/reviews/pr-123

# Run tests
npm test  # or: cargo test

# Start dev server (assigned port shown during setup)
npm run dev  # Server will start on assigned port (e.g., 3042)

# Make changes if needed
# Your main workspace at ~/projects/myapp is untouched
```

## Cleanup

### Remove a Review Environment

```bash
# Clean up PR #123 review
chaba cleanup --pr 123
```

**What happens:**
- Removes the worktree
- Cleans up sandbox environment
- Frees assigned port

### List Active Reviews

```bash
chaba list
```

**Output:**
```
Active review environments:
  PR #123  ~/reviews/pr-123  feature/new-api     (2 days old)
  PR #456  ~/reviews/pr-456  bugfix/auth-error   (5 hours old)
```

### Auto-Cleanup

If `auto_cleanup: true` in config, old review environments are automatically removed after `keep_days`.

## Advanced Usage

### Custom Worktree Location

```bash
chaba review --pr 123 --worktree ~/custom/path
```

### Skip Dependency Installation

```bash
chaba review --pr 123 --no-deps
```

### Use Specific Agent

```bash
chaba review --pr 123 --agent codex
```

### Parallel Reviews

```bash
# Review multiple PRs simultaneously
chaba review --pr 123 &
chaba review --pr 456 &
```

Each review gets its own worktree and port assignment.

## Troubleshooting

### "Worktree already exists"

```bash
# Remove the existing worktree first
chaba cleanup --pr 123

# Or use --force to replace
chaba review --pr 123 --force
```

### "Port already in use"

Chaba automatically assigns the next available port. Check your config's `port_range_start` if needed.

### "Failed to fetch PR"

Ensure you're authenticated:
```bash
gh auth status
```

### AI Agent Not Responding

Check agent logs:
```bash
chaba logs --pr 123 --agent claude
```

## Next Steps

- Read [Configuration Guide](configuration.md) for advanced settings
- See [Architecture](architecture.md) to understand how Chaba works
- Check [CONTRIBUTING.md](../../CONTRIBUTING.md) to help improve Chaba

---

**Happy reviewing! 🍵**
